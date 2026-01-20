use std::env;
use std::fs;
use std::process;
use hippocrates_engine::parser::parse_plan;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: hippocrates-cli <file_path>");
        process::exit(1);
    }

    let file_path = &args[1];
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            process::exit(1);
        }
    };

    match parse_plan(&content) {
        Ok(plan) => {
            match hippocrates_engine::runtime::validator::validate_file(&plan) {
                Ok(_) => {
                    println!("{{ \"status\": \"valid\" }}");
                    process::exit(0);
                }
                Err(errors) => {
                    // e is Vec<EngineError>, use serde_json to serialize it
                    // We might want to wrap it or just output the list.
                    // The original CLI error output was a single object.
                    // If errors.len() == 1, maybe output single object?
                    // User said "Make sure the cli detects the same errors as we see in the editor."
                    // Editor might show multiple errors.
                    // Let's output the array if multiple, or object if single?
                    // Or usually strict tools output list.
                    // But for consistency with parser error (Process::exit(1) on first error?),
                    // parser `Err(e)` returns single error.
                    // Let's just output the list. Parser error can be wrapped in list too for consistency?
                    // "Now use the cli to check and correct".
                    // Let's just output the list of errors.
                    match serde_json::to_string_pretty(&errors) {
                        Ok(json) => println!("{}", json),
                        Err(err) => eprintln!("Error serializing errors: {}", err),
                    }
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            // e is EngineError
            match serde_json::to_string_pretty(&e) {
                Ok(json) => println!("{}", json),
                Err(err) => eprintln!("Error serializing error: {}", err),
            }
            process::exit(1);
        }
    }
}
