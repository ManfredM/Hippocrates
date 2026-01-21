
mod fixture_loader;

use hippocrates_engine::parser;
use hippocrates_engine::runtime::validator;

#[test]
#[ignore] // TODO: Fix regression: timeframe_vars not populated correctly
fn test_validator_fails_missing_not_enough_data() {
    let input = r#"<value> is a number:
    valid values:
        0 points ... 10 points
    calculation:
        timeframe for analysis is 2 days ago ... now:
            value = count of <something>.

<plan> is a plan:
    during plan:
        timeframe for analysis is between 2 days ago ... now:
            <value> = count of <something>.
        assess <value>:
            0 points ... 10 points:
                show message "covered".
"#;

    let result = parser::parse_plan(input.trim());
    assert!(result.is_err());
    let err_msg = result.err().unwrap();
    assert!(err_msg.message.contains("depends on a timeframe calculation but does not handle 'Not enough data'"));
}

#[test]
fn test_validator_passes_with_not_enough_data() {
    let input = r#"<value> is a number:
    valid values:
        0 <points> ... 10 <points>
    calculation:
        timeframe for analysis is 2 days ago ... now:
            <value> = count of <something>.
    
<plan> is a plan:
    during plan:
        timeframe for analysis is between 2 days ago ... now:
            <value> = count of <something>.
        assess <value>:
            Not enough data:
                show message "waiting".
            0 <points> ... 10 <points>:
                show message "covered".
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);

    assert!(result.is_ok());
}

#[test]
fn test_validate_copd_plan() {
    use fixture_loader::{load_scenario, ScenarioKind};

    let input = load_scenario("tests/fixtures/runtime_plans.hipp", "copd_plan", ScenarioKind::Pass);
    let plan = parser::parse_plan(&input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_ok(), "Validation failed for COPD plan: {:?}", result.err());
}

#[test]
fn test_validator_integer_gap_message() {
    let input = r#"<val> is a number:
    valid values:
        0 <points> ... 10 <points>

<plan> is a plan:
    during plan:
        <val> = 0 <points>.
        assess <val>:
            0 <points> ... 5 <points>:
                show message "Lower half".
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    let msg = &errors[0].message;
    println!("Error message: {}", msg);
    // User expects "6 ... 10" or similar integer formatting logic.
    // Our logic produces "between 6 and 10".
    assert!(msg.contains("end: 6 ... 10"));
}

#[test]
fn test_validator_float_gap() {
    let input = r#"<val> is a number:
    valid values:
        0.0 mg ... 10.0 mg

<plan> is a plan:
    during plan:
        <val> = 0.0 mg.
        assess <val>:
            0.0 mg ... 5.5 mg:
                show message "Lower part".
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err());
    
    let errors = result.unwrap_err();
    let msg = &errors[0].message;
    println!("Float Gap Error message: {}", msg);
    // 0.0...10.0 implies precision 1 (0.1). 
    // Logic: step 0.1. current 5.5. start_gap 5.6.
    // Expect: "5.6 ... 10.0"
    assert!(msg.contains("end: 5.6 ... 10.0"));
}

#[test]
fn test_precision_gaps() {
    // Case 1: Float precision (0.0 ... 10.0)
    let input = r#"<val> is a number:
    valid values:
        0.0 mm ... 10.0 mm

<plan> is a plan:
    during plan:
        <val> = 0.0 mm.
        assess <val>:
            0.0 mm ... 5.5 mm:
                show message "Lower part".
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let msg = &errors[0].message;
    // Expect: missing 5.6 ... 10.0
    // The validator format_val uses precision 1.
    println!("Precision 1 Error: {}", msg);
    assert!(msg.contains("end: 5.6 ... 10.0"));

    // Case 2: Integer precision (0 ... 10)
    let input2 = r#"<val> is a number:
    valid values:
        0 <points> ... 10 <points>

<plan> is a plan:
    during plan:
        <val> = 0 <points>.
        assess <val>:
            0 <points> ... 5 <points>:
                show message "Lower part".
"#;
    let plan2 = parser::parse_plan(input2.trim()).expect("Failed to parse 2");
    let result2 = validator::validate_file(&plan2);
    assert!(result2.is_err());
    let errors2 = result2.unwrap_err();
    let msg2 = &errors2[0].message;
    println!("Precision 0 Error: {}", msg2);
    // Expect: missing 6 ... 10
    // Note: The unit 'points' may or may not be in message depending on format_val implementation.
    // Assuming format_val only prints number for now.
    assert!(msg2.contains("end: 6")); 
    assert!(msg2.contains("10"));
}

#[test]
fn test_range_overlap() {
    let input = r#"<Temp> is a number:
    valid values:
        35.0 °C ... 42.0 °C

<plan> is a plan:
    during plan:
        <Temp> = 35.0 °C.
        assess <Temp>:
            38.0 °C ... 42.0 °C:
                show message "Fever".
            35.0 °C ... 38.0 °C:
                show message "Normal".
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let msg = &errors[0].message;
    println!("Overlap Error: {}", msg);
    assert!(msg.contains("Constraint Violation"));
    assert!(msg.contains("covered multiple times"));
}

#[test]
fn test_unit_requirement() {
    // 1. Missing units -> Error
    let input = r#"<Steps> is a number:
    valid values:
        0 ... 10000
    "#;
    let result = parser::parse_plan(input.trim());
    assert!(result.is_err(), "Expected parser error for unitless range");


    // 2. Consistent units (Custom) -> OK
    let input2 = r#"<Steps> is a number:
    valid values:
        0 <steps> ... 10000 <steps>
    "#;
    let plan = parser::parse_plan(input2.trim()).expect("Failed to parse valid units");
    let result2 = validator::validate_file(&plan);
    assert!(result2.is_ok());

    // 3. Mixed units -> Error
    let input3 = r#"<Steps> is a number:
    valid values:
        0 ... 10000 <steps>
    "#;
    let result3 = parser::parse_plan(input3.trim());
    assert!(result3.is_err(), "Expected parser error for mixed unitless range");
}

#[test]
fn test_validation_error_line_number() {
    // Reproduction of user issue: Line number missing for unit parsing error
    let input = r#"
<Temperature> is a number:
    valid values:
        35.0 ... 42.0
"#;
    let result = parser::parse_plan(input.trim());
    assert!(result.is_err(), "Expected parser error for unitless range");
    let err = result.err().unwrap();
    assert!(err.line > 0, "Error line number should be > 0, got {}", err.line);
}

#[test]
fn test_reproduce_missing_error() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 10 kg

<plan> is a plan:
    during plan:
        assess <val>:
            0 kg ... 5 kg:
                show message "Lower half covered".
"#;
    let plan = hippocrates_engine::parser::parse_plan(input).expect("Failed to parse");
    let result = hippocrates_engine::runtime::validator::validate_file(&plan);
    
    assert!(result.is_err(), "Validator should have returned an error");

    let errors = result.unwrap_err();
    assert!(!errors.is_empty(), "Should have at least one error");
}

#[test]
fn test_unitless_assess_fails() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 100 kg

<plan> is a plan:
    during plan:
        <val> = 0 kg.
        assess <val>:
            0 ... 100:
                show message "Done".
"#;
    let result = hippocrates_engine::parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for unitless assess range");
}

#[test]
fn test_unitless_definition_fails() {
    let input = r#"
<val> is a number:
    valid values:
        0 ... 100
"#;
    let result = hippocrates_engine::parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for unitless definition");
}

#[test]
fn test_unitless_definition_constraint_fails() {
    let input = r#"
<val> is a number:
    valid values:
        0 ... 100
"#;
    let result = hippocrates_engine::parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for unitless constraint");
}
