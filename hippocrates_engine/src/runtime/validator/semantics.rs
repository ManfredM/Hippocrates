use crate::ast::{Definition, Property, Statement, StatementKind, Expression, ConditionalTarget, RangeSelector, ContextItem, PlanBlock};
use crate::domain::EngineError;
use std::collections::{HashSet, HashMap};

pub fn check_drugs(defs: &HashMap<String, Definition>, valid_units: &HashSet<String>, errors: &mut Vec<EngineError>) {
    for def in defs.values() {
        if let crate::ast::Definition::Drug(dd) = def {
            for prop in &dd.properties {
                if let Property::Ingredients(ingredients) = prop {
                    for ing in ingredients {
                        if let crate::domain::Unit::Custom(name) = &ing.unit {
                            if !valid_units.contains(name) {
                                errors.push(EngineError {
                                    message: format!("Undefined unit '{}' in drug ingredient '{}'", name, ing.name),
                                    line: 0,
                                    column: 0,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn check_unit_definitions(defs: &HashMap<String, Definition>, errors: &mut Vec<EngineError>) {
    for def in defs.values() {
        if let Definition::Unit(ud) = def {
            let mut aliases = Vec::new();
            aliases.push(ud.name.as_str());
            for name in &ud.plurals {
                aliases.push(name.as_str());
            }
            for name in &ud.singulars {
                aliases.push(name.as_str());
            }
            for name in &ud.abbreviations {
                aliases.push(name.as_str());
            }

            for alias in aliases {
                if is_builtin_unit(alias) {
                    errors.push(EngineError {
                        message: format!(
                            "Built-in units cannot be redefined. Custom unit '{}' conflicts with built-in unit '{}'.",
                            ud.name, alias
                        ),
                        line: 0,
                        column: 0,
                    });
                }
            }
        }
    }
}

pub fn check_addressees(defs: &HashMap<String, Definition>, errors: &mut Vec<EngineError>) {
    for def in defs.values() {
        if let crate::ast::Definition::Addressee(ad) = def {
            for prop in &ad.properties {
                if let Property::ContactInfo(details) = prop {
                    for detail in details {
                        if let crate::ast::ContactDetail::Email(e) = detail {
                            if !e.contains('@') {
                                errors.push(EngineError {
                                    message: format!("Invalid email format '{}' for addressee '{}'", e, ad.name),
                                    line: 0,
                                    column: 0,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn check_value_definitions(defs: &HashMap<String, Definition>, errors: &mut Vec<EngineError>) {
     for def in defs.values() {
        if let crate::ast::Definition::Value(vd) = def {
             let mut line_hint = 0;
             for prop in &vd.properties {
                 if let Property::ValidValues(stmts) = prop {
                     if let Some(stmt) = stmts.first() {
                         line_hint = stmt.line;
                     }
                 }
             }

             // 1. Check for missing Valid Values
             let needs_values = matches!(vd.value_type, crate::domain::ValueType::Number | crate::domain::ValueType::Enumeration);
             if needs_values {
                 let has_valid_values = vd.properties.iter().any(|p| matches!(p, Property::ValidValues(_)));
                 if !has_valid_values {
                      errors.push(EngineError {
                          message: format!("Definition of '{}' is invalid: Missing 'valid values' property.", vd.name),
                          line: 0,
                          column: 0,
                      });
                 }
             }

             // 2. Check for missing Units (Strict Mode: Numbers MUST have units)
             if matches!(vd.value_type, crate::domain::ValueType::Number) {
                 let has_explicit_unit = vd.properties.iter().any(|p| matches!(p, Property::Unit(_)));
                 
                 // If no explicit unit, check if valid values use Quantities (implied unit)
                 let mut has_implied_unit = false;
                 let mut has_unitless_range = false;

                 for prop in &vd.properties {
                     if let Property::ValidValues(stmts) = prop {
                         for stmt in stmts {
                             if let StatementKind::Constraint(_, _, sel) = &stmt.kind {
                                 // Check range selector elements for Unit vs Number
                                 check_selector_units(sel, &mut has_implied_unit, &mut has_unitless_range);
                             } else if let StatementKind::EventProgression(_, cases) = &stmt.kind {
                                 for case in cases {
                                     check_selector_units(&case.condition, &mut has_implied_unit, &mut has_unitless_range);
                                 }
                             }
                         }
                     }
                 }

                 if !has_explicit_unit && !has_implied_unit {
                      errors.push(EngineError {
                          message: format!("Numeric values must have a unit. Definition of '{}' is invalid: Number defined without a Unit (e.g. 'unit is mg' or usage of '0 mg ... 100 mg').", vd.name),
                          line: line_hint,
                          column: 0,
                      });
                 } else if has_unitless_range {
                      // Mixed or Unitless ranges
                      errors.push(EngineError {
                          message: format!("Numeric values must have a unit. Definition of '{}' is invalid: Range contains unitless numbers (must use quantities like '10 mg').", vd.name),
                          line: line_hint,
                          column: 0,
                      });
                 }
             }
         }
     }
}

fn check_selector_units(sel: &crate::ast::RangeSelector, has_unit: &mut bool, has_unitless: &mut bool) {
    use crate::ast::{RangeSelector, Expression, Literal};
    
    let check_expr = |e: &Expression, h_u: &mut bool, h_ul: &mut bool| {
        match e {
            Expression::Literal(Literal::Quantity(_, _, _)) => *h_u = true,
            Expression::Literal(Literal::Number(_, _)) => *h_ul = true,
            _ => {} // Expressions/Vars might be ambiguous, ignore for now (assume valid if at least one quantity found?)
        }
    };

    match sel {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
            check_expr(min, has_unit, has_unitless);
            check_expr(max, has_unit, has_unitless);
        }
        RangeSelector::Equals(e) => {
            check_expr(e, has_unit, has_unitless);
        }
        RangeSelector::List(items) => {
            for item in items {
                check_expr(item, has_unit, has_unitless);
            }
        }
        _ => {}
    }
}

pub fn check_timeframe_period_references(defs: &HashMap<String, Definition>, errors: &mut Vec<EngineError>) {
    let period_names: HashSet<String> = defs
        .values()
        .filter_map(|def| if let Definition::Period(p) = def { Some(p.name.clone()) } else { None })
        .collect();

    for def in defs.values() {
        match def {
            Definition::Period(p) => {
                for line in &p.timeframes {
                    for sel in line {
                        check_timeframe_selector(sel, &period_names, 0, errors);
                    }
                }
            }
            Definition::Value(v) => {
                for prop in &v.properties {
                    if let Property::Timeframe(lines) = prop {
                        for line in lines {
                            for sel in line {
                                check_timeframe_selector(sel, &period_names, 0, errors);
                            }
                        }
                    }
                }
            }
            Definition::Context(c) => {
                for item in &c.items {
                    if let ContextItem::Timeframe(sel) = item {
                        check_timeframe_selector(sel, &period_names, 0, errors);
                    }
                }
            }
            Definition::Plan(p) => {
                for block in &p.blocks {
                    match block {
                        PlanBlock::DuringPlan(stmts) => {
                            for stmt in stmts {
                                check_statement_timeframes(stmt, &period_names, errors);
                            }
                        }
                        PlanBlock::Trigger(t) => {
                            for stmt in &t.statements {
                                check_statement_timeframes(stmt, &period_names, errors);
                            }
                        }
                        PlanBlock::Event(e) => {
                            for stmt in &e.statements {
                                check_statement_timeframes(stmt, &period_names, errors);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn check_timeframe_selector(sel: &RangeSelector, period_names: &HashSet<String>, line: usize, errors: &mut Vec<EngineError>) {
    if let RangeSelector::Equals(Expression::Variable(name)) = sel {
        if !period_names.contains(name) {
            errors.push(EngineError {
                message: format!("Timeframe selector references undefined period '{}'.", name),
                line,
                column: 0,
            });
        }
    }
}

fn check_statement_timeframes(stmt: &Statement, period_names: &HashSet<String>, errors: &mut Vec<EngineError>) {
    match &stmt.kind {
        StatementKind::Timeframe(tb) => {
            for (_, sel) in &tb.constraints {
                check_timeframe_selector(sel, period_names, stmt.line, errors);
            }
            for nested in &tb.block {
                check_statement_timeframes(nested, period_names, errors);
            }
        }
        StatementKind::ContextBlock(cb) => {
            for item in &cb.items {
                if let ContextItem::Timeframe(sel) = item {
                    check_timeframe_selector(sel, period_names, stmt.line, errors);
                }
            }
            for nested in &cb.statements {
                check_statement_timeframes(nested, period_names, errors);
            }
        }
        StatementKind::Conditional(cond) => {
            for case in &cond.cases {
                for nested in &case.block {
                    check_statement_timeframes(nested, period_names, errors);
                }
            }
        }
        _ => {}
    }
}
pub fn check_statement_semantics(
    stmt: &Statement,
    enum_vars: &HashSet<String>,
    defined_values: &HashMap<String, Definition>,
    errors: &mut Vec<EngineError>
) {
    match &stmt.kind {
        StatementKind::Assignment(assign) => {
             // Pass only keys to validation for now, or full map if needed?
             // defined_values is HashMap, validate_expression takes HashSet keys.
             // We can rebuild set or change signature.
             // Given current signature of check_statement_semantics takes HashSet in call site?
             // Wait, previous signature was `defined_values: &HashSet<String>`.
             // I need to change signature to `&HashMap` to inspect properties.
             let defined_keys: HashSet<String> = defined_values.keys().cloned().collect();
             let allow_enum_label = enum_vars.contains(&assign.target);
             validate_expression(
                 &assign.expression,
                 enum_vars,
                 &defined_keys,
                 stmt.line,
                 allow_enum_label,
                 errors,
             );
        }
        StatementKind::Action(action) => {
            match action {
                 crate::ast::Action::AskQuestion(q, _) => {
                     if let Some(def) = defined_values.get(q) {
                         if let Definition::Value(vd) = def {
                             // Check 1: Must have Property::Question with AskQuestion
                             let has_question = vd.properties.iter().any(|p| {
                                 if let Property::Question(act) = p {
                                     matches!(act, crate::ast::Action::AskQuestion(_, _))
                                 } else { false }
                             });
                             
                             if !has_question {
                                 errors.push(EngineError {
                                     message: format!("Variable '{}' cannot be asked: Missing 'question:' property with text.", q),
                                     line: stmt.line,
                                     column: 0,
                                 });
                             }

                             // Check 2: Must have ValidValues if Number or Enumeration
                             let has_valid_values = vd.properties.iter().any(|p| matches!(p, Property::ValidValues(_)));
                             let needs_values = matches!(vd.value_type, crate::domain::ValueType::Number | crate::domain::ValueType::Enumeration);
                             
                             if needs_values && !has_valid_values {
                                 errors.push(EngineError {
                                     message: format!("Variable '{}' cannot be asked: Missing 'valid values:' definition (range or enum cases).", q),
                                     line: stmt.line,
                                     column: 0,
                                 });
                             }
                         }
                     } else {
                         errors.push(EngineError {
                             message: format!("Asking question for undefined variable '{}'", q),
                             line: stmt.line,
                             column: 0,
                         });
                     }
                 },
                 crate::ast::Action::ListenFor(q) => {
                     if !defined_values.contains_key(q) {
                         errors.push(EngineError {
                             message: format!("ListenFor action targets undefined variable '{}'", q),
                             line: stmt.line,
                             column: 0,
                         });
                     }
                 },
                 crate::ast::Action::ShowMessage { parts, .. } => {
                     let defined_keys: HashSet<String> = defined_values.keys().cloned().collect();
                     for part in parts {
                         validate_expression(part, enum_vars, &defined_keys, stmt.line, false, errors);
                     }
                 },
                 _ => {}
            }
        },
        StatementKind::Conditional(cond) => {
             let defined_keys: HashSet<String> = defined_values.keys().cloned().collect();
             // Basic expression check
             match &cond.condition {
                 ConditionalTarget::Expression(e) => validate_expression(e, enum_vars, &defined_keys, stmt.line, false, errors),
                 _ => {}
             }
             
             // Recursively check children
             for case in &cond.cases {
                 for s in &case.block {
                     check_statement_semantics(s, enum_vars, defined_values, errors);
                 }
             }
        },
        _ => {}
    }
}

fn is_builtin_unit(name: &str) -> bool {
    matches!(
        name,
        "\u{00B0}F"
            | "\u{00B0}C"
            | "%"
            | "milligrams"
            | "milligram"
            | "mg"
            | "kilograms"
            | "kilogram"
            | "kg"
            | "grams"
            | "gram"
            | "g"
            | "pounds"
            | "pound"
            | "lb"
            | "ounces"
            | "ounce"
            | "oz"
            | "milliliters"
            | "milliliter"
            | "ml"
            | "liters"
            | "liter"
            | "l"
            | "fluid ounces"
            | "fluid ounce"
            | "fl oz"
            | "gallons"
            | "gallon"
            | "gal"
            | "mmHg"
            | "bpm"
            | "mg/dL"
            | "mmol/L"
            | "centimeters"
            | "centimeter"
            | "cm"
            | "millimeters"
            | "millimeter"
            | "mm"
            | "kilometers"
            | "kilometer"
            | "km"
            | "inches"
            | "inch"
            | "feet"
            | "foot"
            | "miles"
            | "mile"
            | "meters"
            | "meter"
            | "m"
            | "years"
            | "months"
            | "weeks"
            | "days"
            | "hours"
            | "minutes"
            | "seconds"
            | "year"
            | "month"
            | "week"
            | "day"
            | "hour"
            | "minute"
            | "second"
    )
}

fn validate_expression(
    expr: &Expression,
    enum_vars: &HashSet<String>,
    defined_values: &HashSet<String>,
    line: usize,
    allow_undefined_identifiers: bool,
    errors: &mut Vec<EngineError>,
) {
    match expr {
        Expression::Variable(name) => {
             if !defined_values.contains(name) && !allow_undefined_identifiers {
                 errors.push(EngineError {
                     message: format!("Undefined variable '{}' in expression", name),
                     line,
                     column: 0,
                 });
             }
        }
        Expression::MeaningOf(name) => {
             if !defined_values.contains(name) {
                 errors.push(EngineError {
                     message: format!("Undefined variable '{}' in meaning expression", name),
                     line,
                     column: 0,
                 });
             }
        }
        Expression::Binary(left, _, right) => {
             validate_expression(left, enum_vars, defined_values, line, allow_undefined_identifiers, errors);
             validate_expression(right, enum_vars, defined_values, line, allow_undefined_identifiers, errors);
        }
        Expression::DateDiff(_, start, end) => {
             validate_expression(start, enum_vars, defined_values, line, allow_undefined_identifiers, errors);
             validate_expression(end, enum_vars, defined_values, line, allow_undefined_identifiers, errors);
        }
        Expression::Statistical(func) => match func {
            crate::ast::StatisticalFunc::CountOf(name, filter) => {
                if enum_vars.contains(name) && filter.is_none() {
                     errors.push(EngineError {
                         message: format!("Validation Error: 'count of {}' requires a specific value to count (e.g. 'count of {} is <Yes>') because it is an Enumeration.", name, name),
                         line,
                         column: 0
                     });
                }
            }
            crate::ast::StatisticalFunc::TrendOf(name) => {
                 if enum_vars.contains(name) {
                    errors.push(EngineError {
                        message: format!("Validation Error: 'trend of {}' is not supported because it is an Enumeration.", name),
                        line,
                        column: 0
                    });
                }
            }
            _ => {}
        },
        _ => {}
    }
}
