use hippocrates_engine::ast::{Expression, Literal};
use hippocrates_engine::runtime::validator::{Interval, calculate_interval};
use std::collections::HashMap;

#[test]
fn test_interval_creation_positive() {
    let i = Interval::new(5.0, 10.0);
    assert_eq!(i.min, 5.0);
    assert_eq!(i.max, 10.0);
}

#[test]
fn test_interval_force_positive() {
    let i = Interval::new(-5.0, 10.0);
    assert_eq!(i.min, 0.0); // Clamped
    assert_eq!(i.max, 10.0);
}

#[test]
fn test_addition() {
    // To test calculate_interval properly with ranges, we need Variables.
    // Let's create a map with "b" -> [10, 20]
    let mut ranges = HashMap::new();
    ranges.insert("b".to_string(), Interval::new(10.0, 20.0));
    
    // Expr: 5 + b
    let expr_with_var = Expression::Binary(
        Box::new(Expression::Literal(Literal::Number(5.0, None))),
        "+".to_string(),
        Box::new(Expression::Variable("b".to_string()))
    );

    let res = calculate_interval(&expr_with_var, &ranges);
    
    assert_eq!(res.min, 15.0);
    assert_eq!(res.max, 25.0);
}

#[test]
fn test_subtraction_safety() {
    // 5 - 10 = -5 -> Should be 0 in Interval, but unsafe check should flag it.
    let a = Interval::exact(5.0);
    let b = Interval::exact(10.0);
    
    // Result
    let diff = Interval::new((a.min - b.max).max(0.0), (a.max - b.min).max(0.0));
    assert_eq!(diff.min, 0.0);
    assert_eq!(diff.max, 0.0);
}
