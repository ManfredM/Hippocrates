use crate::ast::{Expression, Literal, RangeSelector};
use crate::domain::{RuntimeValue};
use crate::runtime::Environment;

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(env: &Environment, expr: &Expression) -> RuntimeValue {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Number(n) => RuntimeValue::Number(*n),
                Literal::String(s) => RuntimeValue::String(s.clone()),
                Literal::Quantity(n, unit) => RuntimeValue::Quantity(*n, unit.clone()),
                Literal::TimeOfDay(s) => RuntimeValue::String(s.clone()),
                Literal::Date(s) => RuntimeValue::String(s.clone()),
            },
            Expression::Variable(name) => {
                if let Some(val) = env.get_value(name) {
                    val.clone()
                } else {
                    RuntimeValue::Void
                }
            }
            Expression::Binary(left, op, right) => {
                let mut l_val = Self::evaluate(env, left);
                let mut r_val = Self::evaluate(env, right);
                
                // Try resolve strings as variables
                if let RuntimeValue::String(s) = &l_val {
                    if let Some(v) = env.get_value(s) { l_val = v.clone(); }
                }
                if let RuntimeValue::String(s) = &r_val {
                     if let Some(v) = env.get_value(s) { r_val = v.clone(); }
                }

                if op == "+" {
                     // If either is String, concat
                     if matches!(l_val, RuntimeValue::String(_)) || matches!(r_val, RuntimeValue::String(_)) {
                         return RuntimeValue::String(format!("{}{}", l_val, r_val));
                     }
                }

                let l = l_val.as_number().unwrap_or(0.0);
                let r = r_val.as_number().unwrap_or(0.0);

                let result = match op.as_str() {
                    "+" => l + r,
                    "-" => l - r,
                    "*" => l * r,
                    "/" => if r != 0.0 { l / r } else { 0.0 }, // Simple safety
                    _ => 0.0,
                };
                RuntimeValue::Number(result)
            }
            Expression::RelativeTime(val, unit, dir) => {
                 let duration = match unit {
                     crate::domain::Unit::Second => chrono::Duration::seconds(*val as i64),
                     crate::domain::Unit::Minute => chrono::Duration::minutes(*val as i64),
                     crate::domain::Unit::Hour => chrono::Duration::hours(*val as i64),
                     crate::domain::Unit::Day => chrono::Duration::days(*val as i64),
                     crate::domain::Unit::Week => chrono::Duration::weeks(*val as i64),
                     _ => chrono::Duration::seconds(0),
                 };
                 
                 let date = match dir {
                     crate::ast::RelativeDirection::Ago => env.now - duration,
                     crate::ast::RelativeDirection::FromNow => env.now + duration,
                 };
                 
                 RuntimeValue::Date(date)
            }
            Expression::Statistical(func) => match func {
                 crate::ast::StatisticalFunc::CountOf(name, filter) => {
                     if let Some(history) = env.get_history(name) {
                         // Filter by Context Timeframe
                         let filtered: Vec<&crate::domain::ValueInstance> = if let Some(ctx) = env.active_context() {
                             if let Some(crate::ast::RangeSelector::Range(min_expr, max_expr)) = &ctx.timeframe {
                                 let min_val = Self::evaluate(env, min_expr);
                                 let max_val = Self::evaluate(env, max_expr);
                                 
                                 if let (Some(min_date), Some(max_date)) = (min_val.as_date(), max_val.as_date()) {
                                     history.iter().filter(|i| i.timestamp >= min_date && i.timestamp <= max_date).collect()
                                 } else {
                                     history.iter().collect()
                                 }
                             } else {
                                 history.iter().collect()
                             }
                         } else {
                             history.iter().collect()
                         };

                         let count = filtered.iter().filter(|i| {
                             if let Some(f_expr) = filter {
                                 let mut target = Self::evaluate(env, f_expr);
                                 // Fallback for undefined variables to string (e.g. is yes)
                                 if let RuntimeValue::Void = target {
                                     if let Expression::Variable(vname) = &**f_expr {
                                         target = RuntimeValue::String(vname.clone());
                                     }
                                 }

                                 match (&i.value, &target) {
                                     (RuntimeValue::String(s1), RuntimeValue::String(s2)) => s1.eq_ignore_ascii_case(s2),
                                     _ => i.value == target
                                 }
                             } else {
                                 true
                             }
                         }).count();
                         
                         RuntimeValue::Number(count as f64)
                     } else {
                         RuntimeValue::Number(0.0)
                     }
                 }
                 crate::ast::StatisticalFunc::TrendOf(name) => {
                     // 1. Get History
                     if let Some(history) = env.get_history(name) {
                         // 2. Filter by Context Timeframe
                         let filtered: Vec<&crate::domain::ValueInstance> = if let Some(ctx) = env.active_context() {
                             if let Some(crate::ast::RangeSelector::Range(min_expr, max_expr)) = &ctx.timeframe {
                                 // Evaluate range bounds
                                 let min_val = Self::evaluate(env, min_expr);
                                 let max_val = Self::evaluate(env, max_expr);
                                 
                                 if let (Some(min_date), Some(max_date)) = (min_val.as_date(), max_val.as_date()) {
                                     history.iter().filter(|i| i.timestamp >= min_date && i.timestamp <= max_date).collect()
                                 } else {
                                     history.iter().collect() // Fallback if bounds aren't dates
                                 }
                             } else {
                                 history.iter().collect()
                             }
                         } else {
                             history.iter().collect()
                         };

                         if filtered.len() < 2 {
                             return RuntimeValue::String("stable".to_string());
                         }

                         // 3. Calculate Slope (Linear Regression)
                         // X = seconds from first point
                         // Y = value
                         let start_time = filtered[0].timestamp;
                         let n = filtered.len() as f64;
                         let mut sum_x = 0.0;
                         let mut sum_y = 0.0;
                         let mut sum_xy = 0.0;
                         let mut sum_xx = 0.0;
                         
                         for item in &filtered {
                             let x = (item.timestamp - start_time).num_seconds() as f64;
                             let y = item.value.as_number().unwrap_or(0.0);
                             
                             sum_x += x;
                             sum_y += y;
                             sum_xy += x * y;
                             sum_xx += x * x;
                         }
                         
                         // Slope = (N*Sum(XY) - Sum(X)*Sum(Y)) / (N*Sum(XX) - Sum(X)^2)
                         let denominator = n * sum_xx - sum_x * sum_x;
                         let slope = if denominator.abs() < f64::EPSILON {
                             0.0
                         } else {
                             (n * sum_xy - sum_x * sum_y) / denominator
                         };
                         
                         // println!("DEBUG: Trend calculation. N={}, Slope={}", n, slope);

                         if slope > 0.0001 { // Epsilon threshold
                             RuntimeValue::String("increase".to_string())
                         } else if slope < -0.0001 {
                             RuntimeValue::String("decrease".to_string())
                         } else {
                             RuntimeValue::String("stable".to_string())
                         }

                     } else {
                         RuntimeValue::String("stable".to_string()) // No history
                     }
                 }
                 _ => RuntimeValue::Number(0.0), // Stub for others
            }
        }
    }

    pub fn check_condition(env: &Environment, selector: &RangeSelector, value: &RuntimeValue) -> bool {
        match selector {
            RangeSelector::Equals(expr) => {
                let target = Self::evaluate(env, expr);
                value == &target
            }
            RangeSelector::Range(min, max) => {
                let min_val = Self::evaluate(env, min).as_number();
                let max_val = Self::evaluate(env, max).as_number();
                let actual = value.as_number();
                
                match (min_val, max_val, actual) {
                    (Some(min), Some(max), Some(val)) => val >= min && val <= max,
                    _ => false,
                }
            }
            RangeSelector::Between(min, max) => {
                 // "between X ... Y" usually acts like Range
                 let min_val = Self::evaluate(env, min).as_number();
                 let max_val = Self::evaluate(env, max).as_number();
                 let actual = value.as_number();
                 
                 match (min_val, max_val, actual) {
                     (Some(min), Some(max), Some(val)) => val >= min && val <= max,
                     _ => false,
                 }
            }
             RangeSelector::GreaterThan(expr) => {
                let target = Self::evaluate(env, expr).as_number();
                let actual = value.as_number();
                match (target, actual) {
                    (Some(t), Some(a)) => a > t,
                    _ => false,
                }
            }
            RangeSelector::List(handlers) => {
                 // If any matches
                 for h in handlers {
                     // TODO: Need recursive check ideally, but `Expression` isn't `RangeSelector`
                     // For now, treat list as list of Equals
                     let target = Self::evaluate(env, h);
                     if value == &target {
                         return true;
                     }
                 }
                 false
            }
            RangeSelector::Comparison(left, op, right) => {
                let l_val = Evaluator::evaluate(env, left);
                let r_val = Evaluator::evaluate(env, right);
                // Simple comparison logic (numbers only for now?)
                // Or generic comparison.
                let l_num = l_val.as_number();
                let r_num = r_val.as_number();
                match op {
                    crate::ast::ConditionOperator::Equals => l_val == r_val, // Or loose equality?
                    crate::ast::ConditionOperator::NotEquals => l_val != r_val,
                    crate::ast::ConditionOperator::GreaterThan => l_num > r_num,
                    crate::ast::ConditionOperator::LessThan => l_num < r_num,
                    crate::ast::ConditionOperator::GreaterThanOrEquals => l_num >= r_num,
                    crate::ast::ConditionOperator::LessThanOrEquals => l_num <= r_num,
                }
            }
            RangeSelector::Default => true,
        }
    }
}
