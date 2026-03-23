// Spec §3.2, §4.1: units, pluralization, and canonicalization.

use hippocrates_engine::domain::{RuntimeValue, Unit};
use hippocrates_engine::parser;
use hippocrates_engine::runtime::executor::Executor;
use hippocrates_engine::runtime::environment::Environment;
use hippocrates_engine::runtime::validator;

// REQ-3.2-01: custom unit pluralization canonicalizes values.
#[test]
fn spec_custom_unit_pluralization_is_canonical() {
    let input = r#"
<drop> is a unit:
    plural is <drops>.

<val> is a number:
    valid values:
        0 <drops> ... 100 <drops>.

<plan> is a plan:
    before plan:
        <val> = 1 <drop> + 1 <drops> + 1 <drop>.
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
            assert_eq!(*n, 3.0);
            assert_eq!(u.to_string(), "drop");
        }
        _ => panic!("Expected Quantity, got {:?}", val),
    }
}

// REQ-3.2-02: standard units work in calculations.
#[test]
fn spec_standard_units_still_work() {
    let input = r#"
<val> is a number:
    valid values:
        0 m ... 100 m.

<plan> is a plan:
    before plan:
        <val> = 5 m + 1 m.
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

// REQ-3.2-03: custom unit abbreviations canonicalize values.
#[test]
fn spec_custom_unit_abbreviation_is_canonical() {
    let input = r#"
<point> is a unit:
    plural is <points>.
    abbreviation is "pts".

<val> is a number:
    valid values:
        0 <pts> ... 100 <pts>.

<plan> is a plan:
    before plan:
        <val> = 5 <pts> + 5 <points>.
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

// REQ-4.1-01: built-in units cannot be redefined.
#[test]
fn spec_builtin_units_cannot_be_redefined() {
    let input = r#"
<kg> is a unit:
    plural is <kgs>.
"#;
    let plan = parser::parse_plan(input.trim()).expect("Failed to parse");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected validation error for built-in unit redefinition");
    let errors = result.err().unwrap();
    assert!(errors.iter().any(|e| e.message.contains("Built-in units cannot be redefined")));
}

// REQ-3.2-04: custom unit quantities parse with definitions.
#[test]
fn spec_custom_unit_quantity_parsing() {
    let input = r#"
<val> is a number:
    valid values:
        0 <points> ... 10 <points>.
"#;
    let result = parser::parse_plan(input);
    assert!(result.is_ok(), "Expected valid parse for custom unit quantity. Error: {:?}", result.err());
}

// REQ-4.1-02: unit conversions are supported within compatible groups.
#[test]
fn spec_unit_conversions_within_groups() {
    let tol = 1e-6;

    let kg_to_g = Unit::Kilogram.convert(1.0, &Unit::Gram).unwrap();
    assert!((kg_to_g - 1000.0).abs() < tol);

    let cm_to_m = Unit::Centimeter.convert(250.0, &Unit::Meter).unwrap();
    assert!((cm_to_m - 2.5).abs() < tol);

    let l_to_ml = Unit::Liter.convert(1.5, &Unit::Milliliter).unwrap();
    assert!((l_to_ml - 1500.0).abs() < tol);

    let c_to_f = Unit::Celsius.convert(0.0, &Unit::Fahrenheit).unwrap();
    assert!((c_to_f - 32.0).abs() < tol);

    let mmol_to_mg = Unit::MmolPerL.convert(1.0, &Unit::MgPerDl).unwrap();
    assert!((mmol_to_mg - 18.0182).abs() < tol);
}

// REQ-4.1-03: calculations and assignments require matching units and precision.
#[test]
fn spec_assignment_requires_unit_and_precision_match() {
    let precision_mismatch = r#"
<dose> is a number:
    valid values:
        0.0 mg ... 10.0 mg.

<plan> is a plan:
    before plan:
        <dose> = 5 mg.
"#;
    let plan = parser::parse_plan(precision_mismatch.trim()).expect("Failed to parse precision mismatch plan");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected precision validation error");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.message.contains("precision")),
        "Expected precision mismatch error, got {:?}",
        errors
    );

    let unit_mismatch = r#"
<dose> is a number:
    valid values:
        0 mg ... 10 mg.

<plan> is a plan:
    before plan:
        <dose> = 5 g.
"#;
    let plan = parser::parse_plan(unit_mismatch.trim()).expect("Failed to parse unit mismatch plan");
    let result = validator::validate_file(&plan);
    assert!(result.is_err(), "Expected unit validation error");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.message.contains("unit")),
        "Expected unit mismatch error, got {:?}",
        errors
    );
}
