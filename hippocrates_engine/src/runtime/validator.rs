use crate::ast::{Plan, Statement, StatementKind, Definition, Property, RangeSelector, Expression, Literal, ConditionalTarget};

pub fn validate_file(plan: &Plan) -> Result<(), String> {
    // 1. Collect Value Definitions and their Valid Ranges
    use std::collections::HashMap;
    let mut value_ranges: HashMap<String, Vec<RangeSelector>> = HashMap::new();

    for def in &plan.definitions {
        if let Definition::Value(vd) = def {
             for prop in &vd.properties {
                 if let Property::ValidValues(stmts) = prop {
                      // Parse statements to find ranges. 
                      // Typically ValidValues uses Statement::EventProgression (hack) or just storage.
                      // Based on parser hack: Statement::EventProgression("value", cases)
                      for s in stmts {
                          if let StatementKind::EventProgression(_, cases) = &s.kind {
                              let mut selectors = Vec::new();
                              for case in cases {
                                  selectors.push(case.condition.clone());
                              }
                              value_ranges.insert(vd.name.clone(), selectors);
                          }
                      }
                 }
             }
        }
    }

    // 2. Validate Statements in Plan Definition
    for def in &plan.definitions {
        if let Definition::Plan(pd) = def {
             for block in &pd.blocks {
                 let statements = match block {
                     crate::ast::PlanBlock::DuringPlan(s) => s,
                     crate::ast::PlanBlock::Event(e) => &e.statements,
                     crate::ast::PlanBlock::Trigger(t) => &t.statements,
                 };
                 
                 for stmt in statements {
                     validate_statement(stmt, &value_ranges)?;
                 }
             }
        }
    }
    
    Ok(())
}

fn validate_statement(stmt: &Statement, value_ranges: &std::collections::HashMap<String, Vec<RangeSelector>>) -> Result<(), String> {
    match &stmt.kind {
        StatementKind::Conditional(cond) => {
            validate_conditional(cond, value_ranges)?;
        }
        StatementKind::EventProgression(target, cases) => {
             // Similar validation for EventProgression
             // Reconstruct a dummy Conditional-like structure or validate directly
             // validate_coverage(target, cases, value_ranges)?;
             // For now, focus on Conditional as that's what user flagged
        }
        _ => {}
    }
    // Recursive check for blocks inside statements? (Not deeply implemented yet for brevity)
    // Actually, blocks in Conditional cases contain statements, so recursion is needed.
    // TODO: Recursion
    Ok(())
}

fn validate_conditional(cond: &crate::ast::Conditional, value_ranges: &std::collections::HashMap<String, Vec<RangeSelector>>) -> Result<(), String> {
    let target_name = match &cond.condition {
        ConditionalTarget::Expression(Expression::Variable(name)) => Some(name.clone()),
        ConditionalTarget::Expression(Expression::Literal(Literal::String(s))) => Some(s.clone()), // Handle "string" vars
        _ => None,
    };

    if let Some(name) = target_name {
        if let Some(valid_selectors) = value_ranges.get(&name) {
             // Check coverage
             check_coverage(&name, valid_selectors, &cond.cases)?;
        }
    }
    
    // Recurse into blocks?
    for case in &cond.cases {
        for s in &case.block {
            validate_statement(s, value_ranges)?;
        }
    }

    Ok(())
}

fn check_coverage(name: &str, valid: &[RangeSelector], cases: &[crate::ast::AssessmentCase]) -> Result<(), String> {
    // Simplified Logic: Integer Range Coverage [min, max]
    // 1. Identify the "Universe" from valid definitions.
    //    Assume simple case: Single Range(min, max).
    // 2. Collect covered intervals from cases.
    // 3. Verify Union(cases) covers Universe.
    
    // Determine Universe
    let mut universe_min = 0; // Default
    let mut universe_max = 0;
    let mut has_range = false;

    // Support single range for now
    for sel in valid {
        if let RangeSelector::Range(Expression::Literal(Literal::Number(min)), Expression::Literal(Literal::Number(max))) = sel {
             universe_min = *min as i64;
             universe_max = *max as i64;
             has_range = true;
             break; // Handle primary range
        }
        // Handle Between?
    }

    if !has_range {
        // Maybe it's an Enumeration? (List of Equals)
        // Check if all Enum options are covered.
        // TODO: Enumeration support
        return Ok(()); 
    }

    // Check Case Coverage for Range
    // Create a Set of covered integers? Or Interval arithmetic.
    // Since expected range is small (0..100), bitset or simple iteration is fine.
    // Let's use simple bitset/hashset of covered integers for robustness if ranges overlap.
    
    let mut covered = std::collections::HashSet::new();

    for case in cases {
        match &case.condition {
             RangeSelector::Range(Expression::Literal(Literal::Number(min)), Expression::Literal(Literal::Number(max))) => {
                 let start = *min as i64;
                 let end = *max as i64;
                 for i in start..=end {
                     covered.insert(i);
                 }
             }
             RangeSelector::Equals(Expression::Literal(Literal::Number(v))) => {
                 covered.insert(*v as i64);
             }
             // Handle others
             _ => {}
        }
    }

    // Verify Universe
    for i in universe_min..=universe_max {
        if !covered.contains(&i) {
            return Err(format!("Constraint Violation: Assessment for '{}' is missing coverage for value {}. Valid range is {}...{}.", name, i, universe_min, universe_max));
        }
    }

    Ok(())
}
