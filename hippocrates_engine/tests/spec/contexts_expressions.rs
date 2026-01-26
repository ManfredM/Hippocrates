// Spec §3.11, §3.12: contexts and expressions.

use hippocrates_engine::ast::{ContextItem, Definition, Expression, Literal, PlanBlock, RelativeDirection, StatementKind, StatisticalFunc};
use hippocrates_engine::domain::Unit;
use hippocrates_engine::parser;
use hippocrates_engine::runtime::{Environment, Executor};
use hippocrates_engine::domain::RuntimeValue;
use chrono::{Utc, Duration};

// REQ-3.11-01: context definitions parse timeframe/data/value filter items.
#[test]
fn spec_context_definition_parsing() {
    let input = r#"
context:
    timeframe: 1 day ago ... now.
    data: <pain>.
    value filter: 0 kg ... 1 kg:
        information "ok".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let ctx = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Context(c) = d { Some(c) } else { None })
        .expect("Context definition not found");

    assert!(ctx.items.iter().any(|i| matches!(i, ContextItem::Timeframe(_))));
    assert!(ctx.items.iter().any(|i| matches!(i, ContextItem::Data(name) if name == "pain")));
    assert!(ctx.items.iter().any(|i| matches!(i, ContextItem::ValueFilter(_))));
}

// REQ-3.11-02: context blocks parse data/value filters and nested statements.
#[test]
fn spec_context_block_items_parsing() {
    let input = r#"
<pain> is a number:
    valid values:
        0 kg ... 10 kg.

<plan> is a plan:
    during plan:
        context for analysis:
            data: <pain>.
            value filter: 0 kg ... 10 kg:
                information "ok".
            information "done".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let ctx_stmt = during
        .iter()
        .find(|s| matches!(s.kind, StatementKind::ContextBlock(_)))
        .expect("Context block not found");

    if let StatementKind::ContextBlock(cb) = &ctx_stmt.kind {
        assert!(cb.items.iter().any(|i| matches!(i, ContextItem::Data(name) if name == "pain")));
        assert!(cb.items.iter().any(|i| matches!(i, ContextItem::ValueFilter(_))));
        assert!(cb.statements.iter().any(|s| matches!(s.kind, StatementKind::Action(_))));
    }
}

// REQ-3.12-01: statistical function expressions parse in assignments.
#[test]
fn spec_statistical_functions_parsing() {
    let input = r#"
<val> is a number:
    valid values:
        0 kg ... 10 kg.

<plan> is a plan:
    during plan:
        <min val> = min of <val>.
        <max val> = max of <val>.
        <avg val> = average of <val> over 5 days.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let mut saw_min = false;
    let mut saw_max = false;
    let mut saw_avg = false;

    for stmt in during {
        if let StatementKind::Assignment(assign) = &stmt.kind {
            match &assign.expression {
                Expression::Statistical(StatisticalFunc::MinOf(name)) => {
                    saw_min = name == "val";
                }
                Expression::Statistical(StatisticalFunc::MaxOf(name)) => {
                    saw_max = name == "val";
                }
                Expression::Statistical(StatisticalFunc::AverageOf(name, _)) => {
                    saw_avg = name == "val";
                }
                _ => {}
            }
        }
    }

    assert!(saw_min, "Expected min of <val>");
    assert!(saw_max, "Expected max of <val>");
    assert!(saw_avg, "Expected average of <val>");
}

// REQ-3.12-07: meaning-of expressions parse in assignments.
#[test]
fn spec_meaning_of_expression_parsing() {
    let input = r#"
<weight> is a number:
    unit is kg.
    valid values:
        0 kg ... 100 kg.

<label> is a string.

<plan> is a plan:
    during plan:
        <label> = meaning of <weight>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let assignment = during
        .iter()
        .find_map(|stmt| if let StatementKind::Assignment(assign) = &stmt.kind { Some(assign) } else { None })
        .expect("Assignment not found");

    match &assignment.expression {
        Expression::MeaningOf(name) => assert_eq!(name, "weight"),
        _ => panic!("Expected MeaningOf expression"),
    }
}

// REQ-3.1-01: time indications parse for now, weekday, and time-of-day.
#[test]
fn spec_time_indications_parsing() {
    let input = r#"
<when> is a time indication.

<plan> is a plan:
    during plan:
        <when> = now.
        <when> = Monday.
        <when> = 08:30.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let mut saw_now = false;
    let mut saw_weekday = false;
    let mut saw_time = false;

    for stmt in during {
        if let StatementKind::Assignment(assign) = &stmt.kind {
            match &assign.expression {
                Expression::Variable(name) if name == "now" => saw_now = true,
                Expression::Literal(Literal::String(s)) if s == "Monday" => saw_weekday = true,
                Expression::Literal(Literal::TimeOfDay(s)) if s == "08:30" => saw_time = true,
                Expression::Literal(Literal::String(s)) if s == "08:30" => saw_time = true,
                _ => {}
            }
        }
    }

    assert!(saw_now, "Expected now time indication");
    assert!(saw_weekday, "Expected weekday time indication");
    assert!(saw_time, "Expected time literal indication");
}

// REQ-3.1-04: date/time literals parse for date and date-time forms.
#[test]
fn spec_date_time_literals_parsing() {
    let input = r#"
<when> is a date/time.

<plan> is a plan:
    during plan:
        <when> = 2026-01-18.
        <when> = 2026-01-18 13:45.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let mut saw_date = false;
    let mut saw_datetime = false;

    for stmt in during {
        if let StatementKind::Assignment(assign) = &stmt.kind {
            match &assign.expression {
                Expression::Literal(Literal::Date(s)) if s == "2026-01-18" => saw_date = true,
                Expression::Literal(Literal::Date(s)) if s == "2026-01-18 13:45" => saw_datetime = true,
                _ => {}
            }
        }
    }

    assert!(saw_date, "Expected date literal to parse");
    assert!(saw_datetime, "Expected date-time literal to parse");
}

// REQ-3.12-06: date diff expressions parse.
#[test]
fn spec_date_diff_parsing() {
    let input = r#"
<delta> is a number:
    valid values:
        0 days ... 365 days.

<plan> is a plan:
    during plan:
        <delta> = days between 2026-01-01 and 2026-01-11.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let assign = during
        .iter()
        .find_map(|s| if let StatementKind::Assignment(a) = &s.kind { Some(a) } else { None })
        .expect("Assignment not found");

    match &assign.expression {
        Expression::DateDiff(unit, _, _) => assert!(matches!(unit, Unit::Day)),
        _ => panic!("Expected DateDiff expression"),
    }
}

// REQ-3.1-02: relative time expressions from now parse.
#[test]
fn spec_relative_time_from_now_parsing() {
    let input = r#"
<when> is a time indication.

<plan> is a plan:
    during plan:
        <when> = 2 days from now.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let assign = during
        .iter()
        .find_map(|s| if let StatementKind::Assignment(a) = &s.kind { Some(a) } else { None })
        .expect("Assignment not found");

    match &assign.expression {
        Expression::RelativeTime(val, unit, dir) => {
            assert_eq!(*val, 2.0);
            assert!(matches!(unit, Unit::Day));
            assert!(matches!(dir, RelativeDirection::FromNow));
        }
        _ => panic!("Expected RelativeTime expression"),
    }
}

// REQ-3.11-03: context for analysis executes with scoped timeframe.
#[test]
fn spec_context_for_analysis_execution() {
    let code = r#"
<pain level> is a number.

<TestPlan> is a plan:
    during plan:
        context for analysis:
            timeframe: 2 days ago ... now.
            assess trend of <pain level>:
                "increase":
                    <log> = "Increasing".
                "stable":
                    <log> = "Stable".
"#;

    let mut env = Environment::new();
    let mut executor = Executor::new(std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)));

    let plan = parser::parse_plan(code).expect("Failed to parse plan");
    env.load_plan(plan);

    let now = Utc::now().naive_utc();
    env.set_time(now);
    env.set_start_time(now - Duration::days(100));

    env.set_value_at("pain level", RuntimeValue::Number(10.0), now - Duration::days(5));
    env.set_value_at("pain level", RuntimeValue::Number(8.0), now - Duration::days(3));
    env.set_value_at("pain level", RuntimeValue::Number(2.0), now - Duration::days(1));
    env.set_value_at("pain level", RuntimeValue::Number(4.0), now);

    executor.execute_plan(&mut env, "TestPlan");

    let res = env.get_value("log").expect("Result should be set");
    assert_eq!(res, &RuntimeValue::String("Increasing".to_string()));
}
