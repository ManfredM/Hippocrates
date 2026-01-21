use crate::ast::{Action, Block, Statement, StatementKind};
use crate::domain::Unit;
use crate::runtime::{Environment, Evaluator, format_identifier, normalize_identifier, scheduler::Scheduler};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, atomic::{AtomicBool, Ordering as AtomicOrdering}};

#[derive(Debug, Clone)]
struct ScheduledEvent {
    time: chrono::NaiveDateTime,
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

#[derive(Debug, Clone, Copy)]
pub enum ExecutionMode {
    RealTime,
    Simulation { speed_factor: Option<f64>, duration: Option<chrono::Duration> },
}

pub struct Executor {
    pub on_step: Option<Box<dyn Fn(usize) + Send>>,
    pub on_log: Option<Box<dyn Fn(String, crate::domain::EventType, chrono::NaiveDateTime) + Send>>,
    pub on_ask: Option<Box<dyn Fn(crate::domain::AskRequest) + Send>>,
    pub mode: ExecutionMode,
    pub input_receiver: Option<std::sync::mpsc::Receiver<crate::domain::InputMessage>>,
    pub stop_signal: Arc<AtomicBool>,
    next_event_time: Option<chrono::NaiveDateTime>,
    pending_inputs: Vec<crate::domain::InputMessage>,
}

impl Executor {
    pub fn new(stop_signal: Arc<AtomicBool>) -> Self {
        Executor {
            on_step: None,
            on_log: None,
            on_ask: None,
            mode: ExecutionMode::RealTime,
            input_receiver: None,
            stop_signal,
            next_event_time: None,
            pending_inputs: Vec::new(),
        }
    }

    pub fn with_activites(
        line_cb: Box<dyn Fn(usize) + Send>,
        log_cb: Box<dyn Fn(String, crate::domain::EventType, chrono::NaiveDateTime) + Send>,
    ) -> Self {
        Executor {
            on_step: Some(line_cb),
            on_log: Some(log_cb),
            on_ask: None,
            mode: ExecutionMode::RealTime,
            input_receiver: None,
            stop_signal: Arc::new(AtomicBool::new(false)),
            next_event_time: None,
            pending_inputs: Vec::new(),
        }
    }

    pub fn set_input_receiver(&mut self, rx: std::sync::mpsc::Receiver<crate::domain::InputMessage>) {
        self.input_receiver = Some(rx);
    }

    pub fn set_mode(&mut self, mode: ExecutionMode) {
        self.mode = mode;
    }

    pub fn drain_inputs(&mut self, env: &mut Environment) {
        let mut incoming = Vec::new();
        if let Some(rx) = &self.input_receiver {
            while let Ok(msg) = rx.try_recv() {
                incoming.push(msg);
            }
        }
        for mut msg in incoming {
            msg.variable = normalize_identifier(&msg.variable);
            self.pending_inputs.push(msg);
        }
        self.pending_inputs.sort_by_key(|item| item.timestamp);
        self.apply_due_inputs(env, false);
    }

    fn drain_inputs_with_triggers(&mut self, env: &mut Environment) {
        let rx_opt = self.input_receiver.take();
        let mut incoming = Vec::new();
        if let Some(rx) = &rx_opt {
            while let Ok(msg) = rx.try_recv() {
                incoming.push(msg);
            }
        }
        for mut msg in incoming {
            msg.variable = normalize_identifier(&msg.variable);
            self.pending_inputs.push(msg);
        }
        self.pending_inputs.sort_by_key(|item| item.timestamp);
        self.input_receiver = rx_opt;
        self.apply_due_inputs(env, true);
    }

    pub fn execute_plan(&mut self, env: &mut Environment, plan_name: &str) {
        env.log(format!("Starting plan: {}", plan_name));

        // Hacky clone for prototype
        let defs = env.definitions.clone();

        if let Some(crate::ast::Definition::Plan(plan_def)) = defs.get(plan_name) {
            // Drain initial inputs (configuration)
            self.drain_inputs(env);


            let mut events = BinaryHeap::new();
            let start_time = env.now;
            let start_of_window = chrono::Duration::days(30);
            let mut start_of_windows: HashMap<String, chrono::NaiveDateTime> = HashMap::new();
            
            let end_time = match self.mode {
                ExecutionMode::Simulation { duration, .. } => duration.map(|d| start_time + d),
                _ => None,
            };

            // Initial Scheduling
            for block in &plan_def.blocks {
                match block {
                    crate::ast::PlanBlock::DuringPlan(stmts) => {
                        self.execute_block(env, stmts);
                    }
                    crate::ast::PlanBlock::Trigger(block) => {
                        match &block.trigger {
                            crate::ast::Trigger::Periodic {
                                interval,
                                interval_unit,
                                duration,
                                ..
                            } => {
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
                                let first_run = start_time
                                    + chrono::Duration::milliseconds(
                                        (interval_secs * 1000.0) as i64,
                                    );

                                events.push(ScheduledEvent {
                                    time: first_run,
                                    kind: EventKind::Periodic {
                                        block: block.clone(),
                                        iteration: 0,
                                        interval_secs,
                                        max_duration: max_dur,
                                    },
                                });
                            }
                            crate::ast::Trigger::StartOf(target) => {
                                if target == plan_name {

                                    events.push(ScheduledEvent {
                                        time: start_time,
                                        kind: EventKind::Periodic {
                                            block: block.clone(),
                                            iteration: 0,
                                            interval_secs: 0.0, // 0.0 means one-shot/no repeat
                                            max_duration: None,
                                        },
                                    });
                                } else if let Some(def) = defs.get(target) {
                                    if let Some((_next, _)) = Scheduler::next_occurrence(def, start_time) {
                                        // ...
                                        println!(
                                            "DEBUG: StartOf in TriggerBlock for other plans not fully supported yet."
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    crate::ast::PlanBlock::Event(block) => {
                        if let crate::ast::Trigger::StartOf(target) = &block.trigger {
                            if let Some(def) = defs.get(target) {
                                let window_end = start_time + start_of_window;

                                let occurrences =
                                    Scheduler::occurrences_in_range(def, start_time, window_end);
                                if occurrences.is_empty() {
                                    println!(
                                        "DEBUG: Could not schedule '{}', no valid time found.",
                                        block.name
                                    );
                                }
                                for (start, _end) in occurrences {
                                    events.push(ScheduledEvent {
                                        time: start,
                                        kind: EventKind::StartOf {
                                            block: block.clone(),
                                            period_name: target.clone(),
                                        },
                                    });
                                }

                                let window_key =
                                    Self::start_of_window_key(&block.name, target);
                                start_of_windows.insert(window_key, window_end);
                            } else {
                                println!("DEBUG: Definition for '{}' not found.", target);
                            }
                        }
                    }
                }
            }

            // Event Loop
            while let Some(event) = events.pop() {
                if self.stop_signal.load(AtomicOrdering::Relaxed) {
                     break;
                }

                let now = event.time;
                self.next_event_time = events.peek().map(|next| next.time);


                if let Some(limit) = end_time {
                    if now > limit {
                        println!(
                            "DEBUG [{}] Simulation time limit reached.",
                            now.format("%Y-%m-%d %H:%M:%S")
                        );
                        break;
                    }
                }

                // Advance time
                // Advance time
                match self.mode {
                    ExecutionMode::RealTime => {
                        if now > env.now {
                            let diff = (now - env.now)
                                .to_std()
                                .unwrap_or(std::time::Duration::from_secs(0));
                            std::thread::sleep(diff);
                        }
                    }
                    ExecutionMode::Simulation { speed_factor, .. } => {
                        if let Some(factor) = speed_factor {
                             if now > env.now {
                                let diff = (now - env.now)
                                    .to_std()
                                    .unwrap_or(std::time::Duration::from_secs(0));
                                // Sleep with speed factor. factor 1.0 = real time. 10.0 = 10x faster (sleep 1/10th)
                                // factor > 0
                                let duration_secs = diff.as_secs_f64();
                                let sleep_secs = duration_secs / factor;
                                std::thread::sleep(std::time::Duration::from_secs_f64(sleep_secs));
                             }
                        }
                        // If speed_factor is None, instant jump (timelapse/instant)
                    }
                }
                env.set_time(now);
                self.drain_inputs_with_triggers(env);

                // Execute
                match event.kind {
                    EventKind::Periodic {
                        block,
                        iteration,
                        interval_secs,
                        max_duration,
                    } => {
                        self.execute_block(env, &block.statements);

                        // Reschedule
                        let next_time =
                            now + chrono::Duration::milliseconds((interval_secs * 1000.0) as i64);
                        let elapsed = next_time - start_time;

                        let stop = if let Some(max) = max_duration {
                            elapsed > max
                        } else {
                            false
                        };

                        if !stop && interval_secs > 0.0 {
                            events.push(ScheduledEvent {
                                time: next_time,
                                kind: EventKind::Periodic {
                                    block,
                                    iteration: iteration + 1,
                                    interval_secs,
                                    max_duration,
                                },
                            });
                        }
                    }
                    EventKind::StartOf { block, period_name } => {
                        self.execute_block(env, &block.statements);

                        let window_key = Self::start_of_window_key(&block.name, &period_name);
                        if let Some(window_end) = start_of_windows.get_mut(&window_key) {
                            let desired_end = now + start_of_window;

                            if desired_end > *window_end {
                                if let Some(def) = defs.get(&period_name) {
                                    let occurrences = Scheduler::occurrences_in_range(
                                        def,
                                        *window_end,
                                        desired_end,
                                    );
                                    for (start, _end) in occurrences {
                                        events.push(ScheduledEvent {
                                            time: start,
                                            kind: EventKind::StartOf {
                                                block: block.clone(),
                                                period_name: period_name.clone(),
                                            },
                                        });
                                    }
                                }
                                *window_end = desired_end;
                            }
                        }
                    }
                }
            }
            self.next_event_time = None;
        } else {
            println!(
                "DEBUG: Plan '{}' not found. Available: {:?}",
                plan_name,
                defs.keys()
            );
            env.log(format!("Plan not found: {}", plan_name));
        }
    }

    fn start_of_window_key(block_name: &str, period_name: &str) -> String {
        format!("{}|{}", block_name, period_name)
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
            StatementKind::Timeframe(block) => {
                if block.for_analysis {
                    let ctx = crate::runtime::environment::EvaluationContext::from_constraints(
                        &env.definitions,
                        &block.constraints,
                    );
                    env.push_context(ctx);
                    for s in &block.block {
                        self.execute_statement(env, s);
                    }
                    env.pop_context();
                } else {
                    for s in &block.block {
                        self.execute_statement(env, s);
                    }
                }
            }
            StatementKind::Action(action) => self.execute_action(env, action),
            StatementKind::Assignment(assign) => {
                let mut val = Evaluator::evaluate(env, &assign.expression);
                if let crate::domain::RuntimeValue::Missing(var_name) = val {
                    env.log(format!("Implicitly asking for missing variable: {}", var_name));
                    let answered = self.ask_question(env, &var_name, None);
                    if !answered {
                        return;
                    }
                    val = Evaluator::evaluate(env, &assign.expression);
                    if matches!(val, crate::domain::RuntimeValue::Missing(_)) {
                        return;
                    }
                }

                if let Some(expected_unit) = env.expected_unit_for_value(&assign.target) {
                    let is_count = matches!(
                        &assign.expression,
                        crate::ast::Expression::Statistical(
                            crate::ast::StatisticalFunc::CountOf(_, _)
                        )
                    );
                    if is_count {
                        if let crate::domain::RuntimeValue::Number(n) = val {
                            val = crate::domain::RuntimeValue::Quantity(n, expected_unit);
                        }
                    }
                }

                env.set_value(&assign.target, val.clone());
                env.log_audit(
                    crate::domain::EventType::Decision,
                    format!("Assigned variable: {} = {}", assign.target, val),
                    Some("Assignment".to_string()),
                );
            }
            StatementKind::Command(cmd) => {
                env.log(format!("Command: {}", cmd));
            }
            StatementKind::Constraint(expr, op, sel) => {
                 // For now, constraints are declarative and enforced by validator or specific logic (like ValidValues)
                 // Just log for debugging
                 env.log(format!("Constraint: {:?} {} {:?}", expr, op, sel));
            }
            StatementKind::NoOp => {}
            StatementKind::EventProgression(target_name, cases) => {
                let val = if let Some(v) = env.get_value(target_name) {
                    v.clone()
                } else {
                    env.log(format!(
                        "Warning: Variable '{}' not found for assessment",
                        target_name
                    ));
                    return;
                };

                // println!("DEBUG: Assessing '{}' (val={:?}) against {} cases", target_name, val, cases.len());

                for case in cases {
                    let selector = &case.condition;
                    let is_match = Evaluator::check_condition(env, selector, &val);

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
                let ctx = crate::runtime::environment::EvaluationContext {
                    timeframe,
                    period: None,
                };

                env.push_context(ctx);
                self.execute_block(env, &cb.statements);
                env.pop_context();
            }

        }
    }

    fn execute_conditional(&mut self, env: &mut Environment, cond_stmt: &crate::ast::Conditional) {
        let mut val = match &cond_stmt.condition {
            crate::ast::ConditionalTarget::Expression(expr) => Evaluator::evaluate(env, expr),
            crate::ast::ConditionalTarget::Confidence(_ident) => {
                // Stub: return high confidence
                crate::domain::RuntimeValue::Number(100.0)
            }
        };

        if let crate::domain::RuntimeValue::Missing(var_name) = val {
            env.log(format!(
                "Implicitly asking for missing variable in conditional: {}",
                var_name
            ));
            let answered = self.ask_question(env, &var_name, None);
            if !answered {
                return;
            }
            val = match &cond_stmt.condition {
                crate::ast::ConditionalTarget::Expression(expr) => Evaluator::evaluate(env, expr),
                crate::ast::ConditionalTarget::Confidence(_ident) => {
                    crate::domain::RuntimeValue::Number(100.0)
                }
            };
            if matches!(val, crate::domain::RuntimeValue::Missing(_)) {
                return;
            }
        }

        println!(
            "DEBUG [{}] Condition Value: {:?}",
            env.now.format("%Y-%m-%d %H:%M:%S"),
            val
        );

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
            let is_match = Evaluator::check_condition(env, selector, &val);
            println!(
                "DEBUG [{}] Checking selector {:?} (match={})",
                env.now.format("%Y-%m-%d %H:%M:%S"),
                selector,
                is_match
            );

            if is_match {
                self.execute_block(env, &case.block);
                break;
            }
        }
    }

    fn emit_log(&self, msg: String, event_type: crate::domain::EventType, timestamp: chrono::NaiveDateTime) {
        if let Some(cb) = &self.on_log {
            cb(msg, event_type, timestamp);
        }
    }

    fn ask_question(
        &mut self,
        env: &mut Environment,
        q: &str,
        block_opt: Option<&Vec<Statement>>,
    ) -> bool {
        // Build AskRequest
        let mut style = crate::domain::QuestionStyle::Text;
        let mut options = Vec::new();
        let mut range = None;
        let mut question_text = q.to_string();
        let variable_name = format_identifier(q);

        // Lookup definition
        let val_def_opt = if let Some(crate::ast::Definition::Value(val_def)) = env.definitions.get(q) {
            Some(val_def.clone())
        } else {
            env.log(format!(
                "DEBUG: Definition for '{}' not found. Available: {:?}",
                q,
                env.definitions.keys()
            ));
            None
        };

        if let Some(val_def) = &val_def_opt {
            // Determine style/options from definition
            for prop in &val_def.properties {
                match prop {
                    crate::ast::Property::ValidValues(stmts) => {
                        // Extract valid values as options
                        for stmt in stmts {
                            if let StatementKind::EventProgression(_, cases) = &stmt.kind {
                                for case in cases {
                                    match &case.condition {
                                        crate::ast::RangeSelector::Equals(expr) => {
                                            let v = Evaluator::evaluate(env, expr);
                                            // If string or enum, add to options
                                            if let crate::domain::RuntimeValue::String(s) = &v {
                                                options.push(s.clone());
                                            } else if let crate::domain::RuntimeValue::Enumeration(s) = &v {
                                                options.push(s.clone());
                                            } else {
                                                options.push(v.to_string());
                                            }
                                        }
                                        crate::ast::RangeSelector::Range(min, max) => {
                                            let min_v = Evaluator::evaluate(env, min);
                                            let max_v = Evaluator::evaluate(env, max);
                                            if let (crate::domain::RuntimeValue::Number(mn), crate::domain::RuntimeValue::Number(mx)) =
                                                (min_v.clone(), max_v.clone())
                                            {
                                                range = Some((mn, mx));
                                            } else if let (crate::domain::RuntimeValue::Quantity(mn, u1), crate::domain::RuntimeValue::Quantity(mx, u2)) =
                                                (min_v, max_v)
                                            {
                                                if u1 == u2 {
                                                    range = Some((mn, mx));
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    crate::ast::Property::Question(action) => {
                        // Check for explicit question text in definition
                        if let Action::AskQuestion(text, _) = action {
                            if !text.is_empty() {
                                question_text = text.clone();
                            }
                        }
                    }
                    crate::ast::Property::Reuse(val, unit) => {
                        // Check if we have a valid value in history
                        let last_entry_data = if let Some(history) = env.get_history(q) {
                            history.last().map(|entry| (entry.value.clone(), entry.timestamp))
                        } else {
                            None
                        };

                        if let Some((val_data, timestamp)) = last_entry_data {
                            // Calculate age
                            let age = env.now - timestamp;
                            let age_secs = age.num_seconds() as f64;

                            let reuse_secs = match unit {
                                Unit::Second => *val,
                                Unit::Minute => *val * 60.0,
                                Unit::Hour => *val * 3600.0,
                                Unit::Day => *val * 86400.0,
                                Unit::Week => *val * 604800.0,
                                Unit::Month => *val * 2592000.0,
                                Unit::Year => *val * 31536000.0,
                                _ => *val, // Fallback
                            };

                            if age_secs < reuse_secs {
                                // Ensure value is not NotEnoughData
                                if let crate::domain::RuntimeValue::NotEnoughData = val_data {
                                    // If value is NotEnoughData, it's not valid to reuse, so ask.
                                } else {
                                    env.log(format!(
                                        "Skipping question '{}', existing value is fresh (Age: {}s < Reuse: {}s)",
                                        q, age_secs, reuse_secs
                                    ));
                                    return true; // Skip asking!
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Infer Style
            if !options.is_empty() {
                style = crate::domain::QuestionStyle::Selection;
            } else if range.is_some() {
                style = crate::domain::QuestionStyle::Numeric;
            } else if val_def.value_type == crate::domain::ValueType::Enumeration {
                style = crate::domain::QuestionStyle::Selection;
            }
        }

        // Emit log with actual question text
        self.emit_log(question_text.clone(), crate::domain::EventType::Question, env.now);

        let mut validation_mode = None;
        let mut validation_timeout = None;

        if let Some(stmts) = block_opt {
            for stmt in stmts {
                if let StatementKind::Action(Action::ValidateAnswer(mode, timeout)) = &stmt.kind {
                    validation_mode = Some(mode.clone());
                    if let Some((val, unit)) = timeout {
                        let secs = match unit {
                            Unit::Second => *val,
                            Unit::Minute => *val * 60.0,
                            _ => *val,
                        };
                        validation_timeout = Some(secs as i64);
                    }
                }
            }
        }

        // Calculate valid_after timestamp from Reuse property if present
        let mut valid_after = None;
        if let Some(val_def) = &val_def_opt {
            for prop in &val_def.properties {
                if let crate::ast::Property::Reuse(val, unit) = prop {
                    let reuse_secs = match unit {
                        Unit::Second => *val,
                        Unit::Minute => *val * 60.0,
                        Unit::Hour => *val * 3600.0,
                        Unit::Day => *val * 86400.0,
                        Unit::Week => *val * 604800.0,
                        Unit::Month => *val * 2592000.0,
                        Unit::Year => *val * 31536000.0,
                        _ => *val,
                    };
                    let reuse_dur = chrono::Duration::milliseconds((reuse_secs * 1000.0) as i64);
                    valid_after = Some((env.now - reuse_dur).and_utc().timestamp_millis());
                }
            }
        }

        // Log the question to audit
        env.log_audit(
            crate::domain::EventType::Question,
            format!("Asked question for '{}': {}", q, question_text),
            Some("AskQuestion".to_string()),
        );

        // Fire callback if present
        if let Some(cb) = &self.on_ask {
            let req = crate::domain::AskRequest {
                variable_name: variable_name.clone(),
                question_text: question_text.clone(),
                style: style.clone(),
                options: options.clone(),
                range,
                validation_mode,
                validation_timeout,
                timestamp: env.now.and_utc().timestamp_millis(),
                valid_after,
            };
            cb(req);
        }

        // Wait for answer via Channel
        self.wait_for_answer(env, q)
    }

    fn wait_for_answer(&mut self, env: &mut Environment, target: &str) -> bool {
        let rx_opt = self.input_receiver.take();
        let mut answered = false;

        if self.apply_due_inputs_for_target(env, target) {
            self.input_receiver = rx_opt;
            return true;
        }

        if let Some(rx) = &rx_opt {
            if let Some(deadline) = self.next_event_time {
                let total_wait = match (deadline - env.now).to_std() {
                    Ok(dur) => dur,
                    Err(_) => std::time::Duration::from_secs(0),
                };
                let start = std::time::Instant::now();

                loop {
                    let elapsed = start.elapsed();
                    if elapsed >= total_wait {
                        break;
                    }

                    let remaining = total_wait.saturating_sub(elapsed);
                    match rx.recv_timeout(remaining) {
                        Ok(msg) => {
                            let normalized = normalize_identifier(&msg.variable);
                            let mut msg = msg;
                            msg.variable = normalized.clone();

                            if let ExecutionMode::Simulation { .. } = self.mode {
                                if msg.timestamp > env.now {
                                    if msg.timestamp > deadline {
                                        self.pending_inputs.push(msg);
                                        self.pending_inputs.sort_by_key(|item| item.timestamp);
                                        break;
                                    }
                                    env.set_time(msg.timestamp);
                                }
                            }

                            if normalized == target {
                                env.set_value_at(&normalized, msg.value.clone(), msg.timestamp);
                                self.emit_log(
                                    format!("Received Answer: {:?}", msg.value),
                                    crate::domain::EventType::Answer,
                                    env.now,
                                );
                                env.log_audit(
                                    crate::domain::EventType::Decision,
                                    format!(
                                        "Answered question for '{}' with value: {:?}",
                                        msg.variable, msg.value
                                    ),
                                    Some("AnswerQuestion".to_string()),
                                );

                                self.check_triggers(env, &normalized);
                                answered = true;
                            } else {
                                env.set_value_at(&normalized, msg.value.clone(), msg.timestamp);
                                self.check_triggers(env, &normalized);
                            }

                            if answered {
                                break;
                            }

                            if self.apply_due_inputs_for_target(env, target) {
                                answered = true;
                                break;
                            }
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            break;
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                            env.log("Input channel disconnected.".to_string());
                            debug_log("EXECUTOR: Channel disconnected.");
                            break;
                        }
                    }
                }
            } else {
                loop {
                    match rx.recv() {
                        Ok(msg) => {
                            let normalized = normalize_identifier(&msg.variable);
                            let mut msg = msg;
                            msg.variable = normalized.clone();

                            if let ExecutionMode::Simulation { .. } = self.mode {
                                if msg.timestamp > env.now {
                                    env.set_time(msg.timestamp);
                                }
                            }

                            if normalized == target {
                                env.set_value_at(&normalized, msg.value.clone(), msg.timestamp);
                                self.emit_log(
                                    format!("Received Answer: {:?}", msg.value),
                                    crate::domain::EventType::Answer,
                                    env.now,
                                );
                                env.log_audit(
                                    crate::domain::EventType::Decision,
                                    format!(
                                        "Answered question for '{}' with value: {:?}",
                                        msg.variable, msg.value
                                    ),
                                    Some("AnswerQuestion".to_string()),
                                );

                                self.check_triggers(env, &normalized);
                                answered = true;
                            } else {
                                env.set_value_at(&normalized, msg.value.clone(), msg.timestamp);
                                self.check_triggers(env, &normalized);
                            }

                            if answered {
                                break;
                            }

                            if self.apply_due_inputs_for_target(env, target) {
                                answered = true;
                                break;
                            }
                        }
                        Err(_) => {
                            env.log("Input channel disconnected.".to_string());
                            debug_log("EXECUTOR: Channel disconnected.");
                            break;
                        }
                    }
                }
            }
        } else {
            env.log("Warning: No input receiver configured. Skipping wait for answer.".to_string());
            debug_log("EXECUTOR: No receiver configured.");
        }

        self.input_receiver = rx_opt;
        answered
    }

    pub fn set_ask_callback(&mut self, cb: Box<dyn Fn(crate::domain::AskRequest) + Send>) {
        self.on_ask = Some(cb);
    }

    fn execute_action(&mut self, env: &mut Environment, action: &Action) {
        match action {
            Action::ShowMessage(parts, stmts_opt) => {
                for _ in 0..2 {
                    let mut full_msg = String::new();
                    let mut missing_var = None;

                    for part in parts {
                        let val = Evaluator::evaluate(env, part);
                        if let crate::domain::RuntimeValue::Missing(var_name) = val {
                            missing_var = Some(var_name);
                            break;
                        }
                        let s = val.to_string();
                        full_msg.push_str(&s);
                    }

                    if let Some(var_name) = missing_var {
                        env.log(format!(
                            "Implicitly asking for missing variable in message: {}",
                            var_name
                        ));
                        let answered = self.ask_question(env, &var_name, None);
                        if !answered {
                            return;
                        }
                        continue;
                    }

                    let message = full_msg;
                    env.log_audit(
                        crate::domain::EventType::Message,
                        message.clone(),
                        Some("ShowMessage".to_string()),
                    );
                    env.log(message.clone());

                    self.emit_log(message, crate::domain::EventType::Message, env.now);

                    if let Some(stmts) = stmts_opt {
                        self.execute_block(env, stmts);
                    }
                    return;
                }
            }
            Action::AskQuestion(q, block_opt) => {
                self.ask_question(env, q, block_opt.as_ref());
            }
            Action::SendInfo(msg_text, vars) => {
                let vals: Vec<String> = vars
                    .iter()
                    .map(|e| format!("{:?}", Evaluator::evaluate(env, e)))
                    .collect();
                let msg = format!("Action: Send Info '{}' with values: {:?}", msg_text, vals);
                env.log(msg.clone());
                self.emit_log(msg, crate::domain::EventType::Log, env.now);
            }
            Action::ListenFor(val) => {
                let msg = format!("Action: Listen For '{}'", val);
                env.log(msg.clone());
                self.emit_log(msg, crate::domain::EventType::Log, env.now);
            }

            Action::StartPeriod => {}
            _ => {}
        }
    }

    fn check_triggers(&mut self, env: &mut Environment, changed_var: &str) {
        let mut blocks_to_run = Vec::new();
        
        for def in env.definitions.values() {
             if let crate::ast::Definition::Plan(plan_def) = def {
                 for block in &plan_def.blocks {
                     match block {
                         crate::ast::PlanBlock::Trigger(trigger_block) => {
                             match &trigger_block.trigger {
                                 crate::ast::Trigger::ChangeOf(var) => {
                                     if var == changed_var {
                                         blocks_to_run.push(trigger_block.statements.clone());
                                     }
                                 }
                                 _ => {}
                             }
                         },
                         crate::ast::PlanBlock::Event(event_block) => {
                             match &event_block.trigger {
                                 crate::ast::Trigger::ChangeOf(var) => {
                                     if var == changed_var {
                                         blocks_to_run.push(event_block.statements.clone());
                                     }
                                 }
                                 _ => {}
                             }
                         },
                         _ => {}
                     }
                 }
             }
        }
        
        for block in blocks_to_run {
            self.execute_block(env, &block);
        }
    }
}

impl Executor {
    fn apply_due_inputs(&mut self, env: &mut Environment, with_triggers: bool) {
        let now = env.now;
        let mut due = Vec::new();
        let mut remaining = Vec::new();
        for msg in self.pending_inputs.drain(..) {
            if msg.timestamp <= now {
                due.push(msg);
            } else {
                remaining.push(msg);
            }
        }
        self.pending_inputs = remaining;

        for msg in due {
            env.set_value_at(&msg.variable, msg.value.clone(), msg.timestamp);
            if with_triggers {
                self.check_triggers(env, &msg.variable);
            }
        }
    }

    fn apply_due_inputs_for_target(&mut self, env: &mut Environment, target: &str) -> bool {
        let now = env.now;
        let mut due = Vec::new();
        let mut remaining = Vec::new();
        let mut answered = false;

        for msg in self.pending_inputs.drain(..) {
            if msg.timestamp <= now {
                due.push(msg);
            } else {
                remaining.push(msg);
            }
        }

        self.pending_inputs = remaining;

        for msg in due {
            if msg.variable == target && !answered {
                env.set_value_at(&msg.variable, msg.value.clone(), msg.timestamp);
                self.emit_log(
                    format!("Received Answer: {:?}", msg.value),
                    crate::domain::EventType::Answer,
                    env.now,
                );
                env.log_audit(
                    crate::domain::EventType::Decision,
                    format!(
                        "Answered question for '{}' with value: {:?}",
                        msg.variable, msg.value
                    ),
                    Some("AnswerQuestion".to_string()),
                );
                self.check_triggers(env, &msg.variable);
                answered = true;
            } else {
                env.set_value_at(&msg.variable, msg.value.clone(), msg.timestamp);
                self.check_triggers(env, &msg.variable);
            }
        }

        answered
    }
}

fn debug_log(msg: &str) {
    use std::io::Write;
    let path = "/tmp/hippocrates_engine.log";
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{} - {}", chrono::Utc::now(), msg);
    }
}
