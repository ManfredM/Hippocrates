use crate::fixture_loader::{load_scenario, ScenarioKind};
use hippocrates_engine::parser;
use hippocrates_engine::runtime::validator;

#[test]
#[ignore = "Non-spec integration/regression"]
fn test_addressee_validation() {

    let input = load_scenario("tests/fixtures/specs.hipp", "addressee_definition", ScenarioKind::Fail);

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");
    let result = validator::validate_file(&plan);

    // Expect validation error due to bad email
    assert!(result.is_err(), "Validation passed but should fail due to invalid email format");
    let errors = result.err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("Invalid email format")), "Errors should contain 'Invalid email format': {:?}", errors);
}
