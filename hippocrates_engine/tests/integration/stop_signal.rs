// DES-43: Stop signal terminates execution gracefully.

use hippocrates_engine::runtime::{Executor, Environment, ExecutionMode};
use hippocrates_engine::parser;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_stop_signal_terminates_execution() {
    let input = r#"
<morning> is a period:
    timeframe:
        between Monday ... Sunday; 00:00 ... 23:59.

<long plan> is a plan:
    every 1 minute for 1 day:
        information "Tick".
"#;

    let mut env = Environment::new();
    let plan = parser::parse_plan(input).expect("Failed to parse");
    env.load_plan(plan);
    env.set_time(Utc::now().naive_utc());

    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();

    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_clone = stop_signal.clone();

    let mut executor = Executor::new(stop_signal);
    executor.set_mode(ExecutionMode::Simulation {
        speed_factor: None,
        duration: Some(chrono::Duration::days(1)),
    });

    executor.on_step = Some(Box::new(|_| {}));
    executor.on_log = Some(Box::new(move |msg, _, _| {
        let mut l = logs_clone.lock().unwrap();
        l.push(msg);
        // After collecting a few events, signal stop
        if l.len() >= 5 {
            stop_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }));

    executor.execute_plan(&mut env, "long plan");

    let captured = logs.lock().unwrap();
    // With 1-day duration at 1-minute intervals, full run would produce ~1440 events.
    // We stopped after 5, so we should have far fewer than full count.
    assert!(
        captured.len() < 100,
        "Stop signal should have terminated execution early, got {} events",
        captured.len()
    );
    assert!(
        captured.len() >= 5,
        "Should have collected at least 5 events before stopping, got {}",
        captured.len()
    );
}
