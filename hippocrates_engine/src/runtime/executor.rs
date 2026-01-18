use crate::runtime::{Environment, Evaluator, scheduler::Scheduler};
use crate::ast::{Statement, StatementKind, Action, Block};
use crate::domain::Unit;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct ScheduledEvent {
    time: chrono::DateTime<chrono::Utc>,
    kind: EventKind,
}

#[derive(Debug, Clone)]
enum EventKind {
    Periodic {
        block: crate::ast::TriggerBlock,
        iteration: u64,
        interval_secs: f64,
        max_duration: Option<chrono::Duration>,
    },
    StartOf {
        block: crate::ast::EventBlock,
        period_name: String,
    },
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for Min-Heap
        other.time.cmp(&self.time)
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExecutionMode {
    RealTime,
    Simulation(std::time::Duration),
}

pub struct Executor {
    pub on_step: Option<Box<dyn Fn(usize) + Send>>,
    pub on_log: Option<Box<dyn Fn(String, chrono::DateTime<chrono::Utc>) + Send>>,
    pub mode: ExecutionMode,
}

impl Executor {
    pub fn new() -> Self {
        Executor { on_step: None, on_log: None, mode: ExecutionMode::RealTime }
    }
    
    pub fn with_activites(
        line_cb: Box<dyn Fn(usize) + Send>,
        log_cb: Box<dyn Fn(String, chrono::DateTime<chrono::Utc>) + Send>
    ) -> Self {
        Executor { 
            on_step: Some(line_cb),
            on_log: Some(log_cb),
            mode: ExecutionMode::RealTime,
        }
    }

    pub fn set_mode(&mut self, mode: ExecutionMode) {
        self.mode = mode;
    }

    pub fn execute_plan(&mut self, env: &mut Environment, plan_name: &str) {
        env.log(format!("Starting plan: {}", plan_name));
        
        // Hacky clone for prototype
        let defs = env.definitions.clone(); 
        
        if let Some(crate::ast::Definition::Plan(plan_def)) = defs.get(plan_name) {
             println!("DEBUG: Found plan '{}', blocks: {}", plan_name, plan_def.blocks.len());
             
             let mut events = BinaryHeap::new();
             let start_time = env.now;
             let end_time = match self.mode {
                 ExecutionMode::RealTime => None,
                 ExecutionMode::Simulation(d) => Some(start_time + chrono::Duration::from_std(d).unwrap_or(chrono::Duration::max_value())),
             };

             // Initial Scheduling
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
                                        Unit::Day => *interval * 86400.0,
                                        Unit::Week => *interval * 604800.0,
                                        Unit::Month => *interval * 2592000.0,
                                        Unit::Year => *interval * 31536000.0,
                                        _ => *interval,
                                    };
                                    
                                    let max_dur = if let Some((d_val, d_unit)) = duration {
                                         let d_secs = match d_unit {
                                            Unit::Second => *d_val,
                                            Unit::Minute => *d_val * 60.0,
                                            Unit::Hour => *d_val * 3600.0,
                                            Unit::Day => *d_val * 86400.0,
                                            Unit::Week => *d_val * 604800.0,
                                            Unit::Month => *d_val * 2592000.0,
                                            Unit::Year => *d_val * 31536000.0,
                                            _ => *d_val,
                                         };
                                         Some(chrono::Duration::milliseconds((d_secs * 1000.0) as i64))
                                    } else {
                                        None
                                    };

                                    // Schedule first run immediately or after interval?
                                    // "every 1 day" usually triggers after 1 day.
                                    // Assuming after interval.
                                    let first_run = start_time + chrono::Duration::milliseconds((interval_secs * 1000.0) as i64);
                                    
                                    events.push(ScheduledEvent {
                                        time: first_run,
                                        kind: EventKind::Periodic {
                                            block: block.clone(),
                                            iteration: 0,
                                            interval_secs,
                                            max_duration: max_dur,
                                        }
                                    });
                               }
                               crate::ast::Trigger::StartOf(target) => {
                                   // Note: StartOf TriggerBlock is unusual? Plan usually uses EventBlock for distinct events.
                                   // But TriggerBlock exists too. Treating same as EventBlock.
                                   if let Some(crate::ast::Definition::Value(val_def)) = defs.get(target) {
                                       if let Some(next) = Scheduler::next_occurrence(val_def, start_time) {
                                            // Convert TriggerBlock to EventBlock-like structure or just execute stmts
                                            // We need a way to schedule "TriggerBlock execution". 
                                            // Reusing EventKind::StartOf logic but with wrappers?
                                            // Actually I'll clone stmts.
                                            // Actually TriggerBlock for StartOf is rare in my examples, usually EventBlock.
                                            // But let's handle it.
                                            // Wait, EventKind::StartOf expects EventBlock.
                                            // I'll make EventKind generic or multiple variants.
                                            // Or simplified: Just store stmts + reschedule info.
                                            // For now, skipping StartOf in TriggerBlock (unlikely usage).
                                            println!("DEBUG: StartOf in TriggerBlock not fully supported yet.");
                                       }
                                   }
                               },
                               _ => {}
                          }
                     }
                     crate::ast::PlanBlock::Event(block) => {
                         if let crate::ast::Trigger::StartOf(target) = &block.trigger {
                             if let Some(crate::ast::Definition::Value(val_def)) = defs.get(target) {
                                 if let Some(next) = Scheduler::next_occurrence(val_def, start_time) {
                                     println!("DEBUG: Scheduled '{}' at {}", block.name, next);
                                     events.push(ScheduledEvent {
                                         time: next,
                                         kind: EventKind::StartOf {
                                             block: block.clone(),
                                             period_name: target.clone(),
                                         }
                                     });
                                 } else {
                                     println!("DEBUG: Could not schedule '{}', no valid time found.", block.name);
                                 }
                             } else {
                                 println!("DEBUG: Definition for '{}' not found.", target);
                             }
                         }
                     }
                 }
             }

             // Event Loop
             while let Some(event) = events.pop() {
                 let now = event.time;
                 
                 if let Some(limit) = end_time {
                     if now > limit {
                         println!("DEBUG: Simulation time limit reached.");
                         break;
                     }
                 }

                 // Advance time
                 match self.mode {
                     ExecutionMode::RealTime => {
                         if now > env.now {
                             let diff = (now - env.now).to_std().unwrap_or(std::time::Duration::from_secs(0));
                             std::thread::sleep(diff);
                         }
                     }
                     ExecutionMode::Simulation(_) => {
                         // Instant jump
                     }
                 }
                 env.set_time(now);

                 // Execute
                 match event.kind {
                     EventKind::Periodic { block, iteration, interval_secs, max_duration } => {
                          self.execute_block(env, &block.statements);
                          
                          // Reschedule
                          let next_time = now + chrono::Duration::milliseconds((interval_secs * 1000.0) as i64);
                          let elapsed = next_time - start_time;
                          
                          let stop = if let Some(max) = max_duration {
                              elapsed > max
                          } else {
                              false
                          };

                          if !stop {
                              events.push(ScheduledEvent {
                                  time: next_time,
                                  kind: EventKind::Periodic { block, iteration: iteration + 1, interval_secs, max_duration }
                              });
                          }
                     }
                     EventKind::StartOf { block, period_name } => {
                          println!("DEBUG: Executing Event '{}'", block.name);
                          self.execute_block(env, &block.statements);
                          
                          // Reschedule next occurrence
                          if let Some(crate::ast::Definition::Value(val_def)) = defs.get(&period_name) {
                              if let Some(next) = Scheduler::next_occurrence(val_def, now) {
                                  // Ensure we don't schedule same time again loop
                                  if next > now {
                                      events.push(ScheduledEvent {
                                          time: next,
                                          kind: EventKind::StartOf { block, period_name }
                                      });
                                  }
                              }
                          }
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
        if stmt.line > 0 {
             if let Some(cb) = &self.on_step {
                 cb(stmt.line);
             }
        }

        match &stmt.kind {
            StatementKind::Action(action) => self.execute_action(env, action),
            StatementKind::Assignment(assign) => {
                let val = Evaluator::evaluate(env, &assign.expression);
                env.set_value(&assign.target, val);
            }
            StatementKind::Command(cmd) => {
                env.log(format!("Command: {}", cmd));
            }
            StatementKind::EventProgression(target_name, cases) => {
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
                             let v = Evaluator::evaluate(env, &v_expr);
                             match (&val, &v) {
                                  (crate::domain::RuntimeValue::Number(a), crate::domain::RuntimeValue::Number(b)) => (a - b).abs() < f64::EPSILON,
                                  _ => val == v,
                             }
                         }
                         crate::ast::RangeSelector::Range(min_expr, max_expr) => {
                             let min = Evaluator::evaluate(env, &min_expr);
                             let max = Evaluator::evaluate(env, &max_expr);
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
            StatementKind::Conditional(cond) => {
                 self.execute_conditional(env, cond);
            }
            StatementKind::ContextBlock(cb) => {
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
        
        // Context-aware resolution
        let val = if let crate::domain::RuntimeValue::String(s) = &val {
             if let Some(resolved) = env.get_value(s) {
                 resolved.clone()
             } else {
                 val
             }
        } else {
            val
        };

        for case in &cond_stmt.cases {
             let selector = &case.condition; 
             let is_match = match selector {
                 crate::ast::RangeSelector::Equals(v_expr) => {
                     let v = Evaluator::evaluate(env, &v_expr);
                     match (&val, &v) {
                         (crate::domain::RuntimeValue::Number(a), crate::domain::RuntimeValue::Number(b)) => (a - b).abs() < f64::EPSILON,
                         _ => val == v,
                     }
                 }
                 crate::ast::RangeSelector::Range(min_expr, max_expr) => {
                     let min = Evaluator::evaluate(env, &min_expr);
                     let max = Evaluator::evaluate(env, &max_expr);
                     match (&val, &min, &max) {
                         (crate::domain::RuntimeValue::Number(v), crate::domain::RuntimeValue::Number(min_v), crate::domain::RuntimeValue::Number(max_v)) => {
                             v >= min_v && v <= max_v
                         }
                         _ => false, 
                     }
                 }
                 _ => false, 
             };
             
             if is_match {
                 self.execute_block(env, &case.block);
                 break;
             }
        }
    }

    fn emit_log(&self, msg: String, timestamp: chrono::DateTime<chrono::Utc>) {
        if let Some(cb) = &self.on_log {
            cb(msg, timestamp);
        }
    }

    fn execute_action(&mut self, env: &mut Environment, action: &Action) {
        match action {
            Action::ShowMessage(parts, _) => {
                let mut full_msg = String::new();
                for part in parts {
                    let val = Evaluator::evaluate(env, part);
                    let s = val.to_string(); 
                    full_msg.push_str(&s);
                }
                let msg = full_msg;
                // env.log is internal debug log, keep as is
                env.log(msg.clone());
                // Emit log with current env time
                self.emit_log(format!("Message: {}", msg), env.now);
            }
            Action::AskQuestion(q, _) => {
                let msg = format!("Action: Ask Question '{}'", q);
                env.log(msg.clone());
                self.emit_log(msg, env.now);
            }
            Action::SendInfo(msg_text, vars) => {
                 let vals: Vec<String> = vars.iter().map(|e| format!("{:?}", Evaluator::evaluate(env, e))).collect();
                 let msg = format!("Action: Send Info '{}' with values: {:?}", msg_text, vals);
                 env.log(msg.clone());
                 self.emit_log(msg, env.now);
            }
            Action::ListenFor(val) => {
                let msg = format!("Action: Listen For '{}'", val);
                env.log(msg.clone());
                self.emit_log(msg, env.now);
            }
             _ => {}
        }
    }
}
