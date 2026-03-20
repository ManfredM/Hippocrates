# Specification Traceability

## §2 Language Principles
- REQ-2-01 — `tests/spec/grammar.rs::spec_identifiers_require_angle_brackets` — identifiers must use angle brackets.
- REQ-2-02 — `tests/spec/grammar.rs::spec_string_literal_rejects_angle_brackets` — string literals must not contain angle brackets.
- REQ-2-03 — `tests/spec/grammar.rs::spec_no_comparison_operators` — comparison operators are not supported; use ranges.
- REQ-2-04 — `tests/spec/grammar.rs::spec_block_requires_newline_after_colon` — block openings require a newline and indented block.

## §3.1 Basic Elements
- REQ-3.1-01 — `tests/spec/contexts_expressions.rs::spec_time_indications_parsing` — time indications parse for now, weekday, and time-of-day.
- REQ-3.1-02 — `tests/spec/contexts_expressions.rs::spec_relative_time_from_now_parsing` — relative time expressions from now parse.
- REQ-3.1-03 — `tests/spec/grammar.rs::spec_inline_colon_requires_block` — inline ':' forms are only allowed where explicitly shown.
- REQ-3.1-04 — `tests/spec/contexts_expressions.rs::spec_date_time_literals_parsing` — date/time literals parse for date and date-time forms.

## §3.2 Units and Quantities
- REQ-3.2-01 — `tests/spec/units.rs::spec_custom_unit_pluralization_is_canonical` — custom unit pluralization canonicalizes values.
- REQ-3.2-02 — `tests/spec/units.rs::spec_standard_units_still_work` — standard units work in calculations.
- REQ-3.2-03 — `tests/spec/units.rs::spec_custom_unit_abbreviation_is_canonical` — custom unit abbreviations canonicalize values.
- REQ-3.2-04 — `tests/spec/units.rs::spec_custom_unit_quantity_parsing` — custom unit quantities parse with definitions.
- REQ-3.2-05 — `tests/spec/grammar.rs::spec_unitless_numeric_literal_fails` — numeric literals must include units.

## §3.3 Program Structure
- REQ-3.3-01 — `tests/spec/fixtures.rs::spec_full_fixture_parses_core_definitions` — multi-definition fixtures parse core definitions.

## §3.4 Values
- REQ-3.4-01 — `tests/spec/values.rs::spec_value_definition_parsing` — value definitions parse from fixtures.
- REQ-3.4-02 — `tests/spec/values.rs::spec_value_type_variants_parse` — value type variants parse correctly.
- REQ-3.4-03 — `tests/spec/values.rs::spec_unit_property_parsing` — unit properties parse in numeric values.
- REQ-3.4-04 — `tests/spec/values.rs::spec_value_timeframe_property_parsing` — value timeframe properties parse.
- REQ-3.4-05 — `tests/spec/values.rs::spec_inheritance_property_parsing` — inheritance properties parse with overrides.
- REQ-3.4-06 — `tests/spec/values.rs::spec_documentation_property_parsing` — documentation properties parse in inline and block forms.
- REQ-3.4-07 — `tests/spec/values.rs::spec_generic_property_parsing` — custom properties parse as generic properties.
- REQ-3.4-08 — `tests/spec/values.rs::spec_value_type_variants_parse` — date/time value type parses.
- REQ-3.4-09 — `tests/spec/grammar.rs::spec_meaning_assessments_not_allowed_in_plans` — meaning assessments are only allowed in value definition blocks.
- REQ-3.4-10 — `tests/spec/grammar.rs::spec_meaning_requires_target_identifier` — meaning properties require an explicit target identifier.
- REQ-3.4-11 — `tests/spec/grammar.rs::spec_meaning_requires_valid_meanings` — meaning properties must declare valid meanings.
- REQ-3.4-12 — `tests/spec/grammar.rs::spec_meaning_labels_require_identifiers` — meaning labels must be identifiers (angle brackets).
- REQ-3.4-13 — `tests/spec/validation.rs::spec_enum_valid_values_require_identifiers` — enumeration valid values are identifiers (angle brackets).

## §3.5 Periods and Plans
- REQ-3.5-01 — `tests/spec/periods_plans.rs::spec_period_definition_parsing` — period definitions parse by name.
- REQ-3.5-02 — `tests/spec/periods_plans.rs::spec_period_parsing_structure` — period timeframe lines parse with range selectors.

## §3.6 Statements, Assessments, and Ranges
- REQ-3.6-01 — `tests/spec/statements_actions.rs::spec_timeframe_block_parsing` — timeframe blocks parse with nested statements.
- REQ-3.6-02 — `tests/spec/statements_actions.rs::spec_timeframe_requires_range_selector` — timeframe selectors require a start and end.
- REQ-3.6-03 — `tests/spec/validation.rs::spec_timeframe_selector_requires_period_definition` — timeframe selector identifiers must refer to defined periods.
- REQ-3.6-04 — `tests/spec/grammar.rs::spec_block_statements_require_period` — statements inside blocks must terminate with a period.
- REQ-3.6-05 — `tests/spec/grammar.rs::spec_blocks_require_colon` — blocks must be introduced with a colon.

## §3.7 Actions and Questions
- REQ-3.7-01 — `tests/spec/statements_actions.rs::spec_question_config_parsing_and_validation` — question configuration parses and validates references.
- REQ-3.7-02 — `tests/spec/statements_actions.rs::spec_message_expiration_parsing` — message expiration attaches to information, warning, and urgent warning actions.
- REQ-3.7-03 — `tests/spec/statements_actions.rs::spec_question_modifiers_parsing` — question modifiers parse (validate/type/style/expire).
- REQ-3.7-04 — `tests/spec/statements_actions.rs::spec_validate_answer_within_parsing` — validate answer within parsing attaches to ask blocks.
- REQ-3.7-05 — `tests/spec/statements_actions.rs::spec_listen_send_start_and_simple_command_parsing` — listen/send/start/simple command actions parse.
- REQ-3.7-06 — `tests/spec/statements_actions.rs::spec_question_expiration_block_parsing` — question expiration blocks parse with reminder statements.
- REQ-3.7-07 — `tests/spec/statements_actions.rs::spec_question_expiration_until_event_trigger_parsing` — question expiration supports until event triggers.
- REQ-3.7-08 — `tests/spec/statements_actions.rs::spec_message_action_keyword_parsing` — information, warning, and urgent warning are accepted as message action keywords.
- REQ-3.7-09 — `tests/spec/statements_actions.rs::spec_message_expiration_parsing` — message actions accept semicolon-separated addressee lists.

## §3.8 Events and Timing
- REQ-3.8-01 — `tests/spec/periods_plans.rs::spec_event_trigger_parsing` — event triggers parse for change/start/periodic.
- REQ-3.8-02 — `tests/spec/periods_plans.rs::spec_event_block_parsing` — event blocks attach statements to triggers.
- REQ-3.8-03 — `tests/spec/execution.rs::spec_scheduler_next_occurrence` — scheduler computes next occurrence for periods.
- REQ-3.8-04 — `tests/spec/periods_plans.rs::spec_event_trigger_duration_and_offset_parsing` — periodic triggers parse duration and offsets.

## §3.9 Communication & Actors
- REQ-3.9-01 — `tests/spec/actors_drugs.rs::spec_addressee_group_and_contact_logic_parsing` — addressee groups and contact logic parse.
- REQ-3.9-02 — `tests/spec/actors_drugs.rs::spec_addressee_contact_details_and_sequence_order_parsing` — contact details and sequence ordering parse.

## §3.10 Medication
- REQ-3.10-01 — `tests/spec/actors_drugs.rs::spec_drug_definition_validation` — drug definition validation rejects undefined units.
- REQ-3.10-02 — `tests/spec/actors_drugs.rs::spec_drug_interactions_parse` — drug interaction properties parse.
- REQ-3.10-03 — `tests/spec/actors_drugs.rs::spec_drug_dosage_and_admin_rules_parsing` — dosage safety and administration rules parse.

## §3.11 Data Contexts
- REQ-3.11-01 — `tests/spec/contexts_expressions.rs::spec_context_definition_parsing` — context definitions parse timeframe/data/value filter items.
- REQ-3.11-02 — `tests/spec/contexts_expressions.rs::spec_context_block_items_parsing` — context blocks parse data/value filters and nested statements.
- REQ-3.11-03 — `tests/spec/contexts_expressions.rs::spec_context_for_analysis_execution` — context for analysis executes with scoped timeframe.

## §3.12 Expressions and Statistical Analysis
- REQ-3.12-01 — `tests/spec/contexts_expressions.rs::spec_statistical_functions_parsing` — statistical function expressions parse in assignments.
- REQ-3.12-02 — `tests/spec/execution.rs::spec_timeframe_filtering` — timeframe filtering applies to statistical evaluations.
- REQ-3.12-03 — `tests/spec/execution.rs::spec_timeframe_variants` — timeframe variants resolve counts over different windows.
- REQ-3.12-04 — `tests/spec/execution.rs::spec_trend_analysis_evaluates` — trend analysis evaluates statistical trends over timeframes.
- REQ-3.12-05 — `tests/spec/validation.rs::spec_statistical_functions_require_timeframe_context` — statistical functions require an analysis timeframe context.
- REQ-3.12-06 — `tests/spec/contexts_expressions.rs::spec_date_diff_parsing` — date diff expressions parse.
- REQ-3.12-07 — `tests/spec/contexts_expressions.rs::spec_meaning_of_expression_parsing` — meaning-of expressions parse in assignments.

## §4.1 Core Unit Groups and Conversion
- REQ-4.1-01 — `tests/spec/units.rs::spec_builtin_units_cannot_be_redefined` — built-in units cannot be redefined.
- REQ-4.1-02 — `tests/spec/units.rs::spec_unit_conversions_within_groups` — unit conversions are supported within compatible groups.
- REQ-4.1-03 — `tests/spec/units.rs::spec_assignment_requires_unit_and_precision_match` — calculations and assignments require matching units and precision.

## §4.2 Required Properties
- REQ-4.2-01 — `tests/spec/validation.rs::spec_unit_requirement_validation` — numeric valid values require units.
- REQ-4.2-02 — `tests/spec/validation.rs::spec_unitless_assess_fails` — assessment ranges require units.
- REQ-4.2-03 — `tests/spec/validation.rs::spec_unitless_definition_fails` — numeric definitions require units.
- REQ-4.2-04 — `tests/spec/validation.rs::spec_ask_requires_question_property` — ask requires a question property on the value.
- REQ-4.2-05 — `tests/spec/validation.rs::spec_validation_error_line_number` — unit requirement errors report line numbers.
- REQ-4.2-06 — `tests/spec/validation.rs::spec_missing_valid_values_fails` — numbers and enumerations must define valid values.
- REQ-4.2-07 — `tests/spec/validation.rs::spec_valid_values_ranges_do_not_overlap` — valid value ranges must not overlap.
- REQ-4.2-07 — `tests/spec/validation.rs::spec_valid_values_datetime_ranges_do_not_overlap` — date/time valid value ranges must not overlap.

## §4.3 Data Flow and Validity
- REQ-4.3-01 — `tests/spec/validation.rs::spec_data_flow_use_before_assignment_fails` — values cannot be used before assignment.
- REQ-4.3-02 — `tests/spec/validation.rs::spec_calculation_does_not_initialize_value` — calculation properties do not seed values.
- REQ-4.3-03 — `tests/spec/validation.rs::spec_statistical_functions_do_not_require_local_init` — statistical functions do not require local initialization.
- REQ-4.3-04 — `tests/spec/validation.rs::spec_listen_and_context_initialize_values` — listen for and context data initialize values.
- REQ-4.3-05 — `tests/spec/validation.rs::spec_meaning_of_requires_question_when_uninitialized` — meaning-of expressions require an askable value when not initialized.
- REQ-4.3-05 — `tests/spec/validation.rs::spec_meaning_of_allows_question_when_uninitialized` — meaning-of is allowed when the value is askable.

## §4.4 Assessment Coverage
- REQ-4.4-01 — `tests/spec/validation.rs::spec_meaning_coverage_gaps_integer` — meaning ranges must cover valid values (integer gaps).
- REQ-4.4-02 — `tests/spec/validation.rs::spec_meaning_coverage_gaps_float` — meaning ranges must cover valid values (float gaps).
- REQ-4.4-03 — `tests/spec/validation.rs::spec_meaning_coverage_disjoint_ranges_ok` — disjoint valid ranges are allowed when fully covered.
- REQ-4.4-04 — `tests/spec/validation.rs::spec_validator_numeric_overlap` — overlapping numeric assessment ranges are invalid.
- REQ-4.4-05 — `tests/spec/validation.rs::spec_validator_enum_duplicate` — duplicate enumeration cases are invalid.
- REQ-4.4-06 — `tests/spec/validation.rs::spec_validator_integer_gap_message` — gap detection reports missing integer spans.
- REQ-4.4-07 — `tests/spec/validation.rs::spec_validator_float_gap_message` — gap detection reports missing float spans.
- REQ-4.4-08 — `tests/spec/validation.rs::spec_precision_gaps` — coverage gaps respect precision for float and integer ranges.
- REQ-4.4-09 — `tests/spec/validation.rs::spec_range_overlap` — overlapping ranges are rejected.
- REQ-4.4-10 — `tests/spec/validation.rs::spec_reproduce_missing_error` — missing coverage yields a validation error.
- REQ-4.4-11 — `tests/spec/validation.rs::spec_trend_requires_full_coverage` — trend assessments require full coverage.
- REQ-4.4-12 — `tests/spec/validation.rs::spec_precision_consistency` — numeric valid value ranges use consistent precision across bounds and intervals.
- REQ-4.4-13 — `tests/spec/validation.rs::spec_meaning_valid_meanings_must_be_used` — valid meanings must be fully used across meaning assessments.
- REQ-4.4-14 — `tests/spec/validation.rs::spec_meaning_invalid_label_rejected` — meaning labels must be drawn from declared valid meanings.

## §4.5 Range Compliance (Pre-Run Validation)
- REQ-4.5-01 — `tests/spec/validation.rs::spec_interval_creation_and_math` — interval math supports range compliance checks.
- REQ-4.5-02 — `tests/spec/validation.rs::spec_assignment_range_compliance_warning` — assignment range compliance fails when out of bounds.

## §4.6 Data Sufficiency
- REQ-4.6-01 — `tests/spec/validation.rs::spec_validator_requires_not_enough_data_case` — timeframe calculations require Not enough data handling.
- REQ-4.6-02 — `tests/spec/validation.rs::spec_validator_passes_with_not_enough_data` — Not enough data handling satisfies sufficiency.
- REQ-4.6-03 — `tests/spec/execution.rs::spec_not_enough_data_evaluation` — runtime evaluation returns NotEnoughData when history is insufficient.
- REQ-4.6-04 — `tests/spec/validation.rs::spec_not_enough_data_requires_statistical_target` — Not enough data is only allowed for statistical assessments.

## §4.7 Date/Time Semantics
- REQ-4.7-01 — `tests/spec/execution.rs::spec_date_time_range_evaluation` — date/time valid value ranges evaluate using date/time and time-of-day semantics.
- REQ-4.7-02 — `tests/spec/execution.rs::spec_date_diff_evaluation` — date diff expressions evaluate to quantities in requested units.

## §5 Execution Model
- REQ-5-01 — `tests/spec/execution.rs::spec_runtime_execution_flow` — runtime executes assignments and actions in order.
- REQ-5-02 — `tests/spec/execution.rs::spec_validity_reuse_timeframe` — reuse timeframes prevent re-asking within the validity window.
- REQ-5-03 — `tests/spec/execution.rs::spec_message_callback_missing_warns` — runtime emits a warning when a message action executes without a message callback.
- REQ-5-04 — `tests/spec/execution.rs::spec_simulation_mode_execution` — simulation mode executes plans at accelerated speed without real-time delays.

## §5.1 Validation Logic
- REQ-5.1-01 — `tests/spec/validation.rs::spec_validate_plan_fixture_suite` — full-plan validation passes for a complete plan.

## §5.2 Input Validation
- REQ-5.2-01 — `tests/spec/execution.rs::spec_numeric_input_precision_rejection` — numeric answers must respect the decimal precision implied by valid values.

## §5.3 Meaning Evaluation
- REQ-5.3-01 — `tests/spec/execution.rs::spec_meaning_of_evaluates` — meaning evaluation returns the assessed meaning.
- REQ-5.3-02 — `tests/spec/execution.rs::spec_meaning_of_missing_value` — meaning evaluation returns Missing when the source value is unknown.
- REQ-5.3-03 — `tests/spec/execution.rs::spec_meaning_of_nested_assessment` — meaning evaluation supports nested assessments.

## Pending / Ignored
None.
