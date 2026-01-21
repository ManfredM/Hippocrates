
use hippocrates_engine::parser;

#[test]
fn test_parse_treating_copd() {
    let input = std::fs::read_to_string("plans/treating_copd.hipp").expect("Failed to read treating_copd.hipp");
    match parser::parse_plan(&input) {
        Ok(_) => println!("treating_copd.hipp parsed successfully"),
        Err(e) => panic!("Failed to parse treating_copd.hipp: {:?}", e),
    }
}

#[test]
fn test_parse_99_bottles_v2() {
    let input = std::fs::read_to_string("plans/99_bottles_v2.hipp").expect("Failed to read 99_bottles_v2.hipp");
    match parser::parse_plan(&input) {
        Ok(_) => println!("99_bottles_v2.hipp parsed successfully"),
        Err(e) => panic!("Failed to parse 99_bottles_v2.hipp: {:?}", e),
    }
}
