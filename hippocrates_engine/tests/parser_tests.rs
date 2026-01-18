use hippocrates_engine::parser;
use hippocrates_engine::domain::{ValueType, Unit};
use hippocrates_engine::ast::{Plan, Statement, Assignment, Expression, Literal, Action};
use hippocrates_engine::parser::{HippocratesParser, Rule};
use pest::Parser;

#[test]
fn test_treating_copd_parsing() {
    let input = std::fs::read_to_string("plans/treating_copd.hipp").expect("Failed to read file");
    let plan = parser::parse_plan(&input).expect("Failed to parse treating_copd.hipp");
    assert!(!plan.definitions.is_empty());
}

#[test]
fn test_simple_plan_parsing() {
    let script = "\"body weight\" is a number:
    valid values: 0 ... 200
    unit: kg

patient_age is a number.

treatment is a plan:
    during plan:
        \"body weight\" = 75 kg.
        show message \"Patient weight recorded\".";


    let plan = parser::parse_plan(script).expect("Failed to parse plan");

    assert_eq!(plan.definitions.len(), 3);
    // ... Check details if needed, but parsing success is main goal now
}

#[test]
fn test_expression_parsing() {
    let inputs = vec![
        "5 days ago",
        "10 minutes from now",
        "75 kg",
        "body weight",
        "now"
    ];
    for input in inputs {
        let mut pairs = HippocratesParser::parse(Rule::expression, input)
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", input, e));
        let pair = pairs.next().unwrap();
        println!("Parsed '{}' as {:?}", input, pair.as_rule());
    }
}

#[test]
fn test_constraint_parsing() {
    let input = "timeframe is between 5 days ago ... now!";
    let mut pairs = HippocratesParser::parse(Rule::constraint, input)
        .unwrap_or_else(|e| panic!("Failed to parse constraint '{}': {}", input, e));
    let pair = pairs.next().unwrap();
    println!("Parsed constraint as {:?}", pair.as_rule());
}

#[test]
fn test_range_selector_parsing() {
    let input = "between 5 days ago ... now";
    let mut pairs = HippocratesParser::parse(Rule::range_selector, input)
        .unwrap_or_else(|e| panic!("Failed to parse range '{}': {}", input, e));
    let pair = pairs.next().unwrap();
    println!("Parsed range as {:?}", pair.as_rule());
}

#[test]
fn test_expression_excludes_keyword_is() {
    let input = "is";
    let pair = hippocrates_engine::parser::HippocratesParser::parse(hippocrates_engine::parser::Rule::expression, input);
    assert!(pair.is_err(), "Expression should NOT match 'is'");
}
#[test]
fn test_99_bottles_parsing() {
    let input = std::fs::read_to_string("plans/99_bottles.hipp").expect("Failed to read 99_bottles.hipp");
    let plan = parser::parse_plan(&input).expect("Failed to parse 99_bottles.hipp");
    // Verify structure
    assert!(!plan.definitions.is_empty());
}
