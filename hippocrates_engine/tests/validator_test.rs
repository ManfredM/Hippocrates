
#[test]
fn test_validator_numeric_overlap() {
    use hippocrates_engine::parser;
    use hippocrates_engine::runtime::validator;
    
    // Explicit overlapping ranges
    let input = r#"
<val> is a number:
    valid values:
        0 ... 100

<plan> is a plan:
    during plan:
        assess <val>:
            1:
                <log> = "one".
            0 ... 40:
                <log> = "low".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    
    assert!(result.is_err(), "Should fail validation due to overlap");
    let errors = result.err().unwrap();
    let has_overlap = errors.iter().any(|e| e.message.contains("Constraint Violation") && e.message.contains("covered multiple times"));
    assert!(has_overlap, "Errors should contain overlap message: {:?}", errors);
}

#[test]
fn test_validator_enum_duplicate() {
    use hippocrates_engine::parser;
    use hippocrates_engine::runtime::validator;
    
    // Duplicate enum values
    let input = r#"
<val> is an enumeration:
    valid values:
        "A"; "B"

<plan> is a plan:
    during plan:
        assess <val>:
            "A":
                <log> = "A1".
            "A":
                <log> = "A2".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    
    // This expects FAILURE. If current implementation doesn't check duplication, this will FAIL the test (pass validation).
    // We expect it to FAIL validation.
    if result.is_ok() {
        panic!("Validation succeeded but should have failed for duplicate enum values!");
    }
}
