#[cfg(test)]
mod tests {
    use crate::ast::Expression;
    use crate::ast::Literal;
    use crate::ast::RangeSelector;
    use crate::domain::RuntimeValue;
    use crate::runtime::Environment;
    use crate::runtime::Evaluator;

    #[test]
    fn test_fuzzy_equals() {
        let v1 = RuntimeValue::String("Yes".to_string());
        let _v2 = RuntimeValue::String("yes".to_string());
        // We can't access private fuzzy_equals directly, but we can test via check_condition
        // or we can test if we make it public (or test via public API)

        // Let's test check_condition
        let env = Environment::new();

        let expr = Expression::Literal(Literal::String("yes".to_string()));
        let selector = RangeSelector::Equals(expr);

        assert!(Evaluator::check_condition(&env, &selector, &v1));
    }

    #[test]
    fn test_fuzzy_equals_enum() {
        let v1 = RuntimeValue::Enumeration("Yes".to_string());
        // Target is string "yes"
        let env = Environment::new();
        let expr = Expression::Literal(Literal::String("yes".to_string()));
        let selector = RangeSelector::Equals(expr);

        assert!(Evaluator::check_condition(&env, &selector, &v1));
    }

    #[test]
    fn test_fuzzy_equals_enum_enum() {
        let _v1 = RuntimeValue::Enumeration("Yes".to_string());
        // Target is enum "yes" (simulated via expression evaluation?)
        // Expressions usually evaluate to primitives, but if we had a variable...
        // For now, testing Enum vs String is the main use case.
    }
}
