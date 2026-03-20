// Spec §4.6, §5: execution model behaviors.

use hippocrates_engine::domain::{EventType, RuntimeValue, Unit};
use hippocrates_engine::parser;
use hippocrates_engine::runtime::{Engine, Environment, Executor, ExecutionMode};
use chrono::{Utc, TimeZone, Duration as ChronoDuration};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

// REQ-4.6-03: runtime evaluation returns NotEnoughData when history is insufficient.
#[test]
fn spec_not_enough_data_evaluation() {
    use hippocrates_engine::ast::{Expression, StatisticalFunc, RangeSelector, RelativeDirection};
    use hippocrates_engine::runtime::evaluator::Evaluator;
    use hippocrates_engine::runtime::environment::{Environment as EvalEnv, EvaluationContext};

    let mut env = EvalEnv::new();
    let start_time = Utc::now().naive_utc();
    env.set_start_time(start_time);
    env.set_time(start_time + ChronoDuration::days(2));

    env.set_value("incident", RuntimeValue::Boolean(true));

    let five_days_ago = Expression::RelativeTime(5.0, Unit::Day, RelativeDirection::Ago);
    let now_expr = Expression::Variable("now".to_string());

    let ctx = EvaluationContext {
        timeframe: Some(RangeSelector::Range(five_days_ago, now_expr)),
        period: None,
    };
    env.push_context(ctx);

    let count_expr = Expression::Statistical(StatisticalFunc::CountOf(
        "incident".to_string(),
        None,
    ));

    let res = Evaluator::evaluate(&env, &count_expr);
    env.pop_context();

    if let RuntimeValue::NotEnoughData = res {
    } else {
        panic!("Expected NotEnoughData, got {:?}", res);
    }

    env.set_time(start_time + ChronoDuration::days(6));
    env.set_value("incident", RuntimeValue::Boolean(true));

    let five_days_ago = Expression::RelativeTime(5.0, Unit::Day, RelativeDirection::Ago);
    let now_expr = Expression::Variable("now".to_string());

    let ctx = EvaluationContext {
        timeframe: Some(RangeSelector::Range(five_days_ago, now_expr)),
        period: None,
    };
    env.push_context(ctx);
    let res_valid = Evaluator::evaluate(&env, &count_expr);
    env.pop_context();

    if let RuntimeValue::Number(n) = res_valid {
        assert!(n >= 1.0, "Expected valid count >= 1.0, got {}", n);
    } else {
        panic!("Expected Number, got {:?}", res_valid);
    }
}

// REQ-4.7-01: date/time valid value ranges evaluate using date/time and time-of-day semantics.
#[test]
fn spec_date_time_range_evaluation() {
    use hippocrates_engine::ast::{Expression, Literal, RangeSelector};
    use hippocrates_engine::runtime::evaluator::Evaluator;
    use chrono::NaiveDate;

    let env = Environment::new();

    let date_range = RangeSelector::Range(
        Expression::Literal(Literal::Date("2026-01-01".to_string())),
        Expression::Literal(Literal::Date("2026-01-10".to_string())),
    );
    let date_value = RuntimeValue::Date(
        NaiveDate::from_ymd_opt(2026, 1, 5)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    );
    assert!(Evaluator::check_condition(&env, &date_range, &date_value));

    let time_range = RangeSelector::Range(
        Expression::Literal(Literal::TimeOfDay("10:00".to_string())),
        Expression::Literal(Literal::TimeOfDay("20:00".to_string())),
    );
    let time_value = RuntimeValue::Date(
        NaiveDate::from_ymd_opt(2026, 1, 5)
            .unwrap()
            .and_hms_opt(15, 0, 0)
            .unwrap(),
    );
    assert!(Evaluator::check_condition(&env, &time_range, &time_value));
}

// REQ-4.7-02: date diff expressions evaluate to quantities in requested units.
#[test]
fn spec_date_diff_evaluation() {
    use hippocrates_engine::ast::{Expression, Literal};
    use hippocrates_engine::runtime::evaluator::Evaluator;

    let env = Environment::new();
    let expr = Expression::DateDiff(
        Unit::Day,
        Box::new(Expression::Literal(Literal::Date("2026-01-01".to_string()))),
        Box::new(Expression::Literal(Literal::Date("2026-01-11".to_string()))),
    );

    let result = Evaluator::evaluate(&env, &expr);
    assert_eq!(result, RuntimeValue::Quantity(10.0, Unit::Day));
}

// REQ-5.3-01: meaning evaluation returns the assessed meaning for the value.
#[test]
fn spec_meaning_of_evaluates() {
    use hippocrates_engine::ast::Expression;
    use hippocrates_engine::runtime::evaluator::Evaluator;

    let input = r#"
<weight> is a number:
    unit is kg.
    valid values:
        1 kg ... 1000 kg.
    question:
        ask "What is the weight".
    meaning of <weight>:
        valid meanings:
            <light>; <heavy>; <super heavy>.
        assess meaning of <weight>:
            1 kg ... 100 kg:
                <light>.
            101 kg ... 900 kg:
                <heavy>.
            901 kg ... 1000 kg:
                <super heavy>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");
    let mut env = Environment::new();
    env.load_plan(plan);
    env.set_value("weight", RuntimeValue::Quantity(70.0, Unit::Kilogram));

    let result = Evaluator::evaluate(&env, &Expression::MeaningOf("weight".to_string()));
    assert_eq!(result, RuntimeValue::String("light".to_string()));
}

// REQ-5.3-02: meaning evaluation returns Missing when the source value is unknown.
#[test]
fn spec_meaning_of_missing_value() {
    use hippocrates_engine::ast::Expression;
    use hippocrates_engine::runtime::evaluator::Evaluator;

    let input = r#"
<weight> is a number:
    unit is kg.
    valid values:
        1 kg ... 1000 kg.
    question:
        ask "What is the weight".
    meaning of <weight>:
        valid meanings:
            <light>; <heavy>.
        assess meaning of <weight>:
            1 kg ... 100 kg:
                <light>.
            101 kg ... 1000 kg:
                <heavy>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");
    let mut env = Environment::new();
    env.load_plan(plan);

    let result = Evaluator::evaluate(&env, &Expression::MeaningOf("weight".to_string()));
    assert_eq!(result, RuntimeValue::Missing("weight".to_string()));
}

// REQ-5.3-03: meaning evaluation supports nested assessments.
#[test]
fn spec_meaning_of_nested_assessment() {
    use hippocrates_engine::ast::Expression;
    use hippocrates_engine::runtime::evaluator::Evaluator;

    let input = r#"
<pediatric age> is a number:
    unit is months.
    valid values:
        0 months ... 12 months.
    question:
        ask "How old is the child".

<age> is a number:
    unit is years.
    valid values:
        0 years ... 150 years.
    question:
        ask "What is your age".
    meaning of <age>:
        valid meanings:
            <just born>; <newborn>; <young>; <best age>; <old>.
        assess meaning of <age>:
            0 years ... 1 year:
                assess <pediatric age>:
                    0 months:
                        <just born>.
                    1 month ... 12 months:
                        <newborn>.
            2 years ... 20 years:
                <young>.
            21 years ... 100 years:
                <best age>.
            101 years ... 150 years:
                <old>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");
    let mut env = Environment::new();
    env.load_plan(plan);
    env.set_value("age", RuntimeValue::Quantity(0.0, Unit::Year));
    env.set_value("pediatric age", RuntimeValue::Quantity(6.0, Unit::Month));

    let result = Evaluator::evaluate(&env, &Expression::MeaningOf("age".to_string()));
    assert_eq!(result, RuntimeValue::String("newborn".to_string()));
}

// REQ-5.2-01: numeric answers must respect the decimal precision implied by valid values.
#[test]
fn spec_numeric_input_precision_rejection() {
    use hippocrates_engine::domain::InputMessage;
    use std::sync::atomic::AtomicBool;
    use std::sync::mpsc;

    let input = r#"
<hours since meal> is a number:
    unit is hours.
    valid values:
        0 hours ... 24 hours.
    question:
        ask "How many hours ago did you eat?".

<intake> is a plan:
    during plan:
        ask <hours since meal>.
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse plan");
    let mut env = Environment::new();
    env.load_plan(plan);

    assert!(
        env.definitions.contains_key("hours since meal"),
        "Expected normalized value definition"
    );

    assert!(
        hippocrates_engine::runtime::input_validation::validate_input_value(
            &env.definitions,
            "hours since meal",
            &RuntimeValue::Number(10.3)
        )
        .is_err(),
        "Expected precision validation to reject decimal input"
    );

    let (tx, rx) = mpsc::channel();
    let mut executor = Executor::new(Arc::new(AtomicBool::new(false)));
    executor.set_input_receiver(rx);

    let tx_clone = tx.clone();
    executor.set_ask_callback(Box::new(move |req: hippocrates_engine::domain::AskRequest| {
        let now = Utc::now().naive_utc();
        let invalid = InputMessage {
            variable: req.variable_name.clone(),
            value: RuntimeValue::Number(10.3),
            timestamp: now,
        };
        let valid = InputMessage {
            variable: req.variable_name.clone(),
            value: RuntimeValue::Number(10.0),
            timestamp: now,
        };
        tx_clone.send(invalid).expect("Failed to send invalid input");
        tx_clone.send(valid).expect("Failed to send valid input");
    }));

    executor.execute_plan(&mut env, "intake");

    let history = env
        .get_history("hours since meal")
        .expect("Expected value history");

    assert!(
        !history.iter().any(|entry| matches!(entry.value, RuntimeValue::Number(n) if (n - 10.3).abs() < 1e-9)),
        "Decimal input should be rejected"
    );

    let last = history.last().expect("Expected final value");
    assert!(
        matches!(last.value, RuntimeValue::Number(n) if (n - 10.0).abs() < 1e-9),
        "Integer input should be accepted"
    );
}

// REQ-5-01: runtime executes assignments and actions in order.
#[test]
fn spec_runtime_execution_flow() {
    let input = r#"
<test plan> is a plan:
    during plan:
        information "Hello World".
        <x> = 10 kg.
        send information "Val is " <x>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");
    let mut engine = Engine::new();
    engine.load_plan(plan);
    engine.execute("test plan");

    let logs = &engine.env.output_log;
    assert!(logs.iter().any(|s| s.contains("Hello World")));
    assert!(
        logs.iter()
            .any(|s| s.contains("Val is ") && s.contains("Quantity(10.0, Kilogram)")),
        "Logs missing Val is Quantity(10.0, Kilogram): {:?}",
        logs
    );

    if let Some(val) = engine.env.get_value("x") {
        assert_eq!(val, &RuntimeValue::Quantity(10.0, Unit::Kilogram));
    } else {
        panic!("Variable x not found");
    }
}

// REQ-5-03: runtime emits a warning when a message action executes without a message callback.
#[test]
fn spec_message_callback_missing_warns() {
    let input = r#"
<plan> is a plan:
    during plan:
        information to <patient> "Hello".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");
    let mut env = Environment::new();
    env.load_plan(plan);

    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();

    let line_cb = Box::new(|_line: usize| {});
    let log_cb = Box::new(move |msg: String, event_type: EventType, _ts| {
        if event_type == EventType::Log {
            logs_clone.lock().unwrap().push(msg);
        }
    });

    let mut executor = Executor::with_activites(line_cb, log_cb);
    executor.execute_plan(&mut env, "plan");

    let logs = logs.lock().unwrap();
    assert!(
        logs.iter().any(|msg| msg.contains("message callback not set")),
        "Expected warning when message callback missing, got: {:?}",
        *logs
    );
}

// REQ-5-02: reuse timeframes prevent re-asking within the validity window.
#[test]
fn spec_validity_reuse_timeframe() {
    use hippocrates_engine::Session;
    use hippocrates_engine::domain::{AskRequest, EventType};
    use std::thread;

    let asked_questions = Arc::new(Mutex::new(Vec::new()));
    let logs = Arc::new(Mutex::new(Vec::new()));

    let aq = asked_questions.clone();
    let l = logs.clone();

    let session = Arc::new(Session::new(
        Box::new(move |req: AskRequest| {
            aq.lock().unwrap().push(req.variable_name);
        }),
        Box::new(move |msg: String, _kind: EventType, _time: chrono::NaiveDateTime| {
            l.lock().unwrap().push(msg);
        }),
    ));

    let script = r#"
<Temp> is a number:
    unit is °C.
    valid values:
        35.0 °C ... 42.0 °C.
    reuse:
        reuse period of value is 5 seconds.

<CheckTemp> is a plan:
    during plan:
        ask <Temp>.
        information "Temp is " + <Temp>.
"#;

    session.run_script(script.to_string(), "CheckTemp".to_string());
    thread::sleep(Duration::from_millis(500));

    session.provide_answer("<Temp>", RuntimeValue::Quantity(37.0, Unit::Celsius));
    thread::sleep(Duration::from_millis(500));

    {
        let inputs = asked_questions.lock().unwrap();
        assert_eq!(inputs.len(), 1, "First run should ask");
    }

    session.run_script(script.to_string(), "CheckTemp".to_string());
    thread::sleep(Duration::from_millis(500));

    {
        let inputs = asked_questions.lock().unwrap();
        assert_eq!(inputs.len(), 1, "Second run should NOT ask (reuse)");
    }

    thread::sleep(Duration::from_secs(6));

    session.run_script(script.to_string(), "CheckTemp".to_string());
    thread::sleep(Duration::from_millis(500));

    {
        let inputs = asked_questions.lock().unwrap();
        assert_eq!(inputs.len(), 2, "Third run SHOULD ask (expired)");
    }

    session.provide_answer("<Temp>", RuntimeValue::Quantity(38.0, Unit::Celsius));
    thread::sleep(Duration::from_millis(500));
}

// REQ-3.12-02: timeframe filtering applies to statistical evaluations.
#[test]
fn spec_timeframe_filtering() {
    let input = r#"
<val> is an enumeration:
    valid values:
        <Yes>.

<filtered count> is a number:
    calculation:
        timeframe for analysis is 5 days ago ... now:
            <value> = count of <val> is <Yes>.
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut env = Environment::new();
    env.load_plan(plan);

    let now = Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - ChronoDuration::days(20));

    let ten_days_ago = now - ChronoDuration::days(10);
    env.set_time(ten_days_ago);
    env.set_value("val", RuntimeValue::Enumeration("Yes".to_string()));

    let one_day_ago = now - ChronoDuration::days(1);
    env.set_time(one_day_ago);
    env.set_value("val", RuntimeValue::Enumeration("Yes".to_string()));

    env.set_time(now);

    let expr = hippocrates_engine::ast::Expression::Variable("filtered count".to_string());
    let result = hippocrates_engine::runtime::Evaluator::evaluate(&env, &expr);

    if let RuntimeValue::Number(n) = result {
        assert_eq!(n, 1.0, "Expected count of 1 (excluding 10 days ago), got {}", n);
    } else {
        panic!("Expected number result, got {:?}", result);
    }
}

// REQ-3.12-03: timeframe variants resolve counts over different windows.
#[test]
fn spec_timeframe_variants() {
    let input = r#"
<val> is an enumeration:
    valid values:
        <Yes>.

<count_old> is a number:
    calculation:
        timeframe for analysis is 15 days ago ... 5 days ago:
            <value> = count of <val> is <Yes>.

<count_recent> is a number:
    calculation:
        timeframe for analysis is 5 days ago ... now:
            <value> = count of <val> is <Yes>.
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut env = Environment::new();
    env.load_plan(plan);

    let now = Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - ChronoDuration::days(20));

    let ten_days_ago = now - ChronoDuration::days(10);
    env.set_time(ten_days_ago);
    env.set_value("val", RuntimeValue::Enumeration("Yes".to_string()));

    let one_day_ago = now - ChronoDuration::days(1);
    env.set_time(one_day_ago);
    env.set_value("val", RuntimeValue::Enumeration("Yes".to_string()));

    env.set_time(now);

    let expr_old = hippocrates_engine::ast::Expression::Variable("count_old".to_string());
    let res_old = hippocrates_engine::runtime::Evaluator::evaluate(&env, &expr_old);
    if let RuntimeValue::Number(n) = res_old {
        assert_eq!(n, 1.0, "Expected count_old to be 1 (10 days ago), got {}", n);
    } else {
        panic!("count_old failed: {:?}", res_old);
    }

    let expr_recent = hippocrates_engine::ast::Expression::Variable("count_recent".to_string());
    let res_recent = hippocrates_engine::runtime::Evaluator::evaluate(&env, &expr_recent);
    if let RuntimeValue::Number(n) = res_recent {
        assert_eq!(n, 1.0, "Expected count_recent to be 1 (1 day ago), got {}", n);
    } else {
        panic!("count_recent failed: {:?}", res_recent);
    }
}

// REQ-3.12-04: trend analysis evaluates statistical trends over timeframes.
#[test]
fn spec_trend_analysis_evaluates() {
    use hippocrates_engine::ast::{Expression, RangeSelector, RelativeDirection, StatisticalFunc};
    use hippocrates_engine::runtime::environment::EvaluationContext;

    let mut env = Environment::new();
    let now = Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - ChronoDuration::days(20));

    env.set_value_at("systolic", RuntimeValue::Number(10.0), now - ChronoDuration::days(5));
    env.set_value_at("systolic", RuntimeValue::Number(20.0), now - ChronoDuration::days(4));
    env.set_value_at("systolic", RuntimeValue::Number(30.0), now - ChronoDuration::days(3));

    let start_expr = Expression::RelativeTime(6.0, Unit::Day, RelativeDirection::Ago);
    let end_expr = Expression::RelativeTime(0.0, Unit::Second, RelativeDirection::Ago);

    let ctx = EvaluationContext {
        timeframe: Some(RangeSelector::Range(start_expr, end_expr)),
        period: None,
    };

    env.push_context(ctx);

    let trend_expr = Expression::Statistical(StatisticalFunc::TrendOf("systolic".to_string()));
    let result = hippocrates_engine::runtime::Evaluator::evaluate(&env, &trend_expr);

    if let RuntimeValue::String(s) = result {
        assert_eq!(s, "increase");
    } else {
        panic!("Expected string result for trend, got {:?}", result);
    }

    env.pop_context();
}

// REQ-3.8-03: scheduler computes next occurrence for periods.
#[test]
fn spec_scheduler_next_occurrence() {
    use hippocrates_engine::runtime::scheduler::Scheduler;
    use hippocrates_engine::ast::Definition;

    let input = r#"
<best period> is a period:
    timeframe:
        between Monday ... Friday; 07:40 ... 07:50.
"#;

    let plan_struct = parser::parse_plan(input).expect("Failed to parse");

    let def = plan_struct
        .definitions
        .iter()
        .find(|d| matches!(d, Definition::Period(_)))
        .expect("Def must exist");

    let now = Utc.with_ymd_and_hms(2026, 1, 18, 12, 0, 0).unwrap().naive_utc();
    let next = Scheduler::next_occurrence(def, now);
    assert!(next.is_some());
}

// STKR-16, DDR-RT-02: value history is append-only.
#[test]
fn spec_environment_append_only_history() {
    let mut env = Environment::new();
    let t1 = Utc.with_ymd_and_hms(2026, 1, 10, 8, 0, 0).unwrap().naive_utc();
    let t2 = Utc.with_ymd_and_hms(2026, 1, 10, 9, 0, 0).unwrap().naive_utc();

    env.set_start_time(t1 - ChronoDuration::hours(1));
    env.set_time(t1);

    env.set_value_at("x", RuntimeValue::Quantity(5.0, Unit::Kilogram), t1);
    env.set_value_at("x", RuntimeValue::Quantity(10.0, Unit::Kilogram), t2);

    let history = env.get_history("x").expect("Expected history for x");
    assert_eq!(history.len(), 2, "Expected 2 history entries, got {}", history.len());

    assert_eq!(history[0].value, RuntimeValue::Quantity(5.0, Unit::Kilogram));
    assert_eq!(history[0].timestamp, t1);

    assert_eq!(history[1].value, RuntimeValue::Quantity(10.0, Unit::Kilogram));
    assert_eq!(history[1].timestamp, t2);

    // Verify original T1 entry is unchanged after adding T2
    let history_again = env.get_history("x").unwrap();
    assert_eq!(history_again[0].value, RuntimeValue::Quantity(5.0, Unit::Kilogram));
    assert_eq!(history_again[0].timestamp, t1);
}

// STKR-19, DDR-RT-02: value history retrieval across timeframe.
#[test]
fn spec_value_history_retrieval() {
    let mut env = Environment::new();
    let start = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap().naive_utc();
    env.set_start_time(start);
    env.set_time(start);

    let t1 = start + ChronoDuration::hours(1);
    let t2 = start + ChronoDuration::hours(3);
    let t3 = start + ChronoDuration::hours(5);

    env.set_value_at("bp", RuntimeValue::Number(120.0), t1);
    env.set_value_at("bp", RuntimeValue::Number(130.0), t2);
    env.set_value_at("bp", RuntimeValue::Number(125.0), t3);

    let history = env.get_history("bp").expect("Expected history for bp");
    assert_eq!(history.len(), 3, "Expected 3 entries, got {}", history.len());

    // Verify correct timestamps and ordering
    assert_eq!(history[0].timestamp, t1);
    assert_eq!(history[0].value, RuntimeValue::Number(120.0));

    assert_eq!(history[1].timestamp, t2);
    assert_eq!(history[1].value, RuntimeValue::Number(130.0));

    assert_eq!(history[2].timestamp, t3);
    assert_eq!(history[2].value, RuntimeValue::Number(125.0));
}

// STKR-06, DDR-RT-08: simulation mode execution completes without real-time delays.
#[test]
fn spec_simulation_mode_execution() {
    let input = r#"
<sim plan> is a plan:
    every 1 hour for 3 hours:
        information "Hourly check".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");
    let mut env = Environment::new();
    env.load_plan(plan);

    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();

    let stop_signal = Arc::new(AtomicBool::new(false));
    let mut executor = Executor::new(stop_signal);
    executor.set_mode(ExecutionMode::Simulation {
        speed_factor: None,
        duration: Some(ChronoDuration::hours(4)),
    });

    executor.on_log = Some(Box::new(move |msg: String, _event_type: EventType, _ts| {
        logs_clone.lock().unwrap().push(msg);
    }));

    let start = std::time::Instant::now();
    executor.execute_plan(&mut env, "sim plan");
    let elapsed = start.elapsed();

    // Simulation should complete quickly, not take real hours
    assert!(
        elapsed < std::time::Duration::from_secs(10),
        "Simulation took too long: {:?} — expected near-instant execution",
        elapsed
    );

    let captured = logs.lock().unwrap();
    assert!(
        captured.iter().any(|msg| msg.contains("Hourly check")),
        "Expected periodic events to fire, got logs: {:?}",
        *captured
    );
}
