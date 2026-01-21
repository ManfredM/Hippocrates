use hippocrates_engine::ast::{Definition, PlanBlock};
use hippocrates_engine::runtime::{Environment, Executor};
use hippocrates_engine::domain::{RuntimeValue, Unit, AskRequest, InputMessage};
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::thread;
use std::time::Duration;

#[test]
fn test_implicit_asking_scenario() {
    let source = std::fs::read_to_string("plans/implicit_ask_scenario.hipp")
        .expect("Failed to read plan file");

    let mut env = Environment::new();
    let plan = hippocrates_engine::parser::parse_plan(&source).expect("Failed to parse plan");
    env.load_plan(plan);

    // Setup channels for interaction
    let (tx_input, rx_input) = std::sync::mpsc::channel();
    
    // Capture logs and questions
    let logs = Arc::new(Mutex::new(Vec::new()));
    let questions = Arc::new(Mutex::new(Vec::new()));
    
    let logs_clone = logs.clone();
    let questions_clone = questions.clone();
    let tx_input_clone = tx_input.clone();
    let now = chrono::Utc::now().naive_utc();
    env.set_start_time(now);
    env.set_time(now);

    let mut executor = Executor::with_activites(
        Box::new(|_| {}),
        Box::new(move |msg, _, _| {
            logs_clone.lock().unwrap().push(msg);
        })
    );
    
    executor.set_input_receiver(rx_input);
    
    executor.set_ask_callback(Box::new(move |req: AskRequest| {
        println!("DEBUG: Callback asked: {}", req.variable_name);
        questions_clone.lock().unwrap().push(req.variable_name.clone());
        
        // Auto-reply logic for test
        if req.variable_name == "user age" {
            let answer = InputMessage {
                variable: "user age".to_string(),
                value: RuntimeValue::Quantity(25.0, Unit::Year),
                timestamp: chrono::Utc::now().naive_utc(),
            };
            // Send answer in a separate thread to simulate user delay/async nature and avoid blocking callback if logic was different
            // But here executor blocks waiting for RX, so we must allow RX to receive.
            // Executor::execute_action calls check triggers? No, here it calls rx.recv() inside the loop.
            // So we need to send to TX *before* executor blocks or from another thread.
            // Since we are IN the callback which is called BEFORE the blocking wait, we can send now.
            // Actually, Executor is currently blocking on rx.recv() *after* calling callback.
            // So sending here fills the buffer.
            tx_input_clone.send(answer).unwrap();
        }
    }));

    // Execute the plan <Age assessment>
    executor.execute_plan(&mut env, "Age assessment");

    // Verification
    let asked = questions.lock().unwrap();
    let captured_logs = logs.lock().unwrap();
    
    println!("Asked: {:?}", *asked);
    println!("Logs: {:?}", *captured_logs);

    assert!(asked.contains(&"user age".to_string()), "Should have asked for 'user age'");
    assert!(
        captured_logs.iter().any(|log| log.contains("Best age")),
        "Should have shown 'Best age' message"
    );
}
