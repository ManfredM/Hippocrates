use crate::domain::{AskRequest, InputMessage, RuntimeValue, EventType};
use crate::runtime::{Environment, Executor};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

pub struct Session {
    common_definitions: Arc<Mutex<HashMap<String, (RuntimeValue, chrono::NaiveDateTime)>>>,
    executors: Arc<Mutex<Vec<mpsc::Sender<InputMessage>>>>,
    pending_requests: Arc<Mutex<HashSet<String>>>,
    on_ask: Arc<dyn Fn(AskRequest) + Send + Sync>,
    on_log: Arc<dyn Fn(String, EventType, chrono::NaiveDateTime) + Send + Sync>,
}

impl Session {
    pub fn new(
        on_ask: Box<dyn Fn(AskRequest) + Send + Sync>,
        on_log: Box<dyn Fn(String, EventType, chrono::NaiveDateTime) + Send + Sync>,
    ) -> Self {
        Session {
            common_definitions: Arc::new(Mutex::new(HashMap::new())),
            executors: Arc::new(Mutex::new(Vec::new())),
            pending_requests: Arc::new(Mutex::new(HashSet::new())),
            on_ask: Arc::from(on_ask),
            on_log: Arc::from(on_log),
        }
    }

    pub fn provide_answer(&self, variable: &str, value: RuntimeValue) {
        let now = chrono::Utc::now().naive_utc();
        // 1. Update common definitions
        {
            let mut defs = self.common_definitions.lock().unwrap();
            defs.insert(variable.to_string(), (value.clone(), now));
        }

        // 2. Remove from pending requests
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.remove(variable);
        }

        // 3. Broadcast to all executors
        let msg = InputMessage {
            variable: variable.to_string(),
            value,
            timestamp: now,
        };

        let mut executors = self.executors.lock().unwrap();
        // Remove disconnected channels while transmitting
        executors.retain(|tx| {
            tx.send(msg.clone()).is_ok()
        });
    }

    pub fn run_script(&self, source: String, plan_name: String) {
        // Clone Arcs to move into thread
        let common_defs = self.common_definitions.clone();
        let executors = self.executors.clone();
        let pending = self.pending_requests.clone();
        let on_ask = self.on_ask.clone();
        let on_log = self.on_log.clone();
        
        thread::spawn(move || {
            // Parse and Prepare Environment
            let mut env = Environment::new();
            match crate::parser::parse_plan(&source) {
                Ok(plan) => {
                    env.load_plan(plan);
                },
                Err(e) => {
                    (on_log)(format!("Parse Error: {}", e), EventType::Log, chrono::Utc::now().naive_utc());
                    return;
                }
            }

            // Create Executor
            let stop_signal = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let mut executor = Executor::new(stop_signal);

            // Channel for this executor
            let (tx, rx) = mpsc::channel();
            executor.set_input_receiver(rx);

            // Clone tx for local use inside this thread (for callback)
            let tx_local = tx.clone();

            // Register this executor and Bootstrap known values
            {
                let mut exec_list = executors.lock().unwrap();
                exec_list.push(tx.clone());

                let defs = common_defs.lock().unwrap();
                for (k, (v, ts)) in defs.iter() {
                    let msg = InputMessage {
                        variable: k.clone(),
                        value: v.clone(),
                        timestamp: *ts,
                    };
                    let _ = tx.send(msg);
                }
            }
            
            // Set callbacks
            let on_ask_clone = on_ask.clone();
            let common_defs_clone = common_defs.clone();
            let pending_clone = pending.clone();
            
            executor.set_ask_callback(Box::new(move |req| {
                let var = req.variable_name.clone();
                
                // Check if already known
                {
                    let defs = common_defs_clone.lock().unwrap();
                    if let Some((val, ts)) = defs.get(&var) {
                        // Check validity if requested
                        let is_valid = if let Some(min_ts) = req.valid_after {
                            ts.and_utc().timestamp_millis() >= min_ts
                        } else {
                            true
                        };

                        if is_valid {
                            // Already known, send answer to myself immediately
                            let msg = InputMessage {
                                variable: var.clone(),
                                value: val.clone(),
                                timestamp: *ts,
                            };
                            let _ = tx_local.send(msg);
                            return;
                        }
                        // If not valid, fall through to user ask
                    }
                }
                
                // Check if already pending
                {
                    let mut p = pending_clone.lock().unwrap();
                    if p.contains(&var) {
                        // Already asked, just wait (do nothing, Session will broadcast when ready)
                        return; 
                    }
                    p.insert(var);
                }
                
                // Call real callback
                (on_ask_clone)(req);
            }));

            // Log callback
            let on_log_clone = on_log.clone();
            executor.on_log = Some(Box::new(move |msg, kind, time| {
                (on_log_clone)(msg, kind, time);
            }));

            // Execute
            executor.execute_plan(&mut env, &plan_name);
        });
    }
}
