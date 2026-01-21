
#[test]
fn test_period_definition_parsing() {
    use hippocrates_engine::parser;
    use hippocrates_engine::ast::Definition;
    use std::fs;

    let input = fs::read_to_string("tests/plans/period_coverage.hipp")
        .expect("Failed to read plan file");

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    let mut morning_found = false;
    let mut week_found = false;

    for def in plan.definitions {
        if let Definition::Period(pd) = def {
            if pd.name == "Morning" {
                morning_found = true;
            } else if pd.name == "Week" {
                week_found = true;
            }
        }
    }

    assert!(morning_found, "Morning period not found");
    assert!(week_found, "Week period not found");
}
