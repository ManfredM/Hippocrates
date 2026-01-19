use hippocrates_engine::runtime::{Executor, Environment, ExecutionMode};
use hippocrates_engine::parser;
use chrono::{Utc, TimeZone};
use std::sync::{Arc, Mutex};

#[test]
fn test_period_simulation_progression() {
    let input = r#"
<best period> is a period:
    timeframe:
        between Monday ... Sunday; 08:00 ... 08:10

<my plan> is a plan:
    <my event> with begin of best period:
        show message "Event Triggered".
"#;

    // 1. Setup Environment
    let mut env = Environment::new();
    let plan = parser::parse_plan(input).expect("Failed to parse");
    env.load_plan(plan);

    // Mock start time to Sunday 07:59 (should trigger soon)
    let start_time = Utc.with_ymd_and_hms(2026, 1, 18, 7, 59, 0).unwrap();
    env.set_time(start_time);

    // 2. Setup Executor with Simulation Mode (instant jump)
    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();

    let mut executor = Executor::with_activites(
        Box::new(|_| {}),
        Box::new(move |msg, _, _| {
            logs_clone.lock().unwrap().push(msg);
        }),
    );
    // Use Simulation mode with no speed factor (instant jumps)
    executor.set_mode(ExecutionMode::Simulation { 
        speed_factor: None,
        duration: Some(chrono::Duration::days(1)) 
    });

    // 3. Execute
    // We expect the executor to find the next occurrence (Sunday 08:00) and jump to it.
    executor.execute_plan(&mut env, "my plan");

    // 4. Verify
    let captured_logs = logs.lock().unwrap();
    println!("Logs: {:?}", *captured_logs);
    
    assert!(captured_logs.iter().any(|msg| msg.contains("Event Triggered")), "Should have triggered event");
}
