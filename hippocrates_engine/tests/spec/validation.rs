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
    meaning of <BloodPressure>:
        valid meanings:
            <Low>; <Normal>.
        0 mmHg ... 90 mmHg:
            meaning of value = <Low>.
        91 mmHg ... 120 mmHg:
            meaning of value = <Normal>.
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
    meaning of <OxygenSat>:
        valid meanings:
            <Low>; <Normal>.
        0.0 mg ... 0.5 mg:
            meaning of value = <Low>.
        0.7 mg ... 1.0 mg:
            meaning of value = <Normal>.
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
    meaning of <WindowedValue>:
        valid meanings:
            <Low>; <High>.
        1 mg ... 3 mg:
            meaning of value = <Low>.
        5 mg ... 10 mg:
            meaning of value = <High>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan input");
    if let Err(errors) = validate_file(&plan) {
        panic!("Expected validation OK for disjoint ranges, got {:?}", errors);
    }
}

// REQ-4.4-13: valid meanings must be fully used across meaning assessments.
#[test]
fn spec_meaning_valid_meanings_must_be_used() {
    use hippocrates_engine::runtime::validator::validate_file;

    let input = r#"
<weight> is a number:
    unit is kg.
    valid values:
        1 kg ... 1000 kg.
    meaning of <weight>:
        valid meanings:
            <light>; <heavy>; <super heavy>.
        assess meaning of <weight>:
            1 kg ... 100 kg:
                <light>.
            101 kg ... 900 kg:
                <heavy>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan input");
    let result = validate_file(&plan);
    assert!(result.is_err(), "Expected validation error for missing valid meaning");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.message.contains("Missing")),
        "Expected error about missing meanings, got {:?}",
        errors
    );
}

// REQ-4.4-14: meaning labels must be drawn from the declared valid meanings list.
#[test]
fn spec_meaning_invalid_label_rejected() {
    use hippocrates_engine::runtime::validator::validate_file;

    let input = r#"
<weight> is a number:
    unit is kg.
    valid values:
        1 kg ... 1000 kg.
    meaning of <weight>:
        valid meanings:
            <light>; <heavy>.
        assess meaning of <weight>:
            1 kg ... 100 kg:
                <light>.
            101 kg ... 900 kg:
                <unknown>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan input");
    let result = validate_file(&plan);
    assert!(result.is_err(), "Expected validation error for invalid meaning label");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.message.contains("invalid meaning")),
        "Expected error about invalid meaning label, got {:?}",
        errors
    );
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
        <A>; <B>.

<plan> is a plan:
    during plan:
        assess <val>:
            <A>:
                <log> = "A1".
            <A>:
                <log> = "A2".
"#;
    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    if result.is_ok() {
        panic!("Validation succeeded but should have failed for duplicate enum values!");
    }
}

// REQ-3.4-13: enumeration valid values must be identifiers (angle brackets).
#[test]
fn spec_enum_valid_values_require_identifiers() {
    let input = r#"
<val> is an enumeration:
    valid values:
        "Yes"; "No".

<plan> is a plan:
    during plan:
        ask <val>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected validation error for enum string literals");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.message.contains("identifiers")),
        "Expected enum identifier error, got {:?}",
        errors
    );
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
                information "covered".
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Validation should fail without a Not enough data case");
    let errors = result.err().unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("must handle 'Not enough data'")),
        "{:?}",
        errors
    );
}

// REQ-3.12-05: statistical functions require an analysis timeframe context.
#[test]
fn spec_statistical_functions_require_timeframe_context() {
    let input = r#"<point> is a unit:
    plural is <points>.

<flag> is an enumeration:
    valid values:
        <Yes>; <No>.

<count> is a number:
    valid values:
        0 <points> ... 10 <points>.

<plan> is a plan:
    during plan:
        <count> = count of <flag> is <Yes>.
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Statistical functions require an analysis timeframe context")),
        "{:?}",
        errors
    );
}

// REQ-4.6-04: Not enough data is only allowed for statistical assessments.
#[test]
fn spec_not_enough_data_requires_statistical_target() {
    let input = r#"<unit> is a unit:
    plural is <units>.

<value> is a number:
    valid values:
        0 <units> ... 1 <unit>.

<plan> is a plan:
    during plan:
        <value> = 0 <units>.
        assess <value>:
            Not enough data:
                information "No data".
            0 <units> ... 1 <unit>:
                information "OK".
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Not enough data is only allowed")),
        "{:?}",
        errors
    );
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
                information "waiting".
            0 <points> ... 10 <points>:
                information "covered".
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
                information "Lower half".
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
                information "Lower part".
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
                information "Lower part".
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
                information "Lower part".
"#;
    let plan2 = parser::parse_plan(input2.trim()).expect("Failed to parse 2");
    let result2 = validator::validate_file(&plan2);
    assert!(result2.is_err());
    let errors2 = result2.unwrap_err();
    let msg2 = &errors2[0].message;
    assert!(msg2.contains("end: 6"));
    assert!(msg2.contains("10"));
}

// REQ-4.4-12: numeric valid value ranges use consistent precision across bounds and intervals.
#[test]
fn spec_precision_consistency() {
    let input = r#"<val> is a number:
    valid values:
        0.0 mg ... 10 mg.
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err());
    let msg = &result.unwrap_err()[0].message;
    assert!(msg.contains("Precision mismatch"));

    let input2 = r#"<val> is a number:
    valid values:
        0.0 mg ... 9.0 mg.
        10.00 mg ... 20.00 mg.
"#;
    let plan2 = parser::parse_plan(input2.trim()).expect("Failed to parse 2");
    let result2 = validator::validate_file(&plan2);
    assert!(result2.is_err());
    let msg2 = &result2.unwrap_err()[0].message;
    assert!(msg2.contains("Precision mismatch"));
}

// REQ-4.2-07: valid value ranges must not overlap.
#[test]
fn spec_valid_values_ranges_do_not_overlap() {
    let input = r#"<val> is a number:
    valid values:
        0 mg ... 10 mg.
        10 mg ... 20 mg.
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected validation error for overlapping valid ranges");
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Valid values") && e.message.contains("overlapping")),
        "Expected overlapping valid values error, got {:?}",
        errors
    );
}

// REQ-4.2-07: valid value ranges must not overlap (date/time and time-of-day).
#[test]
fn spec_valid_values_datetime_ranges_do_not_overlap() {
    let input = r#"<appointment time> is a date/time:
    valid values:
        2026-01-01 08:00 ... 2026-01-10 12:00.
        2026-01-10 12:00 ... 2026-01-15 12:00.
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected validation error for overlapping date/time ranges");
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("date/time ranges")),
        "Expected overlapping date/time ranges error, got {:?}",
        errors
    );

    let input2 = r#"<quiet hours> is a date/time:
    valid values:
        22:00 ... 02:00.
        01:00 ... 03:00.
"#;
    let plan2 = parser::parse_plan(input2.trim()).expect("Failed to parse");
    let result2 = validator::validate_file(&plan2);
    assert!(result2.is_err(), "Expected validation error for overlapping time-of-day ranges");
    let errors2 = result2.unwrap_err();
    assert!(
        errors2
            .iter()
            .any(|e| e.message.contains("time-of-day ranges")),
        "Expected overlapping time-of-day ranges error, got {:?}",
        errors2
    );
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
                information "Fever".
            35.0 °C ... 38.0 °C:
                information "Normal".
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
        information "Hi".
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
        information "Hi".
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
            information "Hi".
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
                information "Lower half covered".
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
                information "Done".
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
        information "Value is " + <val>.
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
        information "Value is " + <val>.
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
        <Yes>; <No>.

<plan> is a plan:
    during plan:
        timeframe for analysis is between 5 days ago ... now:
            information count of <val> is <Yes>.
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_ok(), "Expected statistical functions to bypass local init checks");
}

// REQ-4.3-05: meaning-of requires a question property when the value is not initialized.
#[test]
fn spec_meaning_of_requires_question_when_uninitialized() {
    let input = r#"
<val> is a number:
    unit is kg.
    valid values:
        0 kg ... 10 kg.
    meaning of <val>:
        valid meanings:
            <ok>.
        0 kg ... 10 kg:
            meaning of value = <ok>.

<label> is a string.

<plan> is a plan:
    during plan:
        <label> = meaning of <val>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected data flow validation error");
    let errors = result.err().unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Meaning of") && e.message.contains("question property")),
        "Expected meaning-of question requirement error, got {:?}",
        errors
    );
}

// REQ-4.3-05: meaning-of is allowed when the value is askable.
#[test]
fn spec_meaning_of_allows_question_when_uninitialized() {
    let input = r#"
<val> is a number:
    unit is kg.
    valid values:
        0 kg ... 10 kg.
    question:
        ask "What is the value".
    meaning of <val>:
        valid meanings:
            <ok>.
        0 kg ... 10 kg:
            meaning of value = <ok>.

<label> is a string.

<plan> is a plan:
    during plan:
        <label> = meaning of <val>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_ok(), "Expected validation OK for meaning-of with question");
}

// REQ-4.3-04: listen for and context data initialize values for data flow.
#[test]
fn spec_listen_and_context_initialize_values() {
    let input = r#"
<signal> is an enumeration:
    valid values:
        <Yes>; <No>.

<plan> is a plan:
    during plan:
        listen for <signal>:
            information "Heard".
        information "Value is " + <signal>.
        context for analysis:
            data: <signal>.
            information "Ctx " + <signal>.
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
        timeframe for analysis is between 7 days ago ... now:
            assess trend of <val>:
                "increase":
                    information "Up".
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
