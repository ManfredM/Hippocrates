// Spec §4.2-§4.6: required properties, data flow, coverage, range compliance, and sufficiency.

use hippocrates_engine::parser;
use hippocrates_engine::runtime::validator;
use hippocrates_engine::runtime::validator::{Interval, calculate_interval};
use std::collections::HashMap;

// REQ-4.4-01: meaning ranges must cover valid values (integer gaps).
#[test]
fn spec_meaning_coverage_gaps_integer() {
    use hippocrates_engine::runtime::validator::validate_file;

    let input = r#"
<BloodPressure> is a number:
    unit is mmHg.
    valid values:
        0 mmHg ... 300 mmHg.
    meaning:
        0 mmHg ... 90 mmHg:
            meaning of value = "Low".
        91 mmHg ... 120 mmHg:
            meaning of value = "Normal".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan input");
    match validate_file(&plan) {
        Ok(_) => panic!("Expected validation error for BP, but got OK"),
        Err(errors) => {
            let has_gap_error = errors.iter().any(|e| e.message.contains("121") && e.message.contains("300"));
            assert!(has_gap_error, "Expected error about missing coverage [121...300]");
        }
    }
}

// REQ-4.4-02: meaning ranges must cover valid values (float gaps).
#[test]
fn spec_meaning_coverage_gaps_float() {
    use hippocrates_engine::runtime::validator::validate_file;

    let input = r#"
<OxygenSat> is a number:
    unit is mg.
    valid values:
        0.0 mg ... 1.0 mg.
    meaning:
        0.0 mg ... 0.5 mg:
            meaning of value = "Low".
        0.7 mg ... 1.0 mg:
            meaning of value = "Normal".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan input");
    match validate_file(&plan) {
        Ok(_) => panic!("Expected validation error for Oxygen, but got OK"),
        Err(errors) => {
            let has_gap_error = errors.iter().any(|e| e.message.contains("Gap") || e.message.contains("gap"));
            assert!(has_gap_error, "Expected error about float gap [0.5...0.6]");
        }
    }
}

// REQ-4.4-03: disjoint valid ranges are allowed when fully covered.
#[test]
fn spec_meaning_coverage_disjoint_ranges_ok() {
    use hippocrates_engine::runtime::validator::validate_file;

    let input = r#"
<WindowedValue> is a number:
    unit is mg.
    valid values:
        1 mg ... 3 mg.
        5 mg ... 10 mg.
    meaning:
        1 mg ... 3 mg:
            meaning of value = "Low".
        5 mg ... 10 mg:
            meaning of value = "High".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan input");
    if let Err(errors) = validate_file(&plan) {
        panic!("Expected validation OK for disjoint ranges, got {:?}", errors);
    }
}

// REQ-4.4-04: overlapping numeric assessment ranges are invalid.
#[test]
fn spec_validator_numeric_overlap() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 100 kg.

<plan> is a plan:
    during plan:
        assess <val>:
            1 kg:
                <log> = "one".
            0 kg ... 40 kg:
                <log> = "low".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);

    assert!(result.is_err(), "Should fail validation due to overlap");
    let errors = result.err().unwrap();
    let has_overlap = errors.iter().any(|e| e.message.contains("Constraint Violation") && e.message.contains("covered multiple times"));
    assert!(has_overlap, "Errors should contain overlap message: {:?}", errors);
}

// REQ-4.4-05: duplicate enumeration cases are invalid.
#[test]
fn spec_validator_enum_duplicate() {
    let input = r#"
<val> is an enumeration:
    valid values:
        "A"; "B".

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
    if result.is_ok() {
        panic!("Validation succeeded but should have failed for duplicate enum values!");
    }
}

// REQ-4.6-01: timeframe calculations require Not enough data handling.
#[test]
fn spec_validator_requires_not_enough_data_case() {
    let input = r#"<points> is a unit:
    plural is <points>.

<value> is a number:
    valid values:
        0 <points> ... 10 <points>.
    calculation:
        timeframe for analysis is 2 days ago ... now:
            <value> = count of <something>.

<plan> is a plan:
    during plan:
        timeframe for analysis is between 2 days ago ... now:
            <value> = count of <something>.
        assess <value>:
            0 <points> ... 10 <points>:
                show message "covered".
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Validation should fail without a Not enough data case");
    let errors = result.err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("depends on a timeframe calculation but does not handle 'Not enough data'")), "{:?}", errors);
}

// REQ-4.6-02: Not enough data handling satisfies sufficiency.
#[test]
fn spec_validator_passes_with_not_enough_data() {
    let input = r#"<value> is a number:
    valid values:
        0 <points> ... 10 <points>.
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

// REQ-5.1-01: full-plan validation passes for a complete plan.
#[test]
fn spec_validate_plan_fixture_suite() {
    use crate::fixture_loader::{list_scenarios, load_scenario, ScenarioKind};

    let path = "tests/fixtures/validation_plans.hipp";

    let pass_scenarios = list_scenarios(path, ScenarioKind::Pass);
    assert!(!pass_scenarios.is_empty(), "No PASS scenarios in {}", path);

    for name in pass_scenarios {
        let input = load_scenario(path, &name, ScenarioKind::Pass);
        let plan = parser::parse_plan(&input)
            .unwrap_or_else(|err| panic!("Failed to parse PASS scenario {}: {:?}", name, err));
        let result = validator::validate_file(&plan);
        assert!(
            result.is_ok(),
            "Validation failed for PASS scenario {}: {:?}",
            name,
            result.err()
        );
    }

    let fail_scenarios = list_scenarios(path, ScenarioKind::Fail);
    assert!(!fail_scenarios.is_empty(), "No FAIL scenarios in {}", path);

    for name in fail_scenarios {
        let input = load_scenario(path, &name, ScenarioKind::Fail);
        let plan = parser::parse_plan(&input)
            .unwrap_or_else(|err| panic!("Failed to parse FAIL scenario {}: {:?}", name, err));
        let result = validator::validate_file(&plan);
        assert!(
            result.is_err(),
            "Validation unexpectedly passed for FAIL scenario {}",
            name
        );
    }
}

// REQ-4.4-06: gap detection reports missing integer spans.
#[test]
fn spec_validator_integer_gap_message() {
    let input = r#"<val> is a number:
    valid values:
        0 <points> ... 10 <points>.

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
    assert!(msg.contains("end: 6 ... 10"));
}

// REQ-4.4-07: gap detection reports missing float spans.
#[test]
fn spec_validator_float_gap_message() {
    let input = r#"<val> is a number:
    valid values:
        0.0 mg ... 10.0 mg.

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
    assert!(msg.contains("end: 5.6 ... 10.0"));
}

// REQ-4.4-08: coverage gaps respect precision for float and integer ranges.
#[test]
fn spec_precision_gaps() {
    let input = r#"<val> is a number:
    valid values:
        0.0 mm ... 10.0 mm.

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
    assert!(msg.contains("end: 5.6 ... 10.0"));

    let input2 = r#"<val> is a number:
    valid values:
        0 <points> ... 10 <points>.

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
    assert!(msg2.contains("end: 6"));
    assert!(msg2.contains("10"));
}

// REQ-4.4-09: overlapping ranges are rejected.
#[test]
fn spec_range_overlap() {
    let input = r#"<Temp> is a number:
    valid values:
        35.0 °C ... 42.0 °C.

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
    assert!(msg.contains("Constraint Violation"));
    assert!(msg.contains("covered multiple times"));
}

// REQ-4.2-01: numeric valid values require units.
#[test]
fn spec_unit_requirement_validation() {
    let input = r#"<Steps> is a number:
    valid values:
        0 ... 10000.
    "#;
    let result = parser::parse_plan(input.trim());
    assert!(result.is_err(), "Expected parser error for unitless range");

    let input2 = r#"<Steps> is a number:
    valid values:
        0 <steps> ... 10000 <steps>.
    "#;
    let plan = parser::parse_plan(input2.trim()).expect("Failed to parse valid units");
    let result2 = validator::validate_file(&plan);
    assert!(result2.is_ok());

    let input3 = r#"<Steps> is a number:
    valid values:
        0 ... 10000 <steps>.
    "#;
    let result3 = parser::parse_plan(input3.trim());
    assert!(result3.is_err(), "Expected parser error for mixed unitless range");
}

// REQ-4.2-05: unit requirement errors report line numbers.
#[test]
fn spec_validation_error_line_number() {
    let input = r#"
<Temperature> is a number:
    valid values:
        35.0 ... 42.0.
"#;
    let result = parser::parse_plan(input.trim());
    assert!(result.is_err(), "Expected parser error for unitless range");
    let err = result.err().unwrap();
    assert!(err.line > 0, "Error line number should be > 0, got {}", err.line);
}

// REQ-4.2-06: numbers and enumerations must define valid values.
#[test]
fn spec_missing_valid_values_fails() {
    let input = r#"
<num> is a number:
    unit is kg.

<plan> is a plan:
    during plan:
        show message "Hi".
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected validation error for missing valid values");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("Missing 'valid values'")));

    let input2 = r#"
<state> is an enumeration:
    documentation:
        english: "State".

<plan> is a plan:
    during plan:
        show message "Hi".
"#;
    let plan2 = parser::parse_plan(input2.trim()).expect("Failed to parse");
    let result2 = validator::validate_file(&plan2);
    assert!(result2.is_err(), "Expected validation error for missing valid values on enum");
    let errors2 = result2.unwrap_err();
    assert!(errors2.iter().any(|e| e.message.contains("Missing 'valid values'")));
}

// REQ-3.6-03: timeframe selector identifiers must refer to defined periods.
#[test]
fn spec_timeframe_selector_requires_period_definition() {
    let input = r#"
<plan> is a plan:
    during plan:
        timeframe for analysis is <MissingPeriod>:
            show message "Hi".
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected validation error for unknown period reference");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("MissingPeriod")));
}

// REQ-4.4-10: missing coverage yields a validation error.
#[test]
fn spec_reproduce_missing_error() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 10 kg.

<plan> is a plan:
    during plan:
        assess <val>:
            0 kg ... 5 kg:
                show message "Lower half covered".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);

    assert!(result.is_err(), "Validator should have returned an error");

    let errors = result.unwrap_err();
    assert!(!errors.is_empty(), "Should have at least one error");
}

// REQ-4.2-02: assessment ranges require units.
#[test]
fn spec_unitless_assess_fails() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 100 kg.

<plan> is a plan:
    during plan:
        <val> = 0 kg.
        assess <val>:
            0 ... 100:
                show message "Done".
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for unitless assess range");
}

// REQ-4.2-03: numeric definitions require units.
#[test]
fn spec_unitless_definition_fails() {
    let input = r#"
<val> is a number:
    valid values:
        0 ... 100.
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for unitless definition");
}

// REQ-4.5-01: interval math supports range compliance checks.
#[test]
fn spec_interval_creation_and_math() {
    let i = Interval::new(5.0, 10.0);
    assert_eq!(i.min, 5.0);
    assert_eq!(i.max, 10.0);

    let i = Interval::new(-5.0, 10.0);
    assert_eq!(i.min, 0.0);
    assert_eq!(i.max, 10.0);

    let mut ranges = HashMap::new();
    ranges.insert("b".to_string(), Interval::new(10.0, 20.0));

    let expr_with_var = hippocrates_engine::ast::Expression::Binary(
        Box::new(hippocrates_engine::ast::Expression::Literal(hippocrates_engine::ast::Literal::Number(5.0, None))),
        "+".to_string(),
        Box::new(hippocrates_engine::ast::Expression::Variable("b".to_string())),
    );

    let res = calculate_interval(&expr_with_var, &ranges);

    assert_eq!(res.min, 15.0);
    assert_eq!(res.max, 25.0);

    let a = Interval::exact(5.0);
    let b = Interval::exact(10.0);
    let diff = Interval::new((a.min - b.max).max(0.0), (a.max - b.min).max(0.0));
    assert_eq!(diff.min, 0.0);
    assert_eq!(diff.max, 0.0);
}

// REQ-4.3-01: values cannot be used before assignment.
#[test]
fn spec_data_flow_use_before_assignment_fails() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 10 kg.

<plan> is a plan:
    during plan:
        show message "Value is " + <val>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected data flow validation error");

    let errors = result.err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("used before being assigned")));
}

// REQ-4.3-02: calculation properties do not seed values; plans must assign or ask before use.
#[test]
fn spec_calculation_does_not_initialize_value() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 10 kg.
    calculation:
        <val> = 5 kg.

<plan> is a plan:
    during plan:
        show message "Value is " + <val>.
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected data flow error for calculated value not assigned in plan");
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("used before being assigned")));
}

// REQ-4.3-03: statistical functions read history and do not require local initialization.
#[test]
fn spec_statistical_functions_do_not_require_local_init() {
    let input = r#"
<val> is an enumeration:
    valid values:
        "Yes"; "No".

<plan> is a plan:
    during plan:
        show message count of <val> is "Yes".
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_ok(), "Expected statistical functions to bypass local init checks");
}

// REQ-4.3-04: listen for and context data initialize values for data flow.
#[test]
fn spec_listen_and_context_initialize_values() {
    let input = r#"
<signal> is an enumeration:
    valid values:
        "Yes"; "No".

<plan> is a plan:
    during plan:
        listen for <signal>:
            show message "Heard".
        show message "Value is " + <signal>.
        context for analysis:
            data: <signal>.
            show message "Ctx " + <signal>.
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_ok(), "Expected listen/context data to initialize values");
}

// REQ-4.2-04: ask requires a question property on the value.
#[test]
fn spec_ask_requires_question_property() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 10 kg.

<plan> is a plan:
    during plan:
        ask <val>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected validation error for ask without question");

    let errors = result.err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("does not have a 'question' property")));
}

// REQ-4.4-11: trend assessments require full coverage.
#[test]
fn spec_trend_requires_full_coverage() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 10 kg.

<plan> is a plan:
    during plan:
        assess trend of <val>:
            "increase":
                show message "Up".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected validation error for missing trend coverage");
}

// REQ-4.5-02: assignment range compliance fails when out of bounds.
#[test]
fn spec_assignment_range_compliance_warning() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 10 kg.

<plan> is a plan:
    during plan:
        <val> = 5 kg + 10 kg.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected assignment range validation error");

    let errors = result.err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("Assignment Validity Warning")));
}
