use crate::domain::{RuntimeValue, ValueInstance};
use crate::ast::{Plan, Definition, RangeSelector};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use std::fmt;

pub struct Environment {
    pub values: HashMap<String, Vec<ValueInstance>>,
    pub definitions: HashMap<String, Definition>,
    pub now: DateTime<Utc>,
    pub output_log: Vec<String>,
    pub output_handler: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub context_stack: Vec<EvaluationContext>,
}

#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub timeframe: Option<RangeSelector>,
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
         .field("values", &self.values)
         .field("definitions", &self.definitions)
         .field("now", &self.now)
         .field("output_log", &self.output_log)
         .field("output_handler", &"fn(...)")
         .finish()
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            definitions: HashMap::new(),
            now: Utc::now(),
            output_log: Vec::new(),
            output_handler: None,
            context_stack: Vec::new(),
        }
    }

    pub fn push_context(&mut self, ctx: EvaluationContext) {
        self.context_stack.push(ctx);
    }

    pub fn pop_context(&mut self) {
        self.context_stack.pop();
    }

    pub fn active_context(&self) -> Option<&EvaluationContext> {
        self.context_stack.last()
    }

    pub fn set_output_handler(&mut self, handler: Arc<dyn Fn(String) + Send + Sync>) {
        self.output_handler = Some(handler);
    }

    pub fn set_time(&mut self, time: DateTime<Utc>) {
        self.now = time;
    }

    pub fn load_plan(&mut self, plan: Plan) {
        for def in plan.definitions {
            let name = match &def {
                Definition::Value(v) => {
                    let default = self.default_value_for(&v.value_type);
                    self.set_value(&v.name, default);
                    v.name.clone()
                },
                Definition::Period(p) => p.name.clone(),
                Definition::Plan(p) => p.name.clone(),
                Definition::Drug(d) => d.name.clone(),
                Definition::Addressee(a) => a.name.clone(),
                Definition::Context(_c) => "context".to_string(),
            };
            self.definitions.insert(name, def);
        }
    }

    fn default_value_for(&self, vt: &crate::domain::ValueType) -> RuntimeValue {
        use crate::domain::{ValueType, RuntimeValue};
        match vt {
            ValueType::Number => RuntimeValue::Number(0.0),
            ValueType::Enumeration => RuntimeValue::Enumeration("".to_string()),
            // Initialize others as needed
            _ => RuntimeValue::Void,
        }
    }

    pub fn set_value(&mut self, name: &str, value: RuntimeValue) {
        self.set_value_at(name, value, self.now);
    }

    pub fn set_value_at(&mut self, name: &str, value: RuntimeValue, timestamp: DateTime<Utc>) {
        let instance = ValueInstance {
            value,
            timestamp,
        };
        let history = self.values.entry(name.to_string()).or_insert_with(Vec::new);
        history.push(instance);
        // Sort by timestamp to ensure history is ordered?
        // Ideally yes, but expensive if frequent. For now, assume append-only or sort on read if needed.
        // Let's sort on insert to be safe for analysis.
        history.sort_by_key(|v| v.timestamp);
    }

    pub fn get_value(&self, name: &str) -> Option<&RuntimeValue> {
        self.values.get(name).and_then(|history| history.last()).map(|instance| &instance.value)
    }

    pub fn get_history(&self, name: &str) -> Option<&Vec<ValueInstance>> {
        self.values.get(name)
    }

    pub fn log(&mut self, message: String) {
        if let Some(handler) = &self.output_handler {
            // println!("DEBUG: Calling output handler");
            handler(message.clone());
        } else {
             println!("DEBUG: No output handler set!");
        }
        self.output_log.push(message);
    }
}
