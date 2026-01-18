use hippocrates_engine::ast::{Plan, Definition, Property, StatementKind, RangeSelector, Expression};
use hippocrates_engine::parser;
use hippocrates_engine::runtime::validator;

#[test]
fn test_validator_fails_missing_not_enough_data() {
    let input = r#"<value> is a number:
    valid values: 0 ... 10
    calculation:
        timeframe for analysis is 2 days ago ... now:
            value = count of <something>.

<plan> is a plan:
    during plan:
        assess <value>:
            0 ... 10:
                show message "covered".
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);

    assert!(result.is_err());
    let err_msg = result.err().unwrap();
    assert!(err_msg.contains("depends on a timeframe calculation but does not handle 'Not enough data'"));
}

#[test]
fn test_validator_passes_with_not_enough_data() {
    let input = r#"<value> is a number:
    valid values: 0 ... 10
    calculation:
        timeframe for analysis is 2 days ago ... now:
            value = count of <something>.

<plan> is a plan:
    during plan:
        assess <value>:
            Not enough data:
                show message "waiting".
            0 ... 10:
                show message "covered".
"#;

    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);

    assert!(result.is_ok());
}

#[test]
fn test_validate_copd_plan() {
    let input = std::fs::read_to_string("plans/treating_copd.hipp").expect("Failed to read file");
    let plan = parser::parse_plan(&input).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_ok(), "Validation failed for COPD plan: {:?}", result.err());
}
