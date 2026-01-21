use hippocrates_engine::ast::{ConditionalTarget, Definition, Property, Statement, StatementKind};
use hippocrates_engine::parser;
use std::fs;

#[test]
fn test_full_spec_features() {
    let input = fs::read_to_string("tests/full_spec_test.hipp").expect("Failed to read test file");

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    // Verify Definitions
    assert_eq!(plan.definitions.len(), 12); 

    // Check Drug
    if let Some(Definition::Drug(drug)) = plan
        .definitions
        .iter()
        .find(|d| matches!(d, Definition::Drug(_)))
    {
        assert_eq!(drug.name, "Aspirin");
        // Check properties
        assert!(
            drug.properties
                .iter()
                .any(|p| matches!(p, Property::Ingredients(_)))
        );
        assert!(
            drug.properties
                .iter()
                .any(|p| matches!(p, Property::DosageSafety(_)))
        );
        assert!(
            drug.properties
                .iter()
                .any(|p| matches!(p, Property::Administration(_)))
        );
    } else {
        panic!("Drug definition not found");
    }

    // Check Addressee
    if let Some(Definition::Addressee(addr)) = plan
        .definitions
        .iter()
        .find(|d| matches!(d, Definition::Addressee(_)))
    {
        assert_eq!(addr.name, "Dr. House");
        // Check properties
        assert!(
            addr.properties
                .iter()
                .any(|p| matches!(p, Property::ContactInfo(_)))
        );
    } else {
        panic!("Addressee definition not found");
    }

    // Check Plan Logic
    if let Some(Definition::Plan(p)) = plan
        .definitions
        .iter()
        .find(|d| matches!(d, Definition::Plan(_)))
    {
        assert_eq!(p.name, "Treatment Plan");

        let block = &p.blocks[0]; // During plan
        if let hippocrates_engine::ast::PlanBlock::DuringPlan(stmts) = block {
            // Context block check removed as it's no longer in 'during plan' for this test case


            // Check for conditional
            let cond_opts = stmts
                .iter()
                .find(|s| matches!(s.kind, StatementKind::Conditional(_)));
            assert!(cond_opts.is_some(), "Conditional found");

            if let Some(s) = cond_opts {
                if let StatementKind::Conditional(c) = &s.kind {
                    assert_eq!(c.cases.len(), 2);
                    println!("Debug: Found Conditional with {} cases", c.cases.len());
                }
            }
            // Confidence and Statistical logic checks removed as they are not in 'during plan' anymore
        } else {
            panic!("Expected DuringPlan block");
        }
        
        // Verify we have other blocks
        assert_eq!(p.blocks.len(), 3); 
    } else {
        panic!("Plan definition not found");
    }
}
