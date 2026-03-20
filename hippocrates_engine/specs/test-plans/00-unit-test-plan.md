# Unit Test Plan

| Field         | Value                                      |
|---------------|--------------------------------------------|
| Document ID   | UT                                         |
| V-Model Level | Unit Testing (Right side, bottom)          |
| Verifies      | Detailed Design (DDR-*)                    |

---

## 1. Purpose

Verify individual modules and functions against the detailed design. Each test case targets a specific DDR-* element from the Detailed Design Document and confirms that the module or function behaves as specified.

## 2. Test Environment

| Item           | Detail                                            |
|----------------|---------------------------------------------------|
| Language       | Rust                                              |
| Test framework | `cargo test`                                      |
| Test locations | `tests/spec/` (integration-style unit tests) and inline `#[cfg(test)]` modules in `src/` |
| Command        | `cargo test`                                      |

### Source files under test

| Test file                                  | Module area                        |
|--------------------------------------------|------------------------------------|
| `tests/spec/grammar.rs`                    | Parser grammar rules               |
| `tests/spec/values.rs`                     | Value definitions and properties   |
| `tests/spec/units.rs`                      | Unit system and conversions        |
| `tests/spec/validation.rs`                 | Validator pipeline                 |
| `tests/spec/execution.rs`                  | Runtime engine and evaluator       |
| `tests/spec/periods_plans.rs`              | Periods, plans, and event triggers |
| `tests/spec/statements_actions.rs`         | Statements and action parsing      |
| `tests/spec/actors_drugs.rs`               | Addressees and drug definitions    |
| `tests/spec/contexts_expressions.rs`       | Contexts and expressions           |
| `tests/spec/fixtures.rs`                   | Full fixture smoke tests           |
| `src/runtime/evaluator_tests.rs`           | Evaluator internals (inline)       |

## 3. Entry / Exit Criteria

**Entry criteria**
- Engine compiles successfully (`cargo build`).
- All dependencies resolved (`Cargo.lock` up to date).

**Exit criteria**
- All UT-* test cases pass when running `cargo test`.

---

## 4. Test Cases

### 4.1 UT-PARSER-* --- Parser Tests

Tests in `tests/spec/grammar.rs` verify DDR-PARSER-* elements (PEG grammar, AST construction, indentation handling).

| ID            | Test function                                                           | Description                                                       | DDR Reference    |
|---------------|-------------------------------------------------------------------------|-------------------------------------------------------------------|------------------|
| UT-PARSER-01  | `tests/spec/grammar.rs::spec_block_requires_newline_after_colon`        | Block openings require a newline and indented block.              | DDR-PARSER-03    |
| UT-PARSER-02  | `tests/spec/grammar.rs::spec_identifiers_require_angle_brackets`        | Identifiers must use angle brackets.                              | DDR-PARSER-01    |
| UT-PARSER-03  | `tests/spec/grammar.rs::spec_inline_colon_requires_block`               | Inline ':' forms are only allowed where explicitly shown.         | DDR-PARSER-01    |
| UT-PARSER-04  | `tests/spec/grammar.rs::spec_string_literal_rejects_angle_brackets`     | String literals must not contain angle brackets.                  | DDR-PARSER-01    |
| UT-PARSER-05  | `tests/spec/grammar.rs::spec_no_comparison_operators`                   | Comparison operators are not supported; use ranges.               | DDR-PARSER-01    |
| UT-PARSER-06  | `tests/spec/grammar.rs::spec_unitless_numeric_literal_fails`            | Numeric literals must include units.                              | DDR-PARSER-01    |
| UT-PARSER-07  | `tests/spec/grammar.rs::spec_block_statements_require_period`           | Statements inside blocks must terminate with a period.            | DDR-PARSER-01    |
| UT-PARSER-08  | `tests/spec/grammar.rs::spec_blocks_require_colon`                      | Blocks must be introduced with a colon.                           | DDR-PARSER-01    |
| UT-PARSER-09  | `tests/spec/grammar.rs::spec_meaning_assessments_not_allowed_in_plans`  | Meaning assessments are only allowed in value definition blocks.  | DDR-PARSER-02    |
| UT-PARSER-10  | `tests/spec/grammar.rs::spec_meaning_requires_target_identifier`        | Meaning properties require an explicit target identifier.         | DDR-PARSER-02    |
| UT-PARSER-11  | `tests/spec/grammar.rs::spec_meaning_requires_valid_meanings`           | Meaning properties must declare valid meanings.                   | DDR-PARSER-02    |
| UT-PARSER-12  | `tests/spec/grammar.rs::spec_meaning_labels_require_identifiers`        | Meaning labels must be identifiers (angle brackets).              | DDR-PARSER-02    |

### 4.2 UT-DOM-* --- Domain Model Tests

Tests verifying DDR-DOM-* elements (RuntimeValue, Unit, EventType, AskRequest, ValueType).

| ID         | Test function                                                           | Description                                                              | DDR Reference |
|------------|-------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------|
| UT-DOM-01  | `tests/spec/values.rs::spec_value_type_variants_parse`                  | Value type variants (string, time indication, date/time, period, plan, drug, addressee) parse correctly. | DDR-DOM-06    |
| UT-DOM-02  | `tests/spec/units.rs::spec_unit_conversions_within_groups`              | Unit conversions within compatible groups (kg/g, cm/m, L/mL, C/F, mmol/mg). | DDR-DOM-02    |
| UT-DOM-03  | `src/runtime/evaluator_tests.rs::test_fuzzy_equals`                     | Fuzzy string equality via check_condition.                               | DDR-DOM-01    |
| UT-DOM-04  | `src/runtime/evaluator_tests.rs::test_fuzzy_equals_enum`                | Fuzzy equality for enumeration values.                                   | DDR-DOM-01    |
| UT-DOM-05  | `src/runtime/evaluator_tests.rs::test_fuzzy_equals_enum_enum`           | Fuzzy equality for enum-to-enum comparison (stub).                       | DDR-DOM-01    |

### 4.3 UT-VAL-* --- Validator Tests

Tests in `tests/spec/validation.rs` verify DDR-VAL-* elements (semantic checks, interval math, data flow, coverage analysis, error reporting).

| ID          | Test function                                                                   | Description                                                              | DDR Reference |
|-------------|---------------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------|
| UT-VAL-01   | `tests/spec/validation.rs::spec_meaning_coverage_gaps_integer`                  | Meaning ranges must cover valid values (integer gaps).                   | DDR-VAL-05    |
| UT-VAL-02   | `tests/spec/validation.rs::spec_meaning_coverage_gaps_float`                    | Meaning ranges must cover valid values (float gaps).                     | DDR-VAL-05    |
| UT-VAL-03   | `tests/spec/validation.rs::spec_meaning_coverage_disjoint_ranges_ok`            | Disjoint valid ranges pass when fully covered.                           | DDR-VAL-05    |
| UT-VAL-04   | `tests/spec/validation.rs::spec_meaning_valid_meanings_must_be_used`            | Valid meanings must be fully used across meaning assessments.             | DDR-VAL-05    |
| UT-VAL-05   | `tests/spec/validation.rs::spec_meaning_invalid_label_rejected`                 | Meaning labels must be drawn from declared valid meanings.               | DDR-VAL-02    |
| UT-VAL-06   | `tests/spec/validation.rs::spec_validator_numeric_overlap`                      | Overlapping numeric assessment ranges are invalid.                       | DDR-VAL-03    |
| UT-VAL-07   | `tests/spec/validation.rs::spec_validator_enum_duplicate`                       | Duplicate enumeration cases are invalid.                                 | DDR-VAL-05    |
| UT-VAL-08   | `tests/spec/validation.rs::spec_enum_valid_values_require_identifiers`          | Enumeration valid values must be identifiers.                            | DDR-VAL-02    |
| UT-VAL-09   | `tests/spec/validation.rs::spec_validator_requires_not_enough_data_case`        | Timeframe calculations require Not enough data handling.                 | DDR-VAL-02    |
| UT-VAL-10   | `tests/spec/validation.rs::spec_statistical_functions_require_timeframe_context` | Statistical functions require an analysis timeframe context.             | DDR-VAL-02    |
| UT-VAL-11   | `tests/spec/validation.rs::spec_not_enough_data_requires_statistical_target`    | Not enough data is only allowed for statistical assessments.             | DDR-VAL-02    |
| UT-VAL-12   | `tests/spec/validation.rs::spec_validator_passes_with_not_enough_data`          | Not enough data handling satisfies sufficiency.                          | DDR-VAL-02    |
| UT-VAL-13   | `tests/spec/validation.rs::spec_validate_plan_fixture_suite`                    | Full-plan validation passes/fails for fixture scenarios.                 | DDR-VAL-01    |
| UT-VAL-14   | `tests/spec/validation.rs::spec_validator_integer_gap_message`                  | Gap detection reports missing integer spans.                             | DDR-VAL-06    |
| UT-VAL-15   | `tests/spec/validation.rs::spec_validator_float_gap_message`                    | Gap detection reports missing float spans.                               | DDR-VAL-06    |
| UT-VAL-16   | `tests/spec/validation.rs::spec_precision_gaps`                                 | Coverage gaps respect precision for float and integer ranges.            | DDR-VAL-03    |
| UT-VAL-17   | `tests/spec/validation.rs::spec_precision_consistency`                          | Numeric valid value ranges use consistent precision across bounds.       | DDR-VAL-03    |
| UT-VAL-18   | `tests/spec/validation.rs::spec_valid_values_ranges_do_not_overlap`             | Valid value ranges must not overlap.                                     | DDR-VAL-03    |
| UT-VAL-19   | `tests/spec/validation.rs::spec_valid_values_datetime_ranges_do_not_overlap`    | Date/time valid value ranges must not overlap.                           | DDR-VAL-03    |
| UT-VAL-20   | `tests/spec/validation.rs::spec_range_overlap`                                  | Overlapping assessment ranges are rejected.                              | DDR-VAL-03    |
| UT-VAL-21   | `tests/spec/validation.rs::spec_unit_requirement_validation`                    | Numeric valid values require units.                                      | DDR-VAL-02    |
| UT-VAL-22   | `tests/spec/validation.rs::spec_validation_error_line_number`                   | Validation errors report line numbers.                                   | DDR-VAL-06    |
| UT-VAL-23   | `tests/spec/validation.rs::spec_missing_valid_values_fails`                     | Numbers and enumerations must define valid values.                       | DDR-VAL-02    |
| UT-VAL-24   | `tests/spec/validation.rs::spec_timeframe_selector_requires_period_definition`  | Timeframe selector identifiers must refer to defined periods.            | DDR-VAL-02    |
| UT-VAL-25   | `tests/spec/validation.rs::spec_reproduce_missing_error`                        | Missing coverage yields a validation error.                              | DDR-VAL-05    |
| UT-VAL-26   | `tests/spec/validation.rs::spec_unitless_assess_fails`                          | Assessment ranges require units.                                         | DDR-VAL-02    |
| UT-VAL-27   | `tests/spec/validation.rs::spec_unitless_definition_fails`                      | Numeric definitions require units.                                       | DDR-VAL-02    |
| UT-VAL-28   | `tests/spec/validation.rs::spec_interval_creation_and_math`                     | Interval math supports range compliance checks.                         | DDR-VAL-03    |
| UT-VAL-29   | `tests/spec/validation.rs::spec_data_flow_use_before_assignment_fails`          | Values cannot be used before assignment.                                 | DDR-VAL-04    |
| UT-VAL-30   | `tests/spec/validation.rs::spec_calculation_does_not_initialize_value`          | Calculation properties do not seed values.                               | DDR-VAL-04    |
| UT-VAL-31   | `tests/spec/validation.rs::spec_statistical_functions_do_not_require_local_init` | Statistical functions do not require local initialization.               | DDR-VAL-04    |
| UT-VAL-32   | `tests/spec/validation.rs::spec_meaning_of_requires_question_when_uninitialized` | Meaning-of requires question property when value is uninitialized.       | DDR-VAL-04    |
| UT-VAL-33   | `tests/spec/validation.rs::spec_meaning_of_allows_question_when_uninitialized`  | Meaning-of is allowed when the value has a question property.            | DDR-VAL-04    |
| UT-VAL-34   | `tests/spec/validation.rs::spec_listen_and_context_initialize_values`           | Listen for and context data initialize values for data flow.             | DDR-VAL-04    |
| UT-VAL-35   | `tests/spec/validation.rs::spec_ask_requires_question_property`                 | Ask requires a question property on the value.                           | DDR-VAL-02    |
| UT-VAL-36   | `tests/spec/validation.rs::spec_trend_requires_full_coverage`                   | Trend assessments require full coverage.                                 | DDR-VAL-05    |
| UT-VAL-37   | `tests/spec/validation.rs::spec_assignment_range_compliance_warning`             | Assignment range compliance fails when out of bounds.                    | DDR-VAL-03    |

### 4.4 UT-RT-* --- Runtime Tests

Tests in `tests/spec/execution.rs` verify DDR-RT-* elements (Engine, Environment, Executor, Evaluator, Scheduler, Session, Input Validation).

| ID         | Test function                                                             | Description                                                                   | DDR Reference |
|------------|---------------------------------------------------------------------------|-------------------------------------------------------------------------------|---------------|
| UT-RT-01   | `tests/spec/execution.rs::spec_not_enough_data_evaluation`                | Runtime evaluation returns NotEnoughData when history is insufficient.         | DDR-RT-04     |
| UT-RT-02   | `tests/spec/execution.rs::spec_date_time_range_evaluation`                | Date/time valid value ranges evaluate using date/time and time-of-day semantics. | DDR-RT-04     |
| UT-RT-03   | `tests/spec/execution.rs::spec_date_diff_evaluation`                      | Date diff expressions evaluate to quantities in requested units.              | DDR-RT-04     |
| UT-RT-04   | `tests/spec/execution.rs::spec_meaning_of_evaluates`                      | Meaning evaluation returns the assessed meaning for the value.                | DDR-RT-04     |
| UT-RT-05   | `tests/spec/execution.rs::spec_meaning_of_missing_value`                  | Meaning evaluation returns Missing when the source value is unknown.          | DDR-RT-04     |
| UT-RT-06   | `tests/spec/execution.rs::spec_meaning_of_nested_assessment`              | Meaning evaluation supports nested assessments.                               | DDR-RT-04     |
| UT-RT-07   | `tests/spec/execution.rs::spec_numeric_input_precision_rejection`         | Numeric answers must respect the decimal precision implied by valid values.   | DDR-RT-07     |
| UT-RT-08   | `tests/spec/execution.rs::spec_runtime_execution_flow`                    | Runtime executes assignments and actions in order.                             | DDR-RT-01     |
| UT-RT-09   | `tests/spec/execution.rs::spec_message_callback_missing_warns`            | Runtime emits a warning when a message callback is not set.                   | DDR-RT-03     |
| UT-RT-10   | `tests/spec/execution.rs::spec_validity_reuse_timeframe`                  | Reuse timeframes prevent re-asking within the validity window.                | DDR-RT-06     |
| UT-RT-11   | `tests/spec/execution.rs::spec_timeframe_filtering`                       | Timeframe filtering applies to statistical evaluations.                       | DDR-RT-04     |
| UT-RT-12   | `tests/spec/execution.rs::spec_timeframe_variants`                        | Timeframe variants resolve counts over different windows.                     | DDR-RT-04     |
| UT-RT-13   | `tests/spec/execution.rs::spec_trend_analysis_evaluates`                  | Trend analysis evaluates statistical trends over timeframes.                  | DDR-RT-04     |
| UT-RT-14   | `tests/spec/execution.rs::spec_scheduler_next_occurrence`                 | Scheduler computes next occurrence for periods.                               | DDR-RT-05     |
| UT-RT-15   | `tests/spec/execution.rs::spec_environment_append_only_history`           | Verifies append-only value history.                                           | DDR-RT-02     |
| UT-RT-16   | `tests/spec/execution.rs::spec_value_history_retrieval`                   | Verifies value history retrieval with timestamps.                             | DDR-RT-02     |
| UT-RT-17   | `tests/spec/execution.rs::spec_simulation_mode_execution`                 | Verifies simulation mode completes without delays.                            | DDR-RT-08     |

### 4.5 UT-VALUES-* --- Value Definition Tests

Tests in `tests/spec/values.rs` verify DDR-PARSER-02 (AST node hierarchy) and DDR-DOM-06 (ValueType) for value definition parsing.

| ID            | Test function                                                       | Description                                                          | DDR Reference |
|---------------|---------------------------------------------------------------------|----------------------------------------------------------------------|---------------|
| UT-VALUES-01  | `tests/spec/values.rs::spec_value_definition_parsing`               | Value definitions parse from fixtures.                               | DDR-PARSER-02 |
| UT-VALUES-02  | `tests/spec/values.rs::spec_value_type_variants_parse`              | Value type variants parse correctly.                                 | DDR-PARSER-02 |
| UT-VALUES-03  | `tests/spec/values.rs::spec_unit_property_parsing`                  | Unit properties parse in numeric values.                             | DDR-PARSER-02 |
| UT-VALUES-04  | `tests/spec/values.rs::spec_value_timeframe_property_parsing`       | Value timeframe properties parse.                                    | DDR-PARSER-02 |
| UT-VALUES-05  | `tests/spec/values.rs::spec_inheritance_property_parsing`           | Inheritance properties parse with overrides.                         | DDR-PARSER-02 |
| UT-VALUES-06  | `tests/spec/values.rs::spec_documentation_property_parsing`         | Documentation properties parse in inline and block forms.            | DDR-PARSER-02 |
| UT-VALUES-07  | `tests/spec/values.rs::spec_generic_property_parsing`               | Custom properties parse as generic properties.                       | DDR-PARSER-02 |

### 4.6 UT-UNITS-* --- Unit System Tests

Tests in `tests/spec/units.rs` verify DDR-DOM-02 (Unit) and DDR-PARSER-01 (grammar rules for units).

| ID           | Test function                                                          | Description                                                       | DDR Reference |
|--------------|------------------------------------------------------------------------|-------------------------------------------------------------------|---------------|
| UT-UNITS-01  | `tests/spec/units.rs::spec_custom_unit_pluralization_is_canonical`      | Custom unit pluralization canonicalizes values.                   | DDR-DOM-02    |
| UT-UNITS-02  | `tests/spec/units.rs::spec_standard_units_still_work`                  | Standard units work in calculations.                              | DDR-DOM-02    |
| UT-UNITS-03  | `tests/spec/units.rs::spec_custom_unit_abbreviation_is_canonical`      | Custom unit abbreviations canonicalize values.                    | DDR-DOM-02    |
| UT-UNITS-04  | `tests/spec/units.rs::spec_builtin_units_cannot_be_redefined`          | Built-in units cannot be redefined.                               | DDR-VAL-02    |
| UT-UNITS-05  | `tests/spec/units.rs::spec_custom_unit_quantity_parsing`               | Custom unit quantities parse with definitions.                    | DDR-PARSER-01 |
| UT-UNITS-06  | `tests/spec/units.rs::spec_unit_conversions_within_groups`             | Unit conversions are supported within compatible groups.           | DDR-DOM-02    |
| UT-UNITS-07  | `tests/spec/units.rs::spec_assignment_requires_unit_and_precision_match` | Assignments require matching units and precision.                 | DDR-VAL-03    |

### 4.7 UT-PERIODS-* --- Period and Plan Tests

Tests in `tests/spec/periods_plans.rs` verify DDR-PARSER-02 (AST node hierarchy for periods, plans, triggers).

| ID             | Test function                                                                      | Description                                                      | DDR Reference |
|----------------|------------------------------------------------------------------------------------|------------------------------------------------------------------|---------------|
| UT-PERIODS-01  | `tests/spec/periods_plans.rs::spec_period_definition_parsing`                      | Period definitions parse by name.                                | DDR-PARSER-02 |
| UT-PERIODS-02  | `tests/spec/periods_plans.rs::spec_period_parsing_structure`                       | Period timeframe lines parse with range selectors.               | DDR-PARSER-02 |
| UT-PERIODS-03  | `tests/spec/periods_plans.rs::spec_event_trigger_parsing`                          | Event triggers parse for change/start/periodic.                  | DDR-PARSER-02 |
| UT-PERIODS-04  | `tests/spec/periods_plans.rs::spec_event_trigger_duration_and_offset_parsing`      | Periodic triggers parse duration and offsets.                    | DDR-PARSER-02 |
| UT-PERIODS-05  | `tests/spec/periods_plans.rs::spec_event_block_parsing`                            | Event blocks attach statements to triggers.                      | DDR-PARSER-02 |

### 4.8 UT-ACTIONS-* --- Action and Statement Tests

Tests in `tests/spec/statements_actions.rs` verify DDR-PARSER-02 (AST node hierarchy for statements and actions).

| ID             | Test function                                                                              | Description                                                             | DDR Reference |
|----------------|--------------------------------------------------------------------------------------------|-------------------------------------------------------------------------|---------------|
| UT-ACTIONS-01  | `tests/spec/statements_actions.rs::spec_timeframe_block_parsing`                           | Timeframe blocks parse with nested statements.                          | DDR-PARSER-02 |
| UT-ACTIONS-02  | `tests/spec/statements_actions.rs::spec_question_config_parsing_and_validation`             | Question configuration parses and validates references.                 | DDR-PARSER-02 |
| UT-ACTIONS-03  | `tests/spec/statements_actions.rs::spec_message_expiration_parsing`                        | Message expiration attaches to information, warning, and urgent warning. | DDR-PARSER-02 |
| UT-ACTIONS-04  | `tests/spec/statements_actions.rs::spec_message_action_keyword_parsing`                    | Information, warning, and urgent warning are accepted as message keywords. | DDR-PARSER-02 |
| UT-ACTIONS-05  | `tests/spec/statements_actions.rs::spec_question_modifiers_parsing`                        | Question modifiers parse (validate/type/style/expire).                  | DDR-PARSER-02 |
| UT-ACTIONS-06  | `tests/spec/statements_actions.rs::spec_question_expiration_block_parsing`                 | Question expiration blocks parse with reminder statements.              | DDR-PARSER-02 |
| UT-ACTIONS-07  | `tests/spec/statements_actions.rs::spec_question_expiration_until_event_trigger_parsing`   | Question expiration supports until event triggers.                      | DDR-PARSER-02 |
| UT-ACTIONS-08  | `tests/spec/statements_actions.rs::spec_validate_answer_within_parsing`                    | Validate answer within parsing attaches to ask blocks.                  | DDR-PARSER-02 |
| UT-ACTIONS-09  | `tests/spec/statements_actions.rs::spec_listen_send_start_and_simple_command_parsing`      | Listen/send/start/simple command actions parse.                         | DDR-PARSER-02 |
| UT-ACTIONS-10  | `tests/spec/statements_actions.rs::spec_timeframe_requires_range_selector`                 | Timeframe selectors require a start and end.                            | DDR-PARSER-01 |

### 4.9 UT-ACTORS-* --- Actor and Drug Tests

Tests in `tests/spec/actors_drugs.rs` verify DDR-PARSER-02 (AST node hierarchy for addressees and drugs) and DDR-VAL-02 (semantic validation of drug definitions).

| ID            | Test function                                                                            | Description                                                         | DDR Reference |
|---------------|------------------------------------------------------------------------------------------|---------------------------------------------------------------------|---------------|
| UT-ACTORS-01  | `tests/spec/actors_drugs.rs::spec_drug_definition_validation`                            | Drug definition validation rejects undefined units.                 | DDR-VAL-02    |
| UT-ACTORS-02  | `tests/spec/actors_drugs.rs::spec_addressee_group_and_contact_logic_parsing`              | Addressee groups and contact logic parse.                           | DDR-PARSER-02 |
| UT-ACTORS-03  | `tests/spec/actors_drugs.rs::spec_drug_interactions_parse`                                | Drug interaction properties parse.                                  | DDR-PARSER-02 |
| UT-ACTORS-04  | `tests/spec/actors_drugs.rs::spec_addressee_contact_details_and_sequence_order_parsing`   | Contact details and sequence contact order parse.                   | DDR-PARSER-02 |
| UT-ACTORS-05  | `tests/spec/actors_drugs.rs::spec_drug_dosage_and_admin_rules_parsing`                   | Dosage safety and administration rules parse.                       | DDR-PARSER-02 |

### 4.10 UT-CTX-* --- Context and Expression Tests

Tests in `tests/spec/contexts_expressions.rs` verify DDR-PARSER-02 (AST node hierarchy for contexts and expressions) and DDR-RT-04 (Evaluator for context execution).

| ID          | Test function                                                                        | Description                                                          | DDR Reference |
|-------------|--------------------------------------------------------------------------------------|----------------------------------------------------------------------|---------------|
| UT-CTX-01   | `tests/spec/contexts_expressions.rs::spec_context_definition_parsing`                | Context definitions parse timeframe/data/value filter items.         | DDR-PARSER-02 |
| UT-CTX-02   | `tests/spec/contexts_expressions.rs::spec_context_block_items_parsing`               | Context blocks parse data/value filters and nested statements.       | DDR-PARSER-02 |
| UT-CTX-03   | `tests/spec/contexts_expressions.rs::spec_statistical_functions_parsing`              | Statistical function expressions parse in assignments.               | DDR-PARSER-02 |
| UT-CTX-04   | `tests/spec/contexts_expressions.rs::spec_meaning_of_expression_parsing`             | Meaning-of expressions parse in assignments.                         | DDR-PARSER-02 |
| UT-CTX-05   | `tests/spec/contexts_expressions.rs::spec_time_indications_parsing`                  | Time indications parse for now, weekday, and time-of-day.            | DDR-PARSER-02 |
| UT-CTX-06   | `tests/spec/contexts_expressions.rs::spec_date_time_literals_parsing`                | Date/time literals parse for date and date-time forms.               | DDR-PARSER-02 |
| UT-CTX-07   | `tests/spec/contexts_expressions.rs::spec_date_diff_parsing`                         | Date diff expressions parse.                                         | DDR-PARSER-02 |
| UT-CTX-08   | `tests/spec/contexts_expressions.rs::spec_relative_time_from_now_parsing`            | Relative time expressions from now parse.                            | DDR-PARSER-02 |
| UT-CTX-09   | `tests/spec/contexts_expressions.rs::spec_context_for_analysis_execution`            | Context for analysis executes with scoped timeframe.                 | DDR-RT-04     |

### 4.11 UT-FIX-* --- Fixture Tests

Tests in `tests/spec/fixtures.rs` verify DDR-PARSER-04 (parser entry point) across a complete multi-definition fixture.

| ID         | Test function                                                          | Description                                                  | DDR Reference |
|------------|------------------------------------------------------------------------|--------------------------------------------------------------|---------------|
| UT-FIX-01  | `tests/spec/fixtures.rs::spec_full_fixture_parses_core_definitions`    | Multi-definition fixtures parse core definitions.            | DDR-PARSER-04 |

### 4.12 UT-FFI-* --- FFI Interface Tests

Tests in `tests/ffi.rs` verify DDR-FFI-* elements (C-compatible API functions).

| UT ID | Test Function | Description | DDR Reference |
|-------|--------------|-------------|---------------|
| UT-FFI-01 | `tests/ffi.rs::ffi_parse_json_valid` | Verifies parse returns Ok JSON for valid input | DDR-FFI-01 |
| UT-FFI-02 | `tests/ffi.rs::ffi_parse_json_invalid` | Verifies parse returns Err JSON for invalid input | DDR-FFI-01 |
| UT-FFI-03 | `tests/ffi.rs::ffi_validate_file_valid_and_invalid` | Verifies validation, error count, and error retrieval | DDR-FFI-08 |
| UT-FFI-04 | `tests/ffi.rs::ffi_engine_lifecycle` | Verifies engine create/free lifecycle | DDR-FFI-02 |
| UT-FFI-05 | `tests/ffi.rs::ffi_engine_load_valid` | Verifies loading valid plan | DDR-FFI-03 |
| UT-FFI-06 | `tests/ffi.rs::ffi_engine_load_invalid` | Verifies loading invalid plan returns error | DDR-FFI-03 |
| UT-FFI-07 | `tests/ffi.rs::ffi_get_periods` | Verifies period retrieval as JSON | DDR-FFI-09 |
| UT-FFI-08 | `tests/ffi.rs::ffi_set_time` | Verifies setting engine time | DDR-FFI-07 |
| UT-FFI-09 | `tests/ffi.rs::ffi_enable_simulation` | Verifies enabling simulation mode | DDR-FFI-10 |
| UT-FFI-10 | `tests/ffi.rs::ffi_stop` | Verifies stop signal | DDR-FFI-11 |

### 4.13 UT-FMT-* --- Formatter Tests

Tests in `tests/formatter.rs` verify DDR-FMT-01 (formatter `format_script`).

| UT ID | Test Function | Description | DDR Reference |
|-------|--------------|-------------|---------------|
| UT-FMT-01 | `tests/formatter.rs::formatter_round_trip_parse_format_parse` | Round-trip parse-format-parse | DDR-FMT-01 |
| UT-FMT-02 | `tests/formatter.rs::formatter_handles_all_definition_types` | All definition types format correctly | DDR-FMT-01 |

---

## 5. Coverage Summary

| Module group | DDR elements covered                                   | Test count |
|--------------|--------------------------------------------------------|------------|
| PARSER       | DDR-PARSER-01, DDR-PARSER-02, DDR-PARSER-03, DDR-PARSER-04 | 12     |
| DOM          | DDR-DOM-01, DDR-DOM-02, DDR-DOM-06                     | 5          |
| VAL          | DDR-VAL-01 through DDR-VAL-06                          | 37         |
| RT           | DDR-RT-01, DDR-RT-02, DDR-RT-03, DDR-RT-04, DDR-RT-05, DDR-RT-06, DDR-RT-07, DDR-RT-08 | 17   |
| VALUES       | DDR-PARSER-02                                          | 7          |
| UNITS        | DDR-DOM-02, DDR-PARSER-01, DDR-VAL-02, DDR-VAL-03      | 7          |
| PERIODS      | DDR-PARSER-02                                          | 5          |
| ACTIONS      | DDR-PARSER-01, DDR-PARSER-02                           | 10         |
| ACTORS       | DDR-PARSER-02, DDR-VAL-02                              | 5          |
| CTX          | DDR-PARSER-02, DDR-RT-04                               | 9          |
| FIX          | DDR-PARSER-04                                          | 1          |
| FFI          | DDR-FFI-01, DDR-FFI-02, DDR-FFI-03, DDR-FFI-07, DDR-FFI-08, DDR-FFI-09, DDR-FFI-10, DDR-FFI-11 | 10 |
| FMT          | DDR-FMT-01                                             | 2          |
| **Total**    |                                                        | **127**    |

### Gaps

| DDR element     | Status   | Notes                                                                |
|-----------------|----------|----------------------------------------------------------------------|
| DDR-FFI-01..19  | Partial  | UT-FFI-01..10 cover DDR-FFI-01, -02, -03, -07, -08, -09, -10, -11. Remaining FFI functions (DDR-FFI-04..06, -12..19) are tested at the integration level via Swift/C bindings. |
| DDR-RT-02       | Covered  | UT-RT-15 and UT-RT-16 verify append-only history and value history retrieval. |
| DDR-RT-08       | Covered  | UT-RT-17 verifies simulation mode execution without real-time delays. |
| DDR-FMT-01      | Covered  | UT-FMT-01 and UT-FMT-02 verify round-trip formatting and all definition types. |

---

## Revision History

| Version | Date       | Author | Changes         |
|---------|------------|--------|-----------------|
| 1.0     | 2026-03-20 | ---    | Initial release |
| 1.1     | 2026-03-20 | ---    | Added UT-FFI-01..10 (FFI tests), UT-RT-15..17 (environment/history/simulation), UT-FMT-01..02 (formatter). Closed DDR-FFI, DDR-RT-02, DDR-RT-08, DDR-FMT-01 gaps. |
