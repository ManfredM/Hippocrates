use crate::ast::{
    ConditionalTarget, ContextItem, Definition, Expression, Literal, Plan, Property, RangeSelector,
    Statement, StatementKind,
};

pub fn validate_file(plan: &Plan) -> Result<(), String> {
    // 1. Collect Value Definitions and their Valid Ranges
    use std::collections::{HashMap, HashSet};
    let mut value_ranges: HashMap<String, Vec<RangeSelector>> = HashMap::new();

    let mut timeframe_vars: HashSet<String> = HashSet::new();
    let mut enum_vars: HashSet<String> = HashSet::new();

    for def in &plan.definitions {
        if let Definition::Value(vd) = def {
            for prop in &vd.properties {
                match prop {
                    Property::ValidValues(stmts) => {
                        // Parse statements to find ranges.
                        for s in stmts {
                            if let StatementKind::EventProgression(_, cases) = &s.kind {
                                let mut selectors = Vec::new();
                                for case in cases {
                                    selectors.push(case.condition.clone());
                                }
                                value_ranges.insert(vd.name.clone(), selectors);
                            }
                        }
                    }
                    Property::Calculation(stmts) => {
                        for stmt in stmts {
                            // Check for Timeframe context
                            if let StatementKind::ContextBlock(cb) = &stmt.kind {
                                for item in &cb.items {
                                    if let ContextItem::Timeframe(_) = item {
                                        timeframe_vars.insert(vd.name.clone());
                                    }
                                }
                            }
                        }
                        // Validate statements in calculation
                        for stmt in stmts {
                            validate_statement(stmt, &value_ranges, &timeframe_vars, &enum_vars)?;
                        }
                    }
                    _ => {}
                }
            }

             if let crate::domain::ValueType::Enumeration = vd.value_type {
                enum_vars.insert(vd.name.clone());
            }
        }
        }


    // 2. Validate Statements in Plan Definition
    for def in &plan.definitions {
        if let Definition::Plan(pd) = def {
            for block in &pd.blocks {
                let statements = match block {
                    crate::ast::PlanBlock::DuringPlan(s) => s,
                    crate::ast::PlanBlock::Event(e) => &e.statements,
                    crate::ast::PlanBlock::Trigger(t) => &t.statements,
                };

                for stmt in statements {
                    validate_statement(stmt, &value_ranges, &timeframe_vars, &enum_vars)?;
                }
            }
        }
    }

    Ok(())
}

fn validate_statement(
    stmt: &Statement,
    value_ranges: &std::collections::HashMap<String, Vec<RangeSelector>>,
    timeframe_vars: &std::collections::HashSet<String>,
    enum_vars: &std::collections::HashSet<String>,
) -> Result<(), String> {
    match &stmt.kind {
        StatementKind::Assignment(assign) => {
             validate_expression(&assign.expression, enum_vars)?;
        }
        StatementKind::Conditional(cond) => {
            validate_conditional(cond, value_ranges, timeframe_vars, enum_vars)?;
        }
        StatementKind::ContextBlock(cb) => {
            for s in &cb.statements {
                 validate_statement(s, value_ranges, timeframe_vars, enum_vars)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_expression(
    expr: &Expression,
    enum_vars: &std::collections::HashSet<String>,
) -> Result<(), String> {
    match expr {
        Expression::Binary(left, _, right) => {
            validate_expression(left, enum_vars)?;
            validate_expression(right, enum_vars)?;
        }
        Expression::Statistical(func) => match func {
            crate::ast::StatisticalFunc::CountOf(name, filter) => {
                if enum_vars.contains(name) {
                    if filter.is_none() {
                        return Err(format!(
                            "Validation Error: 'count of {}' requires a specific value to count (e.g. 'count of {} is \"Yes\"') because it is an Enumeration.",
                            name, name
                        ));
                    }
                }
                 if let Some(f_expr) = filter {
                    validate_expression(f_expr, enum_vars)?;
                }
            }
            crate::ast::StatisticalFunc::TrendOf(name) => {
                 if enum_vars.contains(name) {
                    return Err(format!(
                        "Validation Error: 'trend of {}' is not supported because it is an Enumeration. Trend analysis requires numeric values.",
                        name
                    ));
                }
            }
            crate::ast::StatisticalFunc::AverageOf(_, period) => {
                validate_expression(period, enum_vars)?;
            }
            _ => {}
        },
        Expression::FunctionCall(_, args) => {
            for arg in args {
                validate_expression(arg, enum_vars)?;
            }
        }
        Expression::InterpolatedString(parts) => {
            for part in parts {
                validate_expression(part, enum_vars)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_conditional(
    cond: &crate::ast::Conditional,
    value_ranges: &std::collections::HashMap<String, Vec<RangeSelector>>,
    timeframe_vars: &std::collections::HashSet<String>,
    enum_vars: &std::collections::HashSet<String>,
) -> Result<(), String> {
    let target_name = match &cond.condition {
        ConditionalTarget::Expression(Expression::Variable(name)) => Some(name.clone()),
        ConditionalTarget::Expression(Expression::Literal(Literal::String(s))) => Some(s.clone()),
        _ => None,
    };

    if let Some(name) = target_name {
        // 1. Check Numeric Coverage
        if let Some(valid_selectors) = value_ranges.get(&name) {
            check_coverage(&name, valid_selectors, &cond.cases)?;
        }
        // 2. Check Data Sufficiency Coverage
        if timeframe_vars.contains(&name) {
            check_data_sufficiency(&name, &cond.cases)?;
        }
    }

    // Recurse into blocks
    for case in &cond.cases {
        for s in &case.block {
            validate_statement(s, value_ranges, timeframe_vars, enum_vars)?;
        }
    }

    Ok(())
}

fn check_data_sufficiency(
    name: &str,
    cases: &[crate::ast::AssessmentCase],
) -> Result<(), String> {
    let has_not_enough_data = cases.iter().any(|c| matches!(c.condition, RangeSelector::NotEnoughData));

    if !has_not_enough_data {
        return Err(format!(
            "Missing Case: Assessment for '{}' depends on a timeframe calculation but does not handle 'Not enough data'.",
            name
        ));
    }
    Ok(())
}

fn check_coverage(
    name: &str,
    valid: &[RangeSelector],
    cases: &[crate::ast::AssessmentCase],
) -> Result<(), String> {
    // interval-based coverage check for f64 ranges.
    
    // 1. Determine Universe [min, max]
    let mut universe_min = f64::NEG_INFINITY;
    let mut universe_max = f64::INFINITY;
    let mut has_range = false;

    for sel in valid {
        if let RangeSelector::Range(
            Expression::Literal(Literal::Number(min)),
            Expression::Literal(Literal::Number(max)),
        ) = sel
        {
            universe_min = *min;
            universe_max = *max;
            has_range = true;
            break; 
        }
    }

    if !has_range {
        return Ok(());
    }

    // 2. Collect Intervals from Cases
    struct Interval { start: f64, end: f64 }
    let mut intervals: Vec<Interval> = Vec::new();

    for case in cases {
        match &case.condition {
            RangeSelector::Range(
                Expression::Literal(Literal::Number(min)),
                Expression::Literal(Literal::Number(max)),
            ) => {
                intervals.push(Interval { start: *min, end: *max });
            }
            RangeSelector::Condition(op, Expression::Literal(Literal::Number(val))) => {
                let v = *val;
                match op {
                    crate::ast::ConditionOperator::GreaterThan => intervals.push(Interval { start: v, end: f64::INFINITY }),
                    crate::ast::ConditionOperator::GreaterThanOrEquals => intervals.push(Interval { start: v, end: f64::INFINITY }),
                    crate::ast::ConditionOperator::LessThan => intervals.push(Interval { start: f64::NEG_INFINITY, end: v }),
                    crate::ast::ConditionOperator::LessThanOrEquals => intervals.push(Interval { start: f64::NEG_INFINITY, end: v }),
                    _ => {}
                }
            }
            RangeSelector::Equals(Expression::Literal(Literal::Number(v))) => {
                intervals.push(Interval { start: *v, end: *v });
            }
            _ => {}
        }
    }

    // 3. Clamp Intervals to Universe
    let mut clamped: Vec<Interval> = Vec::new();
    for i in intervals {
        let start = i.start.max(universe_min);
        let end = i.end.min(universe_max);
        if start <= end { // Valid overlap
             clamped.push(Interval { start, end });
        }
    }

    // 4. Sort and Sweep
    // Sort by start
    clamped.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));

    let mut current = universe_min;
    let epsilon = 0.00001; 

    for i in clamped {
        // If there's a significant gap before this interval starts
        if i.start > current + epsilon {
            return Err(format!(
                "Constraint Violation: Assessment for '{}' is incomplete. Uncovered gap detected between {:.2} and {:.2}. Valid range is {:.1}...{:.1}.",
                name, current, i.start, universe_min, universe_max
            ));
        }
        // Extend current reach
        if i.end > current {
            current = i.end;
        }
    }

    // Check end gap
    if current < universe_max - epsilon {
        return Err(format!(
            "Constraint Violation: Assessment for '{}' is incomplete. Uncovered gap at the end between {:.2} and {:.2}. Valid range is {:.1}...{:.1}.",
            name, current, universe_max, universe_min, universe_max
        ));
    }

    Ok(())
}
