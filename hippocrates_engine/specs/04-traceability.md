# Traceability Matrix (TM)

**Document ID:** TM
**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-20

---

## 1. Overview

This document establishes bidirectional traceability across the Hippocrates V-Model lifecycle. Every requirement on the left side of the V (stakeholder requirements, system requirements, system design, detailed design) has corresponding verification evidence on the right side (acceptance tests, system tests, integration tests, unit tests).

The matrix is the primary signal for completeness: if any left-side artifact lacks a right-side verification entry, it represents a gap that must be addressed before regulatory submission. Conversely, every test case traces back to a requirement, ensuring no orphaned tests exist.

Traceability flows in two directions:

- **Forward (top-down):** STKR -> REQ -> DES -> DDR, ensuring each level of decomposition is justified by a higher-level need.
- **Backward (bottom-up):** UT -> IT -> ST -> AT, ensuring each test verifies a design or requirement artifact.

---

## 2. Stakeholder Requirements to System Requirements

This table inverts the traceability header from `01-system-requirements.md`, showing which REQ specification sections satisfy each STKR.

| STKR | REQ Sections | Description |
|------|-------------|-------------|
| STKR-01 | §3.7, §3.9, §3.10, §5 | Care plan execution |
| STKR-02 | §2, §3.1-3.3, §3.8 (REQ-3.8-07) | Readability by medical professionals |
| STKR-03 | _(No REQ sections; satisfied by DES-01..03, DES-18, DES-30..34)_ | Embeddable runtime (architectural) |
| STKR-04 | §5.1 | Validation without execution |
| STKR-05 | §3.5, §3.7, §3.8, §5 | Event-driven execution |
| STKR-06 | _(No REQ sections; satisfied by DES-41)_ | Simulation mode (architectural) |
| STKR-10 | §3.6, §4.4 | Completeness |
| STKR-11 | §3.1-3.3, §4.2 | Readability over defaults |
| STKR-12 | §3.4, §5.3 | Meaningful values |
| STKR-13 | §2, §3.6, §3.11 | Contextual statements |
| STKR-14 | §2, §3.9 | Separation of concerns |
| STKR-15 | §3.4, §4.2, §4.5 | Value constraints |
| STKR-16 | _(No REQ sections; satisfied by DES-15, DES-26)_ | Immutable history (architectural) |
| STKR-17 | _(No REQ sections)_ | Localization support (future) |
| STKR-18 | §3.2, §3.10, §4.1, §4.7 | Medical domain awareness |
| STKR-19 | _(No REQ sections; satisfied by DES-15, DES-26)_ | Automatic persistence (architectural) |
| STKR-20 | §3.5 | Plan autonomy |
| STKR-30 | §2 | No comparison operators |
| STKR-31 | §3.2, §4.1 | Unit discipline |
| STKR-32 | §4.4 | Exhaustive assessment coverage |
| STKR-33 | §4.3 | Data flow validation |
| STKR-34 | §5.2 | Input precision validation |
| STKR-35 | §3.12, §4.6 | Data sufficiency handling |
| STKR-36 | §3.7 | Plan completion actions (`after plan:` block) |
| STKR-40 | _(Meta-requirement)_ | Requirements traceability |
| STKR-41 | _(Meta-requirement)_ | Reproducible verification |
| STKR-42 | _(Meta-requirement)_ | Class II documentation readiness |

---

## 3. System Requirements to System Design

This table maps REQ specification sections to DES-* design elements, derived from the traceability section of `02-system-design.md`.

| REQ Section | DES IDs | Description |
|-------------|---------|-------------|
| §2 Language Principles | DES-10 | Parser enforces syntax rules (angle brackets, no comparisons, blocks) |
| §3.1-3.3 Grammar | DES-10, DES-11 | Parser and AST representation |
| §3.2 Units | DES-15, DES-21 | Environment unit map; Chrono for time units |
| §3.4 Values | DES-15, DES-16 | Environment state store; Evaluator for meaning resolution |
| §3.5 Periods/Plans | DES-13, DES-14 | Executor event loop; Scheduler period calculations |
| §3.6 Statements | DES-10, DES-12 | Parser grammar; Validator completeness checks |
| §3.7 Actions/Questions | DES-13, DES-18 | Executor action handling; FFI callbacks |
| §3.8 Events/Timing | DES-13, DES-14, DES-21 | Executor, Scheduler, Chrono |
| §3.9 Communication | DES-18, DES-31 | FFI callback model |
| §3.10 Medication | DES-10, DES-12 | Parser drug definitions; Validator drug checks |
| §3.11 Data Contexts | DES-15, DES-16 | Environment context stack; Evaluator scoping |
| §3.12 Expressions/Stats | DES-16 | Evaluator statistical functions |
| §4.1 Units/Conversion | DES-12, DES-15 | Validator unit checks; Environment unit map |
| §4.2 Required Properties | DES-12 | Validator semantic checks |
| §4.3 Data Flow | DES-12 | Validator data flow analysis |
| §4.4 Coverage | DES-12 | Validator coverage analysis |
| §4.5 Range Compliance | DES-12 | Validator interval validation |
| §4.6 Data Sufficiency | DES-12, DES-16 | Validator sufficiency checks; Evaluator NotEnoughData |
| §4.7 Date/Time | DES-16, DES-21 | Evaluator date handling; Chrono |
| §5 Execution | DES-13, DES-15 | Executor event loop; Environment state |
| §5.1 Validation | DES-12 | Validator pipeline |
| §5.2 Input Validation | DES-15, DES-42 | Environment input validation; Input channel |
| §5.3 Meaning Evaluation | DES-16 | Evaluator meaning resolution |

---

## 4. System Design to Detailed Design

This table maps DES-* design elements to DDR-* detailed design elements, derived from the traceability section of `03-detailed-design.md`.

| DES ID | DDR ID Range | Component |
|--------|-------------|-----------|
| DES-10 | DDR-PARSER-01..05 | Parser (PEG grammar, AST construction, indentation, entry point, ordinal/bare-unit sugar) |
| DES-11 | DDR-DOM-01..06, DDR-PARSER-02 | AST representation and domain model types |
| DES-12 | DDR-VAL-01..06 | Multi-layer validator (pipeline, semantics, intervals, data flow, coverage, error reporting) |
| DES-13 | DDR-RT-01, DDR-RT-03, DDR-RT-08, DDR-RT-10 | Runtime executor (Engine struct, Executor, execution modes, after plan) |
| DES-14 | DDR-RT-05 | Scheduler |
| DES-15 | DDR-DOM-01..06, DDR-RT-02, DDR-RT-07 | Environment (state store, domain types, input validation) |
| DES-16 | DDR-RT-04 | Evaluator |
| DES-17 | DDR-RT-06 | Session (multi-plan coordinator) |
| DES-18 | DDR-FFI-01..19 | FFI layer (all C-compatible API functions) |
| DES-19 | DDR-FMT-01 | Formatter |

---

## 5. System Requirements to Test Cases (Adopted)

This section adopts the content from `tests/spec/TRACEABILITY.md` verbatim. All 93 REQ-to-test-function mappings are listed below, organized by specification section.

### 5.1 -- Section 2: Language Principles
- REQ-2-01 -- `tests/spec/grammar.rs::spec_identifiers_require_angle_brackets` -- identifiers must use angle brackets.
- REQ-2-02 -- `tests/spec/grammar.rs::spec_string_literal_rejects_angle_brackets` -- string literals must not contain angle brackets.
- REQ-2-03 -- `tests/spec/grammar.rs::spec_no_comparison_operators` -- comparison operators are not supported; use ranges.
- REQ-2-04 -- `tests/spec/grammar.rs::spec_block_requires_newline_after_colon` -- block openings require a newline and indented block.

### 5.2 -- Section 3.1: Basic Elements
- REQ-3.1-01 -- `tests/spec/contexts_expressions.rs::spec_time_indications_parsing` -- time indications parse for now, weekday, and time-of-day.
- REQ-3.1-02 -- `tests/spec/contexts_expressions.rs::spec_relative_time_from_now_parsing` -- relative time expressions from now parse.
- REQ-3.1-03 -- `tests/spec/grammar.rs::spec_inline_colon_requires_block` -- inline ':' forms are only allowed where explicitly shown.
- REQ-3.1-04 -- `tests/spec/contexts_expressions.rs::spec_date_time_literals_parsing` -- date/time literals parse for date and date-time forms.

### 5.3 -- Section 3.2: Units and Quantities
- REQ-3.2-01 -- `tests/spec/units.rs::spec_custom_unit_pluralization_is_canonical` -- custom unit pluralization canonicalizes values.
- REQ-3.2-02 -- `tests/spec/units.rs::spec_standard_units_still_work` -- standard units work in calculations.
- REQ-3.2-03 -- `tests/spec/units.rs::spec_custom_unit_abbreviation_is_canonical` -- custom unit abbreviations canonicalize values.
- REQ-3.2-04 -- `tests/spec/units.rs::spec_custom_unit_quantity_parsing` -- custom unit quantities parse with definitions.
- REQ-3.2-05 -- `tests/spec/grammar.rs::spec_unitless_numeric_literal_fails` -- numeric literals must include units.

### 5.4 -- Section 3.3: Program Structure
- REQ-3.3-01 -- `tests/spec/fixtures.rs::spec_full_fixture_parses_core_definitions` -- multi-definition fixtures parse core definitions.

### 5.5 -- Section 3.4: Values
- REQ-3.4-01 -- `tests/spec/values.rs::spec_value_definition_parsing` -- value definitions parse from fixtures.
- REQ-3.4-02 -- `tests/spec/values.rs::spec_value_type_variants_parse` -- value type variants parse correctly.
- REQ-3.4-03 -- `tests/spec/values.rs::spec_unit_property_parsing` -- unit properties parse in numeric values.
- REQ-3.4-04 -- `tests/spec/values.rs::spec_value_timeframe_property_parsing` -- value timeframe properties parse.
- REQ-3.4-05 -- `tests/spec/values.rs::spec_inheritance_property_parsing` -- inheritance properties parse with overrides.
- REQ-3.4-06 -- `tests/spec/values.rs::spec_documentation_property_parsing` -- documentation properties parse in inline and block forms.
- REQ-3.4-07 -- `tests/spec/values.rs::spec_generic_property_parsing` -- custom properties parse as generic properties.
- REQ-3.4-08 -- `tests/spec/values.rs::spec_value_type_variants_parse` -- date/time value type parses.
- REQ-3.4-09 -- `tests/spec/grammar.rs::spec_meaning_assessments_not_allowed_in_plans` -- meaning assessments are only allowed in value definition blocks.
- REQ-3.4-10 -- `tests/spec/grammar.rs::spec_meaning_requires_target_identifier` -- meaning properties require an explicit target identifier.
- REQ-3.4-11 -- `tests/spec/grammar.rs::spec_meaning_requires_valid_meanings` -- meaning properties must declare valid meanings.
- REQ-3.4-12 -- `tests/spec/grammar.rs::spec_meaning_labels_require_identifiers` -- meaning labels must be identifiers (angle brackets).
- REQ-3.4-13 -- `tests/spec/validation.rs::spec_enum_valid_values_require_identifiers` -- enumeration valid values are identifiers (angle brackets).

### 5.6 -- Section 3.5: Periods and Plans
- REQ-3.5-01 -- `tests/spec/periods_plans.rs::spec_period_definition_parsing` -- period definitions parse by name.
- REQ-3.5-02 -- `tests/spec/periods_plans.rs::spec_period_parsing_structure` -- period timeframe lines parse with range selectors.

### 5.7 -- Section 3.6: Statements, Assessments, and Ranges
- REQ-3.6-01 -- `tests/spec/statements_actions.rs::spec_timeframe_block_parsing` -- timeframe blocks parse with nested statements.
- REQ-3.6-02 -- `tests/spec/statements_actions.rs::spec_timeframe_requires_range_selector` -- timeframe selectors require a start and end.
- REQ-3.6-03 -- `tests/spec/validation.rs::spec_timeframe_selector_requires_period_definition` -- timeframe selector identifiers must refer to defined periods.
- REQ-3.6-04 -- `tests/spec/grammar.rs::spec_block_statements_require_period` -- statements inside blocks must terminate with a period.
- REQ-3.6-05 -- `tests/spec/grammar.rs::spec_blocks_require_colon` -- blocks must be introduced with a colon.

### 5.8 -- Section 3.7: Actions and Questions
- REQ-3.7-01 -- `tests/spec/statements_actions.rs::spec_question_config_parsing_and_validation` -- question configuration parses and validates references.
- REQ-3.7-02 -- `tests/spec/statements_actions.rs::spec_message_expiration_parsing` -- message expiration attaches to information, warning, and urgent warning actions.
- REQ-3.7-03 -- `tests/spec/statements_actions.rs::spec_question_modifiers_parsing` -- question modifiers parse (validate/type/style/expire).
- REQ-3.7-04 -- `tests/spec/statements_actions.rs::spec_validate_answer_within_parsing` -- validate answer within parsing attaches to ask blocks.
- REQ-3.7-05 -- `tests/spec/statements_actions.rs::spec_listen_send_start_and_simple_command_parsing` -- listen/send/start/simple command actions parse.
- REQ-3.7-06 -- `tests/spec/statements_actions.rs::spec_question_expiration_block_parsing` -- question expiration blocks parse with reminder statements.
- REQ-3.7-07 -- `tests/spec/statements_actions.rs::spec_question_expiration_until_event_trigger_parsing` -- question expiration supports until event triggers.
- REQ-3.7-08 -- `tests/spec/statements_actions.rs::spec_message_action_keyword_parsing` -- information, warning, and urgent warning are accepted as message action keywords.
- REQ-3.7-09 -- `tests/spec/statements_actions.rs::spec_message_expiration_parsing` -- message actions accept semicolon-separated addressee lists.
- REQ-3.7-10 -- `tests/spec/periods_plans.rs::spec_after_plan_block_parsing` -- `after plan:` block parses into `PlanBlock::AfterPlan` AST node.

### 5.9 -- Section 3.8: Events and Timing
- REQ-3.8-01 -- `tests/spec/periods_plans.rs::spec_event_trigger_parsing` -- event triggers parse for change/start/periodic.
- REQ-3.8-02 -- `tests/spec/periods_plans.rs::spec_event_block_parsing` -- event blocks attach statements to triggers.
- REQ-3.8-03 -- `tests/spec/execution.rs::spec_scheduler_next_occurrence` -- scheduler computes next occurrence for periods.
- REQ-3.8-04 -- `tests/spec/periods_plans.rs::spec_event_trigger_duration_and_offset_parsing` -- periodic triggers parse duration and offsets.
- REQ-3.8-05 -- `tests/spec/periods_plans.rs::spec_event_trigger_time_of_day_parsing` -- periodic triggers parse `at <time>` clause.
- REQ-3.8-06 -- `tests/integration/simulation.rs::test_period_based_repetition_within_duration` -- period-based triggers fire at every occurrence within duration window.
- REQ-3.8-07 -- `tests/spec/periods_plans.rs::spec_bare_unit_trigger_parsing` -- bare unit triggers (`every day`) parse to interval=1.0. `tests/spec/periods_plans.rs::spec_ordinal_trigger_parsing` -- ordinal triggers (`every third day`) parse to interval=3.0.

### 5.10 -- Section 3.9: Communication and Actors
- REQ-3.9-01 -- `tests/spec/actors_drugs.rs::spec_addressee_group_and_contact_logic_parsing` -- addressee groups and contact logic parse.
- REQ-3.9-02 -- `tests/spec/actors_drugs.rs::spec_addressee_contact_details_and_sequence_order_parsing` -- contact details and sequence ordering parse.

### 5.11 -- Section 3.10: Medication
- REQ-3.10-01 -- `tests/spec/actors_drugs.rs::spec_drug_definition_validation` -- drug definition validation rejects undefined units.
- REQ-3.10-02 -- `tests/spec/actors_drugs.rs::spec_drug_interactions_parse` -- drug interaction properties parse.
- REQ-3.10-03 -- `tests/spec/actors_drugs.rs::spec_drug_dosage_and_admin_rules_parsing` -- dosage safety and administration rules parse.

### 5.12 -- Section 3.11: Data Contexts
- REQ-3.11-01 -- `tests/spec/contexts_expressions.rs::spec_context_definition_parsing` -- context definitions parse timeframe/data/value filter items.
- REQ-3.11-02 -- `tests/spec/contexts_expressions.rs::spec_context_block_items_parsing` -- context blocks parse data/value filters and nested statements.
- REQ-3.11-03 -- `tests/spec/contexts_expressions.rs::spec_context_for_analysis_execution` -- context for analysis executes with scoped timeframe.

### 5.13 -- Section 3.12: Expressions and Statistical Analysis
- REQ-3.12-01 -- `tests/spec/contexts_expressions.rs::spec_statistical_functions_parsing` -- statistical function expressions parse in assignments.
- REQ-3.12-02 -- `tests/spec/execution.rs::spec_timeframe_filtering` -- timeframe filtering applies to statistical evaluations.
- REQ-3.12-03 -- `tests/spec/execution.rs::spec_timeframe_variants` -- timeframe variants resolve counts over different windows.
- REQ-3.12-04 -- `tests/spec/execution.rs::spec_trend_analysis_evaluates` -- trend analysis evaluates statistical trends over timeframes.
- REQ-3.12-05 -- `tests/spec/validation.rs::spec_statistical_functions_require_timeframe_context` -- statistical functions require an analysis timeframe context.
- REQ-3.12-06 -- `tests/spec/contexts_expressions.rs::spec_date_diff_parsing` -- date diff expressions parse.
- REQ-3.12-07 -- `tests/spec/contexts_expressions.rs::spec_meaning_of_expression_parsing` -- meaning-of expressions parse in assignments.

### 5.14 -- Section 4.1: Core Unit Groups and Conversion
- REQ-4.1-01 -- `tests/spec/units.rs::spec_builtin_units_cannot_be_redefined` -- built-in units cannot be redefined.
- REQ-4.1-02 -- `tests/spec/units.rs::spec_unit_conversions_within_groups` -- unit conversions are supported within compatible groups.
- REQ-4.1-03 -- `tests/spec/units.rs::spec_assignment_requires_unit_and_precision_match` -- calculations and assignments require matching units and precision.

### 5.15 -- Section 4.2: Required Properties
- REQ-4.2-01 -- `tests/spec/validation.rs::spec_unit_requirement_validation` -- numeric valid values require units.
- REQ-4.2-02 -- `tests/spec/validation.rs::spec_unitless_assess_fails` -- assessment ranges require units.
- REQ-4.2-03 -- `tests/spec/validation.rs::spec_unitless_definition_fails` -- numeric definitions require units.
- REQ-4.2-04 -- `tests/spec/validation.rs::spec_ask_requires_question_property` -- ask requires a question property on the value.
- REQ-4.2-05 -- `tests/spec/validation.rs::spec_validation_error_line_number` -- unit requirement errors report line numbers.
- REQ-4.2-06 -- `tests/spec/validation.rs::spec_missing_valid_values_fails` -- numbers and enumerations must define valid values.
- REQ-4.2-07 -- `tests/spec/validation.rs::spec_valid_values_ranges_do_not_overlap` -- valid value ranges must not overlap.
- REQ-4.2-07 -- `tests/spec/validation.rs::spec_valid_values_datetime_ranges_do_not_overlap` -- date/time valid value ranges must not overlap.

### 5.16 -- Section 4.3: Data Flow and Validity
- REQ-4.3-01 -- `tests/spec/validation.rs::spec_data_flow_use_before_assignment_fails` -- values cannot be used before assignment.
- REQ-4.3-02 -- `tests/spec/validation.rs::spec_calculation_does_not_initialize_value` -- calculation properties do not seed values.
- REQ-4.3-03 -- `tests/spec/validation.rs::spec_statistical_functions_do_not_require_local_init` -- statistical functions do not require local initialization.
- REQ-4.3-04 -- `tests/spec/validation.rs::spec_listen_and_context_initialize_values` -- listen for and context data initialize values.
- REQ-4.3-05 -- `tests/spec/validation.rs::spec_meaning_of_requires_question_when_uninitialized` -- meaning-of expressions require an askable value when not initialized.
- REQ-4.3-05 -- `tests/spec/validation.rs::spec_meaning_of_allows_question_when_uninitialized` -- meaning-of is allowed when the value is askable.

### 5.17 -- Section 4.4: Assessment Coverage
- REQ-4.4-01 -- `tests/spec/validation.rs::spec_meaning_coverage_gaps_integer` -- meaning ranges must cover valid values (integer gaps).
- REQ-4.4-02 -- `tests/spec/validation.rs::spec_meaning_coverage_gaps_float` -- meaning ranges must cover valid values (float gaps).
- REQ-4.4-03 -- `tests/spec/validation.rs::spec_meaning_coverage_disjoint_ranges_ok` -- disjoint valid ranges are allowed when fully covered.
- REQ-4.4-04 -- `tests/spec/validation.rs::spec_validator_numeric_overlap` -- overlapping numeric assessment ranges are invalid.
- REQ-4.4-05 -- `tests/spec/validation.rs::spec_validator_enum_duplicate` -- duplicate enumeration cases are invalid.
- REQ-4.4-06 -- `tests/spec/validation.rs::spec_validator_integer_gap_message` -- gap detection reports missing integer spans.
- REQ-4.4-07 -- `tests/spec/validation.rs::spec_validator_float_gap_message` -- gap detection reports missing float spans.
- REQ-4.4-08 -- `tests/spec/validation.rs::spec_precision_gaps` -- coverage gaps respect precision for float and integer ranges.
- REQ-4.4-09 -- `tests/spec/validation.rs::spec_range_overlap` -- overlapping ranges are rejected.
- REQ-4.4-10 -- `tests/spec/validation.rs::spec_reproduce_missing_error` -- missing coverage yields a validation error.
- REQ-4.4-11 -- `tests/spec/validation.rs::spec_trend_requires_full_coverage` -- trend assessments require full coverage.
- REQ-4.4-12 -- `tests/spec/validation.rs::spec_precision_consistency` -- numeric valid value ranges use consistent precision across bounds and intervals.
- REQ-4.4-13 -- `tests/spec/validation.rs::spec_meaning_valid_meanings_must_be_used` -- valid meanings must be fully used across meaning assessments.
- REQ-4.4-14 -- `tests/spec/validation.rs::spec_meaning_invalid_label_rejected` -- meaning labels must be drawn from declared valid meanings.

### 5.18 -- Section 4.5: Range Compliance
- REQ-4.5-01 -- `tests/spec/validation.rs::spec_interval_creation_and_math` -- interval math supports range compliance checks.
- REQ-4.5-02 -- `tests/spec/validation.rs::spec_assignment_range_compliance_warning` -- assignment range compliance fails when out of bounds.

### 5.19 -- Section 4.6: Data Sufficiency
- REQ-4.6-01 -- `tests/spec/validation.rs::spec_validator_requires_not_enough_data_case` -- timeframe calculations require Not enough data handling.
- REQ-4.6-02 -- `tests/spec/validation.rs::spec_validator_passes_with_not_enough_data` -- Not enough data handling satisfies sufficiency.
- REQ-4.6-03 -- `tests/spec/execution.rs::spec_not_enough_data_evaluation` -- runtime evaluation returns NotEnoughData when history is insufficient.
- REQ-4.6-04 -- `tests/spec/validation.rs::spec_not_enough_data_requires_statistical_target` -- Not enough data is only allowed for statistical assessments.

### 5.20 -- Section 4.7: Date/Time Semantics
- REQ-4.7-01 -- `tests/spec/execution.rs::spec_date_time_range_evaluation` -- date/time valid value ranges evaluate using date/time and time-of-day semantics.
- REQ-4.7-02 -- `tests/spec/execution.rs::spec_date_diff_evaluation` -- date diff expressions evaluate to quantities in requested units.

### 5.21 -- Section 5: Execution Model
- REQ-5-01 -- `tests/spec/execution.rs::spec_runtime_execution_flow` -- runtime executes assignments and actions in order.
- REQ-5-02 -- `tests/spec/execution.rs::spec_validity_reuse_timeframe` -- reuse timeframes prevent re-asking within the validity window.
- REQ-5-03 -- `tests/spec/execution.rs::spec_message_callback_missing_warns` -- runtime emits a warning when a message action executes without a message callback.
- REQ-5-05 -- `tests/integration/simulation.rs::test_time_pinned_periodic_trigger` -- time-pinned triggers fire at specified time, not plan start time.

### 5.22 -- Section 5.1: Validation Logic
- REQ-5.1-01 -- `tests/spec/validation.rs::spec_validate_plan_fixture_suite` -- full-plan validation passes for a complete plan.

### 5.23 -- Section 5.2: Input Validation
- REQ-5.2-01 -- `tests/spec/execution.rs::spec_numeric_input_precision_rejection` -- numeric answers must respect the decimal precision implied by valid values.

### 5.24 -- Section 5.3: Meaning Evaluation
- REQ-5.3-01 -- `tests/spec/execution.rs::spec_meaning_of_evaluates` -- meaning evaluation returns the assessed meaning.
- REQ-5.3-02 -- `tests/spec/execution.rs::spec_meaning_of_missing_value` -- meaning evaluation returns Missing when the source value is unknown.
- REQ-5.3-03 -- `tests/spec/execution.rs::spec_meaning_of_nested_assessment` -- meaning evaluation supports nested assessments.

---

## 6. Test Plan Coverage Summary

| V-Model Level | Test Plan | Test ID Prefix | # Test Cases | Coverage |
|---|---|---|---|---|
| Stakeholder Req (STKR) | `test-plans/03-acceptance-test-plan.md` | AT-\* | 27 | 100% (all 27 STKR covered) |
| System Req (REQ) | `test-plans/02-system-test-plan.md` | ST-\* | 96 | 100% (94 unique REQ IDs, 96 test cases) |
| System Design (DES) | `test-plans/01-integration-test-plan.md` | IT-\* | 28 | 14/33 DES elements directly covered (~42%) |
| Detailed Design (DDR) | `test-plans/00-unit-test-plan.md` | UT-\* | 130 | 27/30 DDR elements covered (~90%) |

**Notes on DES coverage:** DES elements not covered by integration tests (DES-01, DES-02, DES-03, DES-19, DES-20..DES-26, DES-30, DES-33, DES-34, DES-40) are either architectural constraints verified by successful compilation, dependency declarations, or host-side concerns outside the scope of Rust integration tests. DES-43 (stop signal) is now covered by IT-25.

**Notes on DDR coverage:** DDR-FFI-01..19 are partially covered by UT-FFI-01..10 at the unit level; remaining FFI functions are tested at the integration level via Swift/C bindings. DDR-RT-02, DDR-RT-08, and DDR-FMT-01 gaps have been closed by UT-RT-15..17 and UT-FMT-01..02.

---

## 7. Full V-Model Matrix

The master matrix traces each stakeholder requirement through all V-Model levels.

| STKR | REQ (section) | DES | DDR | UT | IT | ST | AT | Status |
|------|--------------|-----|-----|----|----|----|----|----|
| STKR-01 | §3.7, §3.9, §3.10, §5 | DES-10, DES-13, DES-16, DES-18 | DDR-PARSER-01..04, DDR-RT-01, DDR-RT-03, DDR-RT-04 | UT-PARSER-\*, UT-RT-08, UT-RT-09, UT-ACTIONS-\*, UT-ACTORS-\* | IT-02, IT-07..IT-12 | ST-3.7-\*, ST-3.9-\*, ST-3.10-\*, ST-5-\* | AT-01 | Covered |
| STKR-02 | §2, §3.1-3.3, §3.8 (REQ-3.8-07) | DES-10, DES-11, DES-19 | DDR-PARSER-01..05, DDR-FMT-01 | UT-PARSER-\*, UT-VALUES-\*, UT-FIX-01, UT-CTX-05..08, UT-PERIODS-08..09 | IT-03..IT-06 | ST-2-\*, ST-3.1-\*, ST-3.2-\*, ST-3.3-01, ST-3.8-07 | AT-02 | Covered |
| STKR-03 | _(architectural)_ | DES-01, DES-02, DES-03, DES-18, DES-22, DES-24, DES-30..DES-34 | DDR-FFI-01..19 | UT-FFI-01..10 | IT-23 | _(no ST)_ | AT-03 | Covered |
| STKR-04 | §5.1 | DES-12 | DDR-VAL-01..06 | UT-VAL-01..37 | IT-01, IT-02 | ST-5.1-01 | AT-04 | Covered |
| STKR-05 | §3.5, §3.7, §3.8, §5 | DES-10, DES-13, DES-14, DES-17, DES-21, DES-31, DES-40..DES-43 | DDR-RT-01, DDR-RT-03..DDR-RT-06, DDR-RT-08, DDR-RT-09, DDR-PARSER-02, DDR-PARSER-05 | UT-PERIODS-\*, UT-RT-08, UT-RT-10, UT-RT-14 | IT-07..IT-13, IT-16..IT-22 | ST-3.5-\*, ST-3.7-\*, ST-3.8-\*, ST-5-\* | AT-05 | Covered |
| STKR-06 | _(architectural)_ | DES-41 | DDR-RT-08 | UT-RT-17 | IT-07..IT-09, IT-20, IT-21 | ST-5-04 | AT-06 | Covered |
| STKR-10 | §3.6, §4.4 | DES-10, DES-12 | DDR-VAL-05, DDR-PARSER-01..02 | UT-VAL-01..07, UT-VAL-25, UT-VAL-36, UT-ACTIONS-01..02 | IT-01, IT-02 | ST-3.6-\*, ST-4.4-\* | AT-10 | Covered |
| STKR-11 | §3.1-3.3, §4.2 | DES-10, DES-12 | DDR-PARSER-01..02, DDR-VAL-02 | UT-PARSER-\*, UT-VAL-21..27, UT-VAL-35 | IT-01, IT-02 | ST-3.1-\*, ST-3.2-\*, ST-3.3-01, ST-4.2-\* | AT-11 | Covered |
| STKR-12 | §3.4, §5.3 | DES-15, DES-16 | DDR-RT-04, DDR-PARSER-02 | UT-VALUES-\*, UT-RT-04..06 | IT-10, IT-14, IT-15 | ST-3.4-\*, ST-5.3-\* | AT-12 | Covered |
| STKR-13 | §2, §3.6, §3.11 | DES-10, DES-15, DES-16 | DDR-PARSER-01..02, DDR-RT-04 | UT-CTX-\*, UT-ACTIONS-01 | IT-11 | ST-2-\*, ST-3.6-\*, ST-3.11-\* | AT-13 | Covered |
| STKR-14 | §2, §3.9 | DES-01, DES-03, DES-10 | DDR-PARSER-01 | UT-PARSER-\* | _(architectural)_ | ST-2-\*, ST-3.9-\* | AT-14 | Covered |
| STKR-15 | §3.4, §4.2, §4.5 | DES-12, DES-15, DES-42 | DDR-VAL-02..03, DDR-RT-07 | UT-VAL-21..23, UT-VAL-28, UT-VAL-37, UT-RT-07 | IT-02 | ST-3.4-\*, ST-4.2-\*, ST-4.5-\* | AT-15 | Covered |
| STKR-16 | _(architectural)_ | DES-15, DES-26 | DDR-RT-02 | UT-RT-15 | IT-10, IT-14, IT-15 | _(no ST)_ | AT-16 | Covered |
| STKR-17 | _(future)_ | _(none)_ | _(none)_ | _(none)_ | _(none)_ | _(none)_ | AT-17 | Gap |
| STKR-18 | §3.2, §3.10, §4.1, §4.7 | DES-15, DES-21 | DDR-DOM-02, DDR-PARSER-01..02, DDR-VAL-02 | UT-UNITS-\*, UT-ACTORS-01..05, UT-RT-02..03 | IT-07..IT-10 | ST-3.2-\*, ST-3.10-\*, ST-4.1-\*, ST-4.7-\* | AT-18 | Covered |
| STKR-19 | _(architectural)_ | DES-15, DES-26 | DDR-RT-02 | UT-RT-16 | IT-10, IT-14 | _(no ST)_ | AT-19 | Covered |
| STKR-20 | §3.5 | DES-13, DES-14 | DDR-PARSER-02, DDR-RT-01 | UT-PERIODS-01..02, UT-RT-08 | IT-02, IT-07 | ST-3.5-\* | AT-20 | Covered |
| STKR-30 | §2 | DES-10, DES-12 | DDR-PARSER-01 | UT-PARSER-05 | IT-01, IT-02 | ST-2-03 | AT-30 | Covered |
| STKR-31 | §3.2, §4.1 | DES-12, DES-15 | DDR-VAL-02, DDR-DOM-02 | UT-PARSER-06, UT-UNITS-\*, UT-VAL-21..27 | IT-12, IT-13, IT-16, IT-17 | ST-3.2-05, ST-4.1-\* | AT-31 | Covered |
| STKR-32 | §4.4 | DES-12 | DDR-VAL-03, DDR-VAL-05 | UT-VAL-01..07, UT-VAL-14..20, UT-VAL-25, UT-VAL-36 | IT-01, IT-02 | ST-4.4-\* | AT-32 | Covered |
| STKR-33 | §4.3 | DES-12 | DDR-VAL-04 | UT-VAL-29..34 | IT-01, IT-02 | ST-4.3-\* | AT-33 | Covered |
| STKR-34 | §5.2 | DES-15, DES-42 | DDR-RT-07 | UT-RT-07 | IT-16, IT-17 | ST-5.2-01 | AT-34 | Covered |
| STKR-35 | §3.12, §4.6 | DES-12, DES-16 | DDR-VAL-02, DDR-RT-04 | UT-VAL-09..12, UT-RT-01, UT-CTX-09 | IT-11 | ST-3.12-05, ST-4.6-\* | AT-35 | Covered |
| STKR-36 | §3.7 | DES-13 | DDR-RT-10 | UT-PLAN-01 | IT-28 | ST-3.7-10 | AT-36 | Covered |
| STKR-40 | _(meta)_ | _(this document)_ | _(this document)_ | _(N/A)_ | _(N/A)_ | _(N/A)_ | AT-40 | Covered |
| STKR-41 | _(meta)_ | _(build system)_ | _(build system)_ | _(all tests)_ | _(all tests)_ | _(all tests)_ | AT-41 | Covered |
| STKR-42 | _(meta)_ | _(doc set)_ | _(doc set)_ | _(N/A)_ | _(N/A)_ | _(N/A)_ | AT-42 | Covered |

**Legend:**
- **Covered:** All V-Model columns have entries; verification exists at every applicable level.
- **Partial:** Some columns lack direct test coverage, but the requirement is exercised indirectly through higher-level tests or verified by architectural constraints (compilation, linking).
- **Gap:** Critical verification is missing and requires action.

---

## 8. Gap Analysis

### 8.1 DDR Elements Without Unit Tests

The following DDR elements were previously identified as gaps. Most have been closed:

| DDR Element | Description | Severity | Status |
|-------------|-------------|----------|--------|
| DDR-FFI-01..19 | FFI C-API functions | Low | **Partially closed.** UT-FFI-01..10 cover DDR-FFI-01, -02, -03, -07, -08, -09, -10, -11. Remaining FFI functions (DDR-FFI-04..06, -12..19) are tested at integration level via Swift/C host bindings. |
| DDR-RT-02 | Environment struct | Low | **Closed.** UT-RT-15 (append-only history) and UT-RT-16 (value history retrieval) provide dedicated coverage. |
| DDR-RT-08 | Execution modes (simulation vs. real-time) | Medium | **Closed.** UT-RT-17 verifies simulation mode completes without real-time delays. |
| DDR-FMT-01 | Formatter `format_script` | Medium | **Closed.** UT-FMT-01 (round-trip parse-format-parse) and UT-FMT-02 (all definition types) provide coverage. |

### 8.2 DES Elements Without Integration Tests

The following DES elements are not directly covered by integration tests (per `01-integration-test-plan.md`):

| DES Element | Description | Reason |
|-------------|-------------|--------|
| DES-01, DES-02, DES-03 | Language selection, dual crate, C-FFI boundary | Architectural constraints verified by successful compilation and linking. |
| DES-19 | Formatter | **Closed at unit level.** UT-FMT-01 provides a round-trip parse-format-parse test. No integration test needed. |
| DES-20..DES-26 | Dependencies (Pest, Chrono, Serde, etc.) | Dependency declarations; verified by compilation. |
| DES-30, DES-33, DES-34 | FFI lifecycle, memory management, iOS integration | Host-side concerns; verified by the SwiftUI editor integration. |
| DES-40 | Real-time execution mode | Not testable in automated CI without wall-clock delays. |
| DES-43 | Stop signal | **Closed.** IT-25 verifies stop signal terminates a long-running simulation early. |

### 8.3 STKR Without Full Automated Evidence

| STKR | Issue | Status |
|------|-------|--------|
| STKR-03 | FFI/embedding tested only via Swift host; no Rust-level unit test. | **Closed.** UT-FFI-01..10 provide Rust-side FFI unit tests covering parse, validate, engine lifecycle, load, periods, time, simulation, and stop. |
| STKR-06 | Simulation mode has no system-level REQ or ST; tested only at integration level. | **Closed.** ST-5-04 (system test) and UT-RT-17 (unit test) now provide dedicated simulation mode coverage. |
| STKR-16 | Immutable history is architectural; no dedicated ST or UT. | **Closed.** UT-RT-15 verifies append-only value history semantics. |
| STKR-17 | Localization support is not implemented. | **Accepted gap.** Future scope (Priority: May). This is the only remaining accepted gap. |
| STKR-19 | Automatic persistence is architectural; no dedicated ST or UT. | **Closed.** UT-RT-16 verifies value history retrieval with timestamps after recording. |

---

## Revision History

| Version | Date | Changes |
|---|---|---|
| 1.0 | 2026-03-20 | Initial traceability matrix. Full V-Model cross-reference of all specifications, design documents, and test plans. |
| 1.1 | 2026-03-20 | Closed gaps: STKR-03, -06, -16, -19 changed from Partial to Covered. Added REQ-5-04 and ST-5-04. Updated test counts. STKR-17 remains only accepted gap. |
| 1.2 | 2026-03-23 | Added REQ-3.8-05, REQ-3.8-06, REQ-5-05, DDR-RT-09. Updated STKR-05 row. |
| 1.3 | 2026-03-23 | Added STKR-36 chain: REQ-3.7-10, DDR-RT-10, UT-PLAN-01, IT-28, ST-3.7-10, AT-36. |
| 1.4 | 2026-03-23 | Added REQ-3.8-07 (bare unit and ordinal triggers). Added DDR-PARSER-05. Updated STKR-02 and STKR-05 rows. Added UT-PERIODS-08..09, ST-3.8-07. |
