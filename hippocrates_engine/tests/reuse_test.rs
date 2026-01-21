#[cfg(test)]
mod tests {
    use hippocrates_engine::Session;
    use hippocrates_engine::domain::{RuntimeValue, AskRequest, EventType, Unit};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_validity_reuse_timeframe() {
        let asked_questions = Arc::new(Mutex::new(Vec::new()));
        let logs = Arc::new(Mutex::new(Vec::new()));

        let aq = asked_questions.clone();
        let l = logs.clone();
        
        let session = Arc::new(Session::new(
            Box::new(move |req: AskRequest| {
                println!("Asked: {}", req.variable_name);
                aq.lock().unwrap().push(req.variable_name);
            }),
            Box::new(move |msg: String, _kind: EventType, _time: chrono::NaiveDateTime| {
                println!("Log: {}", msg);
                l.lock().unwrap().push(msg);
            }),
        ));

        let script = r#"
<Temp> is a number:
    unit is °C
    valid values:
        35.0 °C ... 42.0 °C
    reuse:
        reuse period of value is 5 seconds.

<CheckTemp> is a plan:
    during plan:
        ask <Temp>.
        show message "Temp is " + <Temp>.
"#;

        // 1. Run Script First Time - Should Ask
        session.run_script(script.to_string(), "CheckTemp".to_string());
        
        // Wait for ask
        thread::sleep(Duration::from_millis(500));
        
        // Provide Answer (creates value at T=0)
        session.provide_answer("<Temp>", RuntimeValue::Quantity(37.0, Unit::Celsius));
        
        // Wait for completion
        thread::sleep(Duration::from_millis(500));
        
        {
            let inputs = asked_questions.lock().unwrap();
            assert_eq!(inputs.len(), 1, "First run should ask");
        }

        // 2. Run Script Second Time (Immediately) - Should NOT Ask (Valid for 5s)
        println!("Running second time (immediate)...");
        session.run_script(script.to_string(), "CheckTemp".to_string());
        
        // Wait for completion
        thread::sleep(Duration::from_millis(500)); // Should skip ask and finish instantly

        {
            let inputs = asked_questions.lock().unwrap();
            assert_eq!(inputs.len(), 1, "Second run should NOT ask (reuse)");
        }
        
        // 3. Wait for Expiration (> 5s)
        println!("Waiting for expiration (6s)...");
        thread::sleep(Duration::from_secs(6));

        // 4. Run Script Third Time - Should Ask Again
        println!("Running third time...");
        session.run_script(script.to_string(), "CheckTemp".to_string());
         // Wait for ask
        thread::sleep(Duration::from_millis(500));
        
        {
             let inputs = asked_questions.lock().unwrap();
             assert_eq!(inputs.len(), 2, "Third run SHOULD ask (expired)");
        }
        
        // Provide answer to finish cleanly
        session.provide_answer("<Temp>", RuntimeValue::Quantity(38.0, Unit::Celsius));
        thread::sleep(Duration::from_millis(500));
    }
}
