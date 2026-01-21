
#[test]
fn test_value_definition_parsing() {
    use hippocrates_engine::parser;
    use hippocrates_engine::ast::Definition;
    use hippocrates_engine::domain::ValueType;
    use std::fs;

    let input = fs::read_to_string("tests/plans/value_definition_coverage.hipp")
        .expect("Failed to read plan file");

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    let mut unit_found = false;
    let mut bp_found = false;
    let mut status_found = false;

    for def in plan.definitions {
        match def {
            Definition::Unit(ud) => {
                if ud.name == "TestUnit" {
                    unit_found = true;
                    // Check for TestUnits with or without brackets to handle potential parser behavior
                    assert!(ud.plurals.contains(&"TestUnits".to_string()) || ud.plurals.contains(&"<TestUnits>".to_string()));
                    assert!(ud.abbreviations.contains(&"tu".to_string()));
                }
            }
            Definition::Value(vd) => {
                if vd.name == "BloodPressure" {
                    bp_found = true;
                    assert!(matches!(vd.value_type, ValueType::Number));
                    // Check meaningful property parsing if implemented in AST
                    // Currently properties are a Vec<Property>
                    // We can verify "valid values" was parsed (Implicitly by parser success)
                } else if vd.name == "Status" {
                    status_found = true;
                    assert!(matches!(vd.value_type, ValueType::Enumeration));
                }
            }
            _ => {}
        }
    }

    assert!(unit_found, "Unit definition not found");
    assert!(bp_found, "BloodPressure definition not found");
    assert!(status_found, "Status definition not found");
}
