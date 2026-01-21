mod fixture_loader;

use hippocrates_engine::domain::{RuntimeValue, Unit};
use hippocrates_engine::parser;
use hippocrates_engine::runtime::{Engine, Environment, Executor, ExecutionMode};

#[test]
fn test_runtime_execution_flow() {
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

    // Check logs
    // Check logs
    let logs = &engine.env.output_log;
    println!("Logs: {:?}", logs);
    assert!(logs.iter().any(|s| s.contains("Hello World")));
    // "Val is " with values: [Quantity(10.0, Kilogram)]
    assert!(
        logs.iter()
            .any(|s| s.contains("Val is ") && s.contains("Quantity(10.0, Kilogram)")),
        "Logs missing Val is Quantity(10.0, Kilogram): {:?}", logs
    );

    // Check variable
    if let Some(val) = engine.env.get_value("x") {
        assert_eq!(val, &RuntimeValue::Quantity(10.0, Unit::Kilogram));
    } else {
        panic!("Variable x not found");
    }
}

#[test]
fn test_copd_runtime_setup() {
    use fixture_loader::{load_scenario, ScenarioKind};

    // Load the real file to ensure runtime can handle the structure (even if logic is mocked)
    let input = load_scenario("tests/fixtures/runtime_plans.hipp", "copd_plan", ScenarioKind::Pass);
    let plan = parser::parse_plan(&input).expect("Failed to parse COPD plan");

    let mut engine = Engine::new();
    engine.set_mode(ExecutionMode::Simulation { speed_factor: None, duration: Some(chrono::Duration::days(1)) });
    // engine.executor.simulation_duration = ...; // removed

    engine.load_plan(plan);

    // Set some initial state mimicking a patient context
    engine.set_value("inhaler used", RuntimeValue::Enumeration("Yes".to_string()));

    // execute part? specific plan block execution isn't fully exposed by name in my simple prototype
    // "COPD telehealth" is the plan name.

    engine.execute("COPD telehealth");

    // Just verify it ran without crashing and maybe hit some logs
    // Since "during plan" has "listen for inhaler used", it should log that action.
    // Check that "log" variable was set
    if let Some(history) = engine.env.get_history("log") {
        assert!(history.iter().any(|v| v.value == RuntimeValue::String("Plan started".to_string())));
    } else {
        panic!("'log' variable not set by plan");
    }
}

#[test]
fn test_99_bottles_execution() {
    use fixture_loader::{load_scenario, ScenarioKind};

    let mut env = Environment::new();
    let input = load_scenario("tests/fixtures/runtime_plans.hipp", "bottles_plan", ScenarioKind::Pass);
    let plan = parser::parse_plan(&input).expect("Failed to parse");
    env.load_plan(plan);

    // Initialize required variables that are not auto-set
    // 'empty bottles' should start at 0
    // 'empty bottles' should start at 0 <bottles>
    env.set_value("empty bottles", RuntimeValue::Quantity(0.0, Unit::Custom("bottles".to_string())));

    // Execute plan
    // Execute plan
    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut executor = Executor::new(stop_signal);
    executor.set_mode(ExecutionMode::Simulation { speed_factor: None, duration: None });
    executor.execute_plan(&mut env, "99 bottles of beer");

    // Check logs for lyrics
    // We expect log entries like: "Action: Show Message '99 bottles of beer on the wall...'"
    let logs = &env.output_log;
    // println!("DEBUG: Logs:\n{:?}", logs);

    let has_lyrics = logs.iter().any(|log| log.contains("beer on the wall"));
    assert!(has_lyrics, "Should have lyrics");
    // let has_take_down = logs.iter().any(|log| log.contains("Take one down"));
    // assert!(has_take_down, "Should take one down");
}

#[test]
fn test_trend_analysis() {
    use chrono::{Duration, Utc};
    use hippocrates_engine::ast::{Expression, RangeSelector, RelativeDirection, StatisticalFunc};
    use hippocrates_engine::domain::{RuntimeValue, Unit};
    use hippocrates_engine::runtime::environment::EvaluationContext;
    use hippocrates_engine::runtime::{Environment, Evaluator};

    let mut env = Environment::new();
    let now = Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - Duration::days(20));

    // Inject history: Increasing trend
    // T-5d: 10
    // T-4d: 20
    // T-3d: 30
    env.set_value_at(
        "systolic",
        RuntimeValue::Number(10.0),
        now - Duration::days(5),
    );
    env.set_value_at(
        "systolic",
        RuntimeValue::Number(20.0),
        now - Duration::days(4),
    );
    env.set_value_at(
        "systolic",
        RuntimeValue::Number(30.0),
        now - Duration::days(3),
    );

    // Define Context: 6 days ago ... now
    // Construct expressions manually
    let start_expr = Expression::RelativeTime(6.0, Unit::Day, RelativeDirection::Ago);
    let end_expr = Expression::RelativeTime(0.0, Unit::Second, RelativeDirection::Ago); // effectively now

    let ctx = EvaluationContext {
        timeframe: Some(RangeSelector::Range(start_expr, end_expr)),
        period: None,
    };

    env.push_context(ctx);

    // Evaluate Trend
    let trend_expr = Expression::Statistical(StatisticalFunc::TrendOf("systolic".to_string()));
    let result = Evaluator::evaluate(&env, &trend_expr);

    if let RuntimeValue::String(s) = result {
        assert_eq!(s, "increase");
    } else {
        panic!("Expected string result for trend, got {:?}", result);
    }

    env.pop_context();
}

#[test]
fn test_execution_callback() {
    use std::sync::{Arc, Mutex};

    let input = r#"
<callback plan> is a plan:
    during plan:
        show message "Line 4".
        show message "Line 5".
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
        Box::new(|_, _, _| {}), // No-op log callback
    );
    executor.execute_plan(&mut env, "callback plan");

    let lines = visited_lines.lock().unwrap();
    // Verify that we visited lines.
    // "callback plan" is line 2 (offset)
    // "during plan" is line 3
    // "show message" is line 4
    // "show message" is line 5

    assert!(
        lines.contains(&5),
        "Should visit line 5, visited: {:?}",
        lines
    );
}

#[test]
fn test_current_value_in_calculation() {
    // Reproduce issue: Type mismatch in count filter?
    // User gets Count = 0 when Answer is "Yes" (String). Variable is Enumeration. logic is `count of <val> is "Yes"`.
    let input = r#"
<val> is an enumeration:
    valid values:
        "Yes"; "No"

<count> is a number:
    calculation:
        timeframe for analysis is 5 days ago ... now:
            <value> = count of <val> is "Yes".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut env = Environment::new();
    env.load_plan(plan);
    let now = chrono::Utc::now().naive_utc();
    env.set_start_time(now - chrono::Duration::days(10));
    
    // Default initialization likely Enum("")
    
    // Simulate User Answer as String("Yes") (per screenshot)
    env.set_value("val", RuntimeValue::String("Yes".to_string()));
    
    // Evaluate Count with Filter
    use hippocrates_engine::ast::{Expression, RelativeDirection, RangeSelector, StatisticalFunc};
    use hippocrates_engine::domain::Unit;
    use hippocrates_engine::runtime::{Evaluator};
    use hippocrates_engine::runtime::environment::EvaluationContext;
    
    let start_expr = Expression::RelativeTime(5.0, Unit::Day, RelativeDirection::Ago);
    let end_expr = Expression::RelativeTime(0.0, Unit::Second, RelativeDirection::Ago);
    
    let ctx = EvaluationContext {
        timeframe: Some(RangeSelector::Range(start_expr, end_expr)),
        period: None,
    };
    
    env.push_context(ctx);
    
    // Expression: CountOf("val", Some(Equals(Literal("Yes"))))?
    // Parser converts `count of <val> is "Yes"` to `CountOf("val", Some(Expression))`?
    // Actually, `grammar.pest`: `statistical_func = { "count of" ~ identifier ~ ("is" ~ expression)? }`.
    // So it parses as `CountOf("val", Some(expr))`.
    // The expression is `Literal::String("Yes")`.
    
    // We construct it manually:
    let filter_expr = Expression::Literal(hippocrates_engine::ast::Literal::String("Yes".to_string()));
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
fn test_derived_calculation() {
    // Reproduce issue: Variable with Calculation rule returns Void/0 if not explicitly computed.
    let input = r#"
<val> is an enumeration:
    valid values:
        "Yes"

<derived count> is a number:
    calculation:
        timeframe for analysis is 5 days ago ... now:
            <value> = count of <val> is "Yes".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut env = Environment::new();
    env.load_plan(plan);
    let now = chrono::Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - chrono::Duration::days(10));
    
    // Set dependency
    env.set_value("val", RuntimeValue::String("Yes".to_string()));
    
    // Evaluate Derived Variable
    use hippocrates_engine::ast::{Expression};
    use hippocrates_engine::runtime::{Evaluator};
    
    // Accessing "derived count" should trigger calculation
    let expr = Expression::Variable("derived count".to_string());
    let result = Evaluator::evaluate(&env, &expr);
    
    if let RuntimeValue::Number(n) = result {
        assert_eq!(n, 1.0, "Expected derived count to be calculated as 1, got {}", n);
    } else {
        panic!("Expected number result, got {:?}", result);
    }
}

#[test]
fn test_timeframe_filtering() {
    // Verify "5 days ago ... now" strictly excludes older data.
    let input = r#"
<val> is an enumeration:
    valid values:
        "Yes"

<filtered count> is a number:
    calculation:
        timeframe for analysis is 5 days ago ... now:
            <value> = count of <val> is "Yes".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let mut env = Environment::new();
    env.load_plan(plan);

    use chrono::{Duration, Utc};
    use hippocrates_engine::domain::RuntimeValue;
    use hippocrates_engine::ast::Expression;
    use hippocrates_engine::runtime::Evaluator;

    let now = Utc::now().naive_utc();
    env.set_time(now); // Anchor "now"
    env.set_start_time(now - Duration::days(20));

    // 1. Data point 10 days ago (Should be EXCLUDED)
    let ten_days_ago = now - Duration::days(10);
    env.set_time(ten_days_ago);
    env.set_value("val", RuntimeValue::String("Yes".to_string()));

    // 2. Data point 1 day ago (Should be INCLUDED)
    let one_day_ago = now - Duration::days(1);
    env.set_time(one_day_ago);
    env.set_value("val", RuntimeValue::String("Yes".to_string()));

    // 3. Evaluate at "now"
    env.set_time(now);
    
    // Evaluate Derived Variable
    let expr = Expression::Variable("filtered count".to_string());
    let result = Evaluator::evaluate(&env, &expr);
    
    if let RuntimeValue::Number(n) = result {
        assert_eq!(n, 1.0, "Expected count of 1 (excluding 10 days ago), got {}", n);
    } else {
        panic!("Expected number result, got {:?}", result);
    }
}

#[test]
fn test_timeframe_variants() {
    // Verify "> 5 days ago" and other variants
    let input = r#"
<val> is an enumeration:
    valid values:
        "Yes"

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

    use chrono::{Duration, Utc};
    use hippocrates_engine::domain::RuntimeValue;
    use hippocrates_engine::ast::Expression;
    use hippocrates_engine::runtime::Evaluator;

    let now = Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - Duration::days(20));

    // 1. Data point 10 days ago (Old)
    let ten_days_ago = now - Duration::days(10);
    env.set_time(ten_days_ago);
    env.set_value("val", RuntimeValue::String("Yes".to_string()));

    // 2. Data point 1 day ago (Recent)
    let one_day_ago = now - Duration::days(1);
    env.set_time(one_day_ago);
    env.set_value("val", RuntimeValue::String("Yes".to_string()));

    // Evaluate at "now"
    env.set_time(now);
    
    // Check "count_old" (Should be 1: the 10-day-old event)
    let expr_old = Expression::Variable("count_old".to_string());
    let res_old = Evaluator::evaluate(&env, &expr_old);
    if let RuntimeValue::Number(n) = res_old {
        assert_eq!(n, 1.0, "Expected count_old to be 1 (10 days ago), got {}", n);
    } else {
        panic!("count_old failed: {:?}", res_old);
    }

    // Check "count_recent" (Should be 1: the 1-day-old event)
    let expr_recent = Expression::Variable("count_recent".to_string());
    let res_recent = Evaluator::evaluate(&env, &expr_recent);
    if let RuntimeValue::Number(n) = res_recent {
        assert_eq!(n, 1.0, "Expected count_recent to be 1 (1 day ago), got {}", n);
    } else {
        panic!("count_recent failed: {:?}", res_recent);
    }
}
