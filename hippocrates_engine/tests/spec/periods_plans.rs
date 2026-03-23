// Spec §3.5, §3.8: periods, plans, and event triggers.

use crate::fixture_loader::{load_scenario, ScenarioKind};
use hippocrates_engine::ast::{Definition, PlanBlock, RangeSelector, Trigger};
use hippocrates_engine::parser;

// REQ-3.5-01: period definitions parse by name.
#[test]
fn spec_period_definition_parsing() {
    let input = load_scenario("tests/fixtures/specs.hipp", "period_definition", ScenarioKind::Pass);

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

// REQ-3.5-02: period timeframe lines parse with range selectors.
#[test]
fn spec_period_parsing_structure() {
    let input = r#"
<best inhalation period> is a period:
    timeframe:
        between Monday ... Friday; 07:40 ... 07:50.
        between Saturday ... Sunday; 09:00 ... 09:10.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse period definition");

    assert_eq!(plan.definitions.len(), 1);

    match &plan.definitions[0] {
        Definition::Period(period) => {
            assert_eq!(period.name, "best inhalation period");
            assert_eq!(period.timeframes.len(), 2);

            let first_line = &period.timeframes[0];
            assert_eq!(first_line.len(), 2);
            match &first_line[0] {
                RangeSelector::Range(_, _) | RangeSelector::Between(_, _) => {}
                _ => panic!("Expected range/between selector for days"),
            }
            match &first_line[1] {
                RangeSelector::Range(_, _) | RangeSelector::Between(_, _) => {}
                _ => panic!("Expected range/between selector for time"),
            }

            let second_line = &period.timeframes[1];
            assert_eq!(second_line.len(), 2);

            assert_eq!(period.line, 2);
        }
        _ => panic!("Expected Period definition"),
    }
}

// REQ-3.8-01: event triggers parse for change/start/periodic.
#[test]
fn spec_event_trigger_parsing() {
    let input = load_scenario("tests/fixtures/specs.hipp", "event_trigger", ScenarioKind::Pass);

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    let defs = plan.definitions;
    assert_eq!(defs.len(), 1);

    if let Definition::Plan(plan_def) = &defs[0] {
        assert_eq!(plan_def.name, "TriggerPlan");

        if let PlanBlock::Trigger(tb) = &plan_def.blocks[0] {
            if let Trigger::ChangeOf(var) = &tb.trigger {
                assert_eq!(var, "Status");
            } else {
                panic!("Expected ChangeOf");
            }
        }

        if let PlanBlock::Trigger(tb) = &plan_def.blocks[1] {
            if let Trigger::StartOf(var) = &tb.trigger {
                assert_eq!(var, "Treatment");
            } else {
                panic!("Expected StartOf");
            }
        }

        if let PlanBlock::Trigger(tb) = &plan_def.blocks[2] {
            if let Trigger::Periodic { interval, .. } = &tb.trigger {
                assert_eq!(*interval, 1.0);
            } else {
                panic!("Expected Periodic");
            }
        }

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

// REQ-3.8-04: periodic triggers parse duration and offsets.
#[test]
fn spec_event_trigger_duration_and_offset_parsing() {
    let input = r#"
<treatment period> is a period:
    timeframe:
        08:00 ... 09:00.

<Plan> is a plan:
    every 2 days for 1 week:
        information "A".
    every Monday after <treatment period> for 2 weeks:
        information "B".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");

    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let mut saw_duration = false;
    let mut saw_offset = false;

    for block in &plan_def.blocks {
        if let PlanBlock::Trigger(tb) = block {
            match &tb.trigger {
                Trigger::Periodic {
                    interval,
                    interval_unit,
                    duration,
                    offset,
                    specific_day,
                    ..
                } => {
                    if *interval == 2.0 && matches!(interval_unit, hippocrates_engine::domain::Unit::Day) {
                        assert_eq!(duration.as_ref().map(|(v, _)| *v), Some(1.0));
                        assert!(duration.as_ref().map(|(_, u)| matches!(u, hippocrates_engine::domain::Unit::Week)).unwrap_or(false));
                        assert!(offset.is_none());
                        assert!(specific_day.is_none());
                        saw_duration = true;
                    } else if specific_day.as_deref() == Some("Monday") {
                        assert_eq!(*interval, 1.0);
                        assert!(matches!(interval_unit, hippocrates_engine::domain::Unit::Week));
                        assert_eq!(duration.as_ref().map(|(v, _)| *v), Some(2.0));
                        assert!(duration.as_ref().map(|(_, u)| matches!(u, hippocrates_engine::domain::Unit::Week)).unwrap_or(false));
                        assert_eq!(offset.as_deref(), Some("treatment period"));
                        saw_offset = true;
                    }
                }
                _ => {}
            }
        }
    }

    assert!(saw_duration, "Expected duration-based periodic trigger");
    assert!(saw_offset, "Expected weekday trigger with offset and duration");
}

// REQ-3.8-02: event blocks attach statements to triggers.
#[test]
fn spec_event_block_parsing() {
    let input = r#"
<my period> is a period:
    timeframe:
        08:00 ... 09:00.

<My Plan> is a plan:
    <my event> with begin of <my period>:
        information "Event".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");

    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let event_block = plan_def
        .blocks
        .iter()
        .find_map(|b| if let PlanBlock::Event(e) = b { Some(e) } else { None })
        .expect("Event block not found");

    assert_eq!(event_block.name, "my event");
}

// UT-PERIODS-06: periodic trigger with `at <time>` parses time_of_day.
#[test]
fn spec_event_trigger_time_of_day_parsing() {
    let input = r#"
<Plan> is a plan:
    every 1 day at 08:00 for 9 days:
        information "Morning check".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");

    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    assert_eq!(plan_def.blocks.len(), 1);

    if let PlanBlock::Trigger(tb) = &plan_def.blocks[0] {
        if let Trigger::Periodic {
            interval,
            interval_unit,
            duration,
            time_of_day,
            ..
        } = &tb.trigger
        {
            assert_eq!(*interval, 1.0);
            assert!(matches!(interval_unit, hippocrates_engine::domain::Unit::Day));
            assert_eq!(duration.as_ref().map(|(v, _)| *v), Some(9.0));
            assert_eq!(time_of_day.as_deref(), Some("08:00"));
        } else {
            panic!("Expected Periodic trigger");
        }
    } else {
        panic!("Expected Trigger block");
    }
}

// UT-PERIODS-07: weekday periodic trigger with `at <time>` parses time_of_day.
#[test]
fn spec_event_trigger_weekday_with_time_parsing() {
    let input = r#"
<Plan> is a plan:
    every Monday at 09:30 for 4 weeks:
        information "Weekly check".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan");

    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    assert_eq!(plan_def.blocks.len(), 1);

    if let PlanBlock::Trigger(tb) = &plan_def.blocks[0] {
        if let Trigger::Periodic {
            interval,
            interval_unit,
            duration,
            specific_day,
            time_of_day,
            ..
        } = &tb.trigger
        {
            assert_eq!(*interval, 1.0);
            assert!(matches!(interval_unit, hippocrates_engine::domain::Unit::Week));
            assert_eq!(specific_day.as_deref(), Some("Monday"));
            assert_eq!(duration.as_ref().map(|(v, _)| *v), Some(4.0));
            assert!(
                duration
                    .as_ref()
                    .map(|(_, u)| matches!(u, hippocrates_engine::domain::Unit::Week))
                    .unwrap_or(false)
            );
            assert_eq!(time_of_day.as_deref(), Some("09:30"));
        } else {
            panic!("Expected Periodic trigger");
        }
    } else {
        panic!("Expected Trigger block");
    }
}

// UT-PLAN-01: `after plan:` block parses into PlanBlock::AfterPlan.
#[test]
fn spec_after_plan_block_parsing() {
    let input = r#"
<patient> is an addressee:
    contact information:
        email is "patient@test.com".

<my plan> is a plan:
    during plan:
        information to <patient> "Plan started.".

    after plan:
        information to <patient> "Plan completed.".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse plan with after plan block");

    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    assert_eq!(plan_def.blocks.len(), 2, "Expected 2 plan blocks (during + after)");

    assert!(
        matches!(&plan_def.blocks[0], PlanBlock::DuringPlan(_)),
        "First block should be DuringPlan"
    );
    assert!(
        matches!(&plan_def.blocks[1], PlanBlock::AfterPlan(_)),
        "Second block should be AfterPlan"
    );

    if let PlanBlock::AfterPlan(stmts) = &plan_def.blocks[1] {
        assert!(!stmts.is_empty(), "AfterPlan block should have statements");
    }
}
