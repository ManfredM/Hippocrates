# System Test Plan

| Field         | Value                              |
|---------------|------------------------------------|
| Document ID   | ST                                 |
| V-Model Level | System Testing (Right side)        |
| Verifies      | System Requirements (REQ-*)        |

## 1. Purpose

Verify the complete Hippocrates engine against system requirements. Each REQ-* from the language specification has at least one corresponding test case (ST-*). This plan is derived from the specification traceability matrix.

## 2. Test Environment

| Item           | Detail                                  |
|----------------|-----------------------------------------|
| Language       | Rust                                    |
| Test framework | `cargo test`                            |
| Test location  | `hippocrates_engine/tests/spec/`        |
| Command        | `cargo test --test spec`                |

## 3. Entry / Exit Criteria

**Entry criteria**
- Engine compiles successfully (`cargo build`).
- All dependencies resolved (`Cargo.lock` up to date).

**Exit criteria**
- All ST-* test cases pass when running `cargo test --test spec`.

## 4. Test Cases

### 4.1 — §2 Language Principles

| ID       | REQ       | Test function                                                          | Description                                                  | Pass criteria       |
|----------|-----------|------------------------------------------------------------------------|--------------------------------------------------------------|---------------------|
| ST-2-01  | REQ-2-01  | `tests/spec/grammar.rs::spec_identifiers_require_angle_brackets`       | Identifiers must use angle brackets.                         | Test passes         |
| ST-2-02  | REQ-2-02  | `tests/spec/grammar.rs::spec_string_literal_rejects_angle_brackets`    | String literals must not contain angle brackets.             | Test passes         |
| ST-2-03  | REQ-2-03  | `tests/spec/grammar.rs::spec_no_comparison_operators`                  | Comparison operators are not supported; use ranges.          | Test passes         |
| ST-2-04  | REQ-2-04  | `tests/spec/grammar.rs::spec_block_requires_newline_after_colon`       | Block openings require a newline and indented block.         | Test passes         |

### 4.2 — §3.1 Basic Elements

| ID         | REQ         | Test function                                                              | Description                                                       | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------|-------------------------------------------------------------------|---------------------|
| ST-3.1-01  | REQ-3.1-01  | `tests/spec/contexts_expressions.rs::spec_time_indications_parsing`        | Time indications parse for now, weekday, and time-of-day.         | Test passes         |
| ST-3.1-02  | REQ-3.1-02  | `tests/spec/contexts_expressions.rs::spec_relative_time_from_now_parsing`  | Relative time expressions from now parse.                         | Test passes         |
| ST-3.1-03  | REQ-3.1-03  | `tests/spec/grammar.rs::spec_inline_colon_requires_block`                  | Inline ':' forms are only allowed where explicitly shown.         | Test passes         |
| ST-3.1-04  | REQ-3.1-04  | `tests/spec/contexts_expressions.rs::spec_date_time_literals_parsing`      | Date/time literals parse for date and date-time forms.            | Test passes         |

### 4.3 — §3.2 Units and Quantities

| ID         | REQ         | Test function                                                            | Description                                                  | Pass criteria       |
|------------|-------------|--------------------------------------------------------------------------|--------------------------------------------------------------|---------------------|
| ST-3.2-01  | REQ-3.2-01  | `tests/spec/units.rs::spec_custom_unit_pluralization_is_canonical`        | Custom unit pluralization canonicalizes values.               | Test passes         |
| ST-3.2-02  | REQ-3.2-02  | `tests/spec/units.rs::spec_standard_units_still_work`                    | Standard units work in calculations.                         | Test passes         |
| ST-3.2-03  | REQ-3.2-03  | `tests/spec/units.rs::spec_custom_unit_abbreviation_is_canonical`        | Custom unit abbreviations canonicalize values.               | Test passes         |
| ST-3.2-04  | REQ-3.2-04  | `tests/spec/units.rs::spec_custom_unit_quantity_parsing`                  | Custom unit quantities parse with definitions.               | Test passes         |
| ST-3.2-05  | REQ-3.2-05  | `tests/spec/grammar.rs::spec_unitless_numeric_literal_fails`             | Numeric literals must include units.                         | Test passes         |

### 4.4 — §3.3 Program Structure

| ID         | REQ         | Test function                                                              | Description                                                  | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------|--------------------------------------------------------------|---------------------|
| ST-3.3-01  | REQ-3.3-01  | `tests/spec/fixtures.rs::spec_full_fixture_parses_core_definitions`        | Multi-definition fixtures parse core definitions.            | Test passes         |

### 4.5 — §3.4 Values

| ID         | REQ         | Test function                                                              | Description                                                              | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------------|
| ST-3.4-01  | REQ-3.4-01  | `tests/spec/values.rs::spec_value_definition_parsing`                      | Value definitions parse from fixtures.                                   | Test passes         |
| ST-3.4-02  | REQ-3.4-02  | `tests/spec/values.rs::spec_value_type_variants_parse`                     | Value type variants parse correctly.                                     | Test passes         |
| ST-3.4-03  | REQ-3.4-03  | `tests/spec/values.rs::spec_unit_property_parsing`                         | Unit properties parse in numeric values.                                 | Test passes         |
| ST-3.4-04  | REQ-3.4-04  | `tests/spec/values.rs::spec_value_timeframe_property_parsing`              | Value timeframe properties parse.                                        | Test passes         |
| ST-3.4-05  | REQ-3.4-05  | `tests/spec/values.rs::spec_inheritance_property_parsing`                  | Inheritance properties parse with overrides.                             | Test passes         |
| ST-3.4-06  | REQ-3.4-06  | `tests/spec/values.rs::spec_documentation_property_parsing`                | Documentation properties parse in inline and block forms.                | Test passes         |
| ST-3.4-07  | REQ-3.4-07  | `tests/spec/values.rs::spec_generic_property_parsing`                      | Custom properties parse as generic properties.                           | Test passes         |
| ST-3.4-08  | REQ-3.4-08  | `tests/spec/values.rs::spec_value_type_variants_parse`                     | Date/time value type parses.                                             | Test passes         |
| ST-3.4-09  | REQ-3.4-09  | `tests/spec/grammar.rs::spec_meaning_assessments_not_allowed_in_plans`     | Meaning assessments are only allowed in value definition blocks.         | Test passes         |
| ST-3.4-10  | REQ-3.4-10  | `tests/spec/grammar.rs::spec_meaning_requires_target_identifier`           | Meaning properties require an explicit target identifier.                | Test passes         |
| ST-3.4-11  | REQ-3.4-11  | `tests/spec/grammar.rs::spec_meaning_requires_valid_meanings`              | Meaning properties must declare valid meanings.                          | Test passes         |
| ST-3.4-12  | REQ-3.4-12  | `tests/spec/grammar.rs::spec_meaning_labels_require_identifiers`           | Meaning labels must be identifiers (angle brackets).                     | Test passes         |
| ST-3.4-13  | REQ-3.4-13  | `tests/spec/validation.rs::spec_enum_valid_values_require_identifiers`     | Enumeration valid values are identifiers (angle brackets).               | Test passes         |

### 4.6 — §3.5 Periods and Plans

| ID         | REQ         | Test function                                                            | Description                                                  | Pass criteria       |
|------------|-------------|--------------------------------------------------------------------------|--------------------------------------------------------------|---------------------|
| ST-3.5-01  | REQ-3.5-01  | `tests/spec/periods_plans.rs::spec_period_definition_parsing`            | Period definitions parse by name.                            | Test passes         |
| ST-3.5-02  | REQ-3.5-02  | `tests/spec/periods_plans.rs::spec_period_parsing_structure`             | Period timeframe lines parse with range selectors.           | Test passes         |

### 4.7 — §3.6 Statements, Assessments, and Ranges

| ID         | REQ         | Test function                                                                    | Description                                                              | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------------|
| ST-3.6-01  | REQ-3.6-01  | `tests/spec/statements_actions.rs::spec_timeframe_block_parsing`                 | Timeframe blocks parse with nested statements.                           | Test passes         |
| ST-3.6-02  | REQ-3.6-02  | `tests/spec/statements_actions.rs::spec_timeframe_requires_range_selector`       | Timeframe selectors require a start and end.                             | Test passes         |
| ST-3.6-03  | REQ-3.6-03  | `tests/spec/validation.rs::spec_timeframe_selector_requires_period_definition`   | Timeframe selector identifiers must refer to defined periods.            | Test passes         |
| ST-3.6-04  | REQ-3.6-04  | `tests/spec/grammar.rs::spec_block_statements_require_period`                    | Statements inside blocks must terminate with a period.                   | Test passes         |
| ST-3.6-05  | REQ-3.6-05  | `tests/spec/grammar.rs::spec_blocks_require_colon`                               | Blocks must be introduced with a colon.                                  | Test passes         |

### 4.8 — §3.7 Actions and Questions

| ID         | REQ         | Test function                                                                            | Description                                                                              | Pass criteria       |
|------------|-------------|------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------|---------------------|
| ST-3.7-01  | REQ-3.7-01  | `tests/spec/statements_actions.rs::spec_question_config_parsing_and_validation`           | Question configuration parses and validates references.                                  | Test passes         |
| ST-3.7-02  | REQ-3.7-02  | `tests/spec/statements_actions.rs::spec_message_expiration_parsing`                       | Message expiration attaches to information, warning, and urgent warning actions.          | Test passes         |
| ST-3.7-03  | REQ-3.7-03  | `tests/spec/statements_actions.rs::spec_question_modifiers_parsing`                       | Question modifiers parse (validate/type/style/expire).                                   | Test passes         |
| ST-3.7-04  | REQ-3.7-04  | `tests/spec/statements_actions.rs::spec_validate_answer_within_parsing`                   | Validate answer within parsing attaches to ask blocks.                                   | Test passes         |
| ST-3.7-05  | REQ-3.7-05  | `tests/spec/statements_actions.rs::spec_listen_send_start_and_simple_command_parsing`     | Listen/send/start/simple command actions parse.                                          | Test passes         |
| ST-3.7-06  | REQ-3.7-06  | `tests/spec/statements_actions.rs::spec_question_expiration_block_parsing`                | Question expiration blocks parse with reminder statements.                               | Test passes         |
| ST-3.7-07  | REQ-3.7-07  | `tests/spec/statements_actions.rs::spec_question_expiration_until_event_trigger_parsing`  | Question expiration supports until event triggers.                                       | Test passes         |
| ST-3.7-08  | REQ-3.7-08  | `tests/spec/statements_actions.rs::spec_message_action_keyword_parsing`                   | Information, warning, and urgent warning are accepted as message action keywords.         | Test passes         |
| ST-3.7-09  | REQ-3.7-09  | `tests/spec/statements_actions.rs::spec_message_expiration_parsing`                       | Message actions accept semicolon-separated addressee lists.                              | Test passes         |
| ST-3.7-10  | REQ-3.7-10  | `tests/spec/periods_plans.rs::spec_after_plan_block_parsing`                              | `after plan:` block parses into AST and executes after event loop.                       | Test passes         |

### 4.9 — §3.8 Events and Timing

| ID         | REQ         | Test function                                                                    | Description                                                      | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------------|------------------------------------------------------------------|---------------------|
| ST-3.8-01  | REQ-3.8-01  | `tests/spec/periods_plans.rs::spec_event_trigger_parsing`                        | Event triggers parse for change/start/periodic.                  | Test passes         |
| ST-3.8-02  | REQ-3.8-02  | `tests/spec/periods_plans.rs::spec_event_block_parsing`                          | Event blocks attach statements to triggers.                      | Test passes         |
| ST-3.8-03  | REQ-3.8-03  | `tests/spec/execution.rs::spec_scheduler_next_occurrence`                        | Scheduler computes next occurrence for periods.                  | Test passes         |
| ST-3.8-04  | REQ-3.8-04  | `tests/spec/periods_plans.rs::spec_event_trigger_duration_and_offset_parsing`    | Periodic triggers parse duration and offsets.                    | Test passes         |
| ST-3.8-05  | REQ-3.8-05  | `tests/spec/periods_plans.rs::spec_event_trigger_time_of_day_parsing`            | Periodic triggers parse `at <time>` clause.                      | Test passes         |
| ST-3.8-06  | REQ-3.8-06  | `tests/integration/simulation.rs::test_period_based_repetition_within_duration`   | Period-based triggers fire at every occurrence within duration.   | All occurrences fire |
| ST-3.8-07  | REQ-3.8-07  | `tests/spec/periods_plans.rs::spec_bare_unit_trigger_parsing`, `tests/spec/periods_plans.rs::spec_ordinal_trigger_parsing` | Bare unit (`every day`) and ordinal (`every third day`) triggers parse correctly to numeric intervals. | Test passes |

### 4.10 — §3.9 Communication & Actors

| ID         | REQ         | Test function                                                                            | Description                                                      | Pass criteria       |
|------------|-------------|------------------------------------------------------------------------------------------|------------------------------------------------------------------|---------------------|
| ST-3.9-01  | REQ-3.9-01  | `tests/spec/actors_drugs.rs::spec_addressee_group_and_contact_logic_parsing`              | Addressee groups and contact logic parse.                        | Test passes         |
| ST-3.9-02  | REQ-3.9-02  | `tests/spec/actors_drugs.rs::spec_addressee_contact_details_and_sequence_order_parsing`   | Contact details and sequence ordering parse.                     | Test passes         |

### 4.11 — §3.10 Medication

| ID          | REQ          | Test function                                                              | Description                                                  | Pass criteria       |
|-------------|--------------|----------------------------------------------------------------------------|--------------------------------------------------------------|---------------------|
| ST-3.10-01  | REQ-3.10-01  | `tests/spec/actors_drugs.rs::spec_drug_definition_validation`              | Drug definition validation rejects undefined units.          | Test passes         |
| ST-3.10-02  | REQ-3.10-02  | `tests/spec/actors_drugs.rs::spec_drug_interactions_parse`                 | Drug interaction properties parse.                           | Test passes         |
| ST-3.10-03  | REQ-3.10-03  | `tests/spec/actors_drugs.rs::spec_drug_dosage_and_admin_rules_parsing`     | Dosage safety and administration rules parse.                | Test passes         |

### 4.12 — §3.11 Data Contexts

| ID          | REQ          | Test function                                                                  | Description                                                              | Pass criteria       |
|-------------|--------------|--------------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------------|
| ST-3.11-01  | REQ-3.11-01  | `tests/spec/contexts_expressions.rs::spec_context_definition_parsing`          | Context definitions parse timeframe/data/value filter items.             | Test passes         |
| ST-3.11-02  | REQ-3.11-02  | `tests/spec/contexts_expressions.rs::spec_context_block_items_parsing`         | Context blocks parse data/value filters and nested statements.           | Test passes         |
| ST-3.11-03  | REQ-3.11-03  | `tests/spec/contexts_expressions.rs::spec_context_for_analysis_execution`      | Context for analysis executes with scoped timeframe.                     | Test passes         |

### 4.13 — §3.12 Expressions and Statistical Analysis

| ID          | REQ          | Test function                                                                          | Description                                                              | Pass criteria       |
|-------------|--------------|----------------------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------------|
| ST-3.12-01  | REQ-3.12-01  | `tests/spec/contexts_expressions.rs::spec_statistical_functions_parsing`                | Statistical function expressions parse in assignments.                   | Test passes         |
| ST-3.12-02  | REQ-3.12-02  | `tests/spec/execution.rs::spec_timeframe_filtering`                                    | Timeframe filtering applies to statistical evaluations.                  | Test passes         |
| ST-3.12-03  | REQ-3.12-03  | `tests/spec/execution.rs::spec_timeframe_variants`                                     | Timeframe variants resolve counts over different windows.                | Test passes         |
| ST-3.12-04  | REQ-3.12-04  | `tests/spec/execution.rs::spec_trend_analysis_evaluates`                                | Trend analysis evaluates statistical trends over timeframes.             | Test passes         |
| ST-3.12-05  | REQ-3.12-05  | `tests/spec/validation.rs::spec_statistical_functions_require_timeframe_context`        | Statistical functions require an analysis timeframe context.             | Test passes         |
| ST-3.12-06  | REQ-3.12-06  | `tests/spec/contexts_expressions.rs::spec_date_diff_parsing`                           | Date diff expressions parse.                                             | Test passes         |
| ST-3.12-07  | REQ-3.12-07  | `tests/spec/contexts_expressions.rs::spec_meaning_of_expression_parsing`               | Meaning-of expressions parse in assignments.                             | Test passes         |

### 4.14 — §4.1 Core Unit Groups and Conversion

| ID         | REQ         | Test function                                                                    | Description                                                                  | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------------|------------------------------------------------------------------------------|---------------------|
| ST-4.1-01  | REQ-4.1-01  | `tests/spec/units.rs::spec_builtin_units_cannot_be_redefined`                    | Built-in units cannot be redefined.                                          | Test passes         |
| ST-4.1-02  | REQ-4.1-02  | `tests/spec/units.rs::spec_unit_conversions_within_groups`                       | Unit conversions are supported within compatible groups.                     | Test passes         |
| ST-4.1-03  | REQ-4.1-03  | `tests/spec/units.rs::spec_assignment_requires_unit_and_precision_match`         | Calculations and assignments require matching units and precision.           | Test passes         |

### 4.15 — §4.2 Required Properties

| ID         | REQ         | Test function                                                            | Description                                                      | Pass criteria       |
|------------|-------------|--------------------------------------------------------------------------|------------------------------------------------------------------|---------------------|
| ST-4.2-01  | REQ-4.2-01  | `tests/spec/validation.rs::spec_unit_requirement_validation`             | Numeric valid values require units.                              | Test passes         |
| ST-4.2-02  | REQ-4.2-02  | `tests/spec/validation.rs::spec_unitless_assess_fails`                   | Assessment ranges require units.                                 | Test passes         |
| ST-4.2-03  | REQ-4.2-03  | `tests/spec/validation.rs::spec_unitless_definition_fails`               | Numeric definitions require units.                               | Test passes         |
| ST-4.2-04  | REQ-4.2-04  | `tests/spec/validation.rs::spec_ask_requires_question_property`          | Ask requires a question property on the value.                   | Test passes         |
| ST-4.2-05  | REQ-4.2-05  | `tests/spec/validation.rs::spec_validation_error_line_number`            | Unit requirement errors report line numbers.                     | Test passes         |
| ST-4.2-06  | REQ-4.2-06  | `tests/spec/validation.rs::spec_missing_valid_values_fails`              | Numbers and enumerations must define valid values.               | Test passes         |
| ST-4.2-07a | REQ-4.2-07  | `tests/spec/validation.rs::spec_valid_values_ranges_do_not_overlap`      | Valid value ranges must not overlap.                             | Test passes         |
| ST-4.2-07b | REQ-4.2-07  | `tests/spec/validation.rs::spec_valid_values_datetime_ranges_do_not_overlap` | Date/time valid value ranges must not overlap.               | Test passes         |

### 4.16 — §4.3 Data Flow and Validity

| ID         | REQ         | Test function                                                                        | Description                                                                      | Pass criteria       |
|------------|-------------|--------------------------------------------------------------------------------------|----------------------------------------------------------------------------------|---------------------|
| ST-4.3-01  | REQ-4.3-01  | `tests/spec/validation.rs::spec_data_flow_use_before_assignment_fails`               | Values cannot be used before assignment.                                         | Test passes         |
| ST-4.3-02  | REQ-4.3-02  | `tests/spec/validation.rs::spec_calculation_does_not_initialize_value`               | Calculation properties do not seed values.                                       | Test passes         |
| ST-4.3-03  | REQ-4.3-03  | `tests/spec/validation.rs::spec_statistical_functions_do_not_require_local_init`     | Statistical functions do not require local initialization.                       | Test passes         |
| ST-4.3-04  | REQ-4.3-04  | `tests/spec/validation.rs::spec_listen_and_context_initialize_values`                | Listen for and context data initialize values.                                   | Test passes         |
| ST-4.3-05a | REQ-4.3-05  | `tests/spec/validation.rs::spec_meaning_of_requires_question_when_uninitialized`     | Meaning-of expressions require an askable value when not initialized.            | Test passes         |
| ST-4.3-05b | REQ-4.3-05  | `tests/spec/validation.rs::spec_meaning_of_allows_question_when_uninitialized`       | Meaning-of is allowed when the value is askable.                                 | Test passes         |

### 4.17 — §4.4 Assessment Coverage

| ID         | REQ         | Test function                                                                  | Description                                                                          | Pass criteria       |
|------------|-------------|--------------------------------------------------------------------------------|---------------------------------------------------------------------------------------|---------------------|
| ST-4.4-01  | REQ-4.4-01  | `tests/spec/validation.rs::spec_meaning_coverage_gaps_integer`                 | Meaning ranges must cover valid values (integer gaps).                                | Test passes         |
| ST-4.4-02  | REQ-4.4-02  | `tests/spec/validation.rs::spec_meaning_coverage_gaps_float`                   | Meaning ranges must cover valid values (float gaps).                                  | Test passes         |
| ST-4.4-03  | REQ-4.4-03  | `tests/spec/validation.rs::spec_meaning_coverage_disjoint_ranges_ok`           | Disjoint valid ranges are allowed when fully covered.                                 | Test passes         |
| ST-4.4-04  | REQ-4.4-04  | `tests/spec/validation.rs::spec_validator_numeric_overlap`                     | Overlapping numeric assessment ranges are invalid.                                    | Test passes         |
| ST-4.4-05  | REQ-4.4-05  | `tests/spec/validation.rs::spec_validator_enum_duplicate`                      | Duplicate enumeration cases are invalid.                                              | Test passes         |
| ST-4.4-06  | REQ-4.4-06  | `tests/spec/validation.rs::spec_validator_integer_gap_message`                 | Gap detection reports missing integer spans.                                          | Test passes         |
| ST-4.4-07  | REQ-4.4-07  | `tests/spec/validation.rs::spec_validator_float_gap_message`                   | Gap detection reports missing float spans.                                            | Test passes         |
| ST-4.4-08  | REQ-4.4-08  | `tests/spec/validation.rs::spec_precision_gaps`                                | Coverage gaps respect precision for float and integer ranges.                         | Test passes         |
| ST-4.4-09  | REQ-4.4-09  | `tests/spec/validation.rs::spec_range_overlap`                                 | Overlapping ranges are rejected.                                                      | Test passes         |
| ST-4.4-10  | REQ-4.4-10  | `tests/spec/validation.rs::spec_reproduce_missing_error`                       | Missing coverage yields a validation error.                                           | Test passes         |
| ST-4.4-11  | REQ-4.4-11  | `tests/spec/validation.rs::spec_trend_requires_full_coverage`                  | Trend assessments require full coverage.                                              | Test passes         |
| ST-4.4-12  | REQ-4.4-12  | `tests/spec/validation.rs::spec_precision_consistency`                         | Numeric valid value ranges use consistent precision across bounds and intervals.      | Test passes         |
| ST-4.4-13  | REQ-4.4-13  | `tests/spec/validation.rs::spec_meaning_valid_meanings_must_be_used`           | Valid meanings must be fully used across meaning assessments.                          | Test passes         |
| ST-4.4-14  | REQ-4.4-14  | `tests/spec/validation.rs::spec_meaning_invalid_label_rejected`                | Meaning labels must be drawn from declared valid meanings.                            | Test passes         |

### 4.18 — §4.5 Range Compliance (Pre-Run Validation)

| ID         | REQ         | Test function                                                              | Description                                                      | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------|------------------------------------------------------------------|---------------------|
| ST-4.5-01  | REQ-4.5-01  | `tests/spec/validation.rs::spec_interval_creation_and_math`                | Interval math supports range compliance checks.                  | Test passes         |
| ST-4.5-02  | REQ-4.5-02  | `tests/spec/validation.rs::spec_assignment_range_compliance_warning`       | Assignment range compliance fails when out of bounds.            | Test passes         |

### 4.19 — §4.6 Data Sufficiency

| ID         | REQ         | Test function                                                                    | Description                                                              | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------------|
| ST-4.6-01  | REQ-4.6-01  | `tests/spec/validation.rs::spec_validator_requires_not_enough_data_case`          | Timeframe calculations require Not enough data handling.                 | Test passes         |
| ST-4.6-02  | REQ-4.6-02  | `tests/spec/validation.rs::spec_validator_passes_with_not_enough_data`            | Not enough data handling satisfies sufficiency.                          | Test passes         |
| ST-4.6-03  | REQ-4.6-03  | `tests/spec/execution.rs::spec_not_enough_data_evaluation`                       | Runtime evaluation returns NotEnoughData when history is insufficient.   | Test passes         |
| ST-4.6-04  | REQ-4.6-04  | `tests/spec/validation.rs::spec_not_enough_data_requires_statistical_target`     | Not enough data is only allowed for statistical assessments.             | Test passes         |

### 4.20 — §4.7 Date/Time Semantics

| ID         | REQ         | Test function                                                        | Description                                                                      | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------|----------------------------------------------------------------------------------|---------------------|
| ST-4.7-01  | REQ-4.7-01  | `tests/spec/execution.rs::spec_date_time_range_evaluation`           | Date/time valid value ranges evaluate using date/time and time-of-day semantics. | Test passes         |
| ST-4.7-02  | REQ-4.7-02  | `tests/spec/execution.rs::spec_date_diff_evaluation`                 | Date diff expressions evaluate to quantities in requested units.                 | Test passes         |

### 4.21 — §5 Execution Model

| ID       | REQ       | Test function                                                          | Description                                                                          | Pass criteria       |
|----------|-----------|------------------------------------------------------------------------|--------------------------------------------------------------------------------------|---------------------|
| ST-5-01  | REQ-5-01  | `tests/spec/execution.rs::spec_runtime_execution_flow`                 | Runtime executes assignments and actions in order.                                   | Test passes         |
| ST-5-02  | REQ-5-02  | `tests/spec/execution.rs::spec_validity_reuse_timeframe`               | Reuse timeframes prevent re-asking within the validity window.                       | Test passes         |
| ST-5-03  | REQ-5-03  | `tests/spec/execution.rs::spec_message_callback_missing_warns`         | Runtime emits a warning when a message action executes without a message callback.   | Test passes         |
| ST-5-04  | REQ-5-04  | `tests/spec/execution.rs::spec_simulation_mode_execution`              | Simulation mode executes without real-time delays.                                   | Test completes in under 10 seconds |
| ST-5-05  | REQ-5-05  | `tests/integration/simulation.rs::test_time_pinned_periodic_trigger`   | Time-pinned triggers fire at specified time, not plan start time.                    | Events at 08:00     |

### 4.22 — §5.1 Validation Logic

| ID         | REQ         | Test function                                                        | Description                                              | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------|----------------------------------------------------------|---------------------|
| ST-5.1-01  | REQ-5.1-01  | `tests/spec/validation.rs::spec_validate_plan_fixture_suite`         | Full-plan validation passes for a complete plan.         | Test passes         |

### 4.23 — §5.2 Input Validation

| ID         | REQ         | Test function                                                              | Description                                                                      | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------|----------------------------------------------------------------------------------|---------------------|
| ST-5.2-01  | REQ-5.2-01  | `tests/spec/execution.rs::spec_numeric_input_precision_rejection`          | Numeric answers must respect the decimal precision implied by valid values.       | Test passes         |

### 4.24 — §5.3 Meaning Evaluation

| ID         | REQ         | Test function                                                          | Description                                                      | Pass criteria       |
|------------|-------------|------------------------------------------------------------------------|------------------------------------------------------------------|---------------------|
| ST-5.3-01  | REQ-5.3-01  | `tests/spec/execution.rs::spec_meaning_of_evaluates`                   | Meaning evaluation returns the assessed meaning.                 | Test passes         |
| ST-5.3-02  | REQ-5.3-02  | `tests/spec/execution.rs::spec_meaning_of_missing_value`               | Meaning evaluation returns Missing when the source value is unknown. | Test passes     |
| ST-5.3-03  | REQ-5.3-03  | `tests/spec/execution.rs::spec_meaning_of_nested_assessment`           | Meaning evaluation supports nested assessments.                  | Test passes         |

## 5. Coverage Summary

| Metric                | Count |
|-----------------------|-------|
| Total ST-* test cases | 96    |
| REQ-* entries covered | 94    |
| Noted gaps            | 0     |

Two REQ IDs (REQ-4.2-07 and REQ-4.3-05) each map to two distinct test functions. REQ-3.8-07 maps to two test functions (bare unit and ordinal). This produces 96 test cases from 94 unique requirement IDs. All requirements from the traceability matrix are covered; no gaps are noted.

## Revision History

| Rev | Date       | Author | Description            |
|-----|------------|--------|------------------------|
| 1.0 | 2026-03-20 | —      | Initial version        |
| 1.1 | 2026-03-20 | —      | Added ST-5-04 (simulation mode, REQ-5-04). |
| 1.2 | 2026-03-23 | —      | Added ST-3.8-05, ST-3.8-06, ST-5-05 (time-of-day and period repetition). |
| 1.3 | 2026-03-23 | —      | Added ST-3.7-10 (`after plan:` block). |
| 1.4 | 2026-03-23 | —      | Added ST-3.8-07 (bare unit and ordinal trigger parsing, REQ-3.8-07). |
