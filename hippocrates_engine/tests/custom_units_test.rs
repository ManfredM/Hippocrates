use hippocrates_engine::ast::{Plan, Definition, Property, StatementKind, RangeSelector, Expression, Literal};
use hippocrates_engine::domain::RuntimeValue;
use hippocrates_engine::parser;
use hippocrates_engine::runtime::executor::Executor;
use hippocrates_engine::runtime::environment::Environment;

#[test]
fn test_explicit_pluralization() {
    let input = r#"
drop is a unit:
    plural is "drops"

<val> is a number:
    valid values: 0 drops ... 100 drops

<plan> is a plan:
    during plan:
        val = 1 drop + 1 drops + 1 drop.
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    
    let mut env = Environment::new();
    env.load_plan(plan);

    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut exe = Executor::new(stop_signal);
    
    exe.execute_plan(&mut env, "plan");
    
    let val = env.get_value("val").expect("Val not set");
    match val {
        RuntimeValue::Quantity(n, u) => {
            // n is &f64 because we match on &RuntimeValue
            assert_eq!(*n, 3.0);
            assert_eq!(u.to_string(), "drop");
        }
        _ => panic!("Expected Quantity, got {:?}", val),
    }
}

#[test]
fn test_strict_units_without_definition() {
    let input = r#"
<val> is a number:
    valid values: 0 ... 100

<plan> is a plan:
    during plan:
        val = 5 coins + 1 coin.
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    
    let mut env = Environment::new();
    env.load_plan(plan);

    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut exe = Executor::new(stop_signal);
    
    exe.execute_plan(&mut env, "plan");
    
    let val = env.get_value("val").expect("Val not set");
    match val {
        RuntimeValue::Number(n) => {
             assert_eq!(*n, 6.0);
        }
        RuntimeValue::Quantity(_, u) => {
            panic!("Should have mismatched units! Got matching unit: {:?}", u);
        }
        _ => panic!("Unexpected result {:?}", val),
    }
}

#[test]
fn test_standard_units_still_work() {
    let input = r#"
<val> is a number:
    valid values: 0 ... 100

<plan> is a plan:
    during plan:
        val = 5 meters + 1 meter.
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    
    let mut env = Environment::new();
    env.load_plan(plan);

    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut exe = Executor::new(stop_signal);
    
    exe.execute_plan(&mut env, "plan");
    
    let val = env.get_value("val").expect("Val not set");
    match val {
        RuntimeValue::Quantity(n, u) => {
            assert_eq!(*n, 6.0);
            assert!(format!("{:?}", u).contains("Meter")); 
        }
        _ => panic!("Expected Quantity, got {:?}", val),
    }
}

#[test]
fn test_custom_unit_abbreviation() {
    let input = r#"
point is a unit:
    plural is "points"
    abbreviation is "pts"

<val> is a number:
    valid values: 0 pts ... 100 pts

<plan> is a plan:
    during plan:
        val = 5 pts + 5 points.
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    
    let mut env = Environment::new();
    env.load_plan(plan);

    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut exe = Executor::new(stop_signal);
    
    exe.execute_plan(&mut env, "plan");
    
    let val = env.get_value("val").expect("Val not set");
    match val {
        RuntimeValue::Quantity(n, u) => {
            assert_eq!(*n, 10.0);
            assert_eq!(u.to_string(), "point");
        }
        _ => panic!("Expected Quantity, got {:?}", val),
    }
}
