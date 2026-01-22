//! Non-spec integration/regression tests (ignored by default).

#[path = "fixture_loader.rs"]
mod fixture_loader;

#[path = "integration/addressee_definition.rs"]
mod addressee_definition;
#[path = "integration/analysis_context.rs"]
mod analysis_context;
#[path = "integration/copd_execution.rs"]
mod copd_execution;
#[path = "integration/full_simulation.rs"]
mod full_simulation;
#[path = "integration/golden_master.rs"]
mod golden_master;
#[path = "integration/implicit_asking.rs"]
mod implicit_asking;
#[path = "integration/inspect_ast.rs"]
mod inspect_ast;
#[path = "integration/interactive.rs"]
mod interactive;
#[path = "integration/manual_parsing.rs"]
mod manual_parsing;
#[path = "integration/parallel.rs"]
mod parallel;
#[path = "integration/parser_error.rs"]
mod parser_error;
#[path = "integration/reproduce_hang.rs"]
mod reproduce_hang;
#[path = "integration/runtime.rs"]
mod runtime;
#[path = "integration/scheduler_duplication.rs"]
mod scheduler_duplication;
#[path = "integration/scheduler.rs"]
mod scheduler;
#[path = "integration/simulation.rs"]
mod simulation;
#[path = "integration/units_regression.rs"]
mod units_regression;
