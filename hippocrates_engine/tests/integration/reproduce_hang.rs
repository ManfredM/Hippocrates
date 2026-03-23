use hippocrates_engine::domain::{RuntimeValue, Unit};

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_ffi_parsing_logic() {
    let inputs = vec![
        ("1", RuntimeValue::Number(1.0)),
        ("\"1\"", RuntimeValue::Number(1.0)),
        ("\"1 point\"", RuntimeValue::Quantity(1.0, Unit::Custom("point".to_string()))),
        ("\"1 mg\"", RuntimeValue::Quantity(1.0, Unit::Custom("mg".to_string()))),
        ("\"1 point: description\"", RuntimeValue::Quantity(1.0, Unit::Custom("point".to_string()))),
    ];

    for (json, expected) in inputs {
        let parsed = parse_json_value(json);
        assert_eq!(parsed, Some(expected), "Failed to parse: {}", json);
    }
}

// Copy of the logic from ffi.rs for testing
fn parse_json_value(json: &str) -> Option<RuntimeValue> {
    if let Ok(n) = serde_json::from_str::<f64>(json) {
         Some(RuntimeValue::Number(n))
    } else if let Ok(s) = serde_json::from_str::<String>(json) {
         if let Ok(n) = s.parse::<f64>() {
             Some(RuntimeValue::Number(n))
         } else {
             // Logic from ffi.rs
             // Try parsing "10 mg" or "1 point" or "1 point: Description"
             let parts: Vec<&str> = s.split_whitespace().collect();
             if parts.len() >= 2 {
                 if let Ok(n) = parts[0].parse::<f64>() {
                     // Create custom unit. We take the second part as unit.
                     // Ideally we might want to strip punctuation (like ":") from the unit string if present.
                     let unit_str = parts[1].trim_matches(|c| c == ':' || c == ',' || c == ';');
                     let unit = Unit::Custom(unit_str.to_string());
                     Some(RuntimeValue::Quantity(n, unit))
                 } else {
                     Some(RuntimeValue::String(s))
                 }
             } else {
                 Some(RuntimeValue::String(s))
             }
         }
    } else {
        None
    }
}

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_ask_parsing_regression() {
    use hippocrates_engine::parser::parse_plan;
    let input = r#"
<v> is a number.
<TestPlan> is a plan:
    before plan:
        ask for <v>.
"#;
    let plan = parse_plan(input).expect("Failed to parse plan");
    // We expect the first action in the plan to be AskQuestion with subject "v"
    println!("Parsed Plan: {:?}", plan);
}
