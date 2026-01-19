use hippocrates_engine::parser;
use hippocrates_engine::runtime::Engine;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: hippocrates_engine <file.hipp>");
        std::process::exit(1);
    }

    let filename = &args[1];
    let content = fs::read_to_string(filename).expect("Failed to read file");

    println!("Parsing plan from {}...", filename);
    match parser::parse_plan(&content) {
        Ok(plan) => {
            println!("Plan parsed successfully!");
            println!("Definitions: {}", plan.definitions.len());
            if let Err(errors) = hippocrates_engine::runtime::validator::validate_file(&plan) {
                for e in errors {
                    eprintln!("Validation Error: {}", e);
                }
                std::process::exit(1);
            }
            println!("Plan validated successfully.");

            let mut engine = Engine::new();
            if args.contains(&"--sim".to_string()) {
                println!("Running in Simulation Mode (5 days, timelapse)");
                engine.set_mode(
                    hippocrates_engine::runtime::executor::ExecutionMode::Simulation {
                        speed_factor: None, 
                        duration: None,
                    },
                );
            }

            engine.load_plan(plan.clone());

            // Register output handler to stream updates to console
            engine
                .env
                .set_output_handler(std::sync::Arc::new(|msg: String| {
                    println!("{}", msg);
                }));

            if let Some(plan_name) = plan.definitions.iter().find_map(|d| match d {
                hippocrates_engine::ast::Definition::Plan(p) => Some(p.name.clone()),
                _ => None,
            }) {
                println!("Executing plan '{}'...", plan_name);
                engine.set_value(
                    "empty bottles",
                    hippocrates_engine::domain::RuntimeValue::Number(0.0),
                );
                engine.execute(&plan_name);
                println!("Execution finished.");
            } else {
                println!("No plan definition found to execute.");
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}
