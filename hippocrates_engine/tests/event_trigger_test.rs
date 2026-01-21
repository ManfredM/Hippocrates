mod fixture_loader;

#[test]
fn test_event_trigger_parsing() {
    use fixture_loader::{load_scenario, ScenarioKind};
    use hippocrates_engine::parser;
    use hippocrates_engine::ast::{Definition, PlanBlock, Trigger};

    let input = load_scenario("tests/fixtures/specs.hipp", "event_trigger", ScenarioKind::Pass);
    
    let plan = parser::parse_plan(&input).expect("Failed to parse plan");
    
    let defs = plan.definitions;
    assert_eq!(defs.len(), 1);
    
    if let Definition::Plan(plan_def) = &defs[0] {
        assert_eq!(plan_def.name, "TriggerPlan");
        
        // Block 1: change of <Status>
        if let PlanBlock::Trigger(tb) = &plan_def.blocks[0] {
            if let Trigger::ChangeOf(var) = &tb.trigger {
                 assert_eq!(var, "Status");
            } else {
                panic!("Expected ChangeOf");
            }
        }
        
        // Block 2: begin of <Treatment>
        if let PlanBlock::Trigger(tb) = &plan_def.blocks[1] {
            if let Trigger::StartOf(var) = &tb.trigger {
                 assert_eq!(var, "Treatment");
            } else {
                panic!("Expected StartOf");
            }
        }
        
        // Block 3: every 1 <week> (no brackets for week, just "1 week" quantity)
        if let PlanBlock::Trigger(tb) = &plan_def.blocks[2] {
            if let Trigger::Periodic { interval, .. } = &tb.trigger {
                assert_eq!(*interval, 1.0);
            } else {
                panic!("Expected Periodic");
            }
        }
        
        // Block 4: every Monday after <Start> for 1 year
        if let PlanBlock::Trigger(tb) = &plan_def.blocks[3] {
            if let Trigger::Periodic { interval, specific_day, offset, .. } = &tb.trigger {
                 assert_eq!(*interval, 1.0);
                 assert_eq!(specific_day.as_deref(), Some("Monday"));
                 assert_eq!(offset.as_deref(), Some("Start"));
            } else {
                panic!("Expected Periodic for Monday");
            }
        }
    }
}
