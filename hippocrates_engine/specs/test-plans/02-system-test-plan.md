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

| ID       | Traces       | Test function                                                          | Description                                                  | Pass criteria       |
|----------|-----------|------------------------------------------------------------------------|--------------------------------------------------------------|---------------------|
| ST-HIPP-LANG-001  | REQ-HIPP-LANG-001  | `tests/spec/grammar.rs::spec_identifiers_require_angle_brackets`       | Identifiers must use angle brackets.                         | Test passes         |
| ST-HIPP-LANG-002  | REQ-HIPP-LANG-002  | `tests/spec/grammar.rs::spec_string_literal_rejects_angle_brackets`    | String literals must not contain angle brackets.             | Test passes         |
| ST-HIPP-LANG-003  | REQ-HIPP-LANG-003  | `tests/spec/grammar.rs::spec_no_comparison_operators`                  | Comparison operators are not supported; use ranges.          | Test passes         |
| ST-HIPP-LANG-004  | REQ-HIPP-LANG-004  | `tests/spec/grammar.rs::spec_block_requires_newline_after_colon`       | Block openings require a newline and indented block.         | Test passes         |

### 4.2 — §3.1 Basic Elements

| ID         | Traces         | Test function                                                              | Description                                                       | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------|-------------------------------------------------------------------|---------------------|
| ST-HIPP-BASIC-001  | REQ-HIPP-BASIC-001  | `tests/spec/contexts_expressions.rs::spec_time_indications_parsing`        | Time indications parse for now, weekday, and time-of-day.         | Test passes         |
| ST-HIPP-BASIC-002  | REQ-HIPP-BASIC-002  | `tests/spec/contexts_expressions.rs::spec_relative_time_from_now_parsing`  | Relative time expressions from now parse.                         | Test passes         |
| ST-HIPP-BASIC-003  | REQ-HIPP-BASIC-003  | `tests/spec/grammar.rs::spec_inline_colon_requires_block`                  | Inline ':' forms are only allowed where explicitly shown.         | Test passes         |
| ST-HIPP-BASIC-004  | REQ-HIPP-BASIC-004  | `tests/spec/contexts_expressions.rs::spec_date_time_literals_parsing`      | Date/time literals parse for date and date-time forms.            | Test passes         |

### 4.3 — §3.2 Units and Quantities

| ID         | Traces         | Test function                                                            | Description                                                  | Pass criteria       |
|------------|-------------|--------------------------------------------------------------------------|--------------------------------------------------------------|---------------------|
| ST-HIPP-UNITS-001  | REQ-HIPP-UNITS-001  | `tests/spec/units.rs::spec_custom_unit_pluralization_is_canonical`        | Custom unit pluralization canonicalizes values.               | Test passes         |
| ST-HIPP-UNITS-002  | REQ-HIPP-UNITS-002  | `tests/spec/units.rs::spec_standard_units_still_work`                    | Standard units work in calculations.                         | Test passes         |
| ST-HIPP-UNITS-003  | REQ-HIPP-UNITS-003  | `tests/spec/units.rs::spec_custom_unit_abbreviation_is_canonical`        | Custom unit abbreviations canonicalize values.               | Test passes         |
| ST-HIPP-UNITS-004  | REQ-HIPP-UNITS-004  | `tests/spec/units.rs::spec_custom_unit_quantity_parsing`                  | Custom unit quantities parse with definitions.               | Test passes         |
| ST-HIPP-UNITS-005  | REQ-HIPP-UNITS-005  | `tests/spec/grammar.rs::spec_unitless_numeric_literal_fails`             | Numeric literals must include units.                         | Test passes         |

### 4.4 — §3.3 Program Structure

| ID         | Traces         | Test function                                                              | Description                                                  | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------|--------------------------------------------------------------|---------------------|
| ST-HIPP-PROG-001  | REQ-HIPP-PROG-001  | `tests/spec/fixtures.rs::spec_full_fixture_parses_core_definitions`        | Multi-definition fixtures parse core definitions.            | Test passes         |

### 4.5 — §3.4 Values

| ID         | Traces         | Test function                                                              | Description                                                              | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------------|
| ST-HIPP-VALUE-001  | REQ-HIPP-VALUE-001  | `tests/spec/values.rs::spec_value_definition_parsing`                      | Value definitions parse from fixtures.                                   | Test passes         |
| ST-HIPP-VALUE-002  | REQ-HIPP-VALUE-002  | `tests/spec/values.rs::spec_value_type_variants_parse`                     | Value type variants parse correctly.                                     | Test passes         |
| ST-HIPP-VALUE-003  | REQ-HIPP-VALUE-003  | `tests/spec/values.rs::spec_unit_property_parsing`                         | Unit properties parse in numeric values.                                 | Test passes         |
| ST-HIPP-VALUE-004  | REQ-HIPP-VALUE-004  | `tests/spec/values.rs::spec_value_timeframe_property_parsing`              | Value timeframe properties parse.                                        | Test passes         |
| ST-HIPP-VALUE-005  | REQ-HIPP-VALUE-005  | `tests/spec/values.rs::spec_inheritance_property_parsing`                  | Inheritance properties parse with overrides.                             | Test passes         |
| ST-HIPP-VALUE-006  | REQ-HIPP-VALUE-006  | `tests/spec/values.rs::spec_documentation_property_parsing`                | Documentation properties parse in inline and block forms.                | Test passes         |
| ST-HIPP-VALUE-007  | REQ-HIPP-VALUE-007  | `tests/spec/values.rs::spec_generic_property_parsing`                      | Custom properties parse as generic properties.                           | Test passes         |
| ST-HIPP-VALUE-008  | REQ-HIPP-VALUE-008  | `tests/spec/values.rs::spec_value_type_variants_parse`                     | Date/time value type parses.                                             | Test passes         |
| ST-HIPP-VALUE-009  | REQ-HIPP-VALUE-009  | `tests/spec/grammar.rs::spec_meaning_assessments_not_allowed_in_plans`     | Meaning assessments are only allowed in value definition blocks.         | Test passes         |
| ST-HIPP-VALUE-010  | REQ-HIPP-VALUE-010  | `tests/spec/grammar.rs::spec_meaning_requires_target_identifier`           | Meaning properties require an explicit target identifier.                | Test passes         |
| ST-HIPP-VALUE-011  | REQ-HIPP-VALUE-011  | `tests/spec/grammar.rs::spec_meaning_requires_valid_meanings`              | Meaning properties must declare valid meanings.                          | Test passes         |
| ST-HIPP-VALUE-012  | REQ-HIPP-VALUE-012  | `tests/spec/grammar.rs::spec_meaning_labels_require_identifiers`           | Meaning labels must be identifiers (angle brackets).                     | Test passes         |
| ST-HIPP-VALUE-013  | REQ-HIPP-VALUE-013  | `tests/spec/validation.rs::spec_enum_valid_values_require_identifiers`     | Enumeration valid values are identifiers (angle brackets).               | Test passes         |

### 4.6 — §3.5 Periods and Plans

| ID         | Traces         | Test function                                                            | Description                                                  | Pass criteria       |
|------------|-------------|--------------------------------------------------------------------------|--------------------------------------------------------------|---------------------|
| ST-HIPP-PLAN-001  | REQ-HIPP-PLAN-001  | `tests/spec/periods_plans.rs::spec_period_definition_parsing`            | Period definitions parse by name.                            | Test passes         |
| ST-HIPP-PLAN-002  | REQ-HIPP-PLAN-002  | `tests/spec/periods_plans.rs::spec_period_parsing_structure`             | Period timeframe lines parse with range selectors.           | Test passes         |

### 4.7 — §3.6 Statements, Assessments, and Ranges

| ID         | Traces         | Test function                                                                    | Description                                                              | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------------|
| ST-HIPP-STMT-001  | REQ-HIPP-STMT-001  | `tests/spec/statements_actions.rs::spec_timeframe_block_parsing`                 | Timeframe blocks parse with nested statements.                           | Test passes         |
| ST-HIPP-STMT-002  | REQ-HIPP-STMT-002  | `tests/spec/statements_actions.rs::spec_timeframe_requires_range_selector`       | Timeframe selectors require a start and end.                             | Test passes         |
| ST-HIPP-STMT-003  | REQ-HIPP-STMT-003  | `tests/spec/validation.rs::spec_timeframe_selector_requires_period_definition`   | Timeframe selector identifiers must refer to defined periods.            | Test passes         |
| ST-HIPP-STMT-004  | REQ-HIPP-STMT-004  | `tests/spec/grammar.rs::spec_block_statements_require_period`                    | Statements inside blocks must terminate with a period.                   | Test passes         |
| ST-HIPP-STMT-005  | REQ-HIPP-STMT-005  | `tests/spec/grammar.rs::spec_blocks_require_colon`                               | Blocks must be introduced with a colon.                                  | Test passes         |

### 4.8 — §3.7 Actions and Questions

| ID         | Traces         | Test function                                                                            | Description                                                                              | Pass criteria       |
|------------|-------------|------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------|---------------------|
| ST-HIPP-ACT-001  | REQ-HIPP-ACT-001  | `tests/spec/statements_actions.rs::spec_question_config_parsing_and_validation`           | Question configuration parses and validates references.                                  | Test passes         |
| ST-HIPP-ACT-002  | REQ-HIPP-ACT-002  | `tests/spec/statements_actions.rs::spec_message_expiration_parsing`                       | Message expiration attaches to information, warning, and urgent warning actions.          | Test passes         |
| ST-HIPP-ACT-003  | REQ-HIPP-ACT-003  | `tests/spec/statements_actions.rs::spec_question_modifiers_parsing`                       | Question modifiers parse (validate/type/style/expire).                                   | Test passes         |
| ST-HIPP-ACT-004  | REQ-HIPP-ACT-004  | `tests/spec/statements_actions.rs::spec_validate_answer_within_parsing`                   | Validate answer within parsing attaches to ask blocks.                                   | Test passes         |
| ST-HIPP-ACT-005  | REQ-HIPP-ACT-005  | `tests/spec/statements_actions.rs::spec_listen_send_start_and_simple_command_parsing`     | Listen/send/start/simple command actions parse.                                          | Test passes         |
| ST-HIPP-ACT-006  | REQ-HIPP-ACT-006  | `tests/spec/statements_actions.rs::spec_question_expiration_block_parsing`                | Question expiration blocks parse with reminder statements.                               | Test passes         |
| ST-HIPP-ACT-007  | REQ-HIPP-ACT-007  | `tests/spec/statements_actions.rs::spec_question_expiration_until_event_trigger_parsing`  | Question expiration supports until event triggers.                                       | Test passes         |
| ST-HIPP-ACT-008  | REQ-HIPP-ACT-008  | `tests/spec/statements_actions.rs::spec_message_action_keyword_parsing`                   | Information, warning, and urgent warning are accepted as message action keywords.         | Test passes         |
| ST-HIPP-ACT-009  | REQ-HIPP-ACT-009  | `tests/spec/statements_actions.rs::spec_message_expiration_parsing`                       | Message actions accept semicolon-separated addressee lists.                              | Test passes         |
| ST-HIPP-ACT-010  | REQ-HIPP-ACT-010  | `tests/spec/periods_plans.rs::spec_after_plan_block_parsing`                              | `after plan:` block parses into AST and executes after event loop.                       | Test passes         |

### 4.9 — §3.8 Events and Timing

| ID         | Traces         | Test function                                                                    | Description                                                      | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------------|------------------------------------------------------------------|---------------------|
| ST-HIPP-EVT-001  | REQ-HIPP-EVT-001  | `tests/spec/periods_plans.rs::spec_event_trigger_parsing`                        | Event triggers parse for change/start/periodic.                  | Test passes         |
| ST-HIPP-EVT-002  | REQ-HIPP-EVT-002  | `tests/spec/periods_plans.rs::spec_event_block_parsing`                          | Event blocks attach statements to triggers.                      | Test passes         |
| ST-HIPP-EVT-003  | REQ-HIPP-EVT-003  | `tests/spec/execution.rs::spec_scheduler_next_occurrence`                        | Scheduler computes next occurrence for periods.                  | Test passes         |
| ST-HIPP-EVT-004  | REQ-HIPP-EVT-004  | `tests/spec/periods_plans.rs::spec_event_trigger_duration_and_offset_parsing`    | Periodic triggers parse duration and offsets.                    | Test passes         |
| ST-HIPP-EVT-005  | REQ-HIPP-EVT-005  | `tests/spec/periods_plans.rs::spec_event_trigger_time_of_day_parsing`            | Periodic triggers parse `at <time>` clause.                      | Test passes         |
| ST-HIPP-EVT-006  | REQ-HIPP-EVT-006  | `tests/integration/simulation.rs::test_period_based_repetition_within_duration`   | Period-based triggers fire at every occurrence within duration.   | All occurrences fire |
| ST-HIPP-EVT-007  | REQ-HIPP-EVT-007  | `tests/spec/periods_plans.rs::spec_bare_unit_trigger_parsing`, `tests/spec/periods_plans.rs::spec_ordinal_trigger_parsing` | Bare unit (`every day`) and ordinal (`every third day`) triggers parse correctly to numeric intervals. | Test passes |

### 4.10 — §3.9 Communication & Actors

| ID         | Traces         | Test function                                                                            | Description                                                      | Pass criteria       |
|------------|-------------|------------------------------------------------------------------------------------------|------------------------------------------------------------------|---------------------|
| ST-HIPP-COMM-001  | REQ-HIPP-COMM-001  | `tests/spec/actors_drugs.rs::spec_addressee_group_and_contact_logic_parsing`              | Addressee groups and contact logic parse.                        | Test passes         |
| ST-HIPP-COMM-002  | REQ-HIPP-COMM-002  | `tests/spec/actors_drugs.rs::spec_addressee_contact_details_and_sequence_order_parsing`   | Contact details and sequence ordering parse.                     | Test passes         |

### 4.11 — §3.10 Medication

| ID          | Traces          | Test function                                                              | Description                                                  | Pass criteria       |
|-------------|--------------|----------------------------------------------------------------------------|--------------------------------------------------------------|---------------------|
| ST-HIPP-MED-001  | REQ-HIPP-MED-001  | `tests/spec/actors_drugs.rs::spec_drug_definition_validation`              | Drug definition validation rejects undefined units.          | Test passes         |
| ST-HIPP-MED-002  | REQ-HIPP-MED-002  | `tests/spec/actors_drugs.rs::spec_drug_interactions_parse`                 | Drug interaction properties parse.                           | Test passes         |
| ST-HIPP-MED-003  | REQ-HIPP-MED-003  | `tests/spec/actors_drugs.rs::spec_drug_dosage_and_admin_rules_parsing`     | Dosage safety and administration rules parse.                | Test passes         |

### 4.12 — §3.11 Data Contexts

| ID          | Traces          | Test function                                                                  | Description                                                              | Pass criteria       |
|-------------|--------------|--------------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------------|
| ST-HIPP-CTX-001  | REQ-HIPP-CTX-001  | `tests/spec/contexts_expressions.rs::spec_context_definition_parsing`          | Context definitions parse timeframe/data/value filter items.             | Test passes         |
| ST-HIPP-CTX-002  | REQ-HIPP-CTX-002  | `tests/spec/contexts_expressions.rs::spec_context_block_items_parsing`         | Context blocks parse data/value filters and nested statements.           | Test passes         |
| ST-HIPP-CTX-003  | REQ-HIPP-CTX-003  | `tests/spec/contexts_expressions.rs::spec_context_for_analysis_execution`      | Context for analysis executes with scoped timeframe.                     | Test passes         |

### 4.13 — §3.12 Expressions and Statistical Analysis

| ID          | Traces          | Test function                                                                          | Description                                                              | Pass criteria       |
|-------------|--------------|----------------------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------------|
| ST-HIPP-EXPR-001  | REQ-HIPP-EXPR-001  | `tests/spec/contexts_expressions.rs::spec_statistical_functions_parsing`                | Statistical function expressions parse in assignments.                   | Test passes         |
| ST-HIPP-EXPR-002  | REQ-HIPP-EXPR-002  | `tests/spec/execution.rs::spec_timeframe_filtering`                                    | Timeframe filtering applies to statistical evaluations.                  | Test passes         |
| ST-HIPP-EXPR-003  | REQ-HIPP-EXPR-003  | `tests/spec/execution.rs::spec_timeframe_variants`                                     | Timeframe variants resolve counts over different windows.                | Test passes         |
| ST-HIPP-EXPR-004  | REQ-HIPP-EXPR-004  | `tests/spec/execution.rs::spec_trend_analysis_evaluates`                                | Trend analysis evaluates statistical trends over timeframes.             | Test passes         |
| ST-HIPP-EXPR-005  | REQ-HIPP-EXPR-005  | `tests/spec/validation.rs::spec_statistical_functions_require_timeframe_context`        | Statistical functions require an analysis timeframe context.             | Test passes         |
| ST-HIPP-EXPR-006  | REQ-HIPP-EXPR-006  | `tests/spec/contexts_expressions.rs::spec_date_diff_parsing`                           | Date diff expressions parse.                                             | Test passes         |
| ST-HIPP-EXPR-007  | REQ-HIPP-EXPR-007  | `tests/spec/contexts_expressions.rs::spec_meaning_of_expression_parsing`               | Meaning-of expressions parse in assignments.                             | Test passes         |

### 4.14 — §4.1 Core Unit Groups and Conversion

| ID         | Traces         | Test function                                                                    | Description                                                                  | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------------|------------------------------------------------------------------------------|---------------------|
| ST-HIPP-CORE-001  | REQ-HIPP-CORE-001  | `tests/spec/units.rs::spec_builtin_units_cannot_be_redefined`                    | Built-in units cannot be redefined.                                          | Test passes         |
| ST-HIPP-CORE-002  | REQ-HIPP-CORE-002  | `tests/spec/units.rs::spec_unit_conversions_within_groups`                       | Unit conversions are supported within compatible groups.                     | Test passes         |
| ST-HIPP-CORE-003  | REQ-HIPP-CORE-003  | `tests/spec/units.rs::spec_assignment_requires_unit_and_precision_match`         | Calculations and assignments require matching units and precision.           | Test passes         |
| ST-HIPP-CORE-005  | REQ-HIPP-CORE-005  | `tests/spec/validation.rs::spec_parse_error_human_readable`                      | Parse errors include human-readable descriptions, not raw PEG rule names.   | Test passes         |

### 4.15 — §4.2 Required Properties

| ID         | Traces         | Test function                                                            | Description                                                      | Pass criteria       |
|------------|-------------|--------------------------------------------------------------------------|------------------------------------------------------------------|---------------------|
| ST-HIPP-REQP-001  | REQ-HIPP-REQP-001  | `tests/spec/validation.rs::spec_unit_requirement_validation`             | Numeric valid values require units.                              | Test passes         |
| ST-HIPP-REQP-002  | REQ-HIPP-REQP-002  | `tests/spec/validation.rs::spec_unitless_assess_fails`                   | Assessment ranges require units.                                 | Test passes         |
| ST-HIPP-REQP-003  | REQ-HIPP-REQP-003  | `tests/spec/validation.rs::spec_unitless_definition_fails`               | Numeric definitions require units.                               | Test passes         |
| ST-HIPP-REQP-004  | REQ-HIPP-REQP-004  | `tests/spec/validation.rs::spec_ask_requires_question_property`          | Ask requires a question property on the value.                   | Test passes         |
| ST-HIPP-REQP-005  | REQ-HIPP-REQP-005  | `tests/spec/validation.rs::spec_validation_error_line_number`            | Unit requirement errors report line numbers.                     | Test passes         |
| ST-HIPP-REQP-006  | REQ-HIPP-REQP-006  | `tests/spec/validation.rs::spec_missing_valid_values_fails`              | Numbers and enumerations must define valid values.               | Test passes         |
| ST-4.2-07a | REQ-HIPP-REQP-007  | `tests/spec/validation.rs::spec_valid_values_ranges_do_not_overlap`      | Valid value ranges must not overlap.                             | Test passes         |
| ST-4.2-07b | REQ-HIPP-REQP-007  | `tests/spec/validation.rs::spec_valid_values_datetime_ranges_do_not_overlap` | Date/time valid value ranges must not overlap.               | Test passes         |

### 4.16 — §4.3 Data Flow and Validity

| ID         | Traces         | Test function                                                                        | Description                                                                      | Pass criteria       |
|------------|-------------|--------------------------------------------------------------------------------------|----------------------------------------------------------------------------------|---------------------|
| ST-HIPP-FLOW-001  | REQ-HIPP-FLOW-001  | `tests/spec/validation.rs::spec_data_flow_use_before_assignment_fails`               | Values cannot be used before assignment.                                         | Test passes         |
| ST-HIPP-FLOW-002  | REQ-HIPP-FLOW-002  | `tests/spec/validation.rs::spec_calculation_does_not_initialize_value`               | Calculation properties do not seed values.                                       | Test passes         |
| ST-HIPP-FLOW-003  | REQ-HIPP-FLOW-003  | `tests/spec/validation.rs::spec_statistical_functions_do_not_require_local_init`     | Statistical functions do not require local initialization.                       | Test passes         |
| ST-HIPP-FLOW-004  | REQ-HIPP-FLOW-004  | `tests/spec/validation.rs::spec_listen_and_context_initialize_values`                | Listen for and context data initialize values.                                   | Test passes         |
| ST-4.3-05a | REQ-HIPP-FLOW-005  | `tests/spec/validation.rs::spec_meaning_of_requires_question_when_uninitialized`     | Meaning-of expressions require an askable value when not initialized.            | Test passes         |
| ST-4.3-05b | REQ-HIPP-FLOW-005  | `tests/spec/validation.rs::spec_meaning_of_allows_question_when_uninitialized`       | Meaning-of is allowed when the value is askable.                                 | Test passes         |

### 4.17 — §4.4 Assessment Coverage

| ID         | Traces         | Test function                                                                  | Description                                                                          | Pass criteria       |
|------------|-------------|--------------------------------------------------------------------------------|---------------------------------------------------------------------------------------|---------------------|
| ST-HIPP-COVER-001  | REQ-HIPP-COVER-001  | `tests/spec/validation.rs::spec_meaning_coverage_gaps_integer`                 | Meaning ranges must cover valid values (integer gaps).                                | Test passes         |
| ST-HIPP-COVER-002  | REQ-HIPP-COVER-002  | `tests/spec/validation.rs::spec_meaning_coverage_gaps_float`                   | Meaning ranges must cover valid values (float gaps).                                  | Test passes         |
| ST-HIPP-COVER-003  | REQ-HIPP-COVER-003  | `tests/spec/validation.rs::spec_meaning_coverage_disjoint_ranges_ok`           | Disjoint valid ranges are allowed when fully covered.                                 | Test passes         |
| ST-HIPP-COVER-004  | REQ-HIPP-COVER-004  | `tests/spec/validation.rs::spec_validator_numeric_overlap`                     | Overlapping numeric assessment ranges are invalid.                                    | Test passes         |
| ST-HIPP-COVER-005  | REQ-HIPP-COVER-005  | `tests/spec/validation.rs::spec_validator_enum_duplicate`                      | Duplicate enumeration cases are invalid.                                              | Test passes         |
| ST-HIPP-COVER-006  | REQ-HIPP-COVER-006  | `tests/spec/validation.rs::spec_validator_integer_gap_message`                 | Gap detection reports missing integer spans.                                          | Test passes         |
| ST-HIPP-COVER-007  | REQ-HIPP-COVER-007  | `tests/spec/validation.rs::spec_validator_float_gap_message`                   | Gap detection reports missing float spans.                                            | Test passes         |
| ST-HIPP-COVER-008  | REQ-HIPP-COVER-008  | `tests/spec/validation.rs::spec_precision_gaps`                                | Coverage gaps respect precision for float and integer ranges.                         | Test passes         |
| ST-HIPP-COVER-009  | REQ-HIPP-COVER-009  | `tests/spec/validation.rs::spec_range_overlap`                                 | Overlapping ranges are rejected.                                                      | Test passes         |
| ST-HIPP-COVER-010  | REQ-HIPP-COVER-010  | `tests/spec/validation.rs::spec_reproduce_missing_error`                       | Missing coverage yields a validation error.                                           | Test passes         |
| ST-HIPP-COVER-011  | REQ-HIPP-COVER-011  | `tests/spec/validation.rs::spec_trend_requires_full_coverage`                  | Trend assessments require full coverage.                                              | Test passes         |
| ST-HIPP-COVER-012  | REQ-HIPP-COVER-012  | `tests/spec/validation.rs::spec_precision_consistency`                         | Numeric valid value ranges use consistent precision across bounds and intervals.      | Test passes         |
| ST-HIPP-COVER-013  | REQ-HIPP-COVER-013  | `tests/spec/validation.rs::spec_meaning_valid_meanings_must_be_used`           | Valid meanings must be fully used across meaning assessments.                          | Test passes         |
| ST-HIPP-COVER-014  | REQ-HIPP-COVER-014  | `tests/spec/validation.rs::spec_meaning_invalid_label_rejected`                | Meaning labels must be drawn from declared valid meanings.                            | Test passes         |

### 4.18 — §4.5 Range Compliance (Pre-Run Validation)

| ID         | Traces         | Test function                                                              | Description                                                      | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------|------------------------------------------------------------------|---------------------|
| ST-HIPP-RANGE-001  | REQ-HIPP-RANGE-001  | `tests/spec/validation.rs::spec_interval_creation_and_math`                | Interval math supports range compliance checks.                  | Test passes         |
| ST-HIPP-RANGE-002  | REQ-HIPP-RANGE-002  | `tests/spec/validation.rs::spec_assignment_range_compliance_warning`       | Assignment range compliance fails when out of bounds.            | Test passes         |

### 4.19 — §4.6 Data Sufficiency

| ID         | Traces         | Test function                                                                    | Description                                                              | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------------|--------------------------------------------------------------------------|---------------------|
| ST-HIPP-SUFF-001  | REQ-HIPP-SUFF-001  | `tests/spec/validation.rs::spec_validator_requires_not_enough_data_case`          | Timeframe calculations require Not enough data handling.                 | Test passes         |
| ST-HIPP-SUFF-002  | REQ-HIPP-SUFF-002  | `tests/spec/validation.rs::spec_validator_passes_with_not_enough_data`            | Not enough data handling satisfies sufficiency.                          | Test passes         |
| ST-HIPP-SUFF-003  | REQ-HIPP-SUFF-003  | `tests/spec/execution.rs::spec_not_enough_data_evaluation`                       | Runtime evaluation returns NotEnoughData when history is insufficient.   | Test passes         |
| ST-HIPP-SUFF-004  | REQ-HIPP-SUFF-004  | `tests/spec/validation.rs::spec_not_enough_data_requires_statistical_target`     | Not enough data is only allowed for statistical assessments.             | Test passes         |

### 4.20 — §4.7 Date/Time Semantics

| ID         | Traces         | Test function                                                        | Description                                                                      | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------|----------------------------------------------------------------------------------|---------------------|
| ST-HIPP-DTIME-001  | REQ-HIPP-DTIME-001  | `tests/spec/execution.rs::spec_date_time_range_evaluation`           | Date/time valid value ranges evaluate using date/time and time-of-day semantics. | Test passes         |
| ST-HIPP-DTIME-002  | REQ-HIPP-DTIME-002  | `tests/spec/execution.rs::spec_date_diff_evaluation`                 | Date diff expressions evaluate to quantities in requested units.                 | Test passes         |

### 4.21 — §5 Execution Model

| ID       | Traces       | Test function                                                          | Description                                                                          | Pass criteria       |
|----------|-----------|------------------------------------------------------------------------|--------------------------------------------------------------------------------------|---------------------|
| ST-HIPP-EXEC-001  | REQ-HIPP-EXEC-001  | `tests/spec/execution.rs::spec_runtime_execution_flow`                 | Runtime executes assignments and actions in order.                                   | Test passes         |
| ST-HIPP-EXEC-002  | REQ-HIPP-EXEC-002  | `tests/spec/execution.rs::spec_validity_reuse_timeframe`               | Reuse timeframes prevent re-asking within the validity window.                       | Test passes         |
| ST-HIPP-EXEC-003  | REQ-HIPP-EXEC-003  | `tests/spec/execution.rs::spec_message_callback_missing_warns`         | Runtime emits a warning when a message action executes without a message callback.   | Test passes         |
| ST-HIPP-EXEC-004  | REQ-HIPP-EXEC-004  | `tests/spec/execution.rs::spec_simulation_mode_execution`              | Simulation mode executes without real-time delays.                                   | Test completes in under 10 seconds |
| ST-HIPP-EXEC-005  | REQ-HIPP-EXEC-005  | `tests/integration/simulation.rs::test_time_pinned_periodic_trigger`   | Time-pinned triggers fire at specified time, not plan start time.                    | Events at 08:00     |

### 4.22 — §5.1 Validation Logic

| ID         | Traces         | Test function                                                        | Description                                              | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------|----------------------------------------------------------|---------------------|
| ST-HIPP-VALID-001  | REQ-HIPP-VALID-001  | `tests/spec/validation.rs::spec_validate_plan_fixture_suite`         | Full-plan validation passes for a complete plan.         | Test passes         |
| ST-HIPP-VALID-002  | REQ-HIPP-VALID-002  | `tests/spec/validation.rs::spec_undefined_reference_detection`       | Undeclared references produce errors listing available definitions. | Test passes   |
| ST-HIPP-VALID-003  | REQ-HIPP-VALID-003  | `tests/spec/validation.rs::spec_validation_error_suggestions`        | Validation errors include suggestion with actionable fix. | Test passes        |

### 4.23 — §5.2 Input Validation

| ID         | Traces         | Test function                                                              | Description                                                                      | Pass criteria       |
|------------|-------------|----------------------------------------------------------------------------|----------------------------------------------------------------------------------|---------------------|
| ST-HIPP-INPUT-001  | REQ-HIPP-INPUT-001  | `tests/spec/execution.rs::spec_numeric_input_precision_rejection`          | Numeric answers must respect the decimal precision implied by valid values.       | Test passes         |

### 4.24 — §5.3 Meaning Evaluation

| ID         | Traces         | Test function                                                          | Description                                                      | Pass criteria       |
|------------|-------------|------------------------------------------------------------------------|------------------------------------------------------------------|---------------------|
| ST-HIPP-MEAN-001  | REQ-HIPP-MEAN-001  | `tests/spec/execution.rs::spec_meaning_of_evaluates`                   | Meaning evaluation returns the assessed meaning.                 | Test passes         |
| ST-HIPP-MEAN-002  | REQ-HIPP-MEAN-002  | `tests/spec/execution.rs::spec_meaning_of_missing_value`               | Meaning evaluation returns Missing when the source value is unknown. | Test passes     |
| ST-HIPP-MEAN-003  | REQ-HIPP-MEAN-003  | `tests/spec/execution.rs::spec_meaning_of_nested_assessment`           | Meaning evaluation supports nested assessments.                  | Test passes         |

## 5. Coverage Summary

| Metric                | Count |
|-----------------------|-------|
| Total ST-* test cases | 99    |
| REQ-* entries covered | 97    |
| Noted gaps            | 0     |

Two REQ IDs (REQ-HIPP-REQP-007 and REQ-HIPP-FLOW-005) each map to two distinct test functions. REQ-HIPP-EVT-007 maps to two test functions (bare unit and ordinal). This produces 99 test cases from 97 unique requirement IDs. All requirements from the traceability matrix are covered; no gaps are noted.

## Revision History

| Rev | Date       | Author | Description            |
|-----|------------|--------|------------------------|
| 1.0 | 2026-03-20 | —      | Initial version        |
| 1.1 | 2026-03-20 | —      | Added ST-HIPP-EXEC-004 (simulation mode, REQ-HIPP-EXEC-004). |
| 1.2 | 2026-03-23 | —      | Added ST-HIPP-EVT-005, ST-HIPP-EVT-006, ST-HIPP-EXEC-005 (time-of-day and period repetition). |
| 1.3 | 2026-03-23 | —      | Added ST-HIPP-ACT-010 (`after plan:` block). |
| 1.4 | 2026-03-23 | —      | Added ST-HIPP-EVT-007 (bare unit and ordinal trigger parsing, REQ-HIPP-EVT-007). |
| 1.5 | 2026-03-23 | —      | Added ST-HIPP-CORE-005 (parse error humanization), ST-HIPP-VALID-002 (undefined reference detection), ST-HIPP-VALID-003 (validation error suggestions). Updated counts. |
| 1.6 | 2026-04-19 | —      | Renamed all ST-* test IDs to canonical ST-HIPP-* form and updated `**Traces to:**` references from REQ-N.M to REQ-HIPP-<SECTION>-NNN for V-Model validator/generator compatibility. |
