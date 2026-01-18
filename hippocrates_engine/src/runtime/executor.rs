use crate::ast::{Statement, Action, Block};
use crate::runtime::{Environment, Evaluator};
use crate::domain::Unit;


pub struct Executor {
    // Could hold state here if needed
}

impl Executor {
    pub fn new() -> Self {
        Executor {}
    }

    pub fn execute_plan(&mut self, env: &mut Environment, plan_name: &str) {
        env.log(format!("Starting plan: {}", plan_name));
        
        // Find the plan definition
        // (In a real engine, we'd clone the PlanDef or use RC, for now simple lookup)
        // Accessing env.definitions requires borrowing env.
        // We need to be careful about borrowing rules.

        // Hacky clone for prototype to avoid borrow checker hell in this simple pass
        let defs = env.definitions.clone(); 
        
        if let Some(crate::ast::Definition::Plan(plan_def)) = defs.get(plan_name) {
             println!("DEBUG: Found plan '{}', blocks: {}", plan_name, plan_def.blocks.len());
             for block in &plan_def.blocks {
                 match block {
                     crate::ast::PlanBlock::DuringPlan(stmts) => {
                         println!("DEBUG: Executing DuringPlan block with {} stmts", stmts.len());
                         self.execute_block(env, stmts);
                     }
                     crate::ast::PlanBlock::Trigger(block) => {
                          match &block.trigger {
                               crate::ast::Trigger::Periodic { interval, interval_unit, duration } => {
                                    let interval_secs = match interval_unit {
                                        Unit::Second => *interval,
                                        Unit::Minute => *interval * 60.0,
                                        Unit::Hour => *interval * 3600.0,
                                        _ => *interval,
                                    };
                                    
                                    let duration_secs = if let Some((d_val, d_unit)) = duration {
                                         match d_unit {
                                            Unit::Second => *d_val,
                                            Unit::Minute => *d_val * 60.0,
                                            Unit::Hour => *d_val * 3600.0,
                                            _ => *d_val,
                                         }
                                    } else {
                                        0.0
                                    };
                                    
                                    let iterations = if duration_secs > 0.0 {
                                        (duration_secs / interval_secs) as u64
                                    } else {
                                        1
                                    };
                                    
                                    println!("DEBUG: Running periodic trigger for {} iterations (every {}s)", iterations, interval_secs);
                                    
                                    for i in 0..iterations {
                                         // println!("DEBUG: Loop iteration {}", i);
                                         // println!("DEBUG: Block statements count: {}", block.statements.len());
                                         self.execute_block(env, &block.statements);
                                         if i < iterations - 1 {
                                            std::thread::sleep(std::time::Duration::from_secs_f64(interval_secs));
                                         }
                                    }
                               }
                               crate::ast::Trigger::StartOf(_) => {
                                   self.execute_block(env, &block.statements);
                               },
                               crate::ast::Trigger::ChangeOf(val) => {
                                   println!("DEBUG: Registering listener for ChangeOf({})", val);
                               }
                          }
                     }
                     crate::ast::PlanBlock::Event(block) => {
                         println!("DEBUG: Executing Event block '{}' (trigger: {:?}) with {} stmts", block.name, block.trigger, block.statements.len());
                         self.execute_block(env, &block.statements);
                     }
                 }
             }
        } else {
            println!("DEBUG: Plan '{}' not found. Available: {:?}", plan_name, defs.keys());
            env.log(format!("Plan not found: {}", plan_name));
        }
    }

    pub fn execute_block(&mut self, env: &mut Environment, stmts: &Block) {
        for stmt in stmts {
            self.execute_statement(env, stmt);
        }
    }

    pub fn execute_statement(&mut self, env: &mut Environment, stmt: &Statement) {
        match stmt {
            Statement::Action(action) => self.execute_action(env, action),
            Statement::Assignment(assign) => {
                let val = Evaluator::evaluate(env, &assign.expression);
                env.set_value(&assign.target, val);
            }
            Statement::Command(cmd) => {
                env.log(format!("Command: {}", cmd));
            }
            Statement::EventProgression(target_name, cases) => {
                 let val = if let Some(v) = env.get_value(target_name) {
                     v.clone()
                 } else {
                     env.log(format!("Warning: Variable '{}' not found for assessment", target_name));
                     return;
                 };
                 
                 // println!("DEBUG: Assessing '{}' (val={:?}) against {} cases", target_name, val, cases.len());

                 for case in cases {
                     let selector = &case.condition; 
                     let is_match = match selector {
                         crate::ast::RangeSelector::Equals(v_expr) => {
                             let v = Evaluator::evaluate(env, v_expr);
                             match (&val, &v) {
                                  (crate::domain::RuntimeValue::Number(a), crate::domain::RuntimeValue::Number(b)) => (a - b).abs() < f64::EPSILON,
                                  _ => val == v,
                             }
                         }
                         crate::ast::RangeSelector::Range(min_expr, max_expr) => {
                             let min = Evaluator::evaluate(env, min_expr);
                             let max = Evaluator::evaluate(env, max_expr);
                             match (&val, &min, &max) {
                                 (crate::domain::RuntimeValue::Number(v), crate::domain::RuntimeValue::Number(min_v), crate::domain::RuntimeValue::Number(max_v)) => {
                                     *v >= *min_v && *v <= *max_v
                                 }
                                 _ => false, 
                             }
                         }
                         crate::ast::RangeSelector::Default => true,
                         _ => false, 
                     };
                     
                     // println!("DEBUG: Checking selector {:?} match={}", selector, is_match);
                     
                     if is_match {
                         // println!("DEBUG: Case matched, executing block len={}", case.block.len());
                         self.execute_block(env, &case.block);
                         break;
                     }
                 }
            }
            Statement::Conditional(cond) => {
                 self.execute_conditional(env, cond);
            }
            Statement::ContextBlock(cb) => {
                 // Build evaluation context
                 let mut timeframe = None;
                 for item in &cb.items {
                     if let crate::ast::ContextItem::Timeframe(ts) = item {
                         timeframe = Some(ts.clone());
                     }
                 }
                 let ctx = crate::runtime::environment::EvaluationContext { timeframe };
                 
                 env.push_context(ctx);
                 self.execute_block(env, &cb.statements);
                 env.pop_context();
            }
            _ => { println!("DEBUG: Unimplemented execution for {:?}", stmt); }
        }
    }

    fn execute_conditional(&mut self, env: &mut Environment, cond_stmt: &crate::ast::Conditional) {
        let val = match &cond_stmt.condition {
             crate::ast::ConditionalTarget::Expression(expr) => Evaluator::evaluate(env, expr),
             crate::ast::ConditionalTarget::Confidence(_ident) => {
                  // Stub: return high confidence
                  crate::domain::RuntimeValue::Number(100.0) 
             }
        }; 
        
        // Context-aware resolution: If we evaluated to a String, it might be a variable name (legacy Hippocrates style)
        // e.g. assess "empty bottles" -> "empty bottles" -> lookup value.
        let val = if let crate::domain::RuntimeValue::String(s) = &val {
             if let Some(resolved) = env.get_value(s) {
                 // println!("DEBUG: Resolved string '{}' to value {:?}", s, resolved);
                 resolved.clone()
             } else {
                 val
             }
        } else {
            val
        };

        
        
        // println!("DEBUG: Conditional check logic. Value: {:?}", val);
        
        for case in &cond_stmt.cases {
             let selector = &case.condition; // Single condition per (flattened) case
             let is_match = match selector {
                 crate::ast::RangeSelector::Equals(v_expr) => {
                     let v = Evaluator::evaluate(env, v_expr);
                     match (&val, &v) {
                         (crate::domain::RuntimeValue::Number(a), crate::domain::RuntimeValue::Number(b)) => (a - b).abs() < f64::EPSILON,
                         _ => val == v,
                     }
                 }
                 crate::ast::RangeSelector::Range(min_expr, max_expr) => {
                     let min = Evaluator::evaluate(env, min_expr);
                     let max = Evaluator::evaluate(env, max_expr);
                     match (&val, &min, &max) {
                         (crate::domain::RuntimeValue::Number(v), crate::domain::RuntimeValue::Number(min_v), crate::domain::RuntimeValue::Number(max_v)) => {
                             v >= min_v && v <= max_v
                         }
                         _ => false, 
                     }
                 }
                 // range_selector can parse plain expression as RangeSelector::Equals
                 _ => false, 
             };
             
             if is_match {
                 self.execute_block(env, &case.block);
                 break;
             }
        }
    }

    fn execute_action(&mut self, env: &mut Environment, action: &Action) {
        match action {
            Action::ShowMessage(parts, _) => {
                let mut full_msg = String::new();
                for part in parts {
                    let val = Evaluator::evaluate(env, part);
                    let s = val.to_string(); // Use Display impl
                    full_msg.push_str(&s);
                }
                env.log(full_msg);
            }
// ... rest matches existing
            Action::AskQuestion(q, _) => {
                env.log(format!("Action: Ask Question '{}'", q));
            }
            Action::SendInfo(msg, vars) => {
                 let vals: Vec<String> = vars.iter().map(|e| format!("{:?}", Evaluator::evaluate(env, e))).collect();
                 env.log(format!("Action: Send Info '{}' with values: {:?}", msg, vals));
            }
            Action::ListenFor(val) => {
                env.log(format!("Action: Listen For '{}'", val));
            }
             _ => {}
        }
    }
}
