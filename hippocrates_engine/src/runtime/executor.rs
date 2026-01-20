use crate::ast::{Action, Block, Statement, StatementKind};
use crate::domain::Unit;
use crate::runtime::{Environment, Evaluator, scheduler::Scheduler};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, atomic::{AtomicBool, Ordering as AtomicOrdering}};

#[derive(Debug, Clone)]
struct ScheduledEvent {
    time: chrono::DateTime<chrono::Utc>,
    kind: EventKind,
}

#[derive(Debug, Clone)]
enum EventKind {
    Periodic {
        block: crate::ast::TriggerBlock,
        iteration: u64,
        interval_secs: f64,
        max_duration: Option<chrono::Duration>,
    },
    StartOf {
        block: crate::ast::EventBlock,
        period_name: String,
    },
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for Min-Heap
        other.time.cmp(&self.time)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExecutionMode {
    RealTime,
    Simulation { speed_factor: Option<f64>, duration: Option<chrono::Duration> },
}

pub struct Executor {
    pub on_step: Option<Box<dyn Fn(usize) + Send>>,
    pub on_log: Option<Box<dyn Fn(String, crate::domain::EventType, chrono::DateTime<chrono::Utc>) + Send>>,
    pub on_ask: Option<Box<dyn Fn(crate::domain::AskRequest) + Send>>,
    pub mode: ExecutionMode,
    pub input_receiver: Option<std::sync::mpsc::Receiver<crate::domain::InputMessage>>,
    pub stop_signal: Arc<AtomicBool>,
}

impl Executor {
    pub fn new(stop_signal: Arc<AtomicBool>) -> Self {
        Executor {
            on_step: None,
            on_log: None,
            on_ask: None,
            mode: ExecutionMode::RealTime,
            input_receiver: None,
            stop_signal,
        }
    }

    pub fn with_activites(
        line_cb: Box<dyn Fn(usize) + Send>,
        log_cb: Box<dyn Fn(String, crate::domain::EventType, chrono::DateTime<chrono::Utc>) + Send>,
    ) -> Self {
        Executor {
            on_step: Some(line_cb),
            on_log: Some(log_cb),
            on_ask: None,
            mode: ExecutionMode::RealTime,
            input_receiver: None,
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_input_receiver(&mut self, rx: std::sync::mpsc::Receiver<crate::domain::InputMessage>) {
        self.input_receiver = Some(rx);
    }

    pub fn set_mode(&mut self, mode: ExecutionMode) {
        self.mode = mode;
    }

    pub fn drain_inputs(&mut self, env: &mut Environment) {
        if let Some(rx) = &self.input_receiver {
            while let Ok(msg) = rx.try_recv() {
                env.set_value_at(&msg.variable, msg.value, msg.timestamp);
            }
        }
    }

    pub fn execute_plan(&mut self, env: &mut Environment, plan_name: &str) {
        env.log(format!("Starting plan: {}", plan_name));

        // Hacky clone for prototype
        let defs = env.definitions.clone();

        if let Some(crate::ast::Definition::Plan(plan_def)) = defs.get(plan_name) {
            println!(
                "DEBUG: Found plan '{}', blocks: {}",
                plan_name,
                plan_def.blocks.len()
            );

            // Drain initial inputs (configuration)
            self.drain_inputs(env);

            let mut events = BinaryHeap::new();
            let start_time = env.now;
            
            let end_time = match self.mode {
                ExecutionMode::Simulation { duration, .. } => duration.map(|d| start_time + d),
                _ => None,
            };

            // Initial Scheduling
            for block in &plan_def.blocks {
                match block {
                    crate::ast::PlanBlock::DuringPlan(stmts) => {
                        println!(
                            "DEBUG: Executing DuringPlan block with {} stmts",
                            stmts.len()
                        );
                        self.execute_block(env, stmts);
                    }
                    crate::ast::PlanBlock::Trigger(block) => {
                        match &block.trigger {
                            crate::ast::Trigger::Periodic {
                                interval,
                                interval_unit,
                                duration,
                            } => {
                                let interval_secs = match interval_unit {
                                    Unit::Second => *interval,
                                    Unit::Minute => *interval * 60.0,
                                    Unit::Hour => *interval * 3600.0,
                                    Unit::Day => *interval * 86400.0,
                                    Unit::Week => *interval * 604800.0,
                                    Unit::Month => *interval * 2592000.0,
                                    Unit::Year => *interval * 31536000.0,
                                    _ => *interval,
                                };

                                let max_dur = if let Some((d_val, d_unit)) = duration {
                                    let d_secs = match d_unit {
                                        Unit::Second => *d_val,
                                        Unit::Minute => *d_val * 60.0,
                                        Unit::Hour => *d_val * 3600.0,
                                        Unit::Day => *d_val * 86400.0,
                                        Unit::Week => *d_val * 604800.0,
                                        Unit::Month => *d_val * 2592000.0,
                                        Unit::Year => *d_val * 31536000.0,
                                        _ => *d_val,
                                    };
                                    Some(chrono::Duration::milliseconds((d_secs * 1000.0) as i64))
                                } else {
                                    None
                                };

                                // Schedule first run immediately or after interval?
                                // "every 1 day" usually triggers after 1 day.
                                // Assuming after interval.
                                let first_run = start_time
                                    + chrono::Duration::milliseconds(
                                        (interval_secs * 1000.0) as i64,
                                    );

                                events.push(ScheduledEvent {
                                    time: first_run,
                                    kind: EventKind::Periodic {
                                        block: block.clone(),
                                        iteration: 0,
                                        interval_secs,
                                        max_duration: max_dur,
                                    },
                                });
                            }
                            crate::ast::Trigger::StartOf(target) => {
                                if target == plan_name {
                                    println!("DEBUG: Scheduling immediate StartOf trigger for {}", target);
                                    events.push(ScheduledEvent {
                                        time: start_time,
                                        kind: EventKind::Periodic {
                                            block: block.clone(),
                                            iteration: 0,
                                            interval_secs: 0.0, // 0.0 means one-shot/no repeat
                                            max_duration: None,
                                        },
                                    });
                                } else if let Some(def) = defs.get(target) {
                                    if let Some((_next, _)) = Scheduler::next_occurrence(def, start_time) {
                                        // ...
                                        println!(
                                            "DEBUG: StartOf in TriggerBlock for other plans not fully supported yet."
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    crate::ast::PlanBlock::Event(block) => {
                        if let crate::ast::Trigger::StartOf(target) = &block.trigger {
                            if let Some(def) = defs.get(target) {
                                if let Some((next, _)) = Scheduler::next_occurrence(def, start_time)
                                {
                                    println!("DEBUG: Scheduled '{}' at {}", block.name, next);
                                    events.push(ScheduledEvent {
                                        time: next,
                                        kind: EventKind::StartOf {
                                            block: block.clone(),
                                            period_name: target.clone(),
                                        },
                                    });
                                } else {
                                    println!(
                                        "DEBUG: Could not schedule '{}', no valid time found.",
                                        block.name
                                    );
                                }
                            } else {
                                println!("DEBUG: Definition for '{}' not found.", target);
                            }
                        }
                    }
                }
            }

            // Event Loop
            while let Some(event) = events.pop() {
                if self.stop_signal.load(AtomicOrdering::Relaxed) {
                     println!("DEBUG: Execution stopped by signal.");
                     break;
                }

                let now = event.time;

                if let Some(limit) = end_time {
                    if now > limit {
                        println!("DEBUG: Simulation time limit reached.");
                        break;
                    }
                }

                // Advance time
                // Advance time
                match self.mode {
                    ExecutionMode::RealTime => {
                        if now > env.now {
                            let diff = (now - env.now)
                                .to_std()
                                .unwrap_or(std::time::Duration::from_secs(0));
                            std::thread::sleep(diff);
                        }
                    }
                    ExecutionMode::Simulation { speed_factor, .. } => {
                        if let Some(factor) = speed_factor {
                             if now > env.now {
                                let diff = (now - env.now)
                                    .to_std()
                                    .unwrap_or(std::time::Duration::from_secs(0));
                                // Sleep with speed factor. factor 1.0 = real time. 10.0 = 10x faster (sleep 1/10th)
                                // factor > 0
                                let duration_secs = diff.as_secs_f64();
                                let sleep_secs = duration_secs / factor;
                                std::thread::sleep(std::time::Duration::from_secs_f64(sleep_secs));
                             }
                        }
                        // If speed_factor is None, instant jump (timelapse/instant)
                    }
                }
                env.set_time(now);

                // Execute
                match event.kind {
                    EventKind::Periodic {
                        block,
                        iteration,
                        interval_secs,
                        max_duration,
                    } => {
                        self.execute_block(env, &block.statements);

                        // Reschedule
                        let next_time =
                            now + chrono::Duration::milliseconds((interval_secs * 1000.0) as i64);
                        let elapsed = next_time - start_time;

                        let stop = if let Some(max) = max_duration {
                            elapsed > max
                        } else {
                            false
                        };

                        if !stop && interval_secs > 0.0 {
                            events.push(ScheduledEvent {
                                time: next_time,
                                kind: EventKind::Periodic {
                                    block,
                                    iteration: iteration + 1,
                                    interval_secs,
                                    max_duration,
                                },
                            });
                        }
                    }
                    EventKind::StartOf { block, period_name } => {
                        println!("DEBUG: Executing Event '{}'", block.name);
                        self.execute_block(env, &block.statements);

                        // Reschedule next occurrence
                        if let Some(def) = defs.get(&period_name)
                        {
                            if let Some((next, _)) = Scheduler::next_occurrence(def, now) {
                                // Ensure we don't schedule same time again loop
                                if next > now {
                                    events.push(ScheduledEvent {
                                        time: next,
                                        kind: EventKind::StartOf { block, period_name },
                                    });
                                }
                            }
                        }
                    }
                }
            }
        } else {
            println!(
                "DEBUG: Plan '{}' not found. Available: {:?}",
                plan_name,
                defs.keys()
            );
            env.log(format!("Plan not found: {}", plan_name));
        }
    }

    pub fn execute_block(&mut self, env: &mut Environment, stmts: &Block) {
        for stmt in stmts {
            self.execute_statement(env, stmt);
        }
    }

    pub fn execute_statement(&mut self, env: &mut Environment, stmt: &Statement) {
        if stmt.line > 0 {
            if let Some(cb) = &self.on_step {
                cb(stmt.line);
            }
        }

        match &stmt.kind {
            StatementKind::Action(action) => self.execute_action(env, action),
            StatementKind::Assignment(assign) => {
                let val = Evaluator::evaluate(env, &assign.expression);
                env.set_value(&assign.target, val);
            }
            StatementKind::Command(cmd) => {
                env.log(format!("Command: {}", cmd));
            }
            StatementKind::Constraint(expr, op, sel) => {
                 // For now, constraints are declarative and enforced by validator or specific logic (like ValidValues)
                 // Just log for debugging
                 env.log(format!("Constraint: {:?} {} {:?}", expr, op, sel));
            }
            StatementKind::NoOp => {}
            StatementKind::EventProgression(target_name, cases) => {
                let val = if let Some(v) = env.get_value(target_name) {
                    v.clone()
                } else {
                    env.log(format!(
                        "Warning: Variable '{}' not found for assessment",
                        target_name
                    ));
                    return;
                };

                // println!("DEBUG: Assessing '{}' (val={:?}) against {} cases", target_name, val, cases.len());

                for case in cases {
                    let selector = &case.condition;
                    let is_match = Evaluator::check_condition(env, selector, &val);

                    // println!("DEBUG: Checking selector {:?} match={}", selector, is_match);

                    if is_match {
                        // println!("DEBUG: Case matched, executing block len={}", case.block.len());
                        self.execute_block(env, &case.block);
                        break;
                    }
                }
            }
            StatementKind::Conditional(cond) => {
                self.execute_conditional(env, cond);
            }
            StatementKind::ContextBlock(cb) => {
                // Build evaluation context
                let mut timeframe = None;
                for item in &cb.items {
                    if let crate::ast::ContextItem::Timeframe(ts) = item {
                        timeframe = Some(ts.clone());
                    }
                }
                let ctx = crate::runtime::environment::EvaluationContext { timeframe };

                env.push_context(ctx);
                self.execute_block(env, &cb.statements);
                env.pop_context();
            }

        }
    }

    fn execute_conditional(&mut self, env: &mut Environment, cond_stmt: &crate::ast::Conditional) {
        let val = match &cond_stmt.condition {
            crate::ast::ConditionalTarget::Expression(expr) => Evaluator::evaluate(env, expr),
            crate::ast::ConditionalTarget::Confidence(_ident) => {
                // Stub: return high confidence
                crate::domain::RuntimeValue::Number(100.0)
            }
        };

        // Context-aware resolution
        let val = if let crate::domain::RuntimeValue::String(s) = &val {
            if let Some(resolved) = env.get_value(s) {
                resolved.clone()
            } else {
                val
            }
        } else {
            val
        };

        for case in &cond_stmt.cases {
            let selector = &case.condition;
            let is_match = Evaluator::check_condition(env, selector, &val);

            if is_match {
                self.execute_block(env, &case.block);
                break;
            }
        }
    }

    fn emit_log(&self, msg: String, event_type: crate::domain::EventType, timestamp: chrono::DateTime<chrono::Utc>) {
        if let Some(cb) = &self.on_log {
            cb(msg, event_type, timestamp);
        }
    }

    pub fn set_ask_callback(&mut self, cb: Box<dyn Fn(crate::domain::AskRequest) + Send>) {
        self.on_ask = Some(cb);
    }

    fn execute_action(&mut self, env: &mut Environment, action: &Action) {
        match action {
            Action::ShowMessage(parts, _) => {
                let mut full_msg = String::new();
                for part in parts {
                    let val = Evaluator::evaluate(env, part);
                    let s = val.to_string();
                    full_msg.push_str(&s);
                }
                let msg = full_msg;
                // env.log is internal debug log, keep as is
                env.log(msg.clone());
                // Emit log with current env time
                self.emit_log(msg, crate::domain::EventType::Message, env.now);
            }
            Action::AskQuestion(q, _) => {


                // Build AskRequest
                let mut style = crate::domain::QuestionStyle::Text;
                let mut options = Vec::new();
                let mut range = None;
                let mut question_text = q.clone();
                let variable_name = q.clone();
                // let mut is_defined_var = false; // Unused after removing auto-simulation

                // Lookup definition
                // Hack: We need to access definitions. Environment borrow?
                // We have `env`.
                // Clone needed to avoid immutable borrow of env while mutating env later (log)
                let val_def_opt = if let Some(crate::ast::Definition::Value(val_def)) = env.definitions.get(q) {
                    Some(val_def.clone())
                } else {
                    env.log(format!("DEBUG: Definition for '{}' not found. Available: {:?}", q, env.definitions.keys()));
                    None
                };

                if let Some(val_def) = &val_def_opt {
                    // is_defined_var = true;
                    // Determine style/options from definition
                    for prop in &val_def.properties {
                        match prop {
                            crate::ast::Property::ValidValues(stmts) => {
                                // Extract valid values as options
                                for stmt in stmts {
                                    if let StatementKind::EventProgression(_, cases) = &stmt.kind {
                                        for case in cases {
                                            match &case.condition {
                                                crate::ast::RangeSelector::Equals(expr) => {
                                                     let v = Evaluator::evaluate(env, expr);
                                                     // If string or enum, add to options
                                                     if let crate::domain::RuntimeValue::String(s) = &v {
                                                         options.push(s.clone());
                                                     } else if let crate::domain::RuntimeValue::Enumeration(s) = &v {
                                                         options.push(s.clone());
                                                     } else {
                                                         options.push(v.to_string());
                                                     }
                                                }
                                                crate::ast::RangeSelector::Range(min, max) => {
                                                    let min_v = Evaluator::evaluate(env, min);
                                                    let max_v = Evaluator::evaluate(env, max);
                                                     if let (crate::domain::RuntimeValue::Number(mn), crate::domain::RuntimeValue::Number(mx)) = (min_v, max_v) {
                                                         range = Some((mn, mx));
                                                     }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                            crate::ast::Property::Question(action) => {
                                 // Check for explicit question text in definition
                                 if let Action::AskQuestion(text, _) = action {
                                     // If the definition says: ask "How severe...?"
                                     // We use that text instead of variable name.
                                     // But only if `q` was the variable name.
                                     // The `q` here matches `val_def.name`.
                                     if !text.is_empty() { 
                                         // Check if text is string literal or another var?
                                         // Parsing logic: subject.
                                         // Ideally we check if text looks like a question or string.
                                         question_text = text.clone();
                                     }
                                 }
                             }
                            crate::ast::Property::Reuse(val, unit) => {
                                // Check if we have a valid value in history
                                let last_entry_data = if let Some(history) = env.get_history(q) {
                                    history.last().map(|entry| (entry.value.clone(), entry.timestamp))
                                } else {
                                    None
                                };

                                if let Some((val_data, timestamp)) = last_entry_data {
                                     // Calculate age
                                     let age = env.now - timestamp;
                                     let age_secs = age.num_seconds() as f64;
                                     
                                     let reuse_secs = match unit {
                                         Unit::Second => *val,
                                         Unit::Minute => *val * 60.0,
                                         Unit::Hour => *val * 3600.0,
                                         Unit::Day => *val * 86400.0,
                                         Unit::Week => *val * 604800.0,
                                         Unit::Month => *val * 2592000.0,
                                         Unit::Year => *val * 31536000.0,
                                         _ => *val, // Fallback
                                     };
                                     
                                     
                                     if age_secs < reuse_secs {
                                         // Ensure value is not NotEnoughData
                                         if let crate::domain::RuntimeValue::NotEnoughData = val_data {
                                             // If value is NotEnoughData, it's not valid to reuse, so ask.
                                         } else {
                                             env.log(format!(
                                                 "Skipping question '{}', existing value is fresh (Age: {}s < Reuse: {}s)",
                                                 q, age_secs, reuse_secs
                                             ));
                                             return; // Skip asking!
                                         }
                                     }
                                }
                            }
                            _ => {}
                        }
                    }

                    // Infer Style
                    if !options.is_empty() {
                        style = crate::domain::QuestionStyle::Selection;
                    } else if range.is_some() {
                        style = crate::domain::QuestionStyle::Numeric;
                        // TODO: Check for VAS specifically if I could parse it
                    } else if val_def.value_type == crate::domain::ValueType::Enumeration {
                         // Even if options empty (?), it's selection-like
                         style = crate::domain::QuestionStyle::Selection;
                    }
                }


                // Emit log with actual question text
                let msg = format!("Action: Ask Question '{}'", question_text);

                env.log(msg.clone());
                self.emit_log(question_text.clone(), crate::domain::EventType::Question, env.now);

                let mut validation_mode = None;
                let mut validation_timeout = None;

                // Inspect Action Block for Validation
                if let Action::AskQuestion(_, Some(block)) = action {
                     for stmt in block {
                         if let StatementKind::Action(Action::ValidateAnswer(mode, timeout)) = &stmt.kind {
                             validation_mode = Some(mode.clone());
                             if let Some((val, unit)) = timeout {
                                 let secs = match unit {
                                     Unit::Second => *val,
                                     Unit::Minute => *val * 60.0,
                                     _ => *val, // Default logic
                                 };
                                 validation_timeout = Some(secs as i64);
                             }
                         }
                     }
                }

                // Look for block in definition if not in action?
                // The current Action extraction in executor.rs (line 453) matches keys.
                // We need to match the action correctly at line 453.
                // The match currently is: Action::AskQuestion(q, _)
                // We should capture the second arg: Action::AskQuestion(q, block_opt)
                
                // Let's refine the replacement to be correct in context.
                // We need to look at how we matched `action` in `execute_action`.
                // It was `match action { Action::AskQuestion(q, _) => ...`
                
                // We need to access `block_opt` from `action`. 
                // Since we are inside the match branch, we can't easily re-match `action` binding unless we use it.
                // But `action` is `&Action`.
                
                // Let's execute logic.
                
                if let Action::AskQuestion(_, Some(stmts)) = action {
                     for stmt in stmts {
                          if let StatementKind::Action(Action::ValidateAnswer(mode, timeout)) = &stmt.kind {
                              validation_mode = Some(mode.clone());
                              if let Some((val, unit)) = timeout {
                                   let secs = match unit {
                                     Unit::Second => *val,
                                     Unit::Minute => *val * 60.0,
                                     _ => *val,
                                   };
                                   validation_timeout = Some(secs as i64);
                              }
                          }
                     }
                }

                // ... (existing logic) ...

                // Calculate valid_after timestamp from Reuse property if present
                let mut valid_after = None;
                if let Some(val_def) = &val_def_opt {
                    for prop in &val_def.properties {
                       if let crate::ast::Property::Reuse(val, unit) = prop {
                            let reuse_secs = match unit {
                                Unit::Second => *val,
                                Unit::Minute => *val * 60.0,
                                Unit::Hour => *val * 3600.0,
                                Unit::Day => *val * 86400.0,
                                Unit::Week => *val * 604800.0,
                                Unit::Month => *val * 2592000.0,
                                Unit::Year => *val * 31536000.0,
                                _ => *val,
                            };
                            let reuse_dur = chrono::Duration::milliseconds((reuse_secs * 1000.0) as i64);
                            valid_after = Some((env.now - reuse_dur).timestamp_millis());
                       }
                    }
                }

                // Fire Callback
                if let Some(cb) = &self.on_ask {
                    let req = crate::domain::AskRequest {
                        variable_name: variable_name.clone(),
                        question_text: question_text.clone(),
                        style: style.clone(),
                        options: options.clone(),
                        range,
                        validation_mode,
                        validation_timeout,
                        timestamp: env.now.timestamp_millis(),
                        valid_after,
                    };
                    cb(req);
                }

                // Wait for answer via Channel
                // We take the receiver out to satisfy borrow checker when calling check_triggers (which needs &mut self)
                let rx_opt = self.input_receiver.take();
                
                if let Some(rx) = &rx_opt {
                    env.log(format!("Waiting for answer to '{}'...", q));
                    loop {
                        match rx.recv() {
                            Ok(msg) => {
                                if msg.variable == *q {
                                    env.set_value(&msg.variable, msg.value.clone());
                                    self.emit_log(format!("Received Answer: {:?}", msg.value), crate::domain::EventType::Answer, env.now);
                                    
                                    // Trigger Event Checks - Now safe because we don't hold valid reference to self.input_receiver
                                    // Use the message timestamp to preserve data validity from session
                                    env.set_value_at(&msg.variable, msg.value.clone(), msg.timestamp);
                                    self.check_triggers(env, &msg.variable);
                                    
                                    break;
                                } else {
                                    // Handle out-of-order updates (e.g. setting other variables)
                                    env.set_value_at(&msg.variable, msg.value.clone(), msg.timestamp);
                                    self.check_triggers(env, &msg.variable);
                                }
                            }
                            Err(_) => {
                                env.log("Input channel disconnected.".to_string());
                                break; 
                            }
                        }
                    }
                } else {
                    env.log("Warning: No input receiver configured. Skipping wait for answer.".to_string());
                }
                
                // Put receiver back
                self.input_receiver = rx_opt;
            }
            Action::SendInfo(msg_text, vars) => {
                let vals: Vec<String> = vars
                    .iter()
                    .map(|e| format!("{:?}", Evaluator::evaluate(env, e)))
                    .collect();
                let msg = format!("Action: Send Info '{}' with values: {:?}", msg_text, vals);
                env.log(msg.clone());
                self.emit_log(msg, crate::domain::EventType::Log, env.now);
            }
            Action::ListenFor(val) => {
                let msg = format!("Action: Listen For '{}'", val);
                env.log(msg.clone());
                self.emit_log(msg, crate::domain::EventType::Log, env.now);
            }
            _ => {}
        }
    }

    fn check_triggers(&mut self, env: &mut Environment, changed_var: &str) {
        let mut blocks_to_run = Vec::new();
        
        for def in env.definitions.values() {
             if let crate::ast::Definition::Plan(plan_def) = def {
                 for block in &plan_def.blocks {
                     match block {
                         crate::ast::PlanBlock::Trigger(trigger_block) => {
                             match &trigger_block.trigger {
                                 crate::ast::Trigger::ChangeOf(var) => {
                                     if var == changed_var {
                                         blocks_to_run.push(trigger_block.statements.clone());
                                     }
                                 }
                                 _ => {}
                             }
                         },
                         crate::ast::PlanBlock::Event(event_block) => {
                             match &event_block.trigger {
                                 crate::ast::Trigger::ChangeOf(var) => {
                                     if var == changed_var {
                                         blocks_to_run.push(event_block.statements.clone());
                                     }
                                 }
                                 _ => {}
                             }
                         },
                         _ => {}
                     }
                 }
             }
        }
        
        for block in blocks_to_run {
            self.execute_block(env, &block);
        }
    }
}
