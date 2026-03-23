use hippocrates_engine::runtime::{Engine, ExecutionMode, Executor};
use hippocrates_engine::domain::RuntimeValue;
use hippocrates_engine::parser;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;


#[test]
#[ignore = "Non-spec integration/regression"]
fn test_interactive_execution() {
    let input = r#"
<my var> is an enumeration:
    valid values:
        <A>; <B>.
    question:
        ask "Pick one".

<my range> is a number:
    valid values:
        0 kg ... 10 kg.
    question:
        ask "Pick number".
    
<My Plan> is a plan:
    before plan:
        ask <my var>.
        ask <my range>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");
    let mut engine = Engine::new();
    engine.load_plan(plan);
    engine.set_mode(ExecutionMode::Simulation { speed_factor: None, duration: None });

    // Shared state to coordinate test
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();

    // We need to capture the ASK callback to signal the main thread.
    // The current Engine API allows setting callbacks on `Executor` creation or `Engine`.
    // But `Engine` wrapper hides `Executor` a bit.
    // For this test, let's use `Executor` directly like `runtime_tests.rs` or use FFI pattern if possible?
    // Using `Executor` directly is easier for Rust unit test.

    let mut env = hippocrates_engine::runtime::Environment::new();
    // We need to preload the plan definitions into env? 
    // `Engine::load_plan` puts definitions into `env.definitions`.
    // We can steal the env from the engine? `Engine` has `env` field but it might be private.
    // `Engine` struct in `runtime/mod.rs` is `pub struct Engine { env: Environment, ... }`.
    // If fields are private, we might need to use `hippocrates_engine_new` flow or just parse and put into env manually.
    
    // Let's rely on `Executor::with_activities`.
    
    let ask_signal = pair.clone();
    let ask_callback = Box::new(move |req: hippocrates_engine::domain::AskRequest| {
        println!("Callback: Asking for {} with {:?}", req.variable_name, req);
        let (lock, cvar) = &*ask_signal;
        let mut asked = lock.lock().unwrap();
        *asked = true;
        cvar.notify_one();
    });

    let log_check = Arc::new(Mutex::new(Vec::new()));
    let log_capture = log_check.clone();
    let log_callback = Box::new(move |msg: String, _type: hippocrates_engine::domain::EventType, _ts: chrono::NaiveDateTime| {
        println!("Callback: Log - {}", msg);
        log_capture.lock().unwrap().push(msg);
    });

    let mut executor = Executor::with_activites(
        Box::new(|_| {}), // Dummy line callback
        log_callback
    );
    executor.on_ask = Some(ask_callback);
    executor.set_mode(ExecutionMode::Simulation { speed_factor: None, duration: None });
    
    // Create channel for interaction
    let (tx, rx) = std::sync::mpsc::channel();
    executor.input_receiver = Some(rx);

    // Spawn execution in a separate thread
    // We need to move executor and env to the thread.
    // Env is not thread safe (it's just a struct). But the thread will own it.
    // We also need definitions from engine?
    
    // Copy definitions from engine to env
    env.definitions = engine.env.definitions.clone();

    let exec_handle = thread::spawn(move || {
        executor.execute_plan(&mut env, "My Plan");
        env // Return env to check final state if needed
    });

    // Main thread waits for ASK signal
    // Question 1: <my var> (Enum)
    {
        let (lock, cvar) = &*pair2;
        let mut asked = lock.lock().unwrap();
        while !*asked {
            asked = cvar.wait(asked).unwrap();
        }
        *asked = false; // Reset for next
        println!("Test: Received first question.");
    }
    
    // Send answer for var
    let answer1 = hippocrates_engine::domain::InputMessage {
        variable: "my var".to_string(),
        value: RuntimeValue::Enumeration("A".to_string()),
        timestamp: chrono::Utc::now().naive_utc(),
    };
    tx.send(answer1).expect("Failed to send answer 1");

    // Question 2: <my range> (Range)
    {
        let (lock, cvar) = &*pair2;
        let mut asked = lock.lock().unwrap();
        while !*asked {
            asked = cvar.wait(asked).unwrap();
        }
        *asked = false;
        println!("Test: Received second question.");
    }

    let answer2 = hippocrates_engine::domain::InputMessage {
        variable: "my range".to_string(),
        value: RuntimeValue::Quantity(5.0, hippocrates_engine::domain::Unit::Kilogram),
        timestamp: chrono::Utc::now().naive_utc(),
    };
    tx.send(answer2).expect("Failed to send answer 2");

    // Wait for execution to finish
    let _final_env = exec_handle.join().expect("Execution thread panicked");
    
    println!("Test: Execution finished.");
}
