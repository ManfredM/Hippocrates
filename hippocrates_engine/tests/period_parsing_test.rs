use hippocrates_engine::parser::parse_plan;
use hippocrates_engine::ast::{Definition, RangeSelector};

#[test]
fn test_period_parsing() {
    let input = r#"
<best inhalation period> is a period:
    timeframe:
        between Monday ... Friday; 07:40 ... 07:50
        between Saturday ... Sunday; 09:00 ... 09:10
"#;

    let plan = parse_plan(input).expect("Failed to parse period definition");
    
    assert_eq!(plan.definitions.len(), 1);
    
    match &plan.definitions[0] {
        Definition::Period(period) => {
            assert_eq!(period.name, "best inhalation period");
            assert_eq!(period.timeframes.len(), 2);
            
            // First timeframe line: Mon-Fri; 07:40-07:50
            let first_line = &period.timeframes[0];
            assert_eq!(first_line.len(), 2);
            // We can inspect the RangeSelectors if we want deeper verification,
            // but for now checking structure is good.
            match &first_line[0] {
                 RangeSelector::Range(_, _) | RangeSelector::Between(_, _) => {}, // Good
                 _ => panic!("Expected range/between selector for days"),
            }
            match &first_line[1] {
                 RangeSelector::Range(_, _) | RangeSelector::Between(_, _) => {}, // Good
                 _ => panic!("Expected range/between selector for time"),
            }

            // Second timeframe line
            let second_line = &period.timeframes[1];
            assert_eq!(second_line.len(), 2);
            
            // Check line number (pest lines are 1-based)
            assert_eq!(period.line, 2);
        }
        _ => panic!("Expected Period definition"),
    }
}
