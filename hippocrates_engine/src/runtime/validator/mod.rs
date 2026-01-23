mod intervals;
mod semantics; 
mod data_flow;
mod coverage;

use crate::ast::{Plan, Definition, Property, RangeSelector, Expression, Literal, Statement, StatementKind, ConditionalTarget};
use crate::domain::{EngineError, Unit};
use crate::runtime;
use crate::runtime::input_validation;
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
    let mut value_units = HashMap::new();
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

    let unit_map = build_unit_map(&plan.definitions);

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

            if let crate::domain::ValueType::Enumeration = vd.value_type {
                for prop in &vd.properties {
                    if let Property::ValidValues(stmts) = prop {
                        for stmt in stmts {
                            match &stmt.kind {
                                StatementKind::EventProgression(_, cases) => {
                                    for case in cases {
                                        if !enum_selector_is_identifier(&case.condition) {
                                            errors.push(EngineError {
                                                message: format!(
                                                    "Enumeration '{}' valid values must be identifiers (angle brackets).",
                                                    vd.name
                                                ),
                                                line: case.line,
                                                column: 0,
                                            });
                                        }
                                    }
                                }
                                StatementKind::Constraint(_, _, sel) => {
                                    if !enum_selector_is_identifier(sel) {
                                        errors.push(EngineError {
                                            message: format!(
                                                "Enumeration '{}' valid values must be identifiers (angle brackets).",
                                                vd.name
                                            ),
                                            line: stmt.line,
                                            column: 0,
                                        });
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if matches!(vd.value_type, crate::domain::ValueType::Number) {
                if let Some(unit) = expected_unit_for_value_def(vd, &unit_map) {
                    let normalized = runtime::normalize_identifier(&vd.name);
                    value_units.insert(normalized, unit);
                }

                if let Err(msg) = input_validation::precision_for_value(vd) {
                    let line = vd
                        .properties
                        .iter()
                        .find_map(|prop| match prop {
                            Property::ValidValues(stmts) => stmts.first().map(|stmt| stmt.line),
                            _ => None,
                        })
                        .unwrap_or(0);
                    errors.push(EngineError {
                        message: format!("Precision mismatch for '{}': {}", vd.name, msg),
                        line,
                        column: 0,
                    });
                }

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

    // 1.25 Calculation assignments must match unit + precision of the value definition.
    for def in &plan.definitions {
        if let Definition::Value(vd) = def {
            for prop in &vd.properties {
                if let Property::Calculation(stmts) = prop {
                    check_assignment_units_precision_block(
                        stmts,
                        Some(&vd.name),
                        &value_units,
                        &value_precision,
                        &unit_map,
                        &mut errors,
                    );
                }
            }
        }
    }

    // 1.5 Valid value ranges must not overlap
    for def in &plan.definitions {
        if let Definition::Value(vd) = def {
            let line = vd
                .properties
                .iter()
                .find_map(|prop| match prop {
                    Property::ValidValues(stmts) => stmts.first().map(|stmt| stmt.line),
                    _ => None,
                })
                .unwrap_or(0);

            match vd.value_type {
                crate::domain::ValueType::Number => {
                    let ranges = value_ranges.get(&vd.name).map(Vec::as_slice).unwrap_or(&[]);
                    if ranges.len() < 2 {
                        continue;
                    }
                    let decimals = value_precision.get(&vd.name).and_then(|info| info.decimals);
                    check_valid_values_overlap(&vd.name, ranges, decimals, line, &mut errors);
                }
                crate::domain::ValueType::DateTime | crate::domain::ValueType::TimeIndication => {
                    check_valid_values_datetime_overlap(vd, line, &mut errors);
                }
                _ => {}
            }
        }
    }

    // 1.6 Meaning coverage for value definitions
    for def in &plan.definitions {
        if let Definition::Value(vd) = def {
            for prop in &vd.properties {
                if let Property::Meaning(meaning_def) = prop {
                    let cases = &meaning_def.cases;
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

                    if !meaning_def.valid_meanings.is_empty() {
                        validate_meaning_labels(
                            &vd.name,
                            meaning_def,
                            line,
                            &mut errors,
                        );
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
                          check_assignment_units_precision_stmt(
                              stmt,
                              None,
                              &value_units,
                              &value_precision,
                              &unit_map,
                              &mut errors,
                          );

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
        RangeSelector::List(items) => {
            for item in items {
                if let Some(v) = get_val(item) {
                    intervals.push(Interval::exact(v));
                }
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

fn check_valid_values_overlap(
    name: &str,
    ranges: &[Interval],
    decimals: Option<usize>,
    line: usize,
    errors: &mut Vec<EngineError>,
) {
    if ranges.len() < 2 {
        return;
    }

    let mut sorted = ranges.to_vec();
    sorted.sort_by(|a, b| a.min.partial_cmp(&b.min).unwrap_or(std::cmp::Ordering::Equal));

    let decimals = decimals.unwrap_or(0);
    let step = 10_f64.powi(-(decimals as i32));
    let epsilon = step / 10.0;
    let format_val = |v: f64| -> String { format!("{:.*}", decimals, v) };

    let mut current = sorted[0].clone();
    for range in sorted.into_iter().skip(1) {
        if range.min <= current.max + epsilon {
            errors.push(EngineError {
                message: format!(
                    "Constraint Violation: Valid values for '{}' have overlapping ranges ({} ... {} overlaps {} ... {}).",
                    name,
                    format_val(current.min),
                    format_val(current.max),
                    format_val(range.min),
                    format_val(range.max)
                ),
                line,
                column: 0,
            });
            return;
        }
        if range.max > current.max {
            current = range;
        } else {
            current = range;
        }
    }
}

fn check_valid_values_datetime_overlap(
    vd: &crate::ast::ValueDef,
    line: usize,
    errors: &mut Vec<EngineError>,
) {
    let selectors = collect_valid_value_selectors(vd);
    if selectors.is_empty() {
        return;
    }

    let base = chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let mut date_ranges: Vec<(chrono::NaiveDateTime, chrono::NaiveDateTime)> = Vec::new();
    let mut time_ranges: Vec<(chrono::NaiveTime, chrono::NaiveTime)> = Vec::new();

    for selector in selectors {
        extract_datetime_ranges(selector, base, &mut date_ranges, &mut time_ranges, line, errors);
    }

    if date_ranges.len() > 1 {
        check_datetime_ranges_overlap(&vd.name, &date_ranges, line, errors);
    }
    if time_ranges.len() > 1 {
        check_time_ranges_overlap(&vd.name, &time_ranges, line, errors);
    }
}

fn collect_valid_value_selectors(vd: &crate::ast::ValueDef) -> Vec<&RangeSelector> {
    let mut selectors = Vec::new();
    for prop in &vd.properties {
        if let Property::ValidValues(stmts) = prop {
            for stmt in stmts {
                match &stmt.kind {
                    StatementKind::Constraint(_, _, sel) => selectors.push(sel),
                    StatementKind::EventProgression(_, cases) => {
                        for case in cases {
                            selectors.push(&case.condition);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    selectors
}

fn extract_datetime_ranges(
    selector: &RangeSelector,
    base: chrono::NaiveDateTime,
    date_ranges: &mut Vec<(chrono::NaiveDateTime, chrono::NaiveDateTime)>,
    time_ranges: &mut Vec<(chrono::NaiveTime, chrono::NaiveTime)>,
    line: usize,
    errors: &mut Vec<EngineError>,
) {
    match selector {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
            match (parse_time_of_day(min), parse_time_of_day(max)) {
                (Some(start), Some(end)) => {
                    time_ranges.push((start, end));
                    return;
                }
                (None, None) => {}
                _ => {
                    errors.push(EngineError {
                        message: "Valid value ranges must use consistent date/time or time-of-day bounds."
                            .to_string(),
                        line,
                        column: 0,
                    });
                    return;
                }
            }

            match (parse_datetime_expr(min, base), parse_datetime_expr(max, base)) {
                (Some(start), Some(end)) => {
                    if start > end {
                        errors.push(EngineError {
                            message: "Date/time ranges must have start before end.".to_string(),
                            line,
                            column: 0,
                        });
                    } else {
                        date_ranges.push((start, end));
                    }
                }
                _ => {
                    errors.push(EngineError {
                        message: "Valid value ranges must use date/time literals or relative times."
                            .to_string(),
                        line,
                        column: 0,
                    });
                }
            }
        }
        RangeSelector::Equals(expr) => {
            if let Some(time) = parse_time_of_day(expr) {
                time_ranges.push((time, time));
                return;
            }
            if let Some(dt) = parse_datetime_expr(expr, base) {
                date_ranges.push((dt, dt));
                return;
            }
            errors.push(EngineError {
                message: "Valid value ranges must use date/time literals or time-of-day values."
                    .to_string(),
                line,
                column: 0,
            });
        }
        RangeSelector::List(items) => {
            for item in items {
                extract_datetime_ranges(&RangeSelector::Equals(item.clone()), base, date_ranges, time_ranges, line, errors);
            }
        }
        _ => {}
    }
}

fn parse_datetime_expr(
    expr: &Expression,
    base: chrono::NaiveDateTime,
) -> Option<chrono::NaiveDateTime> {
    match expr {
        Expression::Literal(Literal::Date(value)) => parse_date_time_literal(value),
        Expression::RelativeTime(val, unit, dir) => {
            let amount = *val;
            let sign = match dir {
                crate::ast::RelativeDirection::Ago => -1.0,
                crate::ast::RelativeDirection::FromNow => 1.0,
            };
            let value = amount * sign;
            match unit {
                Unit::Second => Some(base + chrono::Duration::seconds(value as i64)),
                Unit::Minute => Some(base + chrono::Duration::minutes(value as i64)),
                Unit::Hour => Some(base + chrono::Duration::hours(value as i64)),
                Unit::Day => Some(base + chrono::Duration::days(value as i64)),
                Unit::Week => Some(base + chrono::Duration::weeks(value as i64)),
                Unit::Month | Unit::Year => {
                    let months = if matches!(unit, Unit::Year) {
                        (value.trunc() as i32) * 12
                    } else {
                        value.trunc() as i32
                    };
                    Some(shift_months(base, months))
                }
                _ => None,
            }
        }
        Expression::Variable(name) if name == "now" => Some(base),
        _ => None,
    }
}

fn parse_date_time_literal(value: &str) -> Option<chrono::NaiveDateTime> {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M") {
        return Some(dt);
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %-H:%M") {
        return Some(dt);
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return date.and_hms_opt(0, 0, 0);
    }
    None
}

fn parse_time_of_day(expr: &Expression) -> Option<chrono::NaiveTime> {
    match expr {
        Expression::Literal(Literal::TimeOfDay(value)) => {
            chrono::NaiveTime::parse_from_str(value, "%H:%M")
                .or_else(|_| chrono::NaiveTime::parse_from_str(value, "%-H:%M"))
                .ok()
        }
        Expression::Literal(Literal::String(value)) => chrono::NaiveTime::parse_from_str(value, "%H:%M")
            .or_else(|_| chrono::NaiveTime::parse_from_str(value, "%-H:%M"))
            .ok(),
        _ => None,
    }
}

fn check_datetime_ranges_overlap(
    name: &str,
    ranges: &[(chrono::NaiveDateTime, chrono::NaiveDateTime)],
    line: usize,
    errors: &mut Vec<EngineError>,
) {
    if ranges.len() < 2 {
        return;
    }
    let mut sorted = ranges.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let mut current = sorted[0];
    for range in sorted.into_iter().skip(1) {
        if range.0 <= current.1 {
            errors.push(EngineError {
                message: format!(
                    "Constraint Violation: Valid values for '{}' have overlapping date/time ranges.",
                    name
                ),
                line,
                column: 0,
            });
            return;
        }
        if range.1 > current.1 {
            current = range;
        } else {
            current = range;
        }
    }
}

fn check_time_ranges_overlap(
    name: &str,
    ranges: &[(chrono::NaiveTime, chrono::NaiveTime)],
    line: usize,
    errors: &mut Vec<EngineError>,
) {
    use chrono::Timelike;
    if ranges.len() < 2 {
        return;
    }
    let mut intervals: Vec<(i32, i32)> = Vec::new();
    for (start, end) in ranges {
        let start_min = (start.num_seconds_from_midnight() / 60) as i32;
        let end_min = (end.num_seconds_from_midnight() / 60) as i32;
        if start_min <= end_min {
            intervals.push((start_min, end_min));
        } else {
            intervals.push((start_min, 24 * 60));
            intervals.push((0, end_min));
        }
    }

    intervals.sort_by(|a, b| a.0.cmp(&b.0));
    let mut current = intervals[0];
    for range in intervals.into_iter().skip(1) {
        if range.0 <= current.1 {
            errors.push(EngineError {
                message: format!(
                    "Constraint Violation: Valid values for '{}' have overlapping time-of-day ranges.",
                    name
                ),
                line,
                column: 0,
            });
            return;
        }
        if range.1 > current.1 {
            current = range;
        } else {
            current = range;
        }
    }
}

fn shift_months(base: chrono::NaiveDateTime, months: i32) -> chrono::NaiveDateTime {
    use chrono::Datelike;
    let date = base.date();
    let time = base.time();
    let (year, month) = (date.year(), date.month() as i32);
    let total = month - 1 + months;
    let new_year = year + total.div_euclid(12);
    let new_month = total.rem_euclid(12) + 1;
    let last_day = last_day_of_month(new_year, new_month as u32);
    let day = date.day().min(last_day);
    let new_date = chrono::NaiveDate::from_ymd_opt(new_year, new_month as u32, day)
        .unwrap_or(date);
    new_date.and_time(time)
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    use chrono::Datelike;
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_next = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
    let last = first_next - chrono::Duration::days(1);
    last.day()
}

fn build_unit_map(defs: &[Definition]) -> HashMap<String, Unit> {
    let mut unit_map = HashMap::new();
    for def in defs {
        if let Definition::Unit(ud) = def {
            let canonical = Unit::Custom(ud.name.clone());
            unit_map.insert(ud.name.clone(), canonical.clone());
            for name in &ud.singulars {
                unit_map.insert(name.clone(), canonical.clone());
            }
            for name in &ud.plurals {
                unit_map.insert(name.clone(), canonical.clone());
            }
            for name in &ud.abbreviations {
                unit_map.insert(name.clone(), canonical.clone());
            }
        }
    }
    unit_map
}

fn canonicalize_unit(unit: &Unit, unit_map: &HashMap<String, Unit>) -> Unit {
    match unit {
        Unit::Custom(name) => unit_map.get(name).cloned().unwrap_or_else(|| unit.clone()),
        _ => unit.clone(),
    }
}

fn expected_unit_for_value_def(vd: &crate::ast::ValueDef, unit_map: &HashMap<String, Unit>) -> Option<Unit> {
    if !matches!(vd.value_type, crate::domain::ValueType::Number) {
        return None;
    }

    for prop in &vd.properties {
        if let Property::Unit(unit) = prop {
            return Some(canonicalize_unit(unit, unit_map));
        }
    }

    for prop in &vd.properties {
        match prop {
            Property::ValidValues(stmts) => {
                if let Some(unit) = unit_from_statements(stmts, unit_map) {
                    return Some(unit);
                }
            }
            Property::Meaning(meaning_def) => {
                if let Some(unit) = unit_from_cases(&meaning_def.cases, unit_map) {
                    return Some(unit);
                }
            }
            _ => {}
        }
    }

    None
}

fn unit_from_statements(stmts: &[Statement], unit_map: &HashMap<String, Unit>) -> Option<Unit> {
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

fn unit_from_cases(cases: &[crate::ast::AssessmentCase], unit_map: &HashMap<String, Unit>) -> Option<Unit> {
    for case in cases {
        if let Some(unit) = unit_from_selector(&case.condition, unit_map) {
            return Some(unit);
        }
    }
    None
}

fn unit_from_selector(selector: &RangeSelector, unit_map: &HashMap<String, Unit>) -> Option<Unit> {
    match selector {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
            unit_from_expression(min, unit_map).or_else(|| unit_from_expression(max, unit_map))
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

fn unit_from_expression(expr: &Expression, unit_map: &HashMap<String, Unit>) -> Option<Unit> {
    match expr {
        Expression::Literal(Literal::Quantity(_, unit, _)) => {
            Some(canonicalize_unit(unit, unit_map))
        }
        _ => None,
    }
}

fn validate_meaning_labels(
    value_name: &str,
    meaning_def: &crate::ast::MeaningDef,
    line: usize,
    errors: &mut Vec<EngineError>,
) {
    let mut labels: Vec<(String, usize)> = Vec::new();
    let mut invalid_expr_lines = Vec::new();

    for case in &meaning_def.cases {
        collect_meaning_labels_from_block(&case.block, &mut labels, &mut invalid_expr_lines);
    }

    for line in invalid_expr_lines {
        errors.push(EngineError {
            message: format!(
                "Meaning of '{}' must assign an explicit meaning label (identifier or string literal) when 'valid meanings' are declared.",
                value_name
            ),
            line,
            column: 0,
        });
    }

    let valid_set: HashSet<&str> = meaning_def.valid_meanings.iter().map(|s| s.as_str()).collect();
    let mut used_set = HashSet::new();

    for (label, label_line) in labels {
        if !valid_set.contains(label.as_str()) {
            errors.push(EngineError {
                message: format!(
                    "Meaning of '{}' uses invalid meaning '{}'; expected one of: {}.",
                    value_name,
                    label,
                    meaning_def.valid_meanings.join(", ")
                ),
                line: label_line,
                column: 0,
            });
            continue;
        }
        used_set.insert(label);
    }

    let missing: Vec<String> = meaning_def
        .valid_meanings
        .iter()
        .filter(|m| !used_set.contains(*m))
        .cloned()
        .collect();
    if !missing.is_empty() {
        errors.push(EngineError {
            message: format!(
                "Meaning of '{}' does not use all valid meanings. Missing: {}.",
                value_name,
                missing.join(", ")
            ),
            line,
            column: 0,
        });
    }
}

fn collect_meaning_labels_from_block(
    block: &[Statement],
    labels: &mut Vec<(String, usize)>,
    invalid_expr_lines: &mut Vec<usize>,
) {
    for stmt in block {
        match &stmt.kind {
            StatementKind::Assignment(assign) if assign.target == "meaning of value" => {
                if let Some(label) = extract_meaning_label_from_expression(&assign.expression) {
                    labels.push((label, stmt.line));
                } else {
                    invalid_expr_lines.push(stmt.line);
                }
            }
            StatementKind::Action(crate::ast::Action::ListenFor(label)) => {
                labels.push((label.clone(), stmt.line));
            }
            StatementKind::Conditional(cond) => {
                for case in &cond.cases {
                    collect_meaning_labels_from_block(&case.block, labels, invalid_expr_lines);
                }
            }
            StatementKind::ContextBlock(cb) => {
                collect_meaning_labels_from_block(&cb.statements, labels, invalid_expr_lines);
            }
            StatementKind::Timeframe(tb) => {
                collect_meaning_labels_from_block(&tb.block, labels, invalid_expr_lines);
            }
            _ => {}
        }
    }
}

fn extract_meaning_label_from_expression(expr: &Expression) -> Option<String> {
    match expr {
        Expression::Variable(name) => Some(name.clone()),
        _ => None,
    }
}

fn check_assignment_units_precision_block(
    stmts: &[Statement],
    calculation_owner: Option<&str>,
    value_units: &HashMap<String, Unit>,
    value_precision: &HashMap<String, PrecisionInfo>,
    unit_map: &HashMap<String, Unit>,
    errors: &mut Vec<EngineError>,
) {
    for stmt in stmts {
        check_assignment_units_precision_stmt(
            stmt,
            calculation_owner,
            value_units,
            value_precision,
            unit_map,
            errors,
        );
    }
}

fn check_assignment_units_precision_stmt(
    stmt: &Statement,
    calculation_owner: Option<&str>,
    value_units: &HashMap<String, Unit>,
    value_precision: &HashMap<String, PrecisionInfo>,
    unit_map: &HashMap<String, Unit>,
    errors: &mut Vec<EngineError>,
) {
    match &stmt.kind {
        StatementKind::Assignment(assign) => {
            let mut target = runtime::normalize_identifier(&assign.target);
            if target == "value" {
                if let Some(owner) = calculation_owner {
                    target = runtime::normalize_identifier(owner);
                }
            }

            let expected_unit = match value_units.get(&target) {
                Some(unit) => unit.clone(),
                None => return,
            };

            let expected_precision = match value_precision.get(&target).and_then(|info| info.decimals) {
                Some(precision) => precision,
                None => return,
            };

            check_expression_unit_precision(
                &assign.expression,
                &expected_unit,
                expected_precision,
                value_units,
                value_precision,
                unit_map,
                &target,
                stmt.line,
                errors,
            );
        }
        StatementKind::Timeframe(tb) => {
            check_assignment_units_precision_block(
                &tb.block,
                calculation_owner,
                value_units,
                value_precision,
                unit_map,
                errors,
            );
        }
        StatementKind::ContextBlock(cb) => {
            check_assignment_units_precision_block(
                &cb.statements,
                calculation_owner,
                value_units,
                value_precision,
                unit_map,
                errors,
            );
        }
        StatementKind::Conditional(cond) => {
            for case in &cond.cases {
                check_assignment_units_precision_block(
                    &case.block,
                    calculation_owner,
                    value_units,
                    value_precision,
                    unit_map,
                    errors,
                );
            }
        }
        _ => {}
    }
}

fn check_expression_unit_precision(
    expr: &Expression,
    expected_unit: &Unit,
    expected_precision: usize,
    value_units: &HashMap<String, Unit>,
    value_precision: &HashMap<String, PrecisionInfo>,
    unit_map: &HashMap<String, Unit>,
    target: &str,
    line: usize,
    errors: &mut Vec<EngineError>,
) {
    match expr {
        Expression::Literal(Literal::Quantity(_, unit, decimals)) => {
            let unit = canonicalize_unit(unit, unit_map);
            if unit != *expected_unit {
                errors.push(EngineError {
                    message: format!(
                        "Assignment to '{}' requires unit '{}', but found '{}'.",
                        target, expected_unit, unit
                    ),
                    line,
                    column: 0,
                });
            }
            let precision = decimals.unwrap_or(0);
            if precision != expected_precision {
                errors.push(EngineError {
                    message: format!(
                        "Assignment to '{}' requires precision {}, but found {}.",
                        target, expected_precision, precision
                    ),
                    line,
                    column: 0,
                });
            }
        }
        Expression::Literal(Literal::Number(_, decimals)) => {
            let precision = decimals.unwrap_or(0);
            errors.push(EngineError {
                message: format!(
                    "Assignment to '{}' requires unit '{}' with precision {}, but found unitless number (precision {}).",
                    target, expected_unit, expected_precision, precision
                ),
                line,
                column: 0,
            });
        }
        Expression::Variable(name) => {
            let normalized = runtime::normalize_identifier(name);
            if let Some(unit) = value_units.get(&normalized) {
                if *unit != *expected_unit {
                    errors.push(EngineError {
                        message: format!(
                            "Assignment to '{}' requires unit '{}', but '{}' uses unit '{}'.",
                            target, expected_unit, normalized, unit
                        ),
                        line,
                        column: 0,
                    });
                }
                let precision = value_precision
                    .get(&normalized)
                    .and_then(|info| info.decimals)
                    .unwrap_or(0);
                if precision != expected_precision {
                    errors.push(EngineError {
                        message: format!(
                            "Assignment to '{}' requires precision {}, but '{}' uses precision {}.",
                            target, expected_precision, normalized, precision
                        ),
                        line,
                        column: 0,
                    });
                }
            } else {
                errors.push(EngineError {
                    message: format!(
                        "Assignment to '{}' requires numeric values with unit '{}', but '{}' is not numeric.",
                        target, expected_unit, normalized
                    ),
                    line,
                    column: 0,
                });
            }
        }
        Expression::Binary(left, _op, right) => {
            check_expression_unit_precision(
                left,
                expected_unit,
                expected_precision,
                value_units,
                value_precision,
                unit_map,
                target,
                line,
                errors,
            );
            check_expression_unit_precision(
                right,
                expected_unit,
                expected_precision,
                value_units,
                value_precision,
                unit_map,
                target,
                line,
                errors,
            );
        }
        Expression::Statistical(func) => match func {
            crate::ast::StatisticalFunc::CountOf(_, _) => {
                if expected_precision != 0 {
                    errors.push(EngineError {
                        message: format!(
                            "Assignment to '{}' requires precision {}, but 'count of' yields precision 0.",
                            target, expected_precision
                        ),
                        line,
                        column: 0,
                    });
                }
            }
            crate::ast::StatisticalFunc::AverageOf(name, _)
            | crate::ast::StatisticalFunc::MinOf(name)
            | crate::ast::StatisticalFunc::MaxOf(name) => {
                check_expression_unit_precision(
                    &Expression::Variable(name.clone()),
                    expected_unit,
                    expected_precision,
                    value_units,
                    value_precision,
                    unit_map,
                    target,
                    line,
                    errors,
                );
            }
            crate::ast::StatisticalFunc::TrendOf(_) => {
                errors.push(EngineError {
                    message: format!(
                        "Assignment to '{}' requires numeric values; 'trend of' is not numeric.",
                        target
                    ),
                    line,
                    column: 0,
                });
            }
        },
        Expression::DateDiff(unit, _, _) => {
            let unit = canonicalize_unit(unit, unit_map);
            if unit != *expected_unit {
                errors.push(EngineError {
                    message: format!(
                        "Assignment to '{}' requires unit '{}', but date diff returns '{}'.",
                        target, expected_unit, unit
                    ),
                    line,
                    column: 0,
                });
            }
        }
        Expression::MeaningOf(_)
        | Expression::RelativeTime(_, _, _)
        | Expression::InterpolatedString(_)
        | Expression::FunctionCall(_, _)
        | Expression::Literal(Literal::String(_))
        | Expression::Literal(Literal::TimeOfDay(_))
        | Expression::Literal(Literal::Date(_)) => {
            errors.push(EngineError {
                message: format!(
                    "Assignment to '{}' requires numeric values with unit '{}' and precision {}.",
                    target, expected_unit, expected_precision
                ),
                line,
                column: 0,
            });
        }
    }
}

fn extract_defined_strings(sel: &RangeSelector, out: &mut Vec<String>) {
    match sel {
        RangeSelector::Equals(expr) => {
            match expr {
                Expression::Literal(Literal::String(s)) => out.push(s.clone()),
                Expression::Variable(name) => out.push(name.clone()),
                _ => {}
            }
        },
        RangeSelector::List(items) => {
             for item in items {
                 match item {
                     Expression::Literal(Literal::String(s)) => out.push(s.clone()),
                     Expression::Variable(name) => out.push(name.clone()),
                     _ => {}
                 }
             }
        },
        _ => {}
    }
}

fn enum_selector_is_identifier(sel: &RangeSelector) -> bool {
    match sel {
        RangeSelector::Equals(expr) => matches!(expr, Expression::Variable(_)),
        RangeSelector::List(items) => items.iter().all(|item| matches!(item, Expression::Variable(_))),
        _ => false,
    }
}
