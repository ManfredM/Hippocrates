// Non-spec integration/regression tests (ignored by default).

use crate::fixture_loader::{load_scenario, ScenarioKind};
use hippocrates_engine::domain::{RuntimeValue, Unit};
use hippocrates_engine::parser;
use hippocrates_engine::runtime::{Engine, Environment, Executor, ExecutionMode};

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_copd_runtime_setup() {
    let input = load_scenario("tests/fixtures/runtime_plans.hipp", "copd_plan", ScenarioKind::Pass);
    let plan = parser::parse_plan(&input).expect("Failed to parse COPD plan");

    let mut engine = Engine::new();
    engine.set_mode(ExecutionMode::Simulation { speed_factor: None, duration: Some(chrono::Duration::days(1)) });

    engine.load_plan(plan);
    engine.set_value("inhaler used", RuntimeValue::Enumeration("Yes".to_string()));

    engine.execute("COPD telehealth");

    if let Some(history) = engine.env.get_history("log") {
        assert!(history.iter().any(|v| v.value == RuntimeValue::String("Plan started".to_string())));
    } else {
        panic!("'log' variable not set by plan");
    }
}

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_99_bottles_execution() {
    let mut env = Environment::new();
    let input = load_scenario("tests/fixtures/runtime_plans.hipp", "bottles_plan", ScenarioKind::Pass);
    let plan = parser::parse_plan(&input).expect("Failed to parse");
    env.load_plan(plan);

    env.set_value("empty bottles", RuntimeValue::Quantity(0.0, Unit::Custom("bottles".to_string())));

    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut executor = Executor::new(stop_signal);
    executor.set_mode(ExecutionMode::Simulation { speed_factor: None, duration: None });
    executor.execute_plan(&mut env, "99 bottles of beer");

    let logs = &env.output_log;
    let has_lyrics = logs.iter().any(|log| log.contains("beer on the wall"));
    assert!(has_lyrics, "Should have lyrics");
}

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_execution_callback() {
    use std::sync::{Arc, Mutex};

    let input = r#"
<callback plan> is a plan:
    during plan:
        information "Line 4".
        information "Line 5".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut env = Environment::new();
    env.load_plan(plan);

    let visited_lines = Arc::new(Mutex::new(Vec::new()));
    let visited_clone = visited_lines.clone();

    let callback = Box::new(move |line: usize| {
        visited_clone.lock().unwrap().push(line);
    });

    let mut executor = Executor::with_activites(
        callback,
        Box::new(|_, _, _| {}),
    );
    executor.execute_plan(&mut env, "callback plan");

    let lines = visited_lines.lock().unwrap();
    assert!(lines.contains(&5), "Should visit line 5, visited: {:?}", lines);
}

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_current_value_in_calculation() {
    let input = r#"
<val> is an enumeration:
    valid values:
        <Yes>; <No>.

<count> is a number:
    calculation:
        timeframe for analysis is 5 days ago ... now:
            <value> = count of <val> is <Yes>.
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut env = Environment::new();
    env.load_plan(plan);
    let now = chrono::Utc::now().naive_utc();
    env.set_start_time(now - chrono::Duration::days(10));

    env.set_value("val", RuntimeValue::Enumeration("Yes".to_string()));

    use hippocrates_engine::ast::{Expression, RelativeDirection, RangeSelector, StatisticalFunc};
    use hippocrates_engine::runtime::Evaluator;
    use hippocrates_engine::runtime::environment::EvaluationContext;

    let start_expr = Expression::RelativeTime(5.0, Unit::Day, RelativeDirection::Ago);
    let end_expr = Expression::RelativeTime(0.0, Unit::Second, RelativeDirection::Ago);

    let ctx = EvaluationContext {
        timeframe: Some(RangeSelector::Range(start_expr, end_expr)),
        period: None,
    };

    env.push_context(ctx);

    let filter_expr = Expression::Variable("Yes".to_string());
    let count_expr = Expression::Statistical(StatisticalFunc::CountOf("val".to_string(), Some(Box::new(filter_expr))));

    let result = Evaluator::evaluate(&env, &count_expr);

    if let RuntimeValue::Number(n) = result {
        assert_eq!(n, 1.0, "Expected count of 1 for 'Yes', got {}", n);
    } else {
        panic!("Expected number result, got {:?}", result);
    }

    env.pop_context();
}

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_derived_calculation() {
    let input = r#"
<val> is an enumeration:
    valid values:
        <Yes>.

<derived count> is a number:
    calculation:
        timeframe for analysis is 5 days ago ... now:
            <value> = count of <val> is <Yes>.
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut env = Environment::new();
    env.load_plan(plan);
    let now = chrono::Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - chrono::Duration::days(10));

    env.set_value("val", RuntimeValue::Enumeration("Yes".to_string()));

    let expr = hippocrates_engine::ast::Expression::Variable("derived count".to_string());
    let result = hippocrates_engine::runtime::Evaluator::evaluate(&env, &expr);

    if let RuntimeValue::Number(n) = result {
        assert_eq!(n, 1.0, "Expected derived count to be calculated as 1, got {}", n);
    } else {
        panic!("Expected number result, got {:?}", result);
    }
}
