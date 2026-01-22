// Spec §4.6, §5: execution model behaviors.

use hippocrates_engine::domain::{RuntimeValue, Unit};
use hippocrates_engine::parser;
use hippocrates_engine::runtime::{Engine, Environment};
use chrono::{Utc, TimeZone, Duration as ChronoDuration};
use std::sync::{Arc, Mutex};
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

// REQ-5-01: runtime executes assignments and actions in order.
#[test]
fn spec_runtime_execution_flow() {
    let input = r#"
<test plan> is a plan:
    during plan:
        show message "Hello World".
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
        show message "Temp is " + <Temp>.
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
        "Yes".

<filtered count> is a number:
    calculation:
        timeframe for analysis is 5 days ago ... now:
            <value> = count of <val> is "Yes".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut env = Environment::new();
    env.load_plan(plan);

    let now = Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - ChronoDuration::days(20));

    let ten_days_ago = now - ChronoDuration::days(10);
    env.set_time(ten_days_ago);
    env.set_value("val", RuntimeValue::String("Yes".to_string()));

    let one_day_ago = now - ChronoDuration::days(1);
    env.set_time(one_day_ago);
    env.set_value("val", RuntimeValue::String("Yes".to_string()));

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
        "Yes".

<count_old> is a number:
    calculation:
        timeframe for analysis is 15 days ago ... 5 days ago:
            <value> = count of <val> is "Yes".

<count_recent> is a number:
    calculation:
        timeframe for analysis is 5 days ago ... now:
            <value> = count of <val> is "Yes".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut env = Environment::new();
    env.load_plan(plan);

    let now = Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - ChronoDuration::days(20));

    let ten_days_ago = now - ChronoDuration::days(10);
    env.set_time(ten_days_ago);
    env.set_value("val", RuntimeValue::String("Yes".to_string()));

    let one_day_ago = now - ChronoDuration::days(1);
    env.set_time(one_day_ago);
    env.set_value("val", RuntimeValue::String("Yes".to_string()));

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
