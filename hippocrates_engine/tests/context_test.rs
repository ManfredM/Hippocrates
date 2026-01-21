use hippocrates_engine::runtime::{Environment, Executor};
use hippocrates_engine::domain::{RuntimeValue}; // ValueInstance removed as it's internal
use hippocrates_engine::parser::parse_plan;
use chrono::{Utc, Duration};

#[test]
fn test_analysis_context_timeframe() {
    let code = r#"
<pain level> is a number.

<TestPlan> is a plan:
    during plan:
        context for analysis:
            timeframe: 2 days ago ... now
            assess trend of <pain level>:
                "increase":
                    <log> = "Increasing".
                "stable":
                    <log> = "Stable".
"#;

    let mut env = Environment::new();
    let mut executor = Executor::new(std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)));

    // Load Plan
    let plan = parse_plan(code).expect("Failed to parse plan");
    env.load_plan(plan);

    // Populate History
    let now = Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - Duration::days(100));

    // Use set_value_at directly
    // Use set_value_at directly - Internal keys usually stripped of <>
    env.set_value_at("pain level", RuntimeValue::Number(10.0), now - Duration::days(5));
    env.set_value_at("pain level", RuntimeValue::Number(8.0), now - Duration::days(3));
    env.set_value_at("pain level", RuntimeValue::Number(2.0), now - Duration::days(1));
    env.set_value_at("pain level", RuntimeValue::Number(4.0), now);

    // Capture logs isn't enough because we assign to <log> variable which is implicit.
    // Let's define it explicitly in code above if needed, but assignments create vars if missing?
    // In strict mode no. But Environment.set_value behaves loosely?
    // Environment::set_value_at checks definitions? No, just inserts into values HashMap.
    // So implicit declaration works for storage.

    // Execute
    executor.execute_plan(&mut env, "TestPlan");
}

#[test]
fn test_analysis_context_timeframe_full() {
    let code = r#"
<pain level> is a number.
<result> is an enumeration.

<TestPlan> is a plan:
    during plan:
        context for analysis:
            timeframe: 2 days ago ... now
            assess trend of <pain level>:
                "increase":
                    <result> = "Increased".
                "decrease":
                    <result> = "Decreased".
"#;

    let mut env = Environment::new();
    let mut executor = Executor::new(std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)));
    
    let plan = parse_plan(code).expect("Failed to parse plan");
    env.load_plan(plan);

    let now = Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - Duration::days(100));

    env.set_value_at("pain level", RuntimeValue::Number(10.0), now - Duration::days(5));
    env.set_value_at("pain level", RuntimeValue::Number(8.0), now - Duration::days(3));
    env.set_value_at("pain level", RuntimeValue::Number(2.0), now - Duration::days(1));
    env.set_value_at("pain level", RuntimeValue::Number(4.0), now);

    // executor.on_log = Some(Box::new(|msg, _, _| {
    //    println!("LOG: {}", msg);
    // }));

    executor.execute_plan(&mut env, "TestPlan");

    let res = env.get_value("result").expect("Result should be set");
    assert_eq!(res, &RuntimeValue::String("Increased".to_string()));
}
