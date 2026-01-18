use hippocrates_engine::parser;
use hippocrates_engine::ast::{Definition, Property, ConditionalTarget, Statement, StatementKind};
use std::fs;

#[test]
fn test_full_spec_features() {
    let input = fs::read_to_string("tests/full_spec_test.hipp")
        .expect("Failed to read test file");
    
    let plan = parser::parse_plan(&input).expect("Failed to parse plan");
    
    // Verify Definitions
    assert_eq!(plan.definitions.len(), 4); // Drug, Addressee, Number, Plan
    
    // Check Drug
    if let Some(Definition::Drug(drug)) = plan.definitions.iter().find(|d| matches!(d, Definition::Drug(_))) {
        assert_eq!(drug.name, "Painkiller");
        // Check properties
        assert!(drug.properties.iter().any(|p| matches!(p, Property::Ingredients(_))));
        assert!(drug.properties.iter().any(|p| matches!(p, Property::DosageSafety(_))));
        assert!(drug.properties.iter().any(|p| matches!(p, Property::Administration(_))));
    } else {
        panic!("Drug definition not found");
    }

    // Check Addressee
    if let Some(Definition::Addressee(addr)) = plan.definitions.iter().find(|d| matches!(d, Definition::Addressee(_))) {
        assert_eq!(addr.name, "DrSmith");
        // Check properties
        assert!(addr.properties.iter().any(|p| matches!(p, Property::ContactInfo(_))));
    } else {
        panic!("Addressee definition not found");
    }

    // Check Plan Logic
    if let Some(Definition::Plan(p)) = plan.definitions.iter().find(|d| matches!(d, Definition::Plan(_))) {
        assert_eq!(p.name, "AnalysisPlan");
        
        let block = &p.blocks[0]; // During plan
        if let hippocrates_engine::ast::PlanBlock::DuringPlan(stmts) = block {
             // Check Context
             let ctx_opts = stmts.iter().find(|s| matches!(s.kind, StatementKind::ContextBlock(_)));
             
             assert!(ctx_opts.is_some(), "Context block found");
             
             if let Some(s) = ctx_opts {
                 if let StatementKind::ContextBlock(cb) = &s.kind {
                     assert_eq!(cb.items.len(), 1); 
                     println!("Debug: Found ContextBlock with {} items", cb.items.len());
                 }
             }
             
             // Check for conditional
             let cond_opts = stmts.iter().find(|s| matches!(s.kind, StatementKind::Conditional(_)));
             assert!(cond_opts.is_some(), "Conditional found");
             
             if let Some(s) = cond_opts {
                 if let StatementKind::Conditional(c) = &s.kind {
                     assert_eq!(c.cases.len(), 2);
                     println!("Debug: Found Conditional with {} cases", c.cases.len());
                 }
             }
             // Check Confidence logic
             let conf_stmt = stmts.iter().find(|s| {
                 if let Statement { kind: StatementKind::Conditional(c), .. } = s {
                     matches!(c.condition, ConditionalTarget::Confidence(_))
                 } else { false }
             });
             assert!(conf_stmt.is_some(), "Confidence assessment missing");

             // Check Statistical logic (assess count of)
             // This is Conditional(Target::Expression(Expression::Statistical(..)))
             let stat_stmt = stmts.iter().find(|s| {
                 if let Statement { kind: StatementKind::Conditional(c), .. } = s {
                      if let ConditionalTarget::Expression(expr) = &c.condition {
                          matches!(expr, hippocrates_engine::ast::Expression::Statistical(_))
                      } else { false }
                 } else { false }
             });
             assert!(stat_stmt.is_some(), "Statistical assessment missing");
             
        } else {
            panic!("Expected DuringPlan block");
        }
    } else {
        panic!("Plan definition not found");
    }
}
