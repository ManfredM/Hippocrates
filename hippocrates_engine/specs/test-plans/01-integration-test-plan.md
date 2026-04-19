# Integration Test Plan

| Field         | Value                              |
|---------------|------------------------------------|
| Document ID   | IT                                 |
| V-Model Level | Integration Testing (Right side)   |
| Verifies      | System Design (DES-*)              |

## 1. Purpose

Verify that components work together correctly, matching the system design. Integration tests focus on cross-component interactions: parser to validator, parser to environment loader, environment to executor, executor to callbacks, executor to evaluator, executor to scheduler, session to executor (multi-plan), and FFI JSON parsing logic. Each IT-* case corresponds to one or more DES-* design elements.

## 2. Test Environment

| Item           | Detail                                              |
|----------------|------------------------------------------------------|
| Language       | Rust                                                 |
| Test framework | `cargo test`                                         |
| Test location  | `hippocrates_engine/tests/integration/`              |
| Fixtures       | `hippocrates_engine/tests/fixtures/` (`.hipp` files) |
| Command        | `cargo test --test integration -- --ignored`         |

All integration tests are marked `#[ignore]` and require the `--ignored` flag to run.

## 3. Entry / Exit Criteria

**Entry criteria**
- Engine compiles successfully (`cargo build`).
- All unit tests pass (`cargo test`).

**Exit criteria**
- All IT-* test cases pass when running `cargo test --test integration -- --ignored`.

## 4. Test Cases

### 4.1 -- Parse + Validate

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures                       |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|--------------------------------|
| IT-HIPP-001  | `addressee_definition.rs::test_addressee_validation`                   | Parses an addressee definition from a fixture, then validates it; expects an invalid-email validation error. | SYS-HIPP-010, SYS-HIPP-012       | `specs.hipp` (addressee_definition / FAIL) |
| IT-HIPP-002  | `scenario_suite.rs::integration_plan_fixture_suite`                    | Iterates all PASS scenarios in a fixture file: parses, validates, and executes each plan; iterates all FAIL scenarios and asserts validation rejects them. | SYS-HIPP-010, SYS-HIPP-012, SYS-HIPP-013, SYS-HIPP-015 | `integration_plans.hipp`       |

### 4.2 -- Parse + Load (AST Inspection)

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures                       |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|--------------------------------|
| IT-HIPP-003  | `inspect_ast.rs::tests::inspect_copd_period_ast`                       | Parses a COPD plan from a fixture and iterates the AST, printing each definition to verify the parser produced the expected typed tree structure. | SYS-HIPP-010, SYS-HIPP-011       | `runtime_plans.hipp` (copd_plan / PASS) |
| IT-HIPP-004  | `manual_parsing.rs::test_parse_treating_copd`                          | Parses the COPD plan fixture and asserts parsing succeeds.                                    | SYS-HIPP-010               | `runtime_plans.hipp` (copd_plan / PASS) |
| IT-HIPP-005  | `manual_parsing.rs::test_parse_99_bottles_v2`                          | Parses the sing plan fixture and asserts parsing succeeds.                                    | SYS-HIPP-010               | `runtime_plans.hipp` (sing_plan / PASS) |

### 4.3 -- Parse + Error Reporting

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|----------|
| IT-HIPP-006  | `parser_error.rs::test_error_formatting`                               | Feeds invalid source text to the parser and verifies the error message is user-friendly (contains "Expected", no raw Pest diagnostic formatting). | SYS-HIPP-010               | --       |

### 4.4 -- Load + Execute (Runtime Pipeline)

| ID     | Test function                                                          | Description                                                                                   | Traces                    | Fixtures                       |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|-------------------------------|--------------------------------|
| IT-HIPP-007  | `copd_execution.rs::test_reproduce_copd_execution`                     | Parses a COPD plan, loads it into Engine, sets simulation mode with a 2-day duration, provides initial enum values, executes, and asserts specific clinical messages appear in the output log. | SYS-HIPP-010, SYS-HIPP-013, SYS-HIPP-015, SYS-HIPP-041, SYS-HIPP-042 | `runtime_plans.hipp` (copd_plan / PASS) |
| IT-HIPP-008  | `runtime.rs::test_copd_runtime_setup`                                  | Parses COPD plan, loads into Engine in simulation mode, sets an initial enumeration value, executes, and verifies the value history contains a "Plan started" log entry. | SYS-HIPP-010, SYS-HIPP-013, SYS-HIPP-015, SYS-HIPP-041 | `runtime_plans.hipp` (copd_plan / PASS) |
| IT-HIPP-009  | `runtime.rs::test_99_bottles_execution`                                | Parses the 99-bottles plan, loads into Environment, sets an initial quantity value, executes in simulation mode, and asserts lyric output appears in logs. | SYS-HIPP-010, SYS-HIPP-013, SYS-HIPP-015, SYS-HIPP-041 | `runtime_plans.hipp` (bottles_plan / PASS) |
| IT-HIPP-010  | `golden_master.rs::test_golden_master`                                 | Parses a golden-master plan from a fixture, loads into Environment with a fixed start time, executes, and asserts specific audit-log entries for assignments (number, string, enum) and messages (value interpolation, assessment branches). | SYS-HIPP-010, SYS-HIPP-013, SYS-HIPP-015, SYS-HIPP-016 | `specs.hipp` (golden_master / PASS) |
| IT-HIPP-011  | `analysis_context.rs::test_analysis_context_execution`                 | Builds two inline plans exercising assessment and context-for-analysis blocks. Parses, loads, executes statement blocks directly on the executor, and asserts the log callback receives the expected "Not enough data" message. | SYS-HIPP-010, SYS-HIPP-013, SYS-HIPP-015, SYS-HIPP-016 | --       |
| IT-HIPP-012  | `units_regression.rs::test_strict_units_without_definition`            | Parses a plan that assigns a value with undefined custom units, executes it, and asserts the resulting value is a plain number (unit mismatch falls back to unitless). | SYS-HIPP-010, SYS-HIPP-013, SYS-HIPP-015, SYS-HIPP-016 | --       |

### 4.5 -- Executor + Callbacks

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|----------|
| IT-HIPP-013  | `runtime.rs::test_execution_callback`                                  | Parses an inline plan with two information statements, creates an Executor with a line-number callback, executes, and asserts the callback was invoked with the expected line numbers. | SYS-HIPP-013, SYS-HIPP-031       | --       |

### 4.6 -- Executor + Evaluator (Derived Values)

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|----------|
| IT-HIPP-014  | `runtime.rs::test_current_value_in_calculation`                        | Parses a plan with a count-of statistical calculation, pushes an evaluation context with a timeframe, sets a value, and asserts the Evaluator returns the correct count. | SYS-HIPP-015, SYS-HIPP-016       | --       |
| IT-HIPP-015  | `runtime.rs::test_derived_calculation`                                 | Parses a plan with a derived (calculation) value definition, sets a source value, evaluates the derived variable by name, and asserts the Evaluator resolves the calculation chain correctly. | SYS-HIPP-015, SYS-HIPP-016       | --       |

### 4.7 -- Executor + Input Channel (Ask / Interactive)

| ID     | Test function                                                          | Description                                                                                   | Traces                    | Fixtures                       |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|-------------------------------|--------------------------------|
| IT-HIPP-016  | `implicit_asking.rs::test_implicit_asking_scenario`                    | Parses a plan with an implicit ask, sets up an ask callback that auto-replies via the mpsc input channel, executes, and asserts the question was asked and the correct assessment message was logged. | SYS-HIPP-013, SYS-HIPP-042, SYS-HIPP-031        | `runtime_plans.hipp` (implicit_ask / PASS) |
| IT-HIPP-017  | `interactive.rs::test_interactive_execution`                           | Parses a plan with two explicit ask statements (enum and numeric), spawns the executor in a separate thread, uses a Condvar to synchronize on each ask callback, sends answers via the mpsc channel, and verifies execution completes without panic. | SYS-HIPP-013, SYS-HIPP-042, SYS-HIPP-031        | --       |

### 4.8 -- Scheduler Integration

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|----------|
| IT-HIPP-018  | `scheduler.rs::test_scheduler_copd_logic`                             | Parses a period definition, manually loads definitions into a HashMap, and calls `Scheduler::next_occurrence` to verify the scheduler computes the correct next start time. | SYS-HIPP-010, SYS-HIPP-014       | --       |
| IT-HIPP-019  | `scheduler_duplication.rs::test_scheduler_duplication`                 | Parses a period with two timeframe groups (weekday and weekend), calls `next_occurrence` from a Monday morning, verifies the first occurrence is at the weekday time, then advances past it and asserts the next occurrence is the following day (not the weekend group on the same day). | SYS-HIPP-014               | --       |

### 4.9 -- Simulation Mode

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|----------|
| IT-HIPP-020  | `full_simulation.rs::test_period_simulation_progression`               | Parses a plan with a period trigger, sets the environment time to just before the period, creates an executor in instant-simulation mode, executes, and asserts the event fires and the log callback captures the expected message. | SYS-HIPP-013, SYS-HIPP-014, SYS-HIPP-041 | --       |
| IT-HIPP-021  | `simulation.rs::test_occurrences_simulation_logic`                     | Constructs a period definition programmatically, iterates `Scheduler::next_occurrence` in a loop over a 3-day window, and asserts exactly 3 daily occurrences are found at the expected times. | SYS-HIPP-014, SYS-HIPP-041       | --       |

### 4.10 -- Session (Multi-Plan Parallel Execution)

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|----------|
| IT-HIPP-022  | `parallel.rs::tests::test_parallel_execution_consolidated_input`       | Creates a Session, runs two scripts that both ask for the same variable, provides one answer via `Session::provide_answer`, and asserts: the question was de-duplicated (asked once), and both scripts received the value and produced their respective output messages. | SYS-HIPP-017, SYS-HIPP-042       | --       |

### 4.11 -- FFI JSON Parsing Logic

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|----------|
| IT-HIPP-023  | `reproduce_hang.rs::test_ffi_parsing_logic`                            | Tests the FFI JSON value-parsing function with multiple input formats (bare number, quoted number, quantity with unit, quantity with description suffix) and asserts each produces the expected `RuntimeValue` variant. | SYS-HIPP-018, SYS-HIPP-032       | --       |
| IT-HIPP-024  | `reproduce_hang.rs::test_ask_parsing_regression`                       | Parses a plan containing an `ask for` statement and asserts parsing succeeds, verifying the parser handles the ask-for grammar path without regression. | SYS-HIPP-010               | --       |

### 4.12 -- Stop Signal

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|----------|
| IT-HIPP-025  | `stop_signal.rs::test_stop_signal_terminates_execution`                | Verifies stop signal terminates a long-running simulation early.                              | SYS-HIPP-043               | None     |
| IT-HIPP-026  | `simulation.rs::test_time_pinned_periodic_trigger`                    | Simulates a plan with `every 1 day at 08:00 for 3 days` starting at 06:00; asserts events fire at 08:00 each day. | SYS-HIPP-013               | REQ-HIPP-EXEC-005 |
| IT-HIPP-027  | `simulation.rs::test_period_based_repetition_within_duration`          | Simulates `every <period> for 2 weeks` with a Mon-Fri 08:00 period; asserts events fire at every weekday occurrence (10 times). | SYS-HIPP-013, SYS-HIPP-014       | REQ-HIPP-EVT-006 |

### 4.13 -- After Plan Execution

| ID     | Test function                                                          | Description                                                                                   | Traces           | Fixtures |
|--------|------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------|----------------------|----------|
| IT-HIPP-028  | `simulation.rs::test_after_plan_block_execution`                       | Simulates a plan with an `after plan:` block; asserts that statements in the block execute exactly once after the event loop finishes. | SYS-HIPP-013               | None     |

## 5. Coverage Summary

| DES ID  | Description                        | Covered by IT-*                      |
|---------|------------------------------------|--------------------------------------|
| SYS-HIPP-010  | Parser (Pest PEG)                  | IT-HIPP-001 .. IT-HIPP-012, IT-HIPP-018, IT-HIPP-024         |
| SYS-HIPP-011  | AST representation                 | IT-HIPP-003                                |
| SYS-HIPP-012  | Multi-layer validator              | IT-HIPP-001, IT-HIPP-002                         |
| SYS-HIPP-013  | Runtime executor                   | IT-HIPP-002, IT-HIPP-007 .. IT-HIPP-013, IT-HIPP-016, IT-HIPP-017, IT-HIPP-020, IT-HIPP-026, IT-HIPP-027, IT-HIPP-028 |
| SYS-HIPP-014  | Scheduler                          | IT-HIPP-018, IT-HIPP-019, IT-HIPP-020, IT-HIPP-021           |
| SYS-HIPP-015  | Environment (state store)          | IT-HIPP-002, IT-HIPP-007 .. IT-HIPP-012, IT-HIPP-014, IT-HIPP-015  |
| SYS-HIPP-016  | Evaluator                          | IT-HIPP-010 .. IT-HIPP-012, IT-HIPP-014, IT-HIPP-015         |
| SYS-HIPP-017  | Session (multi-plan)               | IT-HIPP-022                                |
| SYS-HIPP-018  | FFI layer                          | IT-HIPP-023                                |
| SYS-HIPP-031  | Callback registration              | IT-HIPP-013, IT-HIPP-016, IT-HIPP-017                  |
| SYS-HIPP-032  | JSON data exchange                 | IT-HIPP-023                                |
| SYS-HIPP-041  | Simulation mode                    | IT-HIPP-007 .. IT-HIPP-009, IT-HIPP-020, IT-HIPP-021         |
| SYS-HIPP-042  | Input channel (mpsc)               | IT-HIPP-007, IT-HIPP-016, IT-HIPP-017, IT-HIPP-022           |
| SYS-HIPP-043  | Stop signal                        | IT-HIPP-025                                |

**Not directly covered by integration tests:**
SYS-HIPP-001 (language selection), SYS-HIPP-002 (dual crate output), SYS-HIPP-003 (C-FFI boundary), SYS-HIPP-019 (formatter), SYS-HIPP-020..SYS-HIPP-026 (dependency choices), SYS-HIPP-030 (lifecycle via FFI), SYS-HIPP-033 (memory management), SYS-HIPP-034 (iOS integration), SYS-HIPP-040 (real-time mode). These are either architectural constraints verified by compilation, dependency declarations, or host-side concerns outside the scope of Rust integration tests.

## Revision History

| Rev | Date       | Author | Description            |
|-----|------------|--------|------------------------|
| 1.0 | 2026-03-20 | --     | Initial version        |
| 1.1 | 2026-03-20 | --     | Added IT-HIPP-025 (stop signal). Closed SYS-HIPP-043 gap. |
| 1.2 | 2026-03-23 | --     | Added IT-HIPP-026 (time-pinned triggers), IT-HIPP-027 (period-based repetition). |
| 1.3 | 2026-03-23 | --     | Added IT-HIPP-028 (after plan block execution). |
| 1.4 | 2026-04-19 | --     | Renamed all IT-* test IDs to canonical IT-HIPP-* form and updated `**Traces to:**` references from DES-* to SYS-HIPP-* for V-Model validator/generator compatibility. |
