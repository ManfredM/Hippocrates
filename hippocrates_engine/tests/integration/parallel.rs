#[cfg(test)]
mod tests {
    use hippocrates_engine::runtime::session::Session;
    use hippocrates_engine::domain::{RuntimeValue, AskRequest, EventType, Unit};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::thread;

    #[test]
    #[ignore = "Non-spec integration/regression"]
    fn test_parallel_execution_consolidated_input() {
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

        let script1 = r#"
<Age> is a number:
    valid values:
        0 years ... 130 years.
<Script1> is a plan:
    before plan:
        ask <Age>.
        information "Script1: Age is " + <Age>.
"#;

        let script2 = r#"
<Age> is a number:
    valid values:
        0 years ... 130 years.
<Script2> is a plan:
    before plan:
        ask <Age>.
        information "Script2: Age is " + <Age>.
"#;

        session.run_script(script1.to_string(), "Script1".to_string());
        
        // Wait a bit to ensure Script1 asks
        thread::sleep(Duration::from_millis(100));
        
        session.run_script(script2.to_string(), "Script2".to_string());
        
        // Wait a bit to ensure Script2 starts and checks input
        thread::sleep(Duration::from_millis(100));

        // provide answer
        println!("Providing answer...");
        session.provide_answer("<Age>", RuntimeValue::Quantity(30.0, Unit::Year));

        // Wait for completion
        thread::sleep(Duration::from_millis(500));

        let inputs = asked_questions.lock().unwrap();
        let messages = logs.lock().unwrap();

        // Verify "Age" was asked exactly once
        assert_eq!(inputs.len(), 1, "Should have asked for Age only once");
        assert_eq!(inputs[0], "<Age>");

        // Verify both scripts ran and printed output
        let s1_ok = messages.iter().any(|m: &String| m.contains("Script1: Age is 30"));
        let s2_ok = messages.iter().any(|m: &String| m.contains("Script2: Age is 30"));
        
        assert!(s1_ok, "Script1 did not finish or output incorrect. Messages: {:?}", messages);
        assert!(s2_ok, "Script2 did not finish or output incorrect. Messages: {:?}", messages);
    }
}
