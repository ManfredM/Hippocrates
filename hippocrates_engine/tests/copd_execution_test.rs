use hippocrates_engine::parser;
use hippocrates_engine::runtime::{Engine, ExecutionMode};
use hippocrates_engine::domain::RuntimeValue;

mod fixture_loader;

#[test]
fn test_reproduce_copd_execution() {
    use fixture_loader::{load_scenario, ScenarioKind};

    let input = load_scenario("tests/fixtures/runtime_plans.hipp", "copd_plan", ScenarioKind::Pass);
    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    let mut engine = Engine::new();
    engine.load_plan(plan);
    
    // Simulate App behavior:
    // 1. Set mode
    engine.set_mode(ExecutionMode::Simulation { 
        speed_factor: None, 
        duration: Some(chrono::Duration::days(2)) 
    });

    // 2. Initial values
    engine.set_value("inhaler used", RuntimeValue::Enumeration("No".to_string()));

    // 3. Execute
    println!("Starting execution...");
    engine.execute("COPD telehealth");
    println!("Execution finished.");

    // 4. Verify some output
    let logs = &engine.env.output_log;
    assert!(!logs.is_empty(), "Logs should not be empty");
    
    // Check if we hit "assess" blocks or specific messages
    // "It's the best time of the day to take your daily shot now"
    let has_message = logs.iter().any(|s| s.contains("It's the best time of the day"));
    assert!(has_message, "Should have scheduled inhalation period and shown message. Logs: {:?}", logs);
}
