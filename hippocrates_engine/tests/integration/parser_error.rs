#[test]
#[ignore = "Non-spec integration/regression"]
fn test_error_formatting() {
    // Input that is mostly valid but has a syntax error
    // "before plan" expects "begin of" or similar inside.
    // Let's just give garbage.
    let input = "not a valid plan";
    
    // parse_plan is public but might need imports
    let result = hippocrates_engine::parser::parse_plan(input);
    
    match result {
        Ok(_) => panic!("Should have failed"),
        Err(e) => {
            println!("Error message: {}", e.message);
            
            // Verify it's not the verbose pest output
            // The verbose output usually contains the line content and caret
            assert!(!e.message.contains("\n  |")); 
            assert!(!e.message.contains("^---"));
            
            // Verify it contains expected information
            assert!(e.message.contains("Expected"));
        }
    }
}
