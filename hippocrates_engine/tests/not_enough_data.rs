use hippocrates_engine::ast::{
    Expression, StatisticalFunc, RangeSelector, RelativeDirection, ConditionOperator
};
use hippocrates_engine::domain::{RuntimeValue, ValueInstance, Unit};
use hippocrates_engine::runtime::evaluator::Evaluator;
use hippocrates_engine::runtime::environment::{Environment, EvaluationContext};
use chrono::{Utc, Duration, TimeZone};

#[test]
fn test_not_enough_data() {
    let mut env = Environment::new();
    let start_time = Utc::now();
    env.set_start_time(start_time);
    env.set_time(start_time + Duration::days(2)); // System running for 2 days

    // Add some data
    env.set_value("incident", RuntimeValue::Boolean(true));

    // 1. Defined Timeframe: "Last 5 days" (5 days ago ... now)
    // History (2 days) < Timeframe (5 days) -> Should be NotEnoughData
    let five_days_ago = Expression::RelativeTime(5.0, Unit::Day, RelativeDirection::Ago);
    let now_expr = Expression::Variable("now".to_string());
    
    // Manually push context as if running a rule
    let ctx = EvaluationContext {
        timeframe: Some(RangeSelector::Range(five_days_ago, now_expr)),
    };
    env.push_context(ctx);

    let count_expr = Expression::Statistical(StatisticalFunc::CountOf(
        "incident".to_string(),
        None,
    ));

    let res = Evaluator::evaluate(&env, &count_expr);
    env.pop_context();

    // Verify NotEnoughData
    if let RuntimeValue::NotEnoughData = res {
        // Success
    } else {
        panic!("Expected NotEnoughData, got {:?}", res);
    }

    // 2. Advance time so system has been running for 6 days
    env.set_time(start_time + Duration::days(6));
    
    // Add another data point
    env.set_value("incident", RuntimeValue::Boolean(true));

    // Same calculation: "Last 5 days"
    // History (6 days) > Timeframe (5 days) -> Should be Number
    let five_days_ago = Expression::RelativeTime(5.0, Unit::Day, RelativeDirection::Ago);
    let now_expr = Expression::Variable("now".to_string());

    let ctx = EvaluationContext {
        timeframe: Some(RangeSelector::Range(five_days_ago, now_expr)),
    };
    env.push_context(ctx);
    let res_valid = Evaluator::evaluate(&env, &count_expr);
    env.pop_context();

    if let RuntimeValue::Number(n) = res_valid {
        // We expect at least one incident (the one we just added)
        // possibly 2 if the old one is still in range (depends on exact seconds)
        assert!(n >= 1.0, "Expected valid count >= 1.0, got {}", n);
    } else {
        panic!("Expected Number, got {:?}", res_valid);
    }
}
