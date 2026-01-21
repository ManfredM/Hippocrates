#[test]
fn test_meaning_coverage_gaps() {
    use hippocrates_engine::parser;
    use hippocrates_engine::runtime::validator::validate_file;

    // Case 1: Integer gap (large)
    let input = r#"
<BloodPressure> is a number:
    unit is mmHg
    valid values:
        0 mmHg ... 300 mmHg
    meaning:
        0 mmHg ... 90 mmHg:
            meaning of value = "Low"
        91 mmHg ... 120 mmHg:
            meaning of value = "Normal"
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan input");
    match validate_file(&plan) {
        Ok(_) => panic!("Expected validation error for BP, but got OK"),
        Err(errors) => {
            let has_gap_error = errors.iter().any(|e| e.message.contains("121") && e.message.contains("300"));
             if !has_gap_error {
                println!("BP Errors: {:?}", errors);
            }
            assert!(has_gap_error, "Expected error about missing coverage [121...300]");
        }
    }
}

#[test]
fn test_float_gap_detection() {
    use hippocrates_engine::parser;
    use hippocrates_engine::runtime::validator::validate_file;

    // Case 2: Float gap (small, < 1.0)
    let input = r#"
<OxygenSat> is a number:
    unit is mg
    valid values:
        0.0 mg ... 1.0 mg
    meaning:
        0.0 mg ... 0.5 mg:
            meaning of value = "Low"
         0.7 mg ... 1.0 mg:
            meaning of value = "Normal"
"#;
    // Missing 0.51...0.59 (approx) or just gap between 0.5 and 0.6.
    
    let plan = parser::parse_plan(input).expect("Failed to parse plan input");
    match validate_file(&plan) {
        Ok(_) => panic!("Expected validation error for Oxygen, but got OK"),
        Err(errors) => {
            // We expect an error about incomplete coverage or gap.
            // With current logic (hardcoded +1.0), this will likely pass incorrectly.
            let has_gap_error = errors.iter().any(|e| e.message.contains("Gap") || e.message.contains("gap"));
            if !has_gap_error {
                println!("Oxygen Errors: {:?}", errors);
            }
            assert!(has_gap_error, "Expected error about float gap [0.5...0.6]");
        }
    }
}

#[test]
fn test_meaning_coverage_disjoint_ranges() {
    use hippocrates_engine::parser;
    use hippocrates_engine::runtime::validator::validate_file;

    let input = r#"
<WindowedValue> is a number:
    unit is mg
    valid values:
        1 mg ... 3 mg;
        5 mg ... 10 mg
    meaning:
        1 mg ... 3 mg:
            meaning of value = "Low"
        5 mg ... 10 mg:
            meaning of value = "High"
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan input");
    if let Err(errors) = validate_file(&plan) {
        println!("Disjoint Range Errors: {:?}", errors);
        panic!("Expected validation OK for disjoint ranges");
    }
}
