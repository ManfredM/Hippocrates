// Spec §3.3, §3.9, §3.10, §3.5: Full fixture smoke coverage.

use crate::fixture_loader::{load_scenario, ScenarioKind};
use hippocrates_engine::ast::{Definition, Property, StatementKind};
use hippocrates_engine::parser;

// REQ-3.3-01: multi-definition fixtures parse core definitions.
#[test]
fn spec_full_fixture_parses_core_definitions() {
    let input = load_scenario("tests/fixtures/specs.hipp", "full_spec", ScenarioKind::Pass);

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    // Verify Definitions
    assert_eq!(plan.definitions.len(), 10);

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
        if let hippocrates_engine::ast::PlanBlock::BeforePlan(stmts) = block {
            // Check for conditional
            let cond_opts = stmts
                .iter()
                .find(|s| matches!(s.kind, StatementKind::Conditional(_)));
            assert!(cond_opts.is_some(), "Conditional found");

            if let Some(s) = cond_opts {
                if let StatementKind::Conditional(c) = &s.kind {
                    assert_eq!(c.cases.len(), 2);
                }
            }
        } else {
            panic!("Expected BeforePlan block");
        }

        // Verify we have other blocks
        assert_eq!(p.blocks.len(), 3);
    } else {
        panic!("Plan definition not found");
    }
}
