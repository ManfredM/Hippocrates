use crate::ast::{Definition, Expression, Literal, Property, RangeSelector, StatementKind, ValueDef};
use crate::domain::{RuntimeValue, ValueType};
use crate::runtime::normalize_identifier;
use std::collections::HashMap;

#[derive(Copy, Clone)]
struct PrecisionInfo {
    decimals: Option<usize>,
}

impl PrecisionInfo {
    fn new() -> Self {
        PrecisionInfo { decimals: None }
    }
}

pub fn validate_input_value(
    definitions: &HashMap<String, Definition>,
    variable: &str,
    value: &RuntimeValue,
) -> Result<(), String> {
    let normalized = normalize_identifier(variable);
    let def = match definitions.get(&normalized) {
        Some(def) => def,
        None => return Ok(()),
    };

    let value_def = match def {
        Definition::Value(v) => v,
        _ => return Ok(()),
    };

    if !matches!(value_def.value_type, ValueType::Number) {
        return Ok(());
    }

    let decimals = precision_for_value(value_def)
        .map_err(|msg| format!("precision mismatch for '{}': {}", value_def.name, msg))?;
    if let Some(decimals) = decimals {
        if !value_has_precision(value, decimals) {
            let label = if decimals == 1 {
                "decimal place"
            } else {
                "decimal places"
            };
            return Err(format!("value must use {} {}", decimals, label));
        }
    }

    Ok(())
}

pub(crate) fn precision_for_value(value_def: &ValueDef) -> Result<Option<usize>, String> {
    let mut precision = PrecisionInfo::new();

    for prop in &value_def.properties {
        if let Property::ValidValues(stmts) = prop {
            for stmt in stmts {
                match &stmt.kind {
                    StatementKind::Constraint(_, _, selector) => {
                        update_precision_from_selector(selector, &mut precision)?;
                    }
                    StatementKind::EventProgression(_, cases) => {
                        for case in cases {
                            update_precision_from_selector(&case.condition, &mut precision)?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(precision.decimals)
}

fn update_precision_from_selector(
    selector: &RangeSelector,
    precision: &mut PrecisionInfo,
) -> Result<(), String> {
    match selector {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
            let min_dec = update_precision_from_expr(min, precision)?;
            let max_dec = update_precision_from_expr(max, precision)?;
            if let (Some(a), Some(b)) = (min_dec, max_dec) {
                if a != b {
                    return Err(format!(
                        "range bounds must use the same precision ({} vs {})",
                        a, b
                    ));
                }
            }
        }
        RangeSelector::Equals(expr) => {
            update_precision_from_expr(expr, precision)?;
        }
        RangeSelector::List(items) => {
            for item in items {
                update_precision_from_expr(item, precision)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn update_precision_from_expr(
    expr: &Expression,
    precision: &mut PrecisionInfo,
) -> Result<Option<usize>, String> {
    let decimals = match expr {
        Expression::Literal(Literal::Number(_, decimals)) => {
            apply_precision(decimals, precision)?
        }
        Expression::Literal(Literal::Quantity(_, _, decimals)) => {
            apply_precision(decimals, precision)?
        }
        _ => None,
    };
    Ok(decimals)
}

fn apply_precision(
    decimals: &Option<usize>,
    precision: &mut PrecisionInfo,
) -> Result<Option<usize>, String> {
    let value = Some(match decimals {
        Some(d) => *d,
        None => 0,
    });

    match precision.decimals {
        Some(existing) => {
            if let Some(value) = value {
                if existing != value {
                    return Err(format!(
                        "all intervals must use the same precision ({} vs {})",
                        existing, value
                    ));
                }
            }
        }
        None => {
            precision.decimals = value;
        }
    }

    Ok(value)
}

fn value_has_precision(value: &RuntimeValue, decimals: usize) -> bool {
    let number = match value {
        RuntimeValue::Number(n) => *n,
        RuntimeValue::Quantity(n, _) => *n,
        _ => return true,
    };

    let scale = 10_f64.powi(decimals as i32);
    let scaled = (number * scale).round();
    (number * scale - scaled).abs() < 1e-9
}
