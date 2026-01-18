use hippocrates_engine::domain::RuntimeValue;
use hippocrates_engine::parser;
use hippocrates_engine::runtime::{Engine, Environment, Executor};

#[test]
fn test_runtime_execution_flow() {
    let input = r#"
"test plan" is a plan:
    during plan:
        show message "Hello World".
        x = 10.
        send information "Val is " x.
"#;
    
    let plan = parser::parse_plan(input).expect("Failed to parse plan");
    let mut engine = Engine::new();
    engine.load_plan(plan);
    engine.execute("test plan");
    
    // Check logs
    assert!(engine.env.output_log.iter().any(|s| s.contains("Hello World")));
    // "Val is " with values: [Number(10.0)]
    assert!(engine.env.output_log.iter().any(|s| s.contains("Val is ") && s.contains("Number(10.0)")));
    
    // Check variable
    if let Some(val) = engine.env.get_value("x") {
        assert_eq!(val, &RuntimeValue::Number(10.0));
    } else {
        panic!("Variable x not found");
    }
}

#[test]
fn test_copd_runtime_setup() {
    // Load the real file to ensure runtime can handle the structure (even if logic is mocked)
    let input = std::fs::read_to_string("plans/treating_copd.hipp").expect("Could not read file");
    let plan = parser::parse_plan(&input).expect("Failed to parse COPD plan");
    
    let mut engine = Engine::new();
    engine.load_plan(plan);
    
    // Set some initial state mimicking a patient context
    engine.set_value("inhaler used", RuntimeValue::Enumeration("Yes".to_string()));
    
    // execute part? specific plan block execution isn't fully exposed by name in my simple prototype
    // "COPD telehealth" is the plan name.
    
    engine.execute("COPD telehealth");
    
    // Just verify it ran without crashing and maybe hit some logs
    // Since "during plan" has "listen for inhaler used", it should log that action.
    // Check that "log" variable was set
    if let Some(val) = engine.env.get_value("log") {
        assert_eq!(val, &RuntimeValue::String("Plan started".to_string()));
    } else {
        panic!("'log' variable not set by plan");
    }
}

#[test]
fn test_99_bottles_execution() {
    let mut env = Environment::new();
    let input = std::fs::read_to_string("plans/99_bottles.hipp").expect("Failed to read 99_bottles.hipp");
    let plan = parser::parse_plan(&input).expect("Failed to parse");
    env.load_plan(plan);
    
    // Initialize required variables that are not auto-set
    // 'empty bottles' should start at 0
    env.set_value("empty bottles", RuntimeValue::Number(0.0));
    
    // Execute plan
    let mut executor = Executor::new();
    executor.execute_plan(&mut env, "99 bottles of beer");
    
    // Check logs for lyrics
    // We expect log entries like: "Action: Show Message '99 bottles of beer on the wall...'"
    let logs = &env.output_log;
    // println!("DEBUG: Logs:\n{:?}", logs);
    
    let has_lyrics = logs.iter().any(|log| log.contains("99 bottles of beer on the wall"));
    let has_take_down = logs.iter().any(|log| log.contains("Take one down"));
    // let has_98 = logs.iter().any(|log| log.contains("98 bottles of beer on the wall")); 
    
    assert!(has_lyrics, "Should sing about 99 bottles");
    assert!(has_take_down, "Should take one down");
}

#[test]
fn test_trend_analysis() {
    use hippocrates_engine::ast::{Expression, StatisticalFunc, RangeSelector, RelativeDirection};
    use hippocrates_engine::domain::{Unit, RuntimeValue};
    use hippocrates_engine::runtime::{Environment, Evaluator};
    use hippocrates_engine::runtime::environment::EvaluationContext;
    use chrono::{Utc, Duration};

    let mut env = Environment::new();
    let now = Utc::now();
    env.set_time(now);

    // Inject history: Increasing trend
    // T-5d: 10
    // T-4d: 20
    // T-3d: 30
    env.set_value_at("systolic", RuntimeValue::Number(10.0), now - Duration::days(5));
    env.set_value_at("systolic", RuntimeValue::Number(20.0), now - Duration::days(4));
    env.set_value_at("systolic", RuntimeValue::Number(30.0), now - Duration::days(3));

    // Define Context: 6 days ago ... now
    // Construct expressions manually
    let start_expr = Expression::RelativeTime(6.0, Unit::Day, RelativeDirection::Ago);
    let end_expr = Expression::RelativeTime(0.0, Unit::Second, RelativeDirection::Ago); // effectively now
    
    let ctx = EvaluationContext {
        timeframe: Some(RangeSelector::Range(start_expr, end_expr)),
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
"callback plan" is a plan:
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
        Box::new(|_, _| {}) // No-op log callback
    );
    executor.execute_plan(&mut env, "callback plan");
    
    let lines = visited_lines.lock().unwrap();
    // Verify that we visited lines. 
    // "callback plan" is line 2 (offset)
    // "during plan" is line 3
    // "show message" is line 4
    // "show message" is line 5
    
    assert!(lines.contains(&4), "Should visit line 4, visited: {:?}", lines);
    assert!(lines.contains(&5), "Should visit line 5, visited: {:?}", lines);
}
