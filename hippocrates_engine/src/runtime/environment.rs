use crate::ast::{
    AssessmentCase, Definition, Expression, Literal, Plan, Property, RangeSelector, Statement,
    StatementKind,
};
use crate::domain::{RuntimeValue, ValueInstance};
use chrono::{Utc, NaiveDateTime};
use std::collections::HashMap;
use std::sync::Arc;

use std::fmt;

pub struct Environment {
    pub values: HashMap<String, Vec<ValueInstance>>,
    pub definitions: HashMap<String, Definition>,
    pub now: NaiveDateTime,
    pub start_time: NaiveDateTime,
    pub output_log: Vec<String>,
    pub output_handler: Option<Arc<dyn Fn(String) + Send + Sync>>,
    // Use RwLock for interior mutability so Evaluator can push context with &Environment
    pub context_stack: std::sync::RwLock<Vec<EvaluationContext>>,
    pub unit_map: HashMap<String, crate::domain::Unit>,
    pub audit_log: Vec<crate::domain::AuditEntry>,
}

#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub timeframe: Option<RangeSelector>,
    pub period: Option<String>,
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

impl EvaluationContext {
    pub fn from_constraints(
        defs: &HashMap<String, Definition>,
        constraints: &[(String, RangeSelector)],
    ) -> Self {
        let mut timeframe = None;
        let mut period = None;

        for (op, selector) in constraints {
            match op.as_str() {
                "during" => {
                    if let Some(name) = Self::period_name_from_selector(defs, selector) {
                        period = Some(name);
                    } else {
                        timeframe = Some(selector.clone());
                    }
                }
                "is" | "after" => {
                    timeframe = Some(selector.clone());
                }
                _ => {}
            }
        }

        EvaluationContext { timeframe, period }
    }

    fn period_name_from_selector(
        defs: &HashMap<String, Definition>,
        selector: &RangeSelector,
    ) -> Option<String> {
        let name = match selector {
            RangeSelector::Equals(expr) => match expr {
                Expression::Variable(name) => name.clone(),
                Expression::Literal(Literal::String(name)) => name.clone(),
                _ => return None,
            },
            _ => return None,
        };

        match defs.get(&name) {
            Some(Definition::Period(_)) => Some(name),
            _ => None,
        }
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            definitions: HashMap::new(),
            now: Utc::now().naive_utc(),
            start_time: Utc::now().naive_utc(),
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
            audit_log: Vec::new(),
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

    pub fn set_start_time(&mut self, time: NaiveDateTime) {
        self.start_time = time;
    }

    pub fn set_output_handler(&mut self, handler: Arc<dyn Fn(String) + Send + Sync>) {
        self.output_handler = Some(handler);
    }

    pub fn set_time(&mut self, time: NaiveDateTime) {
        self.now = time;
    }

    pub fn load_plan(&mut self, plan: Plan) {
        for def in plan.definitions {
            let name = match &def {
                Definition::Value(v) => {
                    let default = self.default_value_for(&v.value_type);
                    // Use standard epoch for defaults so they appear "old" in history
                    // compared to any restored or real data.
                    let epoch = chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap_or(Utc::now()).naive_utc();
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

    pub fn expected_unit_for_value(&self, name: &str) -> Option<crate::domain::Unit> {
        let normalized = super::normalize_identifier(name);
        let def = self.definitions.get(&normalized)?;
        let value_def = match def {
            Definition::Value(v) => v,
            _ => return None,
        };

        if !matches!(value_def.value_type, crate::domain::ValueType::Number) {
            return None;
        }

        for prop in &value_def.properties {
            if let Property::Unit(unit) = prop {
                return Some(canonicalize_unit(unit, &self.unit_map));
            }
        }

        for prop in &value_def.properties {
            match prop {
                Property::ValidValues(stmts) => {
                    if let Some(unit) = unit_from_statements(stmts, &self.unit_map) {
                        return Some(unit);
                    }
                }
                Property::Meaning(cases) => {
                    if let Some(unit) = unit_from_cases(cases, &self.unit_map) {
                        return Some(unit);
                    }
                }
                _ => {}
            }
        }

        None
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

    pub fn set_value_at(&mut self, name: &str, value: RuntimeValue, timestamp: NaiveDateTime) {
        let normalized = super::normalize_identifier(name);
        let value_display = value.to_string();
        let timestamp_display = format_timestamp(timestamp);
        let instance = ValueInstance { value, timestamp };
        let history = self.values.entry(normalized).or_insert_with(Vec::new);
        history.push(instance);
        // Sort by timestamp to ensure history is ordered?
        // Ideally yes, but expensive if frequent. For now, assume append-only or sort on read if needed.
        // Let's sort on insert to be safe for analysis.
        history.sort_by_key(|v| v.timestamp);
        println!(
            "DEBUG: Created value instance {} at {} => {}",
            super::format_identifier(name),
            timestamp_display,
            value_display
        );
    }

    pub fn get_value(&self, name: &str) -> Option<&RuntimeValue> {
        let normalized = super::normalize_identifier(name);
        self.values
            .get(&normalized)
            .and_then(|history| history.last())
            .map(|instance| &instance.value)
    }

    pub fn get_history(&self, name: &str) -> Option<&Vec<ValueInstance>> {
        let normalized = super::normalize_identifier(name);
        self.values.get(&normalized)
    }

    pub fn log(&mut self, message: String) {
        if let Some(handler) = &self.output_handler {
            let formatted = format!("[{}] {}", format_timestamp(self.now), message);
            handler(formatted);
        } else {
            println!("DEBUG: No output handler set!");
        }
        self.output_log.push(message);
    }

    pub fn log_audit(&mut self, event_type: crate::domain::EventType, details: String, context: Option<String>) {
        let entry = crate::domain::AuditEntry {
            timestamp: self.now,
            event_type,
            details,
            context,
        };
        self.audit_log.push(entry);
    }
}

fn canonicalize_unit(
    unit: &crate::domain::Unit,
    unit_map: &HashMap<String, crate::domain::Unit>,
) -> crate::domain::Unit {
    match unit {
        crate::domain::Unit::Custom(name) => unit_map.get(name).cloned().unwrap_or_else(|| {
            crate::domain::Unit::Custom(name.clone())
        }),
        _ => unit.clone(),
    }
}

fn unit_from_statements(
    stmts: &[Statement],
    unit_map: &HashMap<String, crate::domain::Unit>,
) -> Option<crate::domain::Unit> {
    for stmt in stmts {
        let unit = match &stmt.kind {
            StatementKind::Constraint(_, _, selector) => unit_from_selector(selector, unit_map),
            StatementKind::EventProgression(_, cases) => unit_from_cases(cases, unit_map),
            _ => None,
        };
        if unit.is_some() {
            return unit;
        }
    }
    None
}

fn unit_from_cases(
    cases: &[AssessmentCase],
    unit_map: &HashMap<String, crate::domain::Unit>,
) -> Option<crate::domain::Unit> {
    for case in cases {
        if let Some(unit) = unit_from_selector(&case.condition, unit_map) {
            return Some(unit);
        }
    }
    None
}

fn unit_from_selector(
    selector: &RangeSelector,
    unit_map: &HashMap<String, crate::domain::Unit>,
) -> Option<crate::domain::Unit> {
    match selector {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
            unit_from_expression(min, unit_map)
                .or_else(|| unit_from_expression(max, unit_map))
        }
        RangeSelector::Equals(expr) => unit_from_expression(expr, unit_map),
        RangeSelector::List(items) => {
            for item in items {
                if let Some(unit) = unit_from_expression(item, unit_map) {
                    return Some(unit);
                }
            }
            None
        }
        _ => None,
    }
}

fn unit_from_expression(
    expr: &Expression,
    unit_map: &HashMap<String, crate::domain::Unit>,
) -> Option<crate::domain::Unit> {
    match expr {
        Expression::Literal(Literal::Quantity(_, unit, _)) => {
            Some(canonicalize_unit(unit, unit_map))
        }
        _ => None,
    }
}

fn format_timestamp(timestamp: NaiveDateTime) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
}
