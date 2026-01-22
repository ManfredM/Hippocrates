use crate::ast::{Expression, Literal, RangeSelector, StatementKind, ContextItem, Definition, Property};
use crate::domain::{RuntimeValue, Unit};
use crate::runtime::Environment;
use crate::runtime::environment::EvaluationContext;
use crate::runtime::scheduler::Scheduler;
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(env: &Environment, expr: &Expression) -> RuntimeValue {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Number(n, _) => RuntimeValue::Number(*n),
                Literal::String(s) => RuntimeValue::String(s.clone()),
                Literal::Quantity(n, unit, _) => {
                    let u = if let crate::domain::Unit::Custom(s) = unit {
                        if let Some(canonical) = env.unit_map.get(s) {
                            canonical.clone()
                        } else {
                            unit.clone()
                        }
                    } else {
                        unit.clone()
                    };
                    RuntimeValue::Quantity(*n, u)
                },
                Literal::TimeOfDay(s) => RuntimeValue::String(s.clone()),
                Literal::Date(s) => {
                    if let Some(dt) = Self::parse_date_time_literal(s) {
                        RuntimeValue::Date(dt)
                    } else {
                        RuntimeValue::String(s.clone())
                    }
                }
            },
            Expression::Variable(name) => {
                if name == "now" {
                    return RuntimeValue::Date(env.now);
                }

                // Check if variable has a Calculation rule first (Derived Value)
                if let Some(Definition::Value(v_def)) = env.definitions.get(name) {
                    for prop in &v_def.properties {
                        if let Property::Calculation(stmts) = prop {
                            let mut timeframe = None;
                            let mut period = None;
                            let mut result_expr = None;

                            for stmt in stmts {
                                match &stmt.kind {
                                    StatementKind::ContextBlock(cb) => {
                                        for item in &cb.items {
                                            if let ContextItem::Timeframe(ts) = item {
                                                timeframe = Some(ts.clone());
                                                period = None;
                                            }
                                        }
                                        // Also search for assignment inside the context block
                                        for inner in &cb.statements {
                                            if let StatementKind::Assignment(assign) = &inner.kind {
                                                if assign.target == "value" || assign.target == *name {
                                                    result_expr = Some(&assign.expression);
                                                }
                                            }
                                        }
                                    }
                                    StatementKind::Assignment(assign) => {
                                        if assign.target == "value" || assign.target == *name {
                                            result_expr = Some(&assign.expression);
                                        }
                                    }
                                    StatementKind::Timeframe(tb) => {
                                        let ctx = EvaluationContext::from_constraints(
                                            &env.definitions,
                                            &tb.constraints,
                                        );
                                        timeframe = ctx.timeframe;
                                        period = ctx.period;
                                        for inner in &tb.block {
                                            if let StatementKind::Assignment(assign) = &inner.kind {
                                                if assign.target == "value" || assign.target == *name {
                                                     result_expr = Some(&assign.expression);
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            if let Some(expr) = result_expr {
                                let ctx = EvaluationContext { timeframe, period };
                                env.push_context(ctx);
                                let mut res = Self::evaluate(env, expr);
                                env.pop_context();
                                if let Some(expected_unit) = env.expected_unit_for_value(name) {
                                    let is_count = matches!(
                                        expr,
                                        Expression::Statistical(
                                            crate::ast::StatisticalFunc::CountOf(_, _)
                                        )
                                    );
                                    if is_count {
                                        if let RuntimeValue::Number(n) = res {
                                            res = RuntimeValue::Quantity(n, expected_unit);
                                        }
                                    }
                                }
                                return res;
                            }
                        }
                    }
                }

                // If no calculation, return stored value or Void
                // If no calculation, return stored value or check definition for implicit ask
                if let Some(v) = env.get_value(name) {
                    if let RuntimeValue::NotEnoughData = v {
                         // Check if askable
                         if let Some(Definition::Value(v_def)) = env.definitions.get(name) {
                             let has_question = v_def.properties.iter().any(|p| matches!(p, Property::Question(_)));
                             if has_question {
                                 return RuntimeValue::Missing(name.clone());
                             }
                         }
                         return RuntimeValue::NotEnoughData; 
                    }
                    return v.clone();
                }
                
                // If not in env, check definition for implicit ask (e.g. if default not set yet?)
                if let Some(Definition::Value(v_def)) = env.definitions.get(name) {
                     let has_question = v_def.properties.iter().any(|p| matches!(p, Property::Question(_)));
                     if has_question {
                         return RuntimeValue::Missing(name.clone());
                     }
                }
                RuntimeValue::Void
            }
            Expression::Binary(left, op, right) => {
                let mut l_val = Self::evaluate(env, left);
                let mut r_val = Self::evaluate(env, right);

                // Try resolve strings as variables
                if let RuntimeValue::String(s) = &l_val {
                    if let Some(v) = env.get_value(s) {
                        l_val = v.clone();
                    }
                }
                if let RuntimeValue::String(s) = &r_val {
                    if let Some(v) = env.get_value(s) {
                        r_val = v.clone();
                    }
                }

                // Propagate Missing
                if let RuntimeValue::Missing(n) = &l_val { return RuntimeValue::Missing(n.clone()); }
                if let RuntimeValue::Missing(n) = &r_val { return RuntimeValue::Missing(n.clone()); }

                if op == "+" {
                    // If either is String, concat
                    if matches!(l_val, RuntimeValue::String(_))
                        || matches!(r_val, RuntimeValue::String(_))
                    {
                        return RuntimeValue::String(format!("{}{}", l_val, r_val));
                    }
                }

                // Check for unit preservation (Same Unit Math)
                let preserved_unit = if let (RuntimeValue::Quantity(_, u1), RuntimeValue::Quantity(_, u2)) = (&l_val, &r_val) {
                    if u1 == u2 {
                        Some(u1.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                let l = l_val.as_number().unwrap_or(0.0);
                let r = r_val.as_number().unwrap_or(0.0);

                let result = match op.as_str() {
                    "+" => l + r,
                    "-" => l - r,
                    "*" => l * r,
                    "/" => {
                        if r != 0.0 {
                            l / r
                        } else {
                            0.0
                        }
                    } // Simple safety
                    _ => 0.0,
                };

                if let Some(u) = preserved_unit {
                    RuntimeValue::Quantity(result, u)
                } else {
                    RuntimeValue::Number(result)
                }
            }
            Expression::RelativeTime(val, unit, dir) => {
                let date = match unit {
                    crate::domain::Unit::Month | crate::domain::Unit::Year => {
                        let months = if matches!(unit, crate::domain::Unit::Year) {
                            (*val).trunc() as i32 * 12
                        } else {
                            (*val).trunc() as i32
                        };
                        match dir {
                            crate::ast::RelativeDirection::Ago => {
                                Self::shift_months(env.now, -months)
                            }
                            crate::ast::RelativeDirection::FromNow => {
                                Self::shift_months(env.now, months)
                            }
                        }
                    }
                    _ => {
                        let duration = match unit {
                            crate::domain::Unit::Second => chrono::Duration::seconds(*val as i64),
                            crate::domain::Unit::Minute => chrono::Duration::minutes(*val as i64),
                            crate::domain::Unit::Hour => chrono::Duration::hours(*val as i64),
                            crate::domain::Unit::Day => chrono::Duration::days(*val as i64),
                            crate::domain::Unit::Week => chrono::Duration::weeks(*val as i64),
                            _ => chrono::Duration::seconds(0),
                        };

                        match dir {
                            crate::ast::RelativeDirection::Ago => env.now - duration,
                            crate::ast::RelativeDirection::FromNow => env.now + duration,
                        }
                    }
                };

                RuntimeValue::Date(date)
            }
            Expression::DateDiff(unit, start_expr, end_expr) => {
                let start = Self::evaluate(env, start_expr);
                let end = Self::evaluate(env, end_expr);
                if let (Some(start_dt), Some(end_dt)) = (start.as_date(), end.as_date()) {
                    let (from, to) = if start_dt <= end_dt {
                        (start_dt, end_dt)
                    } else {
                        (end_dt, start_dt)
                    };

                    let seconds = (to - from).num_seconds().abs() as f64;
                    let value = match unit {
                        Unit::Minute => seconds / 60.0,
                        Unit::Hour => seconds / 3600.0,
                        Unit::Day => seconds / 86400.0,
                        Unit::Month => Self::diff_in_months(from, to) as f64,
                        Unit::Year => Self::diff_in_years(from, to) as f64,
                        _ => seconds,
                    };
                    RuntimeValue::Quantity(value, unit.clone())
                } else {
                    RuntimeValue::Void
                }
            }
            Expression::Statistical(func) => match func {
                crate::ast::StatisticalFunc::CountOf(name, filter) => {
                                if let Some(ctx) = env.active_context() {
                                    if let Some(selector) = &ctx.timeframe {
                                        // Check for NotEnoughData condition first
                                        let min_date_opt = match selector {
                                            crate::ast::RangeSelector::Between(e1, _) | crate::ast::RangeSelector::Range(e1, _) => {
                                                Self::evaluate(env, &e1).as_date()
                                            }
                                             crate::ast::RangeSelector::Condition(op, e1) => {
                                                if let Some(d) = Self::evaluate(env, &e1).as_date() {
                                                 match op {
                                                     crate::ast::ConditionOperator::GreaterThan | crate::ast::ConditionOperator::GreaterThanOrEquals => Some(d),
                                                     _ => None,
                                                 }
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    };

                                        if let Some(min_date) = min_date_opt {

                                            if min_date < env.start_time {
                                                return RuntimeValue::NotEnoughData;
                                            }
                                        }
                                    }
                                }

                    if let Some(history) = env.get_history(name) {
                        let filtered: Vec<&crate::domain::ValueInstance> =
                            if let Some(ctx) = env.active_context() {
                                history
                                    .iter()
                                    .filter(|i| Self::matches_context(env, &ctx, i.timestamp))
                                    .collect()
                            } else {
                                history.iter().collect()
                            };

                        let count = filtered
                            .iter()
                            .filter(|i| {
                                if let Some(f_expr) = filter {
                                    let mut target = Self::evaluate(env, f_expr);
                                    // Fallback for undefined variables to string (e.g. is yes)
                                    if let RuntimeValue::Void = target {
                                        if let Expression::Variable(vname) = &**f_expr {
                                            target = RuntimeValue::String(vname.clone());
                                        }
                                    }

                                    Self::fuzzy_equals(&i.value, &target)
                                } else {
                                    true
                                }
                            })
                            .count();

                        RuntimeValue::Number(count as f64)
                    } else {
                        RuntimeValue::Number(0.0)
                    }
                }
                crate::ast::StatisticalFunc::TrendOf(name) => {
                                if let Some(ctx) = env.active_context() {
                                    if let Some(selector) = &ctx.timeframe {
                                        // Check for NotEnoughData condition first
                                        let min_date_opt = match selector {
                                            crate::ast::RangeSelector::Between(e1, _) | crate::ast::RangeSelector::Range(e1, _) => {
                                                Self::evaluate(env, &e1).as_date()
                                            }
                                             crate::ast::RangeSelector::Condition(op, e1) => {
                                                if let Some(d) = Self::evaluate(env, &e1).as_date() {
                                                     match op {
                                                         crate::ast::ConditionOperator::GreaterThan | crate::ast::ConditionOperator::GreaterThanOrEquals => Some(d),
                                                         _ => None,
                                                     }
                                                } else {
                                                    None
                                                }
                                            }
                                            _ => None,
                                        };

                                        if let Some(min_date) = min_date_opt {
                                            if min_date < env.start_time {
                                                return RuntimeValue::NotEnoughData;
                                            }
                                        }
                                    }
                                }

                    // 1. Get History
                    if let Some(history) = env.get_history(name) {
                        let filtered: Vec<&crate::domain::ValueInstance> =
                            if let Some(ctx) = env.active_context() {
                                history
                                    .iter()
                                    .filter(|i| Self::matches_context(env, &ctx, i.timestamp))
                                    .collect()
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
                            let x = (item.timestamp - start_time).num_seconds() as f64 / 86400.0;
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

                        if slope > 0.0001 {
                            // Epsilon threshold
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
            },
            Expression::FunctionCall(_, _) => RuntimeValue::Void,
            Expression::InterpolatedString(parts) => {
                let mut result = String::new();
                for part in parts {
                    let val = Self::evaluate(env, part);
                    match val {
                        RuntimeValue::String(s) => result.push_str(&s),
                        RuntimeValue::Number(n) => result.push_str(&n.to_string()),
                        RuntimeValue::Quantity(q, u) => result.push_str(&format!("{} {:?}", q, u)),
                        RuntimeValue::Date(d) => result.push_str(&d.to_string()),
                        RuntimeValue::Boolean(b) => result.push_str(&b.to_string()),
                        RuntimeValue::Enumeration(e) => result.push_str(&e),
                        RuntimeValue::List(l) => result.push_str(&format!("{:?}", l)), // Debug format for now
                        RuntimeValue::Void => {}
                        RuntimeValue::NotEnoughData => result.push_str("Not Enough Data"),
                        RuntimeValue::Missing(_) => result.push_str("Missing"),
                    }
                }
                RuntimeValue::String(result)
            }
        }
    }

    pub fn check_condition(
        env: &Environment,
        selector: &RangeSelector,
        value: &RuntimeValue,
    ) -> bool {
        match selector {
            RangeSelector::Equals(expr) => {
                let target = Self::evaluate(env, expr);
                if let (Some(value_dt), Some(target_dt)) = (value.as_date(), target.as_date()) {
                    return value_dt == target_dt;
                }
                if let (Some(value_time), Some(target_time)) =
                    (Self::time_from_value(value), Self::time_from_value(&target))
                {
                    return value_time == target_time;
                }
                Self::fuzzy_equals(value, &target)
            }
            RangeSelector::Range(min, max) => {
                let min_eval = Self::evaluate(env, min);
                let max_eval = Self::evaluate(env, max);

                if let (Some(min_date), Some(max_date), Some(actual_date)) = (
                    min_eval.as_date(),
                    max_eval.as_date(),
                    value.as_date(),
                ) {
                    return actual_date >= min_date && actual_date <= max_date;
                }

                if let (Some(min_time), Some(max_time), Some(actual_time)) = (
                    Self::time_from_value(&min_eval),
                    Self::time_from_value(&max_eval),
                    Self::time_from_value(value),
                ) {
                    return Self::time_in_range(actual_time, min_time, max_time);
                }

                let min_val = min_eval.as_number();
                let max_val = max_eval.as_number();
                let actual = value.as_number();

                match (min_val, max_val, actual) {
                    (Some(min), Some(max), Some(val)) => val >= min && val <= max,
                    _ => false,
                }
            }
            RangeSelector::Between(min, max) => {
                // "between X ... Y" usually acts like Range
                let min_eval = Self::evaluate(env, min);
                let max_eval = Self::evaluate(env, max);

                if let (Some(min_date), Some(max_date), Some(actual_date)) = (
                    min_eval.as_date(),
                    max_eval.as_date(),
                    value.as_date(),
                ) {
                    return actual_date >= min_date && actual_date <= max_date;
                }

                if let (Some(min_time), Some(max_time), Some(actual_time)) = (
                    Self::time_from_value(&min_eval),
                    Self::time_from_value(&max_eval),
                    Self::time_from_value(value),
                ) {
                    return Self::time_in_range(actual_time, min_time, max_time);
                }

                let min_val = min_eval.as_number();
                let max_val = max_eval.as_number();
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
                    if Self::fuzzy_equals(value, &target) {
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
                    crate::ast::ConditionOperator::Equals => Self::fuzzy_equals(&l_val, &r_val),
                    crate::ast::ConditionOperator::NotEquals => !Self::fuzzy_equals(&l_val, &r_val),
                    crate::ast::ConditionOperator::GreaterThan => l_num > r_num,
                    crate::ast::ConditionOperator::LessThan => l_num < r_num,
                    crate::ast::ConditionOperator::GreaterThanOrEquals => l_num >= r_num,
                    crate::ast::ConditionOperator::LessThanOrEquals => l_num <= r_num,
                }
            }
            RangeSelector::Condition(op, expr) => {
                let r_val = Self::evaluate(env, expr);
                let l_num = value.as_number();
                let r_num = r_val.as_number();
                match op {
                    crate::ast::ConditionOperator::Equals => Self::fuzzy_equals(value, &r_val),
                    crate::ast::ConditionOperator::NotEquals => !Self::fuzzy_equals(value, &r_val),
                    crate::ast::ConditionOperator::GreaterThan => l_num > r_num,
                    crate::ast::ConditionOperator::LessThan => l_num < r_num,
                    crate::ast::ConditionOperator::GreaterThanOrEquals => l_num >= r_num,
                    crate::ast::ConditionOperator::LessThanOrEquals => l_num <= r_num,
                }
            }
            RangeSelector::NotEnoughData => {
                 if let RuntimeValue::NotEnoughData = value {
                    true
                } else {
                    false
                }
            }
            RangeSelector::Default => true,
        }
    }

    fn fuzzy_equals(v1: &RuntimeValue, v2: &RuntimeValue) -> bool {
        match (v1, v2) {
            (RuntimeValue::String(s1), RuntimeValue::String(s2)) => s1.eq_ignore_ascii_case(s2),
            (RuntimeValue::Enumeration(s1), RuntimeValue::String(s2)) => {
                s1.eq_ignore_ascii_case(s2)
            }
            (RuntimeValue::String(s1), RuntimeValue::Enumeration(s2)) => {
                s1.eq_ignore_ascii_case(s2)
            }
            (RuntimeValue::Enumeration(s1), RuntimeValue::Enumeration(s2)) => {
                s1.eq_ignore_ascii_case(s2)
            }
            _ => v1 == v2,
        }
    }

    fn matches_context(
        env: &Environment,
        ctx: &EvaluationContext,
        timestamp: chrono::NaiveDateTime,
    ) -> bool {
        if let Some(selector) = &ctx.timeframe {
            if !Self::check_timeframe_match(env, selector, timestamp) {
                return false;
            }
        }
        if let Some(period_name) = &ctx.period {
            if !Self::check_period_match(env, period_name, timestamp) {
                return false;
            }
        }
        true
    }

    fn check_period_match(
        env: &Environment,
        period_name: &str,
        timestamp: chrono::NaiveDateTime,
    ) -> bool {
        if let Some(def) = env.definitions.get(period_name) {
            Scheduler::is_within_period(def, timestamp)
        } else {
            false
        }
    }

    pub fn check_timeframe_match(
        env: &Environment,
        selector: &RangeSelector,
        timestamp: chrono::NaiveDateTime,
    ) -> bool {
        match selector {
            RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
                let min_val = Self::evaluate(env, min);
                let max_val = Self::evaluate(env, max);
                if let (Some(min_date), Some(max_date)) = (min_val.as_date(), max_val.as_date()) {
                    // Use an exclusive lower bound to avoid off-by-one counts on day-sized windows.
                    timestamp > min_date && timestamp <= max_date
                } else if let (Some(min_time), Some(max_time)) = (
                    Self::time_from_value(&min_val),
                    Self::time_from_value(&max_val),
                ) {
                    Self::time_in_range(timestamp.time(), min_time, max_time)
                } else {
                    false // Invalid bounds
                }
            }
            RangeSelector::Condition(op, expr) => {
                let target = Self::evaluate(env, expr);
                if let Some(target_date) = target.as_date() {
                    match op {
                        crate::ast::ConditionOperator::Equals => timestamp == target_date,
                        crate::ast::ConditionOperator::NotEquals => timestamp != target_date,
                        crate::ast::ConditionOperator::GreaterThan => timestamp > target_date,
                        crate::ast::ConditionOperator::LessThan => timestamp < target_date,
                        crate::ast::ConditionOperator::GreaterThanOrEquals => timestamp >= target_date,
                        crate::ast::ConditionOperator::LessThanOrEquals => timestamp <= target_date,
                    }
                } else {
                    false
                }
            }
            RangeSelector::Equals(expr) => {
                let target = Self::evaluate(env, expr);
                if let Some(target_date) = target.as_date() {
                    timestamp == target_date
                } else {
                    false
                }
            }
            RangeSelector::GreaterThan(expr) => {
                let target = Self::evaluate(env, expr);
                if let Some(target_date) = target.as_date() {
                    timestamp > target_date
                } else {
                    false
                }
            }
            RangeSelector::NotEnoughData => {
                // Not a time filter
                false
            }
            _ => true, // Fallback for unsupported variants? Or false? 
        }
    }

    fn parse_date_time_literal(value: &str) -> Option<NaiveDateTime> {
        if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M") {
            return Some(dt);
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %-H:%M") {
            return Some(dt);
        }
        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            return date.and_hms_opt(0, 0, 0);
        }
        None
    }

    fn time_from_value(value: &RuntimeValue) -> Option<NaiveTime> {
        match value {
            RuntimeValue::Date(dt) => Some(dt.time()),
            RuntimeValue::String(s) => NaiveTime::parse_from_str(s, "%H:%M")
                .or_else(|_| NaiveTime::parse_from_str(s, "%-H:%M"))
                .ok(),
            _ => None,
        }
    }

    fn time_in_range(actual: NaiveTime, start: NaiveTime, end: NaiveTime) -> bool {
        if start <= end {
            actual >= start && actual <= end
        } else {
            actual >= start || actual <= end
        }
    }

    fn shift_months(base: NaiveDateTime, months: i32) -> NaiveDateTime {
        let date = base.date();
        let time = base.time();
        let (year, month) = (date.year(), date.month() as i32);
        let total = month - 1 + months;
        let new_year = year + total.div_euclid(12);
        let new_month = total.rem_euclid(12) + 1;
        let last_day = Self::last_day_of_month(new_year, new_month as u32);
        let day = date.day().min(last_day);
        let new_date = NaiveDate::from_ymd_opt(new_year, new_month as u32, day)
            .unwrap_or(date);
        new_date.and_time(time)
    }

    fn last_day_of_month(year: i32, month: u32) -> u32 {
        let next_month = if month == 12 { 1 } else { month + 1 };
        let next_year = if month == 12 { year + 1 } else { year };
        let first_next = NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 1).unwrap());
        (first_next - chrono::Duration::days(1)).day()
    }

    fn diff_in_months(start: NaiveDateTime, end: NaiveDateTime) -> i64 {
        let mut months = (end.year() - start.year()) as i64 * 12
            + (end.month() as i64 - start.month() as i64);
        if end.day() < start.day()
            || (end.day() == start.day() && end.time() < start.time())
        {
            months -= 1;
        }
        months.max(0)
    }

    fn diff_in_years(start: NaiveDateTime, end: NaiveDateTime) -> i64 {
        let mut years = (end.year() - start.year()) as i64;
        if (end.month(), end.day(), end.time()) < (start.month(), start.day(), start.time()) {
            years -= 1;
        }
        years.max(0)
    }
}
