use crate::ast::{Definition, Statement, StatementKind, Expression, ConditionalTarget};
use crate::domain::EngineError;
use std::collections::{HashSet, HashMap};

#[derive(Clone)]
pub struct FlowState {
    pub initialized: HashSet<String>,
}

impl FlowState {
    pub fn new() -> Self {
        FlowState { initialized: HashSet::new() }
    }
}

pub fn analyze_block(
    stmts: &[Statement],
    parent_state: &FlowState,
    defs: &HashMap<String, Definition>,
    errors: &mut Vec<EngineError>
) -> FlowState {
    let mut current_state = parent_state.clone();

    for stmt in stmts {
        analyze_statement(stmt, &mut current_state, defs, errors);
    }
    
    current_state
}

pub fn analyze_statement(
    stmt: &Statement,
    state: &mut FlowState,
    defs: &HashMap<String, Definition>,
    errors: &mut Vec<EngineError>
) {
    match &stmt.kind {
        StatementKind::Assignment(assign) => {
            // Check RHS uses
            check_expression(&assign.expression, state, defs, stmt.line, errors);
            // Mark LHS initialized
            state.initialized.insert(assign.target.clone());
        }
        StatementKind::Action(action) => {
            match action {
                crate::ast::Action::AskQuestion(var, _) => {
                    // 1. Mark initialized
                    state.initialized.insert(var.clone());
                    
                    // 2. Check if variable has a question property
                    if let Some(crate::ast::Definition::Value(vd)) = defs.get(var) {
                        let has_question = vd.properties.iter().any(|p| matches!(p, crate::ast::Property::Question(_)));
                        if !has_question {
                             errors.push(EngineError {
                                message: format!("Validation Error: Cannot 'Ask {}' because it does not have a 'question' property defined.", var),
                                line: stmt.line,
                                column: 0
                            });
                        }
                    } else if !defs.contains_key(var) {
                        // Undefined variable error handled elsewhere, but good to note
                    }
                },
                crate::ast::Action::ShowMessage(parts, _) => {
                     for part in parts {
                         check_expression(part, state, defs, stmt.line, errors);
                     }
                },
                crate::ast::Action::SendInfo(_, parts) => {
                     for part in parts {
                         check_expression(part, state, defs, stmt.line, errors);
                     }
                },
                crate::ast::Action::ListenFor(var) => {
                    state.initialized.insert(var.clone());
                },
                _ => {}
            }
        },
        StatementKind::Conditional(cond) => {
             // Check condition
             match &cond.condition {
                 ConditionalTarget::Expression(e) => check_expression(e, state, defs, stmt.line, errors),
                 _ => {}
             }
             
             // Branch analysis
             // We do NOT update 'state' with results from branches (Conservative)
             // But we pass current 'state' down.
             for case in &cond.cases {
                  analyze_block(&case.block, state, defs, errors);
             }
        },
        StatementKind::ContextBlock(cb) => {
             // Treat context block as effectively part of flow?
             // Usually Context sets up environment.
             // If items in ContextBlock declare data, we should add to state?
             // "data: <var>" -> implies var is present.
             for item in &cb.items {
                 if let crate::ast::ContextItem::Data(var) = item {
                     state.initialized.insert(var.clone());
                 }
             }
             // Then analyze statements
             for s in &cb.statements {
                 analyze_statement(s, state, defs, errors);
             }
        }
        StatementKind::Timeframe(tb) => {
             // Analyze statements inside timeframe blocks like regular flow.
             for s in &tb.block {
                 analyze_statement(s, state, defs, errors);
             }
        }
        _ => {}
    }
}

fn check_expression(
    expr: &Expression,
    state: &FlowState,
    defs: &HashMap<String, Definition>,
    line: usize,
    errors: &mut Vec<EngineError>
) {
    match expr {
        Expression::Variable(name) => {
             if !state.initialized.contains(name) {
                 errors.push(EngineError {
                     message: format!("Data Flow Error: Variable '{}' used before being assigned or asked.", name),
                     line,
                     column: 0
                 });
             }
        },
        Expression::MeaningOf(name) => {
             if !state.initialized.contains(name) {
                 let askable = defs
                     .get(name)
                     .and_then(|def| {
                         if let Definition::Value(vd) = def {
                             Some(vd.properties.iter().any(|p| matches!(p, crate::ast::Property::Question(_))))
                         } else {
                             None
                         }
                     })
                     .unwrap_or(false);

                 if !askable {
                     errors.push(EngineError {
                         message: format!(
                             "Data Flow Error: Meaning of '{}' requires a question property when the value is not initialized.",
                             name
                         ),
                         line,
                         column: 0
                     });
                 }
             }
        },
        Expression::Binary(l, _, r) => {
            check_expression(l, state, defs, line, errors);
            check_expression(r, state, defs, line, errors);
        },
        Expression::FunctionCall(_, args) => {
            for arg in args { check_expression(arg, state, defs, line, errors); }
        },
        Expression::InterpolatedString(parts) => {
             for part in parts { check_expression(part, state, defs, line, errors); }
        },
        // Statistical functions: Exception!
        // "count of <var>" or "trend of <var>" do not require <var> to be initialized in current scope.
        // They look at execution history (database).
        Expression::Statistical(stat) => {
             match stat {
                 crate::ast::StatisticalFunc::AverageOf(_, period) => check_expression(period, state, defs, line, errors),
                 crate::ast::StatisticalFunc::TrendOf(_var) => {
                      // Trend calculation on history.
                 },
                 crate::ast::StatisticalFunc::CountOf(_var, filter) => {
                      if let Some(f) = filter {
                          check_expression(f, state, defs, line, errors);
                      }
                 },
                 _ => {}
             }
        }
        _ => {}
    }
}
