use crate::ast::Expression;
use crate::ast::Literal;

#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    pub fn new(min: f64, max: f64) -> Self {
        Interval {
            min: min.max(0.0), // Enforce non-negative
            max: max.max(0.0),
        }
    }

    pub fn exact(val: f64) -> Self {
        Self::new(val, val)
    }

    pub fn unbounded() -> Self {
        Self::new(0.0, f64::INFINITY)
    }

    pub fn is_subset_of(&self, other: &Interval) -> bool {
        self.min >= other.min && self.max <= other.max
    }
}

pub fn calculate_interval(
    expr: &Expression,
    defined_ranges: &std::collections::HashMap<String, Interval>, // Assuming pre-calculated ranges for variables
) -> Interval {
    match expr {
        Expression::Literal(lit) => match lit {
            Literal::Number(n, _) => Interval::exact(*n),
            Literal::Quantity(n, _, _) => Interval::exact(*n),
            _ => Interval::new(0.0, 0.0) // Non-numeric literals don't participate in arithmetic
        },
        Expression::Variable(name) => {
            defined_ranges.get(name).cloned().unwrap_or(Interval::unbounded())
        },
        Expression::Binary(left, op, right) => {
            let lhs = calculate_interval(left, defined_ranges);
            let rhs = calculate_interval(right, defined_ranges);
            
            match op.as_str() {
                "+" => Interval::new(lhs.min + rhs.min, lhs.max + rhs.max),
                "-" => {
                    // Result cannot be negative.
                    // New Min = max(0, min_left - max_right)
                    // New Max = max(0, max_left - min_right)
                    Interval::new(
                        (lhs.min - rhs.max).max(0.0),
                        (lhs.max - rhs.min).max(0.0)
                    )
                },
                "*" => Interval::new(lhs.min * rhs.min, lhs.max * rhs.max),
                "/" => {
                    if rhs.min <= 0.0 {
                        // Division by roughly zero or range including zero -> Unbounded
                        Interval::unbounded()
                    } else {
                        Interval::new(lhs.min / rhs.max, lhs.max / rhs.min)
                    }
                },
                _ => Interval::unbounded()
            }
        },
        _ => Interval::unbounded(),
    }
}

// Helper to detect potential negative results in subtraction
pub fn check_subtraction_safety(
    left: &Expression, 
    right: &Expression, 
    defined_ranges: &std::collections::HashMap<String, Interval>
) -> Option<String> {
    let lhs = calculate_interval(left, defined_ranges);
    let rhs = calculate_interval(right, defined_ranges);
    
    // Safety check:
    // If it's possible for lhs < rhs, then subtraction is unsafe.
    // i.e. if lhs.min < rhs.max
    // However, if we assume 0-clamping logic is valid behavior, this is fine.
    // The requirement "support no negative numbers" usually means "crash/error if negative".
    // If the logical result is negative, clamped to 0 is often mathematically wrong for the domain.
    // We will warn if potential for negative exists.
    
    if lhs.min < rhs.max {
        return Some(format!(
            "Potential negative result: {} ({:.1}..{:.1}) - {} ({:.1}..{:.1}) could be negative.",
            "Left", lhs.min, lhs.max, "Right", rhs.min, rhs.max
        ));
    }
    None
}
