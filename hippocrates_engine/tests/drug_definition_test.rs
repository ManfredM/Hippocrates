
mod fixture_loader;

#[test]
fn test_drug_definition_validation() {
    use fixture_loader::{load_scenario, ScenarioKind};
    use hippocrates_engine::parser;
    use hippocrates_engine::runtime::validator;

    let input = load_scenario("tests/fixtures/specs.hipp", "drug_definition", ScenarioKind::Fail);

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");
    let result = validator::validate_file(&plan);

    // Expect validation error due to <BadUnit>
    assert!(result.is_err(), "Validation passed but should fail due to undefined unit <BadUnit>");
    
    let errors = result.err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("Undefined unit")), "Errors should contain 'Undefined unit': {:?}", errors);
}
