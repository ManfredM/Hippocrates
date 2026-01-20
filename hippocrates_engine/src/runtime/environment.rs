use crate::ast::{Definition, Plan, RangeSelector};
use crate::domain::{RuntimeValue, ValueInstance};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use std::fmt;

pub struct Environment {
    pub values: HashMap<String, Vec<ValueInstance>>,
    pub definitions: HashMap<String, Definition>,
    pub now: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub output_log: Vec<String>,
    pub output_handler: Option<Arc<dyn Fn(String) + Send + Sync>>,
    // Use RwLock for interior mutability so Evaluator can push context with &Environment
    pub context_stack: std::sync::RwLock<Vec<EvaluationContext>>,
    pub unit_map: HashMap<String, crate::domain::Unit>,
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
            .field("start_time", &self.start_time)
            .field("output_log", &self.output_log)
            .field("output_handler", &"fn(...)")
            .field("unit_map", &self.unit_map)
            .finish()
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            definitions: HashMap::new(),
            now: Utc::now(),
            start_time: Utc::now(),
            output_log: Vec::new(),
            output_handler: None,
            context_stack: std::sync::RwLock::new(Vec::new()),
            unit_map: HashMap::new(),
            // TODO: Pre-populate standard units?
            // If we want standard units to work regardless of definition, we might need a hardcoded list here.
            // But the parser maps them to Enum variants like Unit::Meter.
            // BUT if the user writes "meter", the parser produces Unit::Meter.
            // If the user writes "meters", the parser now produces Unit::Custom("meters") (implicit 's' removed).
            // So we MUST map "meters" -> Unit::Meter if we want standard behavior.
            // Or we force users/us to update stdlib definitions.
            // For now, let's leave it empty and assume std units are handled by parser logic (which I modified to NOT string strip).
            // Wait! Parser logic for STANDARD units was:
            // "meter" | "meters" => Ok(Unit::Meter).
            // This is untouched!
            // My change only affected the `_ =>` fallthrough for CUSTOM units.
            // So standard units still work fine with plurals hardcoded in parser.
        }
    }

    pub fn push_context(&self, ctx: EvaluationContext) {
        self.context_stack.write().unwrap().push(ctx);
    }

    pub fn pop_context(&self) {
        self.context_stack.write().unwrap().pop();
    }

    pub fn active_context(&self) -> Option<EvaluationContext> {
        self.context_stack.read().unwrap().last().cloned()
    }

    pub fn set_start_time(&mut self, time: DateTime<Utc>) {
        self.start_time = time;
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
                    // Use standard epoch for defaults so they appear "old" in history
                    // compared to any restored or real data.
                    let epoch = chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap_or(Utc::now());
                    self.set_value_at(&v.name, default, epoch);
                    v.name.clone()
                }
                Definition::Period(p) => p.name.clone(),
                Definition::Plan(p) => p.name.clone(),
                Definition::Drug(d) => d.name.clone(),
                Definition::Addressee(a) => a.name.clone(),
                Definition::Context(_c) => "context".to_string(),
                Definition::Unit(u) => {
                    let canonical = crate::domain::Unit::Custom(u.name.clone());
                     self.unit_map.insert(u.name.clone(), canonical.clone());
                     for s in &u.singulars {
                         self.unit_map.insert(s.clone(), canonical.clone());
                     }
                     for p in &u.plurals {
                         self.unit_map.insert(p.clone(), canonical.clone());
                     }
                     for a in &u.abbreviations {
                         self.unit_map.insert(a.clone(), canonical.clone());
                     }
                    u.name.clone()
                }
            };
            self.definitions.insert(name, def);
        }
    }
    
    fn default_value_for(&self, vt: &crate::domain::ValueType) -> RuntimeValue {
        use crate::domain::{RuntimeValue, ValueType};
        match vt {
            ValueType::Number => RuntimeValue::NotEnoughData,
            ValueType::Enumeration => RuntimeValue::NotEnoughData,
            // Initialize others as needed
            _ => RuntimeValue::Void,
        }
    }

    pub fn set_value(&mut self, name: &str, value: RuntimeValue) {
        self.set_value_at(name, value, self.now);
    }

    pub fn set_value_at(&mut self, name: &str, value: RuntimeValue, timestamp: DateTime<Utc>) {
        let instance = ValueInstance { value, timestamp };
        let history = self.values.entry(name.to_string()).or_insert_with(Vec::new);
        history.push(instance);
        // Sort by timestamp to ensure history is ordered?
        // Ideally yes, but expensive if frequent. For now, assume append-only or sort on read if needed.
        // Let's sort on insert to be safe for analysis.
        history.sort_by_key(|v| v.timestamp);
    }

    pub fn get_value(&self, name: &str) -> Option<&RuntimeValue> {
        self.values
            .get(name)
            .and_then(|history| history.last())
            .map(|instance| &instance.value)
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
