// Spec §3.6, §3.7: statements, actions, and question modifiers.

use crate::fixture_loader::{load_scenario, ScenarioKind};
use hippocrates_engine::ast::{Action, PlanBlock, QuestionConfig, StatementKind};
use hippocrates_engine::parser;

// REQ-3.6-01: timeframe blocks parse with nested statements.
#[test]
fn spec_timeframe_block_parsing() {
    let input = load_scenario("tests/fixtures/specs.hipp", "timeframe", ScenarioKind::Pass);

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

    let defs = plan.definitions;
    assert_eq!(defs.len(), 1);

    if let hippocrates_engine::ast::Definition::Plan(plan_def) = &defs[0] {
        assert_eq!(plan_def.name, "TimeframePlan");

        let block = &plan_def.blocks[0];
        if let PlanBlock::DuringPlan(stmts) = block {
            if let StatementKind::Timeframe(block) = &stmts[0].kind {
                assert!(block.for_analysis);
            }

            match &stmts[1].kind {
                StatementKind::Timeframe(block) => {
                    let nested_stmts = &block.block;
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

// REQ-3.7-01: question configuration parses and validates references.
#[test]
fn spec_question_config_parsing_and_validation() {
    let input = load_scenario("tests/fixtures/specs.hipp", "question_config", ScenarioKind::Fail);

    let plan = parser::parse_plan(&input).expect("Failed to parse plan");

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

    let result = hippocrates_engine::runtime::validator::validate_file(&plan);
    assert!(result.is_err(), "Validation passed but should fail due to <UnknownVar>");

    let errors = result.err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("UnknownVar")), "Errors should contain 'UnknownVar': {:?}", errors);
}

// REQ-3.7-02: message expiration attaches to show message.
#[test]
fn spec_message_expiration_parsing() {
    let input = r#"
<plan> is a plan:
    during plan:
        show message "Take your medication now":
            message expires after 15 minutes.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");

    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let hippocrates_engine::ast::Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let show_stmt = during
        .iter()
        .find(|stmt| matches!(stmt.kind, StatementKind::Action(Action::ShowMessage(_, _))))
        .expect("Expected show message statement to parse");

    let mut found = false;
    if let StatementKind::Action(Action::ShowMessage(_, Some(block))) = &show_stmt.kind {
        found = block
            .iter()
            .any(|stmt| matches!(stmt.kind, StatementKind::Action(Action::MessageExpiration(_))));
    }

    assert!(found, "Expected message expiration to be attached to show message");
}

// REQ-3.7-08: say is accepted as a message action keyword.
#[test]
fn spec_say_message_parsing() {
    let input = r#"
<plan> is a plan:
    during plan:
        say "Hello.".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let hippocrates_engine::ast::Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    assert!(
        during.iter().any(|stmt| matches!(stmt.kind, StatementKind::Action(Action::ShowMessage(_, _)))),
        "Expected say statement to parse as ShowMessage"
    );
}

// REQ-3.7-03: question modifiers parse (validate/type/style/expire).
#[test]
fn spec_question_modifiers_parsing() {
    let input = r#"
<temp> is a number:
    valid values:
        0 kg ... 10 kg.
    question:
        ask "Temp?".

<plan> is a plan:
    during plan:
        ask <temp>:
            validate answer twice.
            type of question is "numeric".
            style of question is <Likert>.
            question expires after 1 day.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let hippocrates_engine::ast::Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let ask_stmt = during
        .iter()
        .find(|s| matches!(s.kind, StatementKind::Action(Action::AskQuestion(_, _))))
        .expect("Ask statement not found");

    let mut saw_validate = false;
    let mut saw_type = false;
    let mut saw_style = false;
    let mut saw_expires = false;

    if let StatementKind::Action(Action::AskQuestion(_, Some(block))) = &ask_stmt.kind {
        for stmt in block {
            match &stmt.kind {
                StatementKind::Action(Action::ValidateAnswer(mode, timeout)) => {
                    saw_validate = true;
                    assert!(matches!(mode, hippocrates_engine::domain::ValidationMode::Twice));
                    assert!(timeout.is_none());
                }
                StatementKind::Action(Action::Configure(QuestionConfig::Type(t))) => {
                    saw_type = t == "numeric";
                }
                StatementKind::Action(Action::Configure(QuestionConfig::Style(s))) => {
                    saw_style = s == "Likert";
                }
                StatementKind::Action(Action::Configure(QuestionConfig::Generic(text))) => {
                    if text.contains("question expires after") {
                        saw_expires = true;
                    }
                }
                _ => {}
            }
        }
    }

    assert!(saw_validate, "Expected validate answer modifier");
    assert!(saw_type, "Expected question type modifier");
    assert!(saw_style, "Expected question style modifier");
    assert!(saw_expires, "Expected question expires modifier");
}

// REQ-3.7-06: question expiration blocks parse and include reminder statements.
#[test]
fn spec_question_expiration_block_parsing() {
    let input = r#"
<temp> is a number:
    valid values:
        0 kg ... 10 kg.
    question:
        ask "Temp?".

<plan> is a plan:
    during plan:
        ask <temp>:
            question expires after 1 day:
                show message "Reminder".
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let hippocrates_engine::ast::Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let ask_stmt = during
        .iter()
        .find(|s| matches!(s.kind, StatementKind::Action(Action::AskQuestion(_, _))))
        .expect("Ask statement not found");

    let mut saw_expire = false;
    let mut saw_reminder = false;

    if let StatementKind::Action(Action::AskQuestion(_, Some(block))) = &ask_stmt.kind {
        for stmt in block {
            match &stmt.kind {
                StatementKind::Action(Action::Configure(QuestionConfig::Generic(text))) => {
                    if text.contains("question expires after") {
                        saw_expire = true;
                    }
                }
                StatementKind::Action(Action::ShowMessage(_, _)) => {
                    saw_reminder = true;
                }
                _ => {}
            }
        }
    }

    assert!(saw_expire, "Expected question expiration modifier");
    assert!(saw_reminder, "Expected reminder message inside expiration block");
}

// REQ-3.7-07: question expiration supports until event triggers.
#[test]
fn spec_question_expiration_until_event_trigger_parsing() {
    let input = r#"
<temp> is a number:
    valid values:
        0 kg ... 10 kg.
    question:
        ask "Temp?".

<plan> is a plan:
    during plan:
        ask <temp>:
            question expires after until every Monday.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let hippocrates_engine::ast::Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let ask_stmt = during
        .iter()
        .find(|s| matches!(s.kind, StatementKind::Action(Action::AskQuestion(_, _))))
        .expect("Ask statement not found");

    let mut saw_until = false;
    if let StatementKind::Action(Action::AskQuestion(_, Some(block))) = &ask_stmt.kind {
        for stmt in block {
            if let StatementKind::Action(Action::Configure(QuestionConfig::Generic(text))) = &stmt.kind {
                if text.contains("until every Monday") {
                    saw_until = true;
                }
            }
        }
    }

    assert!(saw_until, "Expected question expiration to include 'until every Monday'");
}

// REQ-3.7-04: validate answer within parsing attaches to ask blocks.
#[test]
fn spec_validate_answer_within_parsing() {
    let input = r#"
<temp> is a number:
    valid values:
        0 kg ... 10 kg.
    question:
        ask "Temp?".

<plan> is a plan:
    during plan:
        ask <temp>:
            validate answer twice within 5 minutes.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let hippocrates_engine::ast::Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let ask_stmt = during
        .iter()
        .find(|s| matches!(s.kind, StatementKind::Action(Action::AskQuestion(_, _))))
        .expect("Ask statement not found");

    if let StatementKind::Action(Action::AskQuestion(_, Some(block))) = &ask_stmt.kind {
        let has_validate = block.iter().any(|stmt| matches!(stmt.kind, StatementKind::Action(Action::ValidateAnswer(_, _))));
        assert!(has_validate, "Expected validate answer modifier");
    }
}

// REQ-3.7-05: listen/send/start/simple command actions parse.
#[test]
fn spec_listen_send_start_and_simple_command_parsing() {
    let input = r#"
<plan> is a plan:
    during plan:
        listen for <signal>:
            show message "Listening".
        send information "Info" <signal>.
        start <plan>.
        <do something>.
"#;

    let plan = parser::parse_plan(input).expect("Failed to parse");
    let plan_def = plan
        .definitions
        .iter()
        .find_map(|d| if let hippocrates_engine::ast::Definition::Plan(p) = d { Some(p) } else { None })
        .expect("Plan definition not found");

    let during = match &plan_def.blocks[0] {
        PlanBlock::DuringPlan(stmts) => stmts,
        _ => panic!("Expected DuringPlan"),
    };

    let mut saw_listen = false;
    let mut saw_send = false;
    let mut saw_start = false;
    let mut saw_simple = false;

    for stmt in during {
        match &stmt.kind {
            StatementKind::Action(Action::ListenFor(name)) => {
                if name == "signal" {
                    saw_listen = true;
                }
                if name == "do something" {
                    saw_simple = true;
                }
            }
            StatementKind::Action(Action::SendInfo(_, _)) => {
                saw_send = true;
            }
            StatementKind::Action(Action::StartPeriod) => {
                saw_start = true;
            }
            _ => {}
        }
    }

    assert!(saw_listen, "Expected listen for action");
    assert!(saw_send, "Expected send information action");
    assert!(saw_start, "Expected start period action");
    assert!(saw_simple, "Expected simple command action");
}

// REQ-3.6-02: timeframe selectors require a start and end; single time indications are invalid.
#[test]
fn spec_timeframe_requires_range_selector() {
    let input = r#"
<plan> is a plan:
    during plan:
        timeframe for analysis is now:
            show message "Hi".
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_err(), "Expected parser error for single time indication timeframe");
}
