mod intervals;
mod semantics; 
mod data_flow;
mod coverage;

use crate::ast::{Plan, Definition, Property, RangeSelector, Expression, Literal, Statement, StatementKind, ConditionalTarget};
use crate::domain::EngineError;
use std::collections::{HashMap, HashSet};

pub use intervals::{Interval, calculate_interval};

#[derive(Copy, Clone)]
struct PrecisionInfo {
    decimals: Option<usize>,
}

impl PrecisionInfo {
    fn new() -> Self {
        PrecisionInfo { decimals: None }
    }
}

pub fn validate_file(plan: &Plan) -> Result<(), Vec<EngineError>> {
    let mut errors = Vec::new(); // Note: legacy->semantics calls below
    // ...

    let mut value_ranges = HashMap::new();
    let mut value_bounds = HashMap::new();
    let mut value_precision = HashMap::new();
    let mut defined_values = HashSet::new();
    let mut valid_units = HashSet::new();
    let mut enum_vars = HashSet::new();
    
    // Definitions Map for Lookups
    let mut defs_map = HashMap::new();
    for def in &plan.definitions {
         match def {
            Definition::Value(v) => { defs_map.insert(v.name.clone(), Definition::Value(v.clone())); },
            Definition::Drug(d) => { defs_map.insert(d.name.clone(), Definition::Drug(d.clone())); },
            Definition::Addressee(a) => { defs_map.insert(a.name.clone(), Definition::Addressee(a.clone())); },
            Definition::Unit(u) => { defs_map.insert(u.name.clone(), Definition::Unit(u.clone())); },
            Definition::Period(p) => { defs_map.insert(p.name.clone(), Definition::Period(p.clone())); },
            Definition::Context(c) => { defs_map.insert("context".to_string(), Definition::Context(c.clone())); }, // Single context?
            Definition::Plan(p) => { defs_map.insert(p.name.clone(), Definition::Plan(p.clone())); },
        }
    }

    // 0. Collect Units
    for def in &plan.definitions {
        if let Definition::Unit(ud) = def {
            valid_units.insert(ud.name.clone());
            valid_units.extend(ud.plurals.clone());
            valid_units.extend(ud.singulars.clone());
        }
    }

    // 1. Collect Variables and Base Intervals
    for def in &plan.definitions {
        if let Definition::Value(vd) = def {
            defined_values.insert(vd.name.clone());
            
            if let crate::domain::ValueType::Enumeration = vd.value_type {
                enum_vars.insert(vd.name.clone());
            }

            if matches!(vd.value_type, crate::domain::ValueType::Number) {
                let mut ranges: Vec<Interval> = Vec::new();
                let mut precision = PrecisionInfo::new();

                // Try to narrow down from Valid Values
                for prop in &vd.properties {
                    if let Property::ValidValues(stmts) = prop {
                        for stmt in stmts {
                             if let crate::ast::StatementKind::Constraint(_, _, sel) = &stmt.kind {
                                 update_precision_from_selector(sel, &mut precision);
                                 ranges.extend(extract_selector_intervals(sel));
                             } else if let crate::ast::StatementKind::EventProgression(_, cases) = &stmt.kind {
                                 // Also check progression cases
                                 for case in cases {
                                     update_precision_from_selector(&case.condition, &mut precision);
                                     ranges.extend(extract_selector_intervals(&case.condition));
                                 }
                             }
                        }
                    }
                }
                let bounds = if ranges.is_empty() {
                    Interval::unbounded()
                } else {
                    let mut min_all = f64::INFINITY;
                    let mut max_all = f64::NEG_INFINITY;
                    for r in &ranges {
                        min_all = min_all.min(r.min);
                        max_all = max_all.max(r.max);
                    }
                    Interval::new(min_all, max_all)
                };
                value_ranges.insert(vd.name.clone(), ranges);
                value_bounds.insert(vd.name.clone(), bounds);
                value_precision.insert(vd.name.clone(), precision);
            }
        }
    }
    
    // Legacy structural checks renamed to Semantics
    semantics::check_unit_definitions(&defs_map, &mut errors);
    semantics::check_drugs(&defs_map, &valid_units, &mut errors);
    semantics::check_addressees(&defs_map, &mut errors);
    semantics::check_value_definitions(&defs_map, &mut errors);
    semantics::check_timeframe_period_references(&defs_map, &mut errors);

    // 1.5 Meaning coverage for value definitions
    for def in &plan.definitions {
        if let Definition::Value(vd) = def {
            for prop in &vd.properties {
                if let Property::Meaning(cases) = prop {
                    let line = cases.first().map(|case| case.line).unwrap_or(0);
                    if enum_vars.contains(&vd.name) {
                        let mut valid_strings = Vec::new();
                        for prop in &vd.properties {
                            if let Property::ValidValues(stmts) = prop {
                                for stmt in stmts {
                                    if let crate::ast::StatementKind::Constraint(_, _, sel) = &stmt.kind {
                                        extract_defined_strings(sel, &mut valid_strings);
                                    } else if let crate::ast::StatementKind::EventProgression(_, cases) = &stmt.kind {
                                        for case in cases {
                                            extract_defined_strings(&case.condition, &mut valid_strings);
                                        }
                                    }
                                }
                            }
                        }

                        let valid_refs: Vec<&str> = valid_strings.iter().map(|s| s.as_str()).collect();
                        if !valid_refs.is_empty() {
                            coverage::check_string_coverage(&vd.name, cases, &valid_refs, line, &mut errors);
                        }
                    } else if let Some(valid_ranges) = value_ranges.get(&vd.name) {
                        if cases.iter().any(|case| selector_has_unitless_number(&case.condition)) {
                            errors.push(EngineError {
                                message: format!(
                                    "Numeric values must have a unit. Meaning for '{}' uses unitless numbers.",
                                    vd.name
                                ),
                                line,
                                column: 0,
                            });
                        }

                        let decimals = value_precision.get(&vd.name).and_then(|info| info.decimals);
                        coverage::check_coverage(&vd.name, valid_ranges, cases, line, decimals, &mut errors);
                    }
                }
            }
        }
    }

    let statistical_values = collect_statistical_values(plan);

    for def in &plan.definitions {
        if let Definition::Value(vd) = def {
            for prop in &vd.properties {
                if let Property::Calculation(stmts) = prop {
                    for stmt in stmts {
                        check_statistical_functions_require_timeframe(stmt, false, &mut errors);
                    }
                }
            }
        }
    }

    // 2. Validate Assignments and Expressions using Intervals AND Data Flow
    for def in &plan.definitions {
        match def {
            Definition::Plan(pd) => {
                 let mut plan_start_state = data_flow::FlowState::new();
                 let mut start_flags = Vec::with_capacity(pd.blocks.len());

                 for block in &pd.blocks {
                      if is_plan_start_block(&pd.name, block) {
                           let statements = match block {
                               crate::ast::PlanBlock::DuringPlan(s) => s,
                               crate::ast::PlanBlock::Trigger(t) => &t.statements,
                               crate::ast::PlanBlock::Event(e) => &e.statements,
                           };
                           plan_start_state =
                               data_flow::analyze_block(statements, &plan_start_state, &defs_map, &mut errors);
                           start_flags.push(true);
                      } else {
                           start_flags.push(false);
                      }
                 }

                 for (block_index, block) in pd.blocks.iter().enumerate() {
                      let statements = match block {
                          crate::ast::PlanBlock::DuringPlan(s) => s,
                          crate::ast::PlanBlock::Event(e) => &e.statements,
                          crate::ast::PlanBlock::Trigger(t) => &t.statements,
                      };
                      
                      // Run Data Flow Analysis for this block
                      if !start_flags[block_index] {
                          data_flow::analyze_block(statements, &plan_start_state, &defs_map, &mut errors);
                      }
                      
                      for stmt in statements {
                          // Run semantic checks (undefined vars, etc)
                          semantics::check_statement_semantics(stmt, &enum_vars, &defs_map, &mut errors);
                          check_statistical_not_enough_data(stmt, &statistical_values, &mut errors);
                          check_statistical_functions_require_timeframe(stmt, false, &mut errors);

                          match &stmt.kind {
                              crate::ast::StatementKind::Assignment(assign) => {
                                  let target_ranges = value_ranges.get(&assign.target);
                                  
                                  if let Some(ranges) = target_ranges {
                                      let expr_intervals = intervals::calculate_interval_set(&assign.expression, &value_ranges);
                                      
                                      if !intervals_within_ranges(&expr_intervals, ranges) {
                                          errors.push(EngineError {
                                              message: format!(
                                                  "Assignment Validity Warning: Value for '{}' may be out of bounds. Expression result ranges ({}) are not fully contained in valid ranges.",
                                                  assign.target,
                                                  format_intervals(&expr_intervals)
                                              ),
                                              line: stmt.line,
                                              column: 0
                                          });
                                      }
                                  } 
                                  
                                  // Check for subtraction safety
                                  if let Expression::Binary(l, op, r) = &assign.expression {
                                      if op == "-" {
                                          if let Some(msg) = intervals::check_subtraction_safety(l, r, &value_bounds) {
                                               errors.push(EngineError {
                                                  message: msg,
                                                  line: stmt.line,
                                                  column: 0
                                              });
                                          }
                                      }
                                  }
                              },
                              crate::ast::StatementKind::Conditional(cond) => {
                                  // Coverage Analysis
                                  match &cond.condition {
                                      crate::ast::ConditionalTarget::Expression(Expression::Variable(name)) => {
                                          if enum_vars.contains(name) {
                                              // Check Enum Coverage
                                              if let Some(Definition::Value(vd)) = defs_map.get(name) {
                                                   let mut valid_strings = Vec::new();
                                                   for prop in &vd.properties {
                                                       if let Property::ValidValues(stmts) = prop {
                                                           for stmt in stmts {
                                                               // Constraints are usually RangeSelectors.
                                                               // For Enums, we expect string literals in RangeSelector::List or Equals?
                                                               // Actually, `valid values: "Red", "Green"` parses as `Expression::Literal(String)`.
                                                               // And wrapped in `Constraint(..., selector)`.
                                                               if let crate::ast::StatementKind::Constraint(_, _, sel) = &stmt.kind {
                                                                    extract_defined_strings(sel, &mut valid_strings);
                                                               }
                                                           }
                                                       }
                                                   }
                                                   
                                                   let valid_strs_refs: Vec<&str> = valid_strings.iter().map(|s| s.as_str()).collect();
                                                   coverage::check_string_coverage(
                                                       name, 
                                                       &cond.cases, 
                                                       &valid_strs_refs, 
                                                       stmt.line, 
                                                       &mut errors
                                                   );
                                              }
                                          } else {
                                              // Numeric Coverage
                                              if let Some(valid_ranges) = value_ranges.get(name) {
                                                  if cond.cases.iter().any(|case| selector_has_unitless_number(&case.condition)) {
                                                      errors.push(EngineError {
                                                          message: format!("Numeric values must have a unit. Assessment for '{}' uses unitless numbers.", name),
                                                          line: stmt.line,
                                                          column: 0,
                                                      });
                                                  }

                                                  let decimals = value_precision.get(name).and_then(|info| info.decimals);
                                                  coverage::check_coverage(name, valid_ranges, &cond.cases, stmt.line, decimals, &mut errors);
                                              }
                                          }
                                      },
                                      crate::ast::ConditionalTarget::Expression(Expression::Statistical(crate::ast::StatisticalFunc::TrendOf(name))) => {
                                          coverage::check_string_coverage(
                                              &format!("trend of {}", name),
                                              &cond.cases,
                                              &["increase", "decrease", "stable"],
                                              stmt.line,
                                              &mut errors
                                          );
                                      },
                                      _ => {}
                                  }
                              },
                              _ => {}
                          }
                      }
                 }
            }
            _ => {}
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        errors.dedup();
        Err(errors)
    }
}

fn collect_statistical_values(plan: &Plan) -> HashSet<String> {
    let mut values = HashSet::new();
    for def in &plan.definitions {
        match def {
            Definition::Value(vd) => {
                if value_depends_on_statistical(vd) {
                    values.insert(vd.name.clone());
                }
            }
            Definition::Plan(pd) => {
                for block in &pd.blocks {
                    let statements = match block {
                        crate::ast::PlanBlock::DuringPlan(s) => s,
                        crate::ast::PlanBlock::Event(e) => &e.statements,
                        crate::ast::PlanBlock::Trigger(t) => &t.statements,
                    };
                    collect_statistical_assignments(statements, &mut values);
                }
            }
            _ => {}
        }
    }
    values
}

fn value_depends_on_statistical(vd: &crate::ast::ValueDef) -> bool {
    for prop in &vd.properties {
        if let Property::Calculation(stmts) = prop {
            if calculation_contains_statistical(stmts) {
                return true;
            }
        }
    }
    false
}

fn calculation_contains_statistical(stmts: &[Statement]) -> bool {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::Assignment(assign) => {
                if expression_contains_statistical(&assign.expression) {
                    return true;
                }
            }
            StatementKind::Timeframe(tb) => {
                if calculation_contains_statistical(&tb.block) {
                    return true;
                }
            }
            StatementKind::ContextBlock(cb) => {
                if calculation_contains_statistical(&cb.statements) {
                    return true;
                }
            }
            StatementKind::Conditional(cond) => {
                for case in &cond.cases {
                    if calculation_contains_statistical(&case.block) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

fn collect_statistical_assignments(stmts: &[Statement], values: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::Assignment(assign) => {
                if expression_contains_statistical(&assign.expression) {
                    values.insert(assign.target.clone());
                }
            }
            StatementKind::Timeframe(tb) => collect_statistical_assignments(&tb.block, values),
            StatementKind::ContextBlock(cb) => collect_statistical_assignments(&cb.statements, values),
            StatementKind::Conditional(cond) => {
                for case in &cond.cases {
                    collect_statistical_assignments(&case.block, values);
                }
            }
            _ => {}
        }
    }
}

fn check_statistical_not_enough_data(
    stmt: &Statement,
    statistical_values: &HashSet<String>,
    errors: &mut Vec<EngineError>,
) {
    match &stmt.kind {
        StatementKind::Conditional(cond) => {
            let mut is_statistical_target = false;
            if let ConditionalTarget::Expression(expr) = &cond.condition {
                if let Expression::Variable(name) = expr {
                    if statistical_values.contains(name) {
                        is_statistical_target = true;
                    }
                }
                if expression_contains_statistical(expr) {
                    is_statistical_target = true;
                }
            }

            let has_not_enough = cond
                .cases
                .iter()
                .any(|case| matches!(case.condition, RangeSelector::NotEnoughData));

            if is_statistical_target && !has_not_enough {
                errors.push(EngineError {
                    message: "Assessment of statistical results must handle 'Not enough data'."
                        .to_string(),
                    line: stmt.line,
                    column: 0,
                });
            }

            if !is_statistical_target && has_not_enough {
                errors.push(EngineError {
                    message:
                        "Not enough data is only allowed when assessing statistical results."
                            .to_string(),
                    line: stmt.line,
                    column: 0,
                });
            }

            for case in &cond.cases {
                for nested in &case.block {
                    check_statistical_not_enough_data(nested, statistical_values, errors);
                }
            }
        }
        StatementKind::Timeframe(tb) => {
            for nested in &tb.block {
                check_statistical_not_enough_data(nested, statistical_values, errors);
            }
        }
        StatementKind::ContextBlock(cb) => {
            for nested in &cb.statements {
                check_statistical_not_enough_data(nested, statistical_values, errors);
            }
        }
        _ => {}
    }
}

fn expression_contains_statistical(expr: &Expression) -> bool {
    match expr {
        Expression::Statistical(_) => true,
        Expression::Binary(left, _op, right) => {
            expression_contains_statistical(left) || expression_contains_statistical(right)
        }
        Expression::DateDiff(_, start, end) => {
            expression_contains_statistical(start) || expression_contains_statistical(end)
        }
        Expression::FunctionCall(_, args) => args.iter().any(expression_contains_statistical),
        Expression::InterpolatedString(parts) => parts.iter().any(expression_contains_statistical),
        _ => false,
    }
}

fn selector_contains_statistical(sel: &RangeSelector) -> bool {
    match sel {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
            expression_contains_statistical(min) || expression_contains_statistical(max)
        }
        RangeSelector::Equals(expr) => expression_contains_statistical(expr),
        RangeSelector::Comparison(left, _op, right) => {
            expression_contains_statistical(left) || expression_contains_statistical(right)
        }
        RangeSelector::Condition(_op, expr) => expression_contains_statistical(expr),
        RangeSelector::List(items) => items.iter().any(expression_contains_statistical),
        _ => false,
    }
}

fn check_statistical_functions_require_timeframe(
    stmt: &Statement,
    has_timeframe_context: bool,
    errors: &mut Vec<EngineError>,
) {
    match &stmt.kind {
        StatementKind::Timeframe(block) => {
            let next_context = has_timeframe_context || block.for_analysis;
            for nested in &block.block {
                check_statistical_functions_require_timeframe(nested, next_context, errors);
            }
        }
        StatementKind::ContextBlock(cb) => {
            let has_timeframe_item = cb
                .items
                .iter()
                .any(|item| matches!(item, crate::ast::ContextItem::Timeframe(_)));
            let next_context = has_timeframe_context || has_timeframe_item;
            for nested in &cb.statements {
                check_statistical_functions_require_timeframe(nested, next_context, errors);
            }
        }
        StatementKind::Assignment(assign) => {
            if expression_contains_statistical(&assign.expression) && !has_timeframe_context {
                errors.push(EngineError {
                    message:
                        "Statistical functions require an analysis timeframe context.".to_string(),
                    line: stmt.line,
                    column: 0,
                });
            }
        }
        StatementKind::Conditional(cond) => {
            if let ConditionalTarget::Expression(expr) = &cond.condition {
                if expression_contains_statistical(expr) && !has_timeframe_context {
                    errors.push(EngineError {
                        message:
                            "Statistical functions require an analysis timeframe context."
                                .to_string(),
                        line: stmt.line,
                        column: 0,
                    });
                }
            }
            for case in &cond.cases {
                if selector_contains_statistical(&case.condition) && !has_timeframe_context {
                    errors.push(EngineError {
                        message:
                            "Statistical functions require an analysis timeframe context."
                                .to_string(),
                        line: stmt.line,
                        column: 0,
                    });
                }
                for nested in &case.block {
                    check_statistical_functions_require_timeframe(
                        nested,
                        has_timeframe_context,
                        errors,
                    );
                }
            }
        }
        StatementKind::Action(action) => {
            match action {
                crate::ast::Action::ShowMessage(exprs, _) => {
                    if exprs.iter().any(expression_contains_statistical) && !has_timeframe_context
                    {
                        errors.push(EngineError {
                            message:
                                "Statistical functions require an analysis timeframe context."
                                    .to_string(),
                            line: stmt.line,
                            column: 0,
                        });
                    }
                }
                crate::ast::Action::SendInfo(_, exprs) => {
                    if exprs.iter().any(expression_contains_statistical) && !has_timeframe_context
                    {
                        errors.push(EngineError {
                            message:
                                "Statistical functions require an analysis timeframe context."
                                    .to_string(),
                            line: stmt.line,
                            column: 0,
                        });
                    }
                }
                crate::ast::Action::MessageExpiration(sel) => {
                    if selector_contains_statistical(sel) && !has_timeframe_context {
                        errors.push(EngineError {
                            message:
                                "Statistical functions require an analysis timeframe context."
                                    .to_string(),
                            line: stmt.line,
                            column: 0,
                        });
                    }
                }
                _ => {}
            }
        }
        StatementKind::Constraint(expr, _op, sel) => {
            if (expression_contains_statistical(expr) || selector_contains_statistical(sel))
                && !has_timeframe_context
            {
                errors.push(EngineError {
                    message:
                        "Statistical functions require an analysis timeframe context.".to_string(),
                    line: stmt.line,
                    column: 0,
                });
            }
        }
        _ => {}
    }
}

fn is_plan_start_block(plan_name: &str, block: &crate::ast::PlanBlock) -> bool {
    match block {
        crate::ast::PlanBlock::DuringPlan(_) => true,
        crate::ast::PlanBlock::Trigger(t) => match &t.trigger {
            crate::ast::Trigger::StartOf(target) => target == plan_name,
            _ => false,
        },
        _ => false,
    }
}


fn extract_selector_intervals(sel: &RangeSelector) -> Vec<Interval> {
    let mut intervals = Vec::new();
    let get_val = |e: &Expression| match e {
        Expression::Literal(Literal::Number(n, _)) => Some(*n),
        Expression::Literal(Literal::Quantity(n, _, _)) => Some(*n),
        _ => None,
    };

    match sel {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
            if let (Some(v1), Some(v2)) = (get_val(min), get_val(max)) {
                intervals.push(Interval::new(v1, v2));
            }
        }
        RangeSelector::Equals(expr) => {
            if let Some(v) = get_val(expr) {
                intervals.push(Interval::exact(v));
            }
        }
        _ => {}
    }

    intervals
}

fn intervals_within_ranges(interval_set: &[Interval], ranges: &[Interval]) -> bool {
    if ranges.is_empty() {
        return true;
    }

    let merged_ranges = intervals::merge_intervals(ranges);
    let merged_intervals = intervals::merge_intervals(interval_set);

    if merged_intervals.is_empty() {
        return true;
    }

    for interval in merged_intervals {
        let mut within_any = false;
        for range in &merged_ranges {
            if interval.min >= range.min && interval.max <= range.max {
                within_any = true;
                break;
            }
        }
        if !within_any {
            return false;
        }
    }

    true
}

fn format_intervals(interval_set: &[Interval]) -> String {
    let merged = intervals::merge_intervals(interval_set);
    if merged.is_empty() {
        return "unbounded".to_string();
    }

    let parts: Vec<String> = merged
        .iter()
        .map(|interval| format!("{:.1}..{:.1}", interval.min, interval.max))
        .collect();
    parts.join("; ")
}

fn update_precision_from_selector(sel: &RangeSelector, precision: &mut PrecisionInfo) {
    match sel {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
            update_precision_from_expr(min, precision);
            update_precision_from_expr(max, precision);
        }
        RangeSelector::Equals(expr) => update_precision_from_expr(expr, precision),
        RangeSelector::List(items) => {
            for item in items {
                update_precision_from_expr(item, precision);
            }
        }
        _ => {}
    }
}

fn update_precision_from_expr(expr: &Expression, precision: &mut PrecisionInfo) {
    match expr {
        Expression::Literal(Literal::Number(_, decimals)) => {
            apply_precision(decimals, precision);
        }
        Expression::Literal(Literal::Quantity(_, _, decimals)) => {
            apply_precision(decimals, precision);
        }
        _ => {}
    }
}

fn apply_precision(decimals: &Option<usize>, precision: &mut PrecisionInfo) {
    match decimals {
        Some(d) => {
            precision.decimals = Some(match precision.decimals {
                Some(current) => current.max(*d),
                None => *d,
            });
        }
        None => {
            if precision.decimals.is_none() {
                precision.decimals = Some(0);
            }
        }
    }
}

fn selector_has_unitless_number(sel: &RangeSelector) -> bool {
    match sel {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
            expr_has_unitless_number(min) || expr_has_unitless_number(max)
        }
        RangeSelector::Equals(expr) => expr_has_unitless_number(expr),
        RangeSelector::List(items) => items.iter().any(expr_has_unitless_number),
        _ => false,
    }
}

fn expr_has_unitless_number(expr: &Expression) -> bool {
    match expr {
        Expression::Literal(Literal::Number(_, _)) => true,
        Expression::Literal(Literal::Quantity(_, _, _)) => false,
        Expression::Binary(left, _, right) => {
            expr_has_unitless_number(left) || expr_has_unitless_number(right)
        }
        _ => false,
    }
}

fn extract_defined_strings(sel: &RangeSelector, out: &mut Vec<String>) {
    match sel {
        RangeSelector::Equals(expr) => {
            if let Expression::Literal(Literal::String(s)) = expr {
                out.push(s.clone());
            }
        },
        RangeSelector::List(items) => {
             for item in items {
                 if let Expression::Literal(Literal::String(s)) = item {
                     out.push(s.clone());
                 }
             }
        },
        _ => {}
    }
}
