use hippocrates_engine::runtime::{Environment, Executor, ExecutionMode};
use hippocrates_engine::parser;
use hippocrates_engine::domain::EventType;
use chrono::{Utc, TimeZone};
use std::sync::{Arc, Mutex};
use std::fs;

#[test]
fn test_golden_master() {
    // 1. Load Golden Plan
    let path = "tests/golden.hipp";
    let input = fs::read_to_string(path).expect("Failed to read golden.hipp");
    
    let plan = parser::parse_plan(&input).expect("Failed to parse golden.hipp");
    
    let mut env = Environment::new();
    env.load_plan(plan);
    
    // Fix time for reproducibility
    let start_time = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap().naive_utc();
    env.set_start_time(start_time);
    env.set_time(start_time);

    // 2. Setup Executor
    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();
    
    let mut executor = Executor::with_activites(
        Box::new(|_| {}),
        Box::new(move |msg, kind, _| {
            // We ignore timestamp in verification to avoid flakiness, or correct it?
            // Audit log captures it.
            // Here we just act as stdout
            logs_clone.lock().unwrap().push((kind, msg));
        }),
    );
    
    // 3. Execute
    let plan_name = "Golden Plan";
    executor.execute_plan(&mut env, plan_name);
    
    // 4. Inspect Audit Log
    let audit_json = serde_json::to_string_pretty(&env.audit_log).expect("Failed to serialize audit log");
    
    println!("--- AUDIT LOG START ---");
    println!("{}", audit_json);
    println!("--- AUDIT LOG END ---");
    
    // 5. Assertions (forming the baseline)
    
    // Check key events exist
    let events = &env.audit_log;
    
    // Expect assignments: Test Number, Test String, Test Enum
    assert!(events.iter().any(|e| e.details.contains("Assigned variable: Test Number = 42 testpoint")));
    assert!(events.iter().any(|e| e.details.contains("Assigned variable: Test String = Hello World")));
    assert!(events.iter().any(|e| e.details.contains("Assigned variable: TestEnum = A")));
    
    // Expect messages
    assert!(events.iter().any(|e| e.event_type == EventType::Message && e.details.contains("Number is 42")));
    assert!(events.iter().any(|e| e.event_type == EventType::Message && e.details.contains("High Value"))); // 42 > 40
    assert!(events.iter().any(|e| e.event_type == EventType::Message && e.details.contains("Enum is A")));

    // Loop assertions removed as grammar doesn't support repeat
}
