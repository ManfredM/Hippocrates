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

Tests in `tests/spec/grammar.rs` verify DET-HIPP-PARSER-* elements (PEG grammar, AST construction, indentation handling).

| ID            | Test function                                                           | Description                                                       | Traces    |
|---------------|-------------------------------------------------------------------------|-------------------------------------------------------------------|------------------|
| UT-HIPP-PARSER-001  | `tests/spec/grammar.rs::spec_block_requires_newline_after_colon`        | Block openings require a newline and indented block.              | DET-HIPP-PARSER-003    |
| UT-HIPP-PARSER-002  | `tests/spec/grammar.rs::spec_identifiers_require_angle_brackets`        | Identifiers must use angle brackets.                              | DET-HIPP-PARSER-001    |
| UT-HIPP-PARSER-003  | `tests/spec/grammar.rs::spec_inline_colon_requires_block`               | Inline ':' forms are only allowed where explicitly shown.         | DET-HIPP-PARSER-001    |
| UT-HIPP-PARSER-004  | `tests/spec/grammar.rs::spec_string_literal_rejects_angle_brackets`     | String literals must not contain angle brackets.                  | DET-HIPP-PARSER-001    |
| UT-HIPP-PARSER-005  | `tests/spec/grammar.rs::spec_no_comparison_operators`                   | Comparison operators are not supported; use ranges.               | DET-HIPP-PARSER-001    |
| UT-HIPP-PARSER-006  | `tests/spec/grammar.rs::spec_unitless_numeric_literal_fails`            | Numeric literals must include units.                              | DET-HIPP-PARSER-001    |
| UT-HIPP-PARSER-007  | `tests/spec/grammar.rs::spec_block_statements_require_period`           | Statements inside blocks must terminate with a period.            | DET-HIPP-PARSER-001    |
| UT-HIPP-PARSER-008  | `tests/spec/grammar.rs::spec_blocks_require_colon`                      | Blocks must be introduced with a colon.                           | DET-HIPP-PARSER-001    |
| UT-HIPP-PARSER-009  | `tests/spec/grammar.rs::spec_meaning_assessments_not_allowed_in_plans`  | Meaning assessments are only allowed in value definition blocks.  | DET-HIPP-PARSER-002    |
| UT-HIPP-PARSER-010  | `tests/spec/grammar.rs::spec_meaning_requires_target_identifier`        | Meaning properties require an explicit target identifier.         | DET-HIPP-PARSER-002    |
| UT-HIPP-PARSER-011  | `tests/spec/grammar.rs::spec_meaning_requires_valid_meanings`           | Meaning properties must declare valid meanings.                   | DET-HIPP-PARSER-002    |
| UT-HIPP-PARSER-012  | `tests/spec/grammar.rs::spec_meaning_labels_require_identifiers`        | Meaning labels must be identifiers (angle brackets).              | DET-HIPP-PARSER-002    |
| UT-HIPP-PARSER-013  | `tests/spec/validation.rs::spec_parse_error_human_readable`             | Syntax error produces human-readable message, not raw Rule names. | DET-HIPP-PARSER-006    |

### 4.2 UT-DOM-* --- Domain Model Tests

Tests verifying DET-HIPP-DOM-* elements (RuntimeValue, Unit, EventType, AskRequest, ValueType).

| ID         | Test function                                                           | Description                                                              | Traces |
|------------|-------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------|
| UT-HIPP-DOM-001  | `tests/spec/values.rs::spec_value_type_variants_parse`                  | Value type variants (string, time indication, date/time, period, plan, drug, addressee) parse correctly. | DET-HIPP-DOM-006    |
| UT-HIPP-DOM-002  | `tests/spec/units.rs::spec_unit_conversions_within_groups`              | Unit conversions within compatible groups (kg/g, cm/m, L/mL, C/F, mmol/mg). | DET-HIPP-DOM-002    |
| UT-HIPP-DOM-003  | `src/runtime/evaluator_tests.rs::test_fuzzy_equals`                     | Fuzzy string equality via check_condition.                               | DET-HIPP-DOM-001    |
| UT-HIPP-DOM-004  | `src/runtime/evaluator_tests.rs::test_fuzzy_equals_enum`                | Fuzzy equality for enumeration values.                                   | DET-HIPP-DOM-001    |
| UT-HIPP-DOM-005  | `src/runtime/evaluator_tests.rs::test_fuzzy_equals_enum_enum`           | Fuzzy equality for enum-to-enum comparison (stub).                       | DET-HIPP-DOM-001    |

### 4.3 UT-VAL-* --- Validator Tests

Tests in `tests/spec/validation.rs` verify DET-HIPP-VAL-* elements (semantic checks, interval math, data flow, coverage analysis, error reporting).

| ID          | Test function                                                                   | Description                                                              | Traces |
|-------------|---------------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------|
| UT-HIPP-VAL-001   | `tests/spec/validation.rs::spec_meaning_coverage_gaps_integer`                  | Meaning ranges must cover valid values (integer gaps).                   | DET-HIPP-VAL-005    |
| UT-HIPP-VAL-002   | `tests/spec/validation.rs::spec_meaning_coverage_gaps_float`                    | Meaning ranges must cover valid values (float gaps).                     | DET-HIPP-VAL-005    |
| UT-HIPP-VAL-003   | `tests/spec/validation.rs::spec_meaning_coverage_disjoint_ranges_ok`            | Disjoint valid ranges pass when fully covered.                           | DET-HIPP-VAL-005    |
| UT-HIPP-VAL-004   | `tests/spec/validation.rs::spec_meaning_valid_meanings_must_be_used`            | Valid meanings must be fully used across meaning assessments.             | DET-HIPP-VAL-005    |
| UT-HIPP-VAL-005   | `tests/spec/validation.rs::spec_meaning_invalid_label_rejected`                 | Meaning labels must be drawn from declared valid meanings.               | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-006   | `tests/spec/validation.rs::spec_validator_numeric_overlap`                      | Overlapping numeric assessment ranges are invalid.                       | DET-HIPP-VAL-003    |
| UT-HIPP-VAL-007   | `tests/spec/validation.rs::spec_validator_enum_duplicate`                       | Duplicate enumeration cases are invalid.                                 | DET-HIPP-VAL-005    |
| UT-HIPP-VAL-008   | `tests/spec/validation.rs::spec_enum_valid_values_require_identifiers`          | Enumeration valid values must be identifiers.                            | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-009   | `tests/spec/validation.rs::spec_validator_requires_not_enough_data_case`        | Timeframe calculations require Not enough data handling.                 | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-010   | `tests/spec/validation.rs::spec_statistical_functions_require_timeframe_context` | Statistical functions require an analysis timeframe context.             | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-011   | `tests/spec/validation.rs::spec_not_enough_data_requires_statistical_target`    | Not enough data is only allowed for statistical assessments.             | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-012   | `tests/spec/validation.rs::spec_validator_passes_with_not_enough_data`          | Not enough data handling satisfies sufficiency.                          | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-013   | `tests/spec/validation.rs::spec_validate_plan_fixture_suite`                    | Full-plan validation passes/fails for fixture scenarios.                 | DET-HIPP-VAL-001    |
| UT-HIPP-VAL-014   | `tests/spec/validation.rs::spec_validator_integer_gap_message`                  | Gap detection reports missing integer spans.                             | DET-HIPP-VAL-006    |
| UT-HIPP-VAL-015   | `tests/spec/validation.rs::spec_validator_float_gap_message`                    | Gap detection reports missing float spans.                               | DET-HIPP-VAL-006    |
| UT-HIPP-VAL-016   | `tests/spec/validation.rs::spec_precision_gaps`                                 | Coverage gaps respect precision for float and integer ranges.            | DET-HIPP-VAL-003    |
| UT-HIPP-VAL-017   | `tests/spec/validation.rs::spec_precision_consistency`                          | Numeric valid value ranges use consistent precision across bounds.       | DET-HIPP-VAL-003    |
| UT-HIPP-VAL-018   | `tests/spec/validation.rs::spec_valid_values_ranges_do_not_overlap`             | Valid value ranges must not overlap.                                     | DET-HIPP-VAL-003    |
| UT-HIPP-VAL-019   | `tests/spec/validation.rs::spec_valid_values_datetime_ranges_do_not_overlap`    | Date/time valid value ranges must not overlap.                           | DET-HIPP-VAL-003    |
| UT-HIPP-VAL-020   | `tests/spec/validation.rs::spec_range_overlap`                                  | Overlapping assessment ranges are rejected.                              | DET-HIPP-VAL-003    |
| UT-HIPP-VAL-021   | `tests/spec/validation.rs::spec_unit_requirement_validation`                    | Numeric valid values require units.                                      | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-022   | `tests/spec/validation.rs::spec_validation_error_line_number`                   | Validation errors report line numbers.                                   | DET-HIPP-VAL-006    |
| UT-HIPP-VAL-023   | `tests/spec/validation.rs::spec_missing_valid_values_fails`                     | Numbers and enumerations must define valid values.                       | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-024   | `tests/spec/validation.rs::spec_timeframe_selector_requires_period_definition`  | Timeframe selector identifiers must refer to defined periods.            | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-025   | `tests/spec/validation.rs::spec_reproduce_missing_error`                        | Missing coverage yields a validation error.                              | DET-HIPP-VAL-005    |
| UT-HIPP-VAL-026   | `tests/spec/validation.rs::spec_unitless_assess_fails`                          | Assessment ranges require units.                                         | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-027   | `tests/spec/validation.rs::spec_unitless_definition_fails`                      | Numeric definitions require units.                                       | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-028   | `tests/spec/validation.rs::spec_interval_creation_and_math`                     | Interval math supports range compliance checks.                         | DET-HIPP-VAL-003    |
| UT-HIPP-VAL-029   | `tests/spec/validation.rs::spec_data_flow_use_before_assignment_fails`          | Values cannot be used before assignment.                                 | DET-HIPP-VAL-004    |
| UT-HIPP-VAL-030   | `tests/spec/validation.rs::spec_calculation_does_not_initialize_value`          | Calculation properties do not seed values.                               | DET-HIPP-VAL-004    |
| UT-HIPP-VAL-031   | `tests/spec/validation.rs::spec_statistical_functions_do_not_require_local_init` | Statistical functions do not require local initialization.               | DET-HIPP-VAL-004    |
| UT-HIPP-VAL-032   | `tests/spec/validation.rs::spec_meaning_of_requires_question_when_uninitialized` | Meaning-of requires question property when value is uninitialized.       | DET-HIPP-VAL-004    |
| UT-HIPP-VAL-033   | `tests/spec/validation.rs::spec_meaning_of_allows_question_when_uninitialized`  | Meaning-of is allowed when the value has a question property.            | DET-HIPP-VAL-004    |
| UT-HIPP-VAL-034   | `tests/spec/validation.rs::spec_listen_and_context_initialize_values`           | Listen for and context data initialize values for data flow.             | DET-HIPP-VAL-004    |
| UT-HIPP-VAL-035   | `tests/spec/validation.rs::spec_ask_requires_question_property`                 | Ask requires a question property on the value.                           | DET-HIPP-VAL-002    |
| UT-HIPP-VAL-036   | `tests/spec/validation.rs::spec_trend_requires_full_coverage`                   | Trend assessments require full coverage.                                 | DET-HIPP-VAL-005    |
| UT-HIPP-VAL-037   | `tests/spec/validation.rs::spec_assignment_range_compliance_warning`             | Assignment range compliance fails when out of bounds.                    | DET-HIPP-VAL-003    |
| UT-HIPP-VAL-038   | `tests/spec/validation.rs::spec_undefined_reference_detection`                  | Undeclared addressee/variable/unit produce errors listing available definitions. | DET-HIPP-VAL-007    |
| UT-HIPP-VAL-039   | `tests/spec/validation.rs::spec_validation_error_suggestions`                   | Coverage gap error includes suggestion with exact missing range.         | DET-HIPP-VAL-008    |

### 4.4 UT-RT-* --- Runtime Tests

Tests in `tests/spec/execution.rs` verify DET-HIPP-RT-* elements (Engine, Environment, Executor, Evaluator, Scheduler, Session, Input Validation).

| ID         | Test function                                                             | Description                                                                   | Traces |
|------------|---------------------------------------------------------------------------|-------------------------------------------------------------------------------|---------------|
| UT-HIPP-RT-001   | `tests/spec/execution.rs::spec_not_enough_data_evaluation`                | Runtime evaluation returns NotEnoughData when history is insufficient.         | DET-HIPP-RT-004     |
| UT-HIPP-RT-002   | `tests/spec/execution.rs::spec_date_time_range_evaluation`                | Date/time valid value ranges evaluate using date/time and time-of-day semantics. | DET-HIPP-RT-004     |
| UT-HIPP-RT-003   | `tests/spec/execution.rs::spec_date_diff_evaluation`                      | Date diff expressions evaluate to quantities in requested units.              | DET-HIPP-RT-004     |
| UT-HIPP-RT-004   | `tests/spec/execution.rs::spec_meaning_of_evaluates`                      | Meaning evaluation returns the assessed meaning for the value.                | DET-HIPP-RT-004     |
| UT-HIPP-RT-005   | `tests/spec/execution.rs::spec_meaning_of_missing_value`                  | Meaning evaluation returns Missing when the source value is unknown.          | DET-HIPP-RT-004     |
| UT-HIPP-RT-006   | `tests/spec/execution.rs::spec_meaning_of_nested_assessment`              | Meaning evaluation supports nested assessments.                               | DET-HIPP-RT-004     |
| UT-HIPP-RT-007   | `tests/spec/execution.rs::spec_numeric_input_precision_rejection`         | Numeric answers must respect the decimal precision implied by valid values.   | DET-HIPP-RT-007     |
| UT-HIPP-RT-008   | `tests/spec/execution.rs::spec_runtime_execution_flow`                    | Runtime executes assignments and actions in order.                             | DET-HIPP-RT-001     |
| UT-HIPP-RT-009   | `tests/spec/execution.rs::spec_message_callback_missing_warns`            | Runtime emits a warning when a message callback is not set.                   | DET-HIPP-RT-003     |
| UT-HIPP-RT-010   | `tests/spec/execution.rs::spec_validity_reuse_timeframe`                  | Reuse timeframes prevent re-asking within the validity window.                | DET-HIPP-RT-006     |
| UT-HIPP-RT-011   | `tests/spec/execution.rs::spec_timeframe_filtering`                       | Timeframe filtering applies to statistical evaluations.                       | DET-HIPP-RT-004     |
| UT-HIPP-RT-012   | `tests/spec/execution.rs::spec_timeframe_variants`                        | Timeframe variants resolve counts over different windows.                     | DET-HIPP-RT-004     |
| UT-HIPP-RT-013   | `tests/spec/execution.rs::spec_trend_analysis_evaluates`                  | Trend analysis evaluates statistical trends over timeframes.                  | DET-HIPP-RT-004     |
| UT-HIPP-RT-014   | `tests/spec/execution.rs::spec_scheduler_next_occurrence`                 | Scheduler computes next occurrence for periods.                               | DET-HIPP-RT-005     |
| UT-HIPP-RT-015   | `tests/spec/execution.rs::spec_environment_append_only_history`           | Verifies append-only value history.                                           | DET-HIPP-RT-002     |
| UT-HIPP-RT-016   | `tests/spec/execution.rs::spec_value_history_retrieval`                   | Verifies value history retrieval with timestamps.                             | DET-HIPP-RT-002     |
| UT-HIPP-RT-017   | `tests/spec/execution.rs::spec_simulation_mode_execution`                 | Verifies simulation mode completes without delays.                            | DET-HIPP-RT-008     |

### 4.5 UT-VALUES-* --- Value Definition Tests

Tests in `tests/spec/values.rs` verify DET-HIPP-PARSER-002 (AST node hierarchy) and DET-HIPP-DOM-006 (ValueType) for value definition parsing.

| ID            | Test function                                                       | Description                                                          | Traces |
|---------------|---------------------------------------------------------------------|----------------------------------------------------------------------|---------------|
| UT-HIPP-VALUES-001  | `tests/spec/values.rs::spec_value_definition_parsing`               | Value definitions parse from fixtures.                               | DET-HIPP-PARSER-002 |
| UT-HIPP-VALUES-002  | `tests/spec/values.rs::spec_value_type_variants_parse`              | Value type variants parse correctly.                                 | DET-HIPP-PARSER-002 |
| UT-HIPP-VALUES-003  | `tests/spec/values.rs::spec_unit_property_parsing`                  | Unit properties parse in numeric values.                             | DET-HIPP-PARSER-002 |
| UT-HIPP-VALUES-004  | `tests/spec/values.rs::spec_value_timeframe_property_parsing`       | Value timeframe properties parse.                                    | DET-HIPP-PARSER-002 |
| UT-HIPP-VALUES-005  | `tests/spec/values.rs::spec_inheritance_property_parsing`           | Inheritance properties parse with overrides.                         | DET-HIPP-PARSER-002 |
| UT-HIPP-VALUES-006  | `tests/spec/values.rs::spec_documentation_property_parsing`         | Documentation properties parse in inline and block forms.            | DET-HIPP-PARSER-002 |
| UT-HIPP-VALUES-007  | `tests/spec/values.rs::spec_generic_property_parsing`               | Custom properties parse as generic properties.                       | DET-HIPP-PARSER-002 |

### 4.6 UT-UNITS-* --- Unit System Tests

Tests in `tests/spec/units.rs` verify DET-HIPP-DOM-002 (Unit) and DET-HIPP-PARSER-001 (grammar rules for units).

| ID           | Test function                                                          | Description                                                       | Traces |
|--------------|------------------------------------------------------------------------|-------------------------------------------------------------------|---------------|
| UT-HIPP-UNITS-001  | `tests/spec/units.rs::spec_custom_unit_pluralization_is_canonical`      | Custom unit pluralization canonicalizes values.                   | DET-HIPP-DOM-002    |
| UT-HIPP-UNITS-002  | `tests/spec/units.rs::spec_standard_units_still_work`                  | Standard units work in calculations.                              | DET-HIPP-DOM-002    |
| UT-HIPP-UNITS-003  | `tests/spec/units.rs::spec_custom_unit_abbreviation_is_canonical`      | Custom unit abbreviations canonicalize values.                    | DET-HIPP-DOM-002    |
| UT-HIPP-UNITS-004  | `tests/spec/units.rs::spec_builtin_units_cannot_be_redefined`          | Built-in units cannot be redefined.                               | DET-HIPP-VAL-002    |
| UT-HIPP-UNITS-005  | `tests/spec/units.rs::spec_custom_unit_quantity_parsing`               | Custom unit quantities parse with definitions.                    | DET-HIPP-PARSER-001 |
| UT-HIPP-UNITS-006  | `tests/spec/units.rs::spec_unit_conversions_within_groups`             | Unit conversions are supported within compatible groups.           | DET-HIPP-DOM-002    |
| UT-HIPP-UNITS-007  | `tests/spec/units.rs::spec_assignment_requires_unit_and_precision_match` | Assignments require matching units and precision.                 | DET-HIPP-VAL-003    |

### 4.7 UT-PERIODS-* --- Period and Plan Tests

Tests in `tests/spec/periods_plans.rs` verify DET-HIPP-PARSER-002 (AST node hierarchy for periods, plans, triggers).

| ID             | Test function                                                                      | Description                                                      | Traces |
|----------------|------------------------------------------------------------------------------------|------------------------------------------------------------------|---------------|
| UT-HIPP-PERIODS-001  | `tests/spec/periods_plans.rs::spec_period_definition_parsing`                      | Period definitions parse by name.                                | DET-HIPP-PARSER-002 |
| UT-HIPP-PERIODS-002  | `tests/spec/periods_plans.rs::spec_period_parsing_structure`                       | Period timeframe lines parse with range selectors.               | DET-HIPP-PARSER-002 |
| UT-HIPP-PERIODS-003  | `tests/spec/periods_plans.rs::spec_event_trigger_parsing`                          | Event triggers parse for change/start/periodic.                  | DET-HIPP-PARSER-002 |
| UT-HIPP-PERIODS-004  | `tests/spec/periods_plans.rs::spec_event_trigger_duration_and_offset_parsing`      | Periodic triggers parse duration and offsets.                    | DET-HIPP-PARSER-002 |
| UT-HIPP-PERIODS-005  | `tests/spec/periods_plans.rs::spec_event_block_parsing`                            | Event blocks attach statements to triggers.                      | DET-HIPP-PARSER-002 |
| UT-HIPP-PERIODS-006  | `tests/spec/periods_plans.rs::spec_event_trigger_time_of_day_parsing`              | Periodic triggers parse `at <time>` clause into `time_of_day`.   | DET-HIPP-RT-009     |
| UT-HIPP-PERIODS-007  | `tests/spec/periods_plans.rs::spec_event_trigger_weekday_with_time_parsing`        | Weekday triggers parse `at <time>` clause.                       | DET-HIPP-RT-009     |
| UT-HIPP-PERIODS-008  | `tests/spec/periods_plans.rs::spec_bare_unit_trigger_parsing`                      | Bare unit trigger `every day at 08:00 for 9 days:` parses to interval=1.0, unit=Day. | DET-HIPP-PARSER-005 |
| UT-HIPP-PERIODS-009  | `tests/spec/periods_plans.rs::spec_ordinal_trigger_parsing`                        | Ordinal trigger `every third day for 12 days:` parses to interval=3.0, unit=Day. | DET-HIPP-PARSER-005 |
| UT-HIPP-PLAN-001     | `tests/spec/periods_plans.rs::spec_after_plan_block_parsing`                       | `after plan:` block parses into `PlanBlock::AfterPlan` AST node. | DET-HIPP-RT-010     |

### 4.8 UT-ACTIONS-* --- Action and Statement Tests

Tests in `tests/spec/statements_actions.rs` verify DET-HIPP-PARSER-002 (AST node hierarchy for statements and actions).

| ID             | Test function                                                                              | Description                                                             | Traces |
|----------------|--------------------------------------------------------------------------------------------|-------------------------------------------------------------------------|---------------|
| UT-HIPP-ACTIONS-001  | `tests/spec/statements_actions.rs::spec_timeframe_block_parsing`                           | Timeframe blocks parse with nested statements.                          | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTIONS-002  | `tests/spec/statements_actions.rs::spec_question_config_parsing_and_validation`             | Question configuration parses and validates references.                 | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTIONS-003  | `tests/spec/statements_actions.rs::spec_message_expiration_parsing`                        | Message expiration attaches to information, warning, and urgent warning. | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTIONS-004  | `tests/spec/statements_actions.rs::spec_message_action_keyword_parsing`                    | Information, warning, and urgent warning are accepted as message keywords. | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTIONS-005  | `tests/spec/statements_actions.rs::spec_question_modifiers_parsing`                        | Question modifiers parse (validate/type/style/expire).                  | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTIONS-006  | `tests/spec/statements_actions.rs::spec_question_expiration_block_parsing`                 | Question expiration blocks parse with reminder statements.              | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTIONS-007  | `tests/spec/statements_actions.rs::spec_question_expiration_until_event_trigger_parsing`   | Question expiration supports until event triggers.                      | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTIONS-008  | `tests/spec/statements_actions.rs::spec_validate_answer_within_parsing`                    | Validate answer within parsing attaches to ask blocks.                  | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTIONS-009  | `tests/spec/statements_actions.rs::spec_listen_send_start_and_simple_command_parsing`      | Listen/send/start/simple command actions parse.                         | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTIONS-010  | `tests/spec/statements_actions.rs::spec_timeframe_requires_range_selector`                 | Timeframe selectors require a start and end.                            | DET-HIPP-PARSER-001 |

### 4.9 UT-ACTORS-* --- Actor and Drug Tests

Tests in `tests/spec/actors_drugs.rs` verify DET-HIPP-PARSER-002 (AST node hierarchy for addressees and drugs) and DET-HIPP-VAL-002 (semantic validation of drug definitions).

| ID            | Test function                                                                            | Description                                                         | Traces |
|---------------|------------------------------------------------------------------------------------------|---------------------------------------------------------------------|---------------|
| UT-HIPP-ACTORS-001  | `tests/spec/actors_drugs.rs::spec_drug_definition_validation`                            | Drug definition validation rejects undefined units.                 | DET-HIPP-VAL-002    |
| UT-HIPP-ACTORS-002  | `tests/spec/actors_drugs.rs::spec_addressee_group_and_contact_logic_parsing`              | Addressee groups and contact logic parse.                           | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTORS-003  | `tests/spec/actors_drugs.rs::spec_drug_interactions_parse`                                | Drug interaction properties parse.                                  | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTORS-004  | `tests/spec/actors_drugs.rs::spec_addressee_contact_details_and_sequence_order_parsing`   | Contact details and sequence contact order parse.                   | DET-HIPP-PARSER-002 |
| UT-HIPP-ACTORS-005  | `tests/spec/actors_drugs.rs::spec_drug_dosage_and_admin_rules_parsing`                   | Dosage safety and administration rules parse.                       | DET-HIPP-PARSER-002 |

### 4.10 UT-CTX-* --- Context and Expression Tests

Tests in `tests/spec/contexts_expressions.rs` verify DET-HIPP-PARSER-002 (AST node hierarchy for contexts and expressions) and DET-HIPP-RT-004 (Evaluator for context execution).

| ID          | Test function                                                                        | Description                                                          | Traces |
|-------------|--------------------------------------------------------------------------------------|----------------------------------------------------------------------|---------------|
| UT-HIPP-CTX-001   | `tests/spec/contexts_expressions.rs::spec_context_definition_parsing`                | Context definitions parse timeframe/data/value filter items.         | DET-HIPP-PARSER-002 |
| UT-HIPP-CTX-002   | `tests/spec/contexts_expressions.rs::spec_context_block_items_parsing`               | Context blocks parse data/value filters and nested statements.       | DET-HIPP-PARSER-002 |
| UT-HIPP-CTX-003   | `tests/spec/contexts_expressions.rs::spec_statistical_functions_parsing`              | Statistical function expressions parse in assignments.               | DET-HIPP-PARSER-002 |
| UT-HIPP-CTX-004   | `tests/spec/contexts_expressions.rs::spec_meaning_of_expression_parsing`             | Meaning-of expressions parse in assignments.                         | DET-HIPP-PARSER-002 |
| UT-HIPP-CTX-005   | `tests/spec/contexts_expressions.rs::spec_time_indications_parsing`                  | Time indications parse for now, weekday, and time-of-day.            | DET-HIPP-PARSER-002 |
| UT-HIPP-CTX-006   | `tests/spec/contexts_expressions.rs::spec_date_time_literals_parsing`                | Date/time literals parse for date and date-time forms.               | DET-HIPP-PARSER-002 |
| UT-HIPP-CTX-007   | `tests/spec/contexts_expressions.rs::spec_date_diff_parsing`                         | Date diff expressions parse.                                         | DET-HIPP-PARSER-002 |
| UT-HIPP-CTX-008   | `tests/spec/contexts_expressions.rs::spec_relative_time_from_now_parsing`            | Relative time expressions from now parse.                            | DET-HIPP-PARSER-002 |
| UT-HIPP-CTX-009   | `tests/spec/contexts_expressions.rs::spec_context_for_analysis_execution`            | Context for analysis executes with scoped timeframe.                 | DET-HIPP-RT-004     |

### 4.11 UT-FIX-* --- Fixture Tests

Tests in `tests/spec/fixtures.rs` verify DET-HIPP-PARSER-004 (parser entry point) across a complete multi-definition fixture.

| ID         | Test function                                                          | Description                                                  | Traces |
|------------|------------------------------------------------------------------------|--------------------------------------------------------------|---------------|
| UT-HIPP-FIX-001  | `tests/spec/fixtures.rs::spec_full_fixture_parses_core_definitions`    | Multi-definition fixtures parse core definitions.            | DET-HIPP-PARSER-004 |

### 4.12 UT-FFI-* --- FFI Interface Tests

Tests in `tests/ffi.rs` verify DET-HIPP-FFI-* elements (C-compatible API functions).

| UT ID | Test Function | Description | Traces |
|-------|--------------|-------------|---------------|
| UT-HIPP-FFI-001 | `tests/ffi.rs::ffi_parse_json_valid` | Verifies parse returns Ok JSON for valid input | DET-HIPP-FFI-001 |
| UT-HIPP-FFI-002 | `tests/ffi.rs::ffi_parse_json_invalid` | Verifies parse returns Err JSON for invalid input | DET-HIPP-FFI-001 |
| UT-HIPP-FFI-003 | `tests/ffi.rs::ffi_validate_file_valid_and_invalid` | Verifies validation, error count, and error retrieval | DET-HIPP-FFI-008 |
| UT-HIPP-FFI-004 | `tests/ffi.rs::ffi_engine_lifecycle` | Verifies engine create/free lifecycle | DET-HIPP-FFI-002 |
| UT-HIPP-FFI-005 | `tests/ffi.rs::ffi_engine_load_valid` | Verifies loading valid plan | DET-HIPP-FFI-003 |
| UT-HIPP-FFI-006 | `tests/ffi.rs::ffi_engine_load_invalid` | Verifies loading invalid plan returns error | DET-HIPP-FFI-003 |
| UT-HIPP-FFI-007 | `tests/ffi.rs::ffi_get_periods` | Verifies period retrieval as JSON | DET-HIPP-FFI-009 |
| UT-HIPP-FFI-008 | `tests/ffi.rs::ffi_set_time` | Verifies setting engine time | DET-HIPP-FFI-007 |
| UT-HIPP-FFI-009 | `tests/ffi.rs::ffi_enable_simulation` | Verifies enabling simulation mode | DET-HIPP-FFI-010 |
| UT-HIPP-FFI-010 | `tests/ffi.rs::ffi_stop` | Verifies stop signal | DET-HIPP-FFI-011 |

### 4.13 UT-FMT-* --- Formatter Tests

Tests in `tests/formatter.rs` verify DET-HIPP-FMT-001 (formatter `format_script`).

| UT ID | Test Function | Description | Traces |
|-------|--------------|-------------|---------------|
| UT-HIPP-FMT-001 | `tests/formatter.rs::formatter_round_trip_parse_format_parse` | Round-trip parse-format-parse | DET-HIPP-FMT-001 |
| UT-HIPP-FMT-002 | `tests/formatter.rs::formatter_handles_all_definition_types` | All definition types format correctly | DET-HIPP-FMT-001 |

---

## 5. Coverage Summary

| Module group | DDR elements covered                                   | Test count |
|--------------|--------------------------------------------------------|------------|
| PARSER       | DET-HIPP-PARSER-001, DET-HIPP-PARSER-002, DET-HIPP-PARSER-003, DET-HIPP-PARSER-004, DET-HIPP-PARSER-005, DET-HIPP-PARSER-006 | 13     |
| DOM          | DET-HIPP-DOM-001, DET-HIPP-DOM-002, DET-HIPP-DOM-006                     | 5          |
| VAL          | DET-HIPP-VAL-001 through DET-HIPP-VAL-008                          | 39         |
| RT           | DET-HIPP-RT-001, DET-HIPP-RT-002, DET-HIPP-RT-003, DET-HIPP-RT-004, DET-HIPP-RT-005, DET-HIPP-RT-006, DET-HIPP-RT-007, DET-HIPP-RT-008 | 17   |
| VALUES       | DET-HIPP-PARSER-002                                          | 7          |
| UNITS        | DET-HIPP-DOM-002, DET-HIPP-PARSER-001, DET-HIPP-VAL-002, DET-HIPP-VAL-003      | 7          |
| PERIODS      | DET-HIPP-PARSER-002, DET-HIPP-PARSER-005, DET-HIPP-RT-010                | 7          |
| PLAN         | DET-HIPP-RT-010                                              | 1          |
| ACTIONS      | DET-HIPP-PARSER-001, DET-HIPP-PARSER-002                           | 10         |
| ACTORS       | DET-HIPP-PARSER-002, DET-HIPP-VAL-002                              | 5          |
| CTX          | DET-HIPP-PARSER-002, DET-HIPP-RT-004                               | 9          |
| FIX          | DET-HIPP-PARSER-004                                          | 1          |
| FFI          | DET-HIPP-FFI-001, DET-HIPP-FFI-002, DET-HIPP-FFI-003, DET-HIPP-FFI-007, DET-HIPP-FFI-008, DET-HIPP-FFI-009, DET-HIPP-FFI-010, DET-HIPP-FFI-011 | 10 |
| FMT          | DET-HIPP-FMT-001                                             | 2          |
| **Total**    |                                                        | **133**    |

### Gaps

| DDR element     | Status   | Notes                                                                |
|-----------------|----------|----------------------------------------------------------------------|
| DET-HIPP-FFI-001..19  | Partial  | UT-HIPP-FFI-001..10 cover DET-HIPP-FFI-001, -02, -03, -07, -08, -09, -10, -11. Remaining FFI functions (DET-HIPP-FFI-004..06, -12..19) are tested at the integration level via Swift/C bindings. |
| DET-HIPP-RT-002       | Covered  | UT-HIPP-RT-015 and UT-HIPP-RT-016 verify append-only history and value history retrieval. |
| DET-HIPP-RT-008       | Covered  | UT-HIPP-RT-017 verifies simulation mode execution without real-time delays. |
| DET-HIPP-FMT-001      | Covered  | UT-HIPP-FMT-001 and UT-HIPP-FMT-002 verify round-trip formatting and all definition types. |

---

## Revision History

| Version | Date       | Author | Changes         |
|---------|------------|--------|-----------------|
| 1.0     | 2026-03-20 | ---    | Initial release |
| 1.1     | 2026-03-20 | ---    | Added UT-HIPP-FFI-001..10 (FFI tests), UT-HIPP-RT-015..17 (environment/history/simulation), UT-HIPP-FMT-001..02 (formatter). Closed DET-HIPP-FFI, DET-HIPP-RT-002, DET-HIPP-RT-008, DET-HIPP-FMT-001 gaps. |
| 1.2     | 2026-03-23 | ---    | Added UT-HIPP-PERIODS-006, UT-HIPP-PERIODS-007 for time-of-day parsing (DET-HIPP-RT-009). |
| 1.3     | 2026-03-23 | ---    | Added UT-HIPP-PLAN-001 for `after plan:` block parsing (DET-HIPP-RT-010). |
| 1.4     | 2026-03-23 | ---    | Added UT-HIPP-PERIODS-008, UT-HIPP-PERIODS-009 for bare unit and ordinal trigger parsing (DET-HIPP-PARSER-005). |
| 1.5     | 2026-03-23 | ---    | Added UT-HIPP-PARSER-013 (parse error humanization, DET-HIPP-PARSER-006), UT-HIPP-VAL-038 (undefined references, DET-HIPP-VAL-007), UT-HIPP-VAL-039 (suggested fixes, DET-HIPP-VAL-008). |
| 1.6     | 2026-04-19 | ---    | Renamed all UT-* test IDs to canonical UT-HIPP-* form and updated `**Traces to:**` references from DDR-* to DET-HIPP-* for V-Model validator/generator compatibility. |
