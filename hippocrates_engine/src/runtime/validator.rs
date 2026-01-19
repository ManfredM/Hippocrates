use crate::ast::{
    ConditionalTarget, ContextItem, Definition, Expression, Literal, Plan, Property, RangeSelector,
    Statement, StatementKind,
};

pub fn validate_file(plan: &Plan) -> Result<(), Vec<crate::domain::EngineError>> {
    // 1. Collect Value Definitions and their Valid Ranges
    use std::collections::{HashMap, HashSet};
    
    let mut value_ranges: HashMap<String, Vec<RangeSelector>> = HashMap::new();
    let mut timeframe_vars: HashSet<String> = HashSet::new();
    let mut enum_vars: HashSet<String> = HashSet::new();
    let mut numeric_vars: HashSet<String> = HashSet::new();

    let mut errors: Vec<crate::domain::EngineError> = Vec::new();

    for def in &plan.definitions {
        if let Definition::Value(vd) = def {
            for prop in &vd.properties {
                match prop {
                    Property::ValidValues(stmts) => {
                        let mut selectors = Vec::new();

                        // Parse statements to find ranges.
                        for s in stmts {
                            if let StatementKind::EventProgression(_, cases) = &s.kind {
                                for case in cases {
                                    selectors.push(case.condition.clone());
                                }
                            } else if let StatementKind::Constraint(_, _, sel) = &s.kind {
                                selectors.push(sel.clone());
                            }
                        }
                        
                        if !selectors.is_empty() {
                            value_ranges.insert(vd.name.clone(), selectors);
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
                            let mut constraints = HashMap::new();
                            validate_statement(stmt, &value_ranges, &timeframe_vars, &enum_vars, &numeric_vars, &mut constraints, &mut errors);
                        }
                    }
                    _ => {}
                }
            }

            if let crate::domain::ValueType::Enumeration = vd.value_type {
                enum_vars.insert(vd.name.clone());
            }
            if let crate::domain::ValueType::Number = vd.value_type {
                numeric_vars.insert(vd.name.clone());
                check_numeric_units(&vd.name, &vd.properties, &mut errors);
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
                    let mut constraints = HashMap::new();
                    validate_statement(stmt, &value_ranges, &timeframe_vars, &enum_vars, &numeric_vars, &mut constraints, &mut errors);
                }
            }
        }
    }

    errors.sort();
    errors.dedup();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_statement(
    stmt: &Statement,
    value_ranges: &std::collections::HashMap<String, Vec<RangeSelector>>,
    timeframe_vars: &std::collections::HashSet<String>,
    enum_vars: &std::collections::HashSet<String>,
    numeric_vars: &std::collections::HashSet<String>,
    constraints: &mut std::collections::HashMap<String, (f64, f64)>,
    errors: &mut Vec<crate::domain::EngineError>,
) {
    match &stmt.kind {
        StatementKind::Assignment(assign) => {
             validate_expression(&assign.expression, enum_vars, stmt.line, errors);
             // Clear constraint on assignment because value changes
             constraints.remove(&assign.target);
        }
        StatementKind::Conditional(cond) => {
            validate_conditional(cond, value_ranges, timeframe_vars, enum_vars, numeric_vars, constraints, stmt.line, errors);
        }
        StatementKind::ContextBlock(cb) => {
            for s in &cb.statements {
                 validate_statement(s, value_ranges, timeframe_vars, enum_vars, numeric_vars, constraints, errors);
            }
        }
        _ => {}
    }
}

fn validate_expression(
    expr: &Expression,
    enum_vars: &std::collections::HashSet<String>,
    line: usize,
    errors: &mut Vec<crate::domain::EngineError>,
) {
    use crate::domain::EngineError;
    match expr {
        Expression::Binary(left, _, right) => {
            validate_expression(left, enum_vars, line, errors);
            validate_expression(right, enum_vars, line, errors);
        }
        Expression::Statistical(func) => match func {
            crate::ast::StatisticalFunc::CountOf(name, filter) => {
                if enum_vars.contains(name) {
                    if filter.is_none() {
                        errors.push(EngineError {
                            message: format!(
                                "Validation Error: 'count of {}' requires a specific value to count (e.g. 'count of {} is \"Yes\"') because it is an Enumeration.",
                                name, name
                            ),
                            line,
                            column: 0
                        });
                    }
                }
                 if let Some(f_expr) = filter {
                    validate_expression(f_expr, enum_vars, line, errors);
                }
            }
            crate::ast::StatisticalFunc::TrendOf(name) => {
                 if enum_vars.contains(name) {
                    errors.push(EngineError {
                        message: format!(
                            "Validation Error: 'trend of {}' is not supported because it is an Enumeration. Trend analysis requires numeric values.",
                            name
                        ),
                        line,
                        column: 0
                    });
                }
            }
            crate::ast::StatisticalFunc::AverageOf(_, period) => {
                validate_expression(period, enum_vars, line, errors);
            }
            _ => {}
        },
        Expression::FunctionCall(_, args) => {
            for arg in args {
                validate_expression(arg, enum_vars, line, errors);
            }
        }
        Expression::InterpolatedString(parts) => {
            for part in parts {
                validate_expression(part, enum_vars, line, errors);
            }
        }
        _ => {}
    }
}

fn validate_conditional(
    cond: &crate::ast::Conditional,
    value_ranges: &std::collections::HashMap<String, Vec<RangeSelector>>,
    timeframe_vars: &std::collections::HashSet<String>,
    enum_vars: &std::collections::HashSet<String>,
    numeric_vars: &std::collections::HashSet<String>,
    constraints: &mut std::collections::HashMap<String, (f64, f64)>,
    line: usize,
    errors: &mut Vec<crate::domain::EngineError>,
) {
    let target_name = match &cond.condition {
        ConditionalTarget::Expression(Expression::Variable(name)) => Some(name.clone()),
        ConditionalTarget::Expression(Expression::Literal(Literal::String(s))) => Some(s.clone()),
        _ => None,
    };

    if let Some(name) = &target_name {
        // 1. Check Numeric Coverage
        if let Some(valid_selectors) = value_ranges.get(name) {
            check_coverage(name, valid_selectors, &cond.cases, line, errors);
        }
        // 2. Check Data Sufficiency Coverage
        if timeframe_vars.contains(name) {
            check_data_sufficiency(name, &cond.cases, line, errors);
        }
        // 3. Check Units for Numeric Variables
        if numeric_vars.contains(name) {
             for case in &cond.cases {
                 check_selector_units(name, &case.condition, case.line, errors);
             }
        }
    }

    // Reachability Analysis
    for case in &cond.cases {
         // Check against parent constraints
         if let Some(name) = &target_name {
             if let Some((min_c, max_c)) = constraints.get(name) {
                 // Determine case range
                 let (min_case, max_case) = resolve_selector_range(&case.condition);
                 
                 // If case is completely outside [min_c, max_c], it's unreachable
                 // Intersection: [max(min_c, min_case), min(max_c, max_case)]
                 let start = min_c.max(min_case);
                 let end = max_c.min(max_case);
                 
                 if start > end {
                     errors.push(crate::domain::EngineError {
                         message: format!(
                             "Unreachable Code: Case for '{}' covers range {}...{}, effectively disjoint from constrained context {}...{}.",
                             name, min_case, max_case, min_c, max_c
                         ),
                         line: case.line,
                         column: 0
                     });
                 }
             }
         }
         
         // Clone constraints for child block
         let mut child_constraints = constraints.clone();
         
         // Narrow constraints if applicable
         if let Some(name) = &target_name {
             let (min_case, max_case) = resolve_selector_range(&case.condition);
             // Union current with case? No, Intersection.
             // We are entering a block where 'name' IS in this range.
             // So new constraint is Intersection(old, case).
              if let Some((old_min, old_max)) = child_constraints.get(name) {
                  let new_min = old_min.max(min_case);
                  let new_max = old_max.min(max_case);
                  child_constraints.insert(name.clone(), (new_min, new_max));
              } else {
                  child_constraints.insert(name.clone(), (min_case, max_case));
              }
         }

        for s in &case.block {
            validate_statement(s, value_ranges, timeframe_vars, enum_vars, numeric_vars, &mut child_constraints, errors);
        }
    }
}

// Helper to resolve rough range from selector
fn resolve_selector_range(sel: &RangeSelector) -> (f64, f64) {
    let extract = |expr: &Expression| -> Option<f64> {
         if let Expression::Literal(lit) = expr {
             match lit {
                 Literal::Number(v, _) => Some(*v),
                 Literal::Quantity(v, _, _) => Some(*v),
                 _ => None
             }
         } else { None }
    };

    match sel {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
             let vn = extract(min).unwrap_or(f64::NEG_INFINITY);
             let vx = extract(max).unwrap_or(f64::INFINITY);
             (vn, vx)
        }
        RangeSelector::Equals(expr) => {
             let v = extract(expr).unwrap_or(f64::NAN);
             if v.is_nan() { (f64::NEG_INFINITY, f64::INFINITY) } else { (v, v) }
        }
        RangeSelector::Condition(op, expr) => {
             let v = extract(expr).unwrap_or(0.0);
             match op {
                 crate::ast::ConditionOperator::GreaterThan => (v + 0.00001, f64::INFINITY),
                 crate::ast::ConditionOperator::GreaterThanOrEquals => (v, f64::INFINITY),
                 crate::ast::ConditionOperator::LessThan => (f64::NEG_INFINITY, v - 0.00001),
                 crate::ast::ConditionOperator::LessThanOrEquals => (f64::NEG_INFINITY, v),
                 _ => (f64::NEG_INFINITY, f64::INFINITY)
             }
        }
        _ => (f64::NEG_INFINITY, f64::INFINITY)
    }
}

fn check_data_sufficiency(
    name: &str,
    cases: &[crate::ast::AssessmentCase],
    line: usize,
    errors: &mut Vec<crate::domain::EngineError>,
) {
    let has_not_enough_data = cases.iter().any(|c| matches!(c.condition, RangeSelector::NotEnoughData));

    if !has_not_enough_data {
        errors.push(crate::domain::EngineError {
            message: format!(
                "Missing Case: Assessment for '{}' depends on a timeframe calculation but does not handle 'Not enough data'.",
                name
            ),
            line,
            column: 0
        });
    }
}

fn check_coverage(
    name: &str,
    valid: &[RangeSelector],
    cases: &[crate::ast::AssessmentCase],
    line: usize,
    errors: &mut Vec<crate::domain::EngineError>,
) {
    // 1. Determine Universe [min, max] and Precision
    let mut universe_min = f64::NEG_INFINITY;
    let mut universe_max = f64::INFINITY;
    let mut has_range = false;
    let mut max_precision: Option<usize> = None;

    let extract_val_prec = |expr: &Expression| -> Option<(f64, Option<usize>)> {
        if let Expression::Literal(lit) = expr {
             match lit {
                 Literal::Number(v, p) => Some((*v, *p)),
                 Literal::Quantity(v, _, p) => Some((*v, *p)),
                 _ => None
             }
        } else { None }
    };

    for sel in valid {
        match sel {
             RangeSelector::Range(min_expr, max_expr) | RangeSelector::Between(min_expr, max_expr) => {
                 if let (Some((min, p1)), Some((max, p2))) = (extract_val_prec(min_expr), extract_val_prec(max_expr)) {
                     universe_min = min;
                     universe_max = max;
                     has_range = true;
                     if let Some(p) = p1 { max_precision = std::cmp::max(max_precision, Some(p)); }
                     if let Some(p) = p2 { max_precision = std::cmp::max(max_precision, Some(p)); }
                     break; 
                 }
             }
             _ => {}
        }
    }

    if !has_range {
        return;
    }

    // println!("DEBUG: Checking coverage for {}, Universe: {}...{}", name, universe_min, universe_max);

    let step = if let Some(p) = max_precision {

        if p == 0 { 1.0 } else { 10f64.powi(-(p as i32)) }
    } else {
        1.0 // Implicit integer
    };

    // 2. Collect Intervals from Cases
    struct Interval { start: f64, end: f64 }
    let mut intervals: Vec<Interval> = Vec::new();

    for case in cases {
        match &case.condition {
            RangeSelector::Range(min_expr, max_expr) | RangeSelector::Between(min_expr, max_expr) => {
                if let (Some((min, _)), Some((max, _))) = (extract_val_prec(min_expr), extract_val_prec(max_expr)) {
                    intervals.push(Interval { start: min, end: max });
                }
            }
            RangeSelector::Condition(op, expr) => {
                if let Some((v, _)) = extract_val_prec(expr) {
                    match op {
                        crate::ast::ConditionOperator::GreaterThan => intervals.push(Interval { start: v, end: f64::INFINITY }),
                        crate::ast::ConditionOperator::GreaterThanOrEquals => intervals.push(Interval { start: v, end: f64::INFINITY }),
                        crate::ast::ConditionOperator::LessThan => intervals.push(Interval { start: f64::NEG_INFINITY, end: v }),
                        crate::ast::ConditionOperator::LessThanOrEquals => intervals.push(Interval { start: f64::NEG_INFINITY, end: v }),
                        _ => {}
                    }
                }
            }
            RangeSelector::Equals(expr) => {
                if let Some((v, _)) = extract_val_prec(expr) {
                    intervals.push(Interval { start: v, end: v });
                }
            }
            _ => {}
        }
    }

    // 3. Clamp Intervals to Universe
    let mut clamped: Vec<Interval> = Vec::new();
    for i in intervals {
        let start = i.start.max(universe_min);
        let end = i.end.min(universe_max);
        if start <= end { 
             clamped.push(Interval { start, end });
        }
    }

    // 4. Sort and Sweep
    clamped.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));

    let mut current = universe_min;
    let mut is_first = true;
    let epsilon = 0.00001; 

    // Helper to format based on precision
    let format_val = |v: f64| -> String {
        if let Some(p) = max_precision {
            format!("{:.1$}", v, p)
        } else {
            // Integer default
            format!("{:.0}", v)
        }
    };

    for i in clamped {
        // Overlap Check 
        if !is_first {
            if i.start < current + step - epsilon {
                 errors.push(crate::domain::EngineError {
                    message: format!(
                    "Constraint Violation: Assessment for '{}' has overlapping or ambiguous ranges. Value {} is covered multiple times. Next range should start at {}.",
                    name, format_val(i.start), format_val(current + step)
                    ),
                    line,
                    column: 0
                });
            }
        }

        // Check for gap
        if i.start > current + epsilon {
            let gap_size = i.start - current;
            let is_valid_step = (gap_size - step).abs() < epsilon;
            
            if !is_valid_step {
                errors.push(crate::domain::EngineError {
                    message: format!(
                    "Constraint Violation: Assessment for '{}' is incomplete. Uncovered gap detected: {} ... {}. Valid range is {}...{}.",
                    name, format_val(current + step), format_val(i.start), format_val(universe_min), format_val(universe_max)
                    ),
                    line,
                    column: 0
                });
            } else {
                // Bridge the gap
                current = i.end; 
            }
        }
        // Extend current reach
        if i.end > current {
            current = i.end;
        }
        is_first = false;
    }

    // Check end gap
    if current < universe_max - epsilon {
         let start_gap = current + step;
         let end_gap = universe_max;
         
        errors.push(crate::domain::EngineError {
            message: format!(
                "Constraint Violation: Assessment for '{}' is incomplete. Uncovered gap at the end: {} ... {}. Valid range is {}...{}.",
                name, format_val(start_gap), format_val(end_gap), format_val(universe_min), format_val(universe_max)
            ),
            line,
            column: 0
        });
    }
}

fn check_numeric_units(name: &str, props: &[Property], errors: &mut Vec<crate::domain::EngineError>) {
    for prop in props {
        if let Property::ValidValues(stmts) = prop {
            for stmt in stmts {
                if let StatementKind::EventProgression(_, cases) = &stmt.kind {
                    for case in cases {
                        check_selector_units(name, &case.condition, stmt.line, errors);
                    }
                } else if let StatementKind::Constraint(_, _, sel) = &stmt.kind {
                     check_selector_units(name, sel, stmt.line, errors);
                }
            }
        }
    }
}

fn check_selector_units(name: &str, sel: &RangeSelector, line: usize, errors: &mut Vec<crate::domain::EngineError>) {
    match sel {
        RangeSelector::Range(min, max) => {
            check_expression_unit(name, min, line, errors);
            check_expression_unit(name, max, line, errors);
        }
        RangeSelector::Between(min, max) => {
             check_expression_unit(name, min, line, errors);
             check_expression_unit(name, max, line, errors);
        }
        RangeSelector::Condition(_, expr) => check_expression_unit(name, expr, line, errors),
        RangeSelector::Equals(expr) => check_expression_unit(name, expr, line, errors),
        RangeSelector::List(exprs) => {
             for e in exprs { check_expression_unit(name, e, line, errors); }
        }
        _ => {}
    }
}

fn check_expression_unit(name: &str, expr: &Expression, line: usize, errors: &mut Vec<crate::domain::EngineError>) {
    if let Expression::Literal(lit) = expr {
        if let Literal::Number(..) = lit {
             errors.push(crate::domain::EngineError {
                 message: format!("Validation Error: Numeric values must have a unit in '{}'.", name),
                 line, 
                 column: 0
             });
        }
    }
}
