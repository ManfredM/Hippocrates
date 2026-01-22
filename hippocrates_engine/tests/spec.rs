//! Specification-linked test suite for Hippocrates Engine.

#[path = "fixture_loader.rs"]
mod fixture_loader;

#[path = "spec/actors_drugs.rs"]
mod actors_drugs;
#[path = "spec/contexts_expressions.rs"]
mod contexts_expressions;
#[path = "spec/execution.rs"]
mod execution;
#[path = "spec/fixtures.rs"]
mod fixtures;
#[path = "spec/grammar.rs"]
mod grammar;
#[path = "spec/periods_plans.rs"]
mod periods_plans;
#[path = "spec/statements_actions.rs"]
mod statements_actions;
#[path = "spec/units.rs"]
mod units;
#[path = "spec/validation.rs"]
mod validation;
#[path = "spec/values.rs"]
mod values;
