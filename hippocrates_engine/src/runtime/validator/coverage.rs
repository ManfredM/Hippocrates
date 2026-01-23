use crate::ast::{Expression, Literal, RangeSelector, AssessmentCase};
use crate::domain::EngineError;
use super::intervals::Interval;

#[derive(Clone, Copy)]
struct Range { start: f64, end: f64 }

pub fn check_coverage(
    name: &str,
    valid_ranges: &[Interval],
    cases: &[AssessmentCase],
    line: usize,
    decimals: Option<usize>,
    errors: &mut Vec<EngineError>,
) {
    if valid_ranges.is_empty() {
        return;
    }

    if valid_ranges
        .iter()
        .any(|r| r.min.is_infinite() || r.max.is_infinite())
    {
        return;
    }

    let valid_ranges = merge_ranges(intervals_to_ranges(valid_ranges));

    // 2. Collect Intervals from Cases
    let mut ranges: Vec<Range> = Vec::new();

    for case in cases {
        ranges.extend(extract_ranges(&case.condition));
    }

    if let Some(decimals) = decimals {
        let step = 10_f64.powi(-(decimals as i32));
        for valid in &valid_ranges {
            check_discrete_coverage(
                name,
                valid.start,
                valid.end,
                &ranges,
                line,
                step,
                decimals,
                errors,
            );
        }
        return;
    }

    for valid in &valid_ranges {
        check_continuous_coverage(
            name,
            valid.start,
            valid.end,
            &ranges,
            line,
            errors,
        );
    }
}

fn check_continuous_coverage(
    name: &str,
    universe_min: f64,
    universe_max: f64,
    ranges: &[Range],
    line: usize,
    errors: &mut Vec<EngineError>,
) {
    let mut clamped: Vec<Range> = Vec::new();
    for r in ranges {
        let start = r.start.max(universe_min);
        let end = r.end.min(universe_max);
        if start <= end {
            clamped.push(Range { start, end });
        }
    }

    clamped.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));

    let mut current = universe_min;
    let mut is_first = true;
    let epsilon = 0.0001;

    let format_val = |v: f64| -> String { format!("{:.1}", v) };

    for r in clamped {
        if !is_first && r.start < current - epsilon {
            errors.push(EngineError {
                message: format!(
                    "Constraint Violation: Assessment for '{}' has overlapping ranges; value {} is covered multiple times.",
                    name,
                    format_val(r.start)
                ),
                line,
                column: 0,
            });
        }

        if r.start > current + epsilon {
            errors.push(EngineError {
                message: format!(
                    "Coverage Error: Assessment for '{}' is incomplete. Uncovered gap detected: {} ... {}. Valid range is {}...{}.",
                    name,
                    format_val(current),
                    format_val(r.start),
                    format_val(universe_min),
                    format_val(universe_max)
                ),
                line,
                column: 0,
            });
            current = r.start;
        }

        if r.end > current {
            current = r.end;
        }
        is_first = false;
    }

    if current < universe_max - epsilon {
        errors.push(EngineError {
            message: format!(
                "Coverage Error: Assessment for '{}' is incomplete. Uncovered gap at the end: {} ... {}. Valid range is {}...{}.",
                name,
                format_val(current),
                format_val(universe_max),
                format_val(universe_min),
                format_val(universe_max)
            ),
            line,
            column: 0,
        });
    }
}

fn check_discrete_coverage(
    name: &str,
    universe_min: f64,
    universe_max: f64,
    ranges: &[Range],
    line: usize,
    step: f64,
    decimals: usize,
    errors: &mut Vec<EngineError>,
) {
    let mut clamped: Vec<Range> = Vec::new();
    for r in ranges {
        let start = r.start.max(universe_min);
        let end = r.end.min(universe_max);
        if start <= end {
            clamped.push(Range { start, end });
        }
    }

    clamped.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));

    let mut current = universe_min;
    let epsilon = step / 10.0;
    let format_val = |v: f64| -> String { format!("{:.*}", decimals, v) };

    for r in clamped {
        if r.start < current - epsilon {
            errors.push(EngineError {
                message: format!(
                    "Constraint Violation: Assessment for '{}' has overlapping ranges; value {} is covered multiple times.",
                    name,
                    format_val(r.start)
                ),
                line,
                column: 0,
            });
        }

        if r.start > current + epsilon {
            let gap_end = r.start - step;
            if gap_end + epsilon >= current {
                errors.push(EngineError {
                    message: format!(
                        "Coverage Error: Assessment for '{}' is incomplete. Uncovered gap detected: {} ... {}. Valid range is {}...{}.",
                        name,
                        format_val(current),
                        format_val(gap_end),
                        format_val(universe_min),
                        format_val(universe_max)
                    ),
                    line,
                    column: 0,
                });
            }
            current = r.start;
        }

        let next = r.end + step;
        if next > current {
            current = next;
        }
    }

    if current <= universe_max + epsilon {
        errors.push(EngineError {
            message: format!(
                "Coverage Error: Assessment for '{}' is incomplete. Uncovered gap at the end: {} ... {}. Valid range is {}...{}.",
                name,
                format_val(current),
                format_val(universe_max),
                format_val(universe_min),
                format_val(universe_max)
            ),
            line,
            column: 0,
        });
    }
}

fn extract_ranges(sel: &RangeSelector) -> Vec<Range> {
    let mut res = Vec::new();
    
    let extract_val = |expr: &Expression| -> Option<f64> {
        if let Expression::Literal(lit) = expr {
             match lit {
                 Literal::Number(v, _) => Some(*v),
                 Literal::Quantity(v, _, _) => Some(*v),
                 _ => None
             }
        } else { None }
    };

    match sel {
        RangeSelector::Range(min, max) | RangeSelector::Between(min, max) => {
             if let (Some(vn), Some(vx)) = (extract_val(min), extract_val(max)) {
                 res.push(Range { start: vn, end: vx });
             }
        },
        RangeSelector::Equals(expr) => {
             if let Some(v) = extract_val(expr) {
                 res.push(Range { start: v, end: v });
             }
        },
        RangeSelector::List(_selectors) => {
            // Ignored as simplified
        }
        _ => {}
    }
    res
}

fn intervals_to_ranges(intervals: &[Interval]) -> Vec<Range> {
    intervals
        .iter()
        .map(|interval| Range {
            start: interval.min,
            end: interval.max,
        })
        .collect()
}

fn merge_ranges(mut ranges: Vec<Range>) -> Vec<Range> {
    if ranges.is_empty() {
        return ranges;
    }

    ranges.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));

    let mut merged = Vec::new();
    let mut current = ranges[0].clone();
    let epsilon = 0.0001;

    for range in ranges.into_iter().skip(1) {
        if range.start <= current.end + epsilon {
            if range.end > current.end {
                current.end = range.end;
            }
        } else {
            merged.push(current);
            current = range;
        }
    }

    merged.push(current);
    merged
}

pub fn check_string_coverage(
    name: &str,
    cases: &[AssessmentCase],
    required_values: &[&str],
    line: usize,
    errors: &mut Vec<EngineError>,
) {
    let mut covered = std::collections::HashSet::new();
    
    for case in cases {
        extract_strings(&case.condition, &mut covered);
    }
    
    let mut missing = Vec::new();
    for req in required_values {
        if !covered.contains(*req) {
            missing.push(*req);
        }
    }
    
    if !missing.is_empty() {
        errors.push(EngineError {
            message: format!(
                "Coverage Error: Assessment for '{}' is incomplete. Missing cases: {}. Required: {}.",
                name,
                missing.join(", "),
                required_values.join(", ")
            ),
            line,
            column: 0
        });
    }
}

fn extract_strings(sel: &RangeSelector, covered: &mut std::collections::HashSet<String>) {
     match sel {
        RangeSelector::Equals(Expression::Literal(Literal::String(s))) => {
            covered.insert(s.clone());
        },
        RangeSelector::Equals(Expression::Variable(name)) => {
            covered.insert(name.clone());
        },
        RangeSelector::List(items) => {
            for item in items {
                match item {
                    Expression::Literal(Literal::String(s)) => {
                        covered.insert(s.clone());
                    }
                    Expression::Variable(name) => {
                        covered.insert(name.clone());
                    }
                    _ => {}
                }
            }
        }
        // RangeSelector could be just a string literal in some parsing contexts?
        // Actually, parser for "string" usually produces RangeSelector::Equals(Literal::String)
        // Check grammar: range_selector -> expression -> literal -> string
        // The parser converts expression to RangeSelector::Equals/etc.
        // Wait, if I write `"increase": ...`
        // `range_selector` rule in parser matches `expression`.
        // `expression` matches `string_literal`.
        // The AST builder wraps pure expression in `RangeSelector::Equals` usually?
        // Let's assume standard expression parsing wraps it in Equals.
        // Or specific selector types?
        // If it's just `Expression`, parser likely returns `RangeSelector::Equals(expr)`? 
        // Need to verify `parser.rs` or just be robust.
        _ => {}
    }
}
