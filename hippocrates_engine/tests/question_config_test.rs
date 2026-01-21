
#[test]
fn test_question_config_parsing_and_validation() {
    use hippocrates_engine::parser;
    use hippocrates_engine::runtime::validator;
    use hippocrates_engine::ast::{StatementKind, PlanBlock, Action, QuestionConfig};
    use std::fs;

    let input = fs::read_to_string("tests/plans/question_config_coverage.hipp")
        .expect("Failed to read plan file");

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    // 1. Verify AST parsing of VAS
    let mut vas_found = false;
    for def in &plan.definitions {
        if let hippocrates_engine::ast::Definition::Plan(pd) = def {
            for block in &pd.blocks {
                if let PlanBlock::DuringPlan(stmts) = block {
                    for stmt in stmts {
                        if let StatementKind::Action(Action::Configure(config)) = &stmt.kind {
                            if let QuestionConfig::VisualAnalogScale(vas) = config {
                                vas_found = true;
                                assert_eq!(vas.best_value, 0.0);
                                assert_eq!(vas.worst_value, 10.0);
                                assert_eq!(vas.best_label.as_deref(), Some("No Pain"));
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(vas_found, "Failed to parse VisualAnalogScale configuration");

    // 2. Verify Validation of Show Message (UnknownVar)
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Validation passed but should fail due to <UnknownVar>");
    
    let errors = result.err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("UnknownVar")), "Errors should contain 'UnknownVar': {:?}", errors);
}
