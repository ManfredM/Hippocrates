
#[test]
fn test_timeframe_parsing() {
    use hippocrates_engine::parser;
    use hippocrates_engine::ast::{Definition, StatementKind, PlanBlock};
    use std::fs;

    let input = fs::read_to_string("tests/plans/timeframe_coverage.hipp")
        .expect("Failed to read plan file");
    
    let plan = parser::parse_plan(&input).expect("Failed to parse plan");
    
    let defs = plan.definitions;
    assert_eq!(defs.len(), 1);
    
    if let Definition::Plan(plan_def) = &defs[0] {
        assert_eq!(plan_def.name, "TimeframePlan");
        
        let block = &plan_def.blocks[0];
        if let PlanBlock::DuringPlan(stmts) = block {
             // 1. timeframe for analysis
             if let StatementKind::Timeframe(block) = &stmts[0].kind {
                 assert!(block.for_analysis);
                 // 2. timeframe during ...
                 // Actually the first statement is "timeframe for analysis", second is "timeframe during ..."
             }
             
             // Check nesting
             // The second top-level stmt in 'during plan' is 'timeframe during ...'
             match &stmts[1].kind {
                 StatementKind::Timeframe(block) => {
                     // check constraint? 
                     // It has a nested timeframe
                     let nested_stmts = &block.block;
                     // 1. show message
                     // 2. timeframe after ...
                     assert!(matches!(nested_stmts[1].kind, StatementKind::Timeframe(_)));
                 }
                 _ => panic!("Expected Timeframe block"),
             }
        } else {
            panic!("Expected DuringPlan block");
        }
    } else {
        panic!("Expected Plan definition");
    }
}
