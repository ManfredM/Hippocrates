mod intervals;
mod semantics; 
mod data_flow;
mod coverage;

use crate::ast::{Plan, Definition, Property, RangeSelector, Expression, Literal};
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

    let mut value_intervals = HashMap::new();
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
            _ => {}
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

            // Default to unbounded
            let mut interval = Interval::unbounded();
            
            let mut precision = PrecisionInfo::new();

            // Try to narrow down from Valid Values
            for prop in &vd.properties {
                if let Property::ValidValues(stmts) = prop {
                    // Extract range union from valid values
                    // For now, simplify: take the min(min) and max(max) of all ranges
                    let mut min_all = f64::INFINITY;
                    let mut max_all = f64::NEG_INFINITY;
                    let mut found = false;
                    
                    for stmt in stmts {
                         if let crate::ast::StatementKind::Constraint(_, _, sel) = &stmt.kind {
                             update_precision_from_selector(sel, &mut precision);
                             if let Some((mn, mx)) = extract_const_range(sel) {
                                 min_all = min_all.min(mn);
                                 max_all = max_all.max(mx);
                                 found = true;
                             }
                         } else if let crate::ast::StatementKind::EventProgression(_, cases) = &stmt.kind {
                             // Also check progression cases
                             for case in cases {
                                 update_precision_from_selector(&case.condition, &mut precision);
                                 if let Some((mn, mx)) = extract_const_range(&case.condition) {
                                     min_all = min_all.min(mn);
                                     max_all = max_all.max(mx);
                                     found = true;
                                 }
                             }
                         }
                    }
                    
                    if found {
                        interval = Interval::new(min_all, max_all);
                    }
                }
            }
            value_intervals.insert(vd.name.clone(), interval);
            value_precision.insert(vd.name.clone(), precision);
        }
    }
    
    // Legacy structural checks renamed to Semantics
    semantics::check_unit_definitions(&defs_map, &mut errors);
    semantics::check_drugs(&defs_map, &valid_units, &mut errors);
    semantics::check_addressees(&defs_map, &mut errors);
    semantics::check_value_definitions(&defs_map, &mut errors);

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
                    } else if let Some(valid_int) = value_intervals.get(&vd.name) {
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
                        coverage::check_coverage(&vd.name, valid_int, cases, line, decimals, &mut errors);
                    }
                }
            }
        }
    }

    // 2. Validate Assignments and Expressions using Intervals AND Data Flow
    for def in &plan.definitions {
        match def {
            Definition::Plan(pd) => {
                 for block in &pd.blocks {
                      let statements = match block {
                          crate::ast::PlanBlock::DuringPlan(s) => s,
                          crate::ast::PlanBlock::Event(e) => &e.statements,
                          crate::ast::PlanBlock::Trigger(t) => &t.statements,
                      };
                      
                      // Run Data Flow Analysis for this block
                      let base_state = data_flow::FlowState::new();
                      data_flow::analyze_block(statements, &base_state, &defs_map, &mut errors);
                      
                      for stmt in statements {
                          // Run semantic checks (undefined vars, etc)
                          semantics::check_statement_semantics(stmt, &enum_vars, &defs_map, &mut errors);

                          match &stmt.kind {
                              crate::ast::StatementKind::Assignment(assign) => {
                                  let target_interval = value_intervals.get(&assign.target);
                                  
                                  if let Some(target_int) = target_interval {
                                      let expr_interval = calculate_interval(&assign.expression, &value_intervals);
                                      
                                      if !expr_interval.is_subset_of(target_int) {
                                          errors.push(EngineError {
                                              message: format!(
                                                  "Assignment Validity Warning: Value for '{}' may be out of bounds. Expression result range ({:.1}..{:.1}) is not fully contained in valid range ({:.1}..{:.1}).",
                                                  assign.target, expr_interval.min, expr_interval.max, target_int.min, target_int.max
                                              ),
                                              line: stmt.line,
                                              column: 0
                                          });
                                      }
                                  } 
                                  
                                  // Check for subtraction safety
                                  if let Expression::Binary(l, op, r) = &assign.expression {
                                      if op == "-" {
                                          if let Some(msg) = intervals::check_subtraction_safety(l, r, &value_intervals) {
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
                                              if let Some(valid_int) = value_intervals.get(name) {
                                                  if cond.cases.iter().any(|case| selector_has_unitless_number(&case.condition)) {
                                                      errors.push(EngineError {
                                                          message: format!("Numeric values must have a unit. Assessment for '{}' uses unitless numbers.", name),
                                                          line: stmt.line,
                                                          column: 0,
                                                      });
                                                  }

                                                  let decimals = value_precision.get(name).and_then(|info| info.decimals);
                                                  coverage::check_coverage(name, valid_int, &cond.cases, stmt.line, decimals, &mut errors);
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


fn extract_const_range(sel: &RangeSelector) -> Option<(f64, f64)> {
    match sel {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
             // simplified extraction logic
             let get_val = |e: &Expression| match e {
                 Expression::Literal(Literal::Number(n, _)) => *n,
                 Expression::Literal(Literal::Quantity(n, _, _)) => *n,
                 _ => f64::NAN
             };
             
             let v1 = get_val(min);
             let v2 = get_val(max);
             if !v1.is_nan() && !v2.is_nan() {
                 Some((v1, v2))
             } else { None }
        }
        _ => None
    }
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
