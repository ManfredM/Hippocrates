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
| SREQ-HIPP-001 | REQ-HIPP-ACT-001, REQ-HIPP-ACT-002, REQ-HIPP-ACT-003, REQ-HIPP-ACT-004, REQ-HIPP-ACT-005, REQ-HIPP-ACT-006, REQ-HIPP-ACT-007, REQ-HIPP-ACT-008, REQ-HIPP-ACT-009, REQ-HIPP-ACT-010, REQ-HIPP-COMM-001, REQ-HIPP-COMM-002, REQ-HIPP-MED-001, REQ-HIPP-MED-002, REQ-HIPP-MED-003, REQ-HIPP-EXEC-001, REQ-HIPP-EXEC-002, REQ-HIPP-EXEC-003, REQ-HIPP-EXEC-004, REQ-HIPP-EXEC-005 | Care plan execution |
| SREQ-HIPP-002 | REQ-HIPP-LANG-001, REQ-HIPP-LANG-002, REQ-HIPP-LANG-003, REQ-HIPP-LANG-004, REQ-HIPP-BASIC-001, REQ-HIPP-BASIC-002, REQ-HIPP-BASIC-003, REQ-HIPP-BASIC-004, REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-002, REQ-HIPP-UNITS-003, REQ-HIPP-UNITS-004, REQ-HIPP-UNITS-005, REQ-HIPP-PROG-001, REQ-HIPP-EVT-001, REQ-HIPP-EVT-002, REQ-HIPP-EVT-003, REQ-HIPP-EVT-004, REQ-HIPP-EVT-005, REQ-HIPP-EVT-006, REQ-HIPP-EVT-007 | Readability by medical professionals |
| SREQ-HIPP-003 | _(No REQ sections; satisfied by SYS-HIPP-001..03, SYS-HIPP-018, SYS-HIPP-030..34)_ | Embeddable runtime (architectural) |
| SREQ-HIPP-004 | REQ-HIPP-VALID-001, REQ-HIPP-VALID-002, REQ-HIPP-VALID-003 | Validation without execution |
| SREQ-HIPP-005 | REQ-HIPP-PLAN-001, REQ-HIPP-PLAN-002, REQ-HIPP-ACT-001, REQ-HIPP-ACT-002, REQ-HIPP-ACT-003, REQ-HIPP-ACT-004, REQ-HIPP-ACT-005, REQ-HIPP-ACT-006, REQ-HIPP-ACT-007, REQ-HIPP-ACT-008, REQ-HIPP-ACT-009, REQ-HIPP-ACT-010, REQ-HIPP-EVT-001, REQ-HIPP-EVT-002, REQ-HIPP-EVT-003, REQ-HIPP-EVT-004, REQ-HIPP-EVT-005, REQ-HIPP-EVT-006, REQ-HIPP-EVT-007, REQ-HIPP-EXEC-001, REQ-HIPP-EXEC-002, REQ-HIPP-EXEC-003, REQ-HIPP-EXEC-004, REQ-HIPP-EXEC-005 | Event-driven execution |
| SREQ-HIPP-006 | _(No REQ sections; satisfied by SYS-HIPP-041)_ | Simulation mode (architectural) |
| SREQ-HIPP-010 | REQ-HIPP-STMT-001, REQ-HIPP-STMT-002, REQ-HIPP-STMT-003, REQ-HIPP-STMT-004, REQ-HIPP-STMT-005, REQ-HIPP-COVER-001, REQ-HIPP-COVER-002, REQ-HIPP-COVER-003, REQ-HIPP-COVER-004, REQ-HIPP-COVER-005, REQ-HIPP-COVER-006, REQ-HIPP-COVER-007, REQ-HIPP-COVER-008, REQ-HIPP-COVER-009, REQ-HIPP-COVER-010, REQ-HIPP-COVER-011, REQ-HIPP-COVER-012, REQ-HIPP-COVER-013, REQ-HIPP-COVER-014 | Completeness |
| SREQ-HIPP-011 | REQ-HIPP-BASIC-001, REQ-HIPP-BASIC-002, REQ-HIPP-BASIC-003, REQ-HIPP-BASIC-004, REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-002, REQ-HIPP-UNITS-003, REQ-HIPP-UNITS-004, REQ-HIPP-UNITS-005, REQ-HIPP-PROG-001, REQ-HIPP-REQP-001, REQ-HIPP-REQP-002, REQ-HIPP-REQP-003, REQ-HIPP-REQP-004, REQ-HIPP-REQP-005, REQ-HIPP-REQP-006, REQ-HIPP-REQP-007 | Readability over defaults |
| SREQ-HIPP-012 | REQ-HIPP-VALUE-001, REQ-HIPP-VALUE-002, REQ-HIPP-VALUE-003, REQ-HIPP-VALUE-004, REQ-HIPP-VALUE-005, REQ-HIPP-VALUE-006, REQ-HIPP-VALUE-007, REQ-HIPP-VALUE-008, REQ-HIPP-VALUE-009, REQ-HIPP-VALUE-010, REQ-HIPP-VALUE-011, REQ-HIPP-VALUE-012, REQ-HIPP-VALUE-013, REQ-HIPP-MEAN-001, REQ-HIPP-MEAN-002, REQ-HIPP-MEAN-003 | Meaningful values |
| SREQ-HIPP-013 | REQ-HIPP-LANG-001, REQ-HIPP-LANG-002, REQ-HIPP-LANG-003, REQ-HIPP-LANG-004, REQ-HIPP-STMT-001, REQ-HIPP-STMT-002, REQ-HIPP-STMT-003, REQ-HIPP-STMT-004, REQ-HIPP-STMT-005, REQ-HIPP-CTX-001, REQ-HIPP-CTX-002, REQ-HIPP-CTX-003 | Contextual statements |
| SREQ-HIPP-014 | REQ-HIPP-LANG-001, REQ-HIPP-LANG-002, REQ-HIPP-LANG-003, REQ-HIPP-LANG-004, REQ-HIPP-COMM-001, REQ-HIPP-COMM-002 | Separation of concerns |
| SREQ-HIPP-015 | REQ-HIPP-VALUE-001, REQ-HIPP-VALUE-002, REQ-HIPP-VALUE-003, REQ-HIPP-VALUE-004, REQ-HIPP-VALUE-005, REQ-HIPP-VALUE-006, REQ-HIPP-VALUE-007, REQ-HIPP-VALUE-008, REQ-HIPP-VALUE-009, REQ-HIPP-VALUE-010, REQ-HIPP-VALUE-011, REQ-HIPP-VALUE-012, REQ-HIPP-VALUE-013, REQ-HIPP-REQP-001, REQ-HIPP-REQP-002, REQ-HIPP-REQP-003, REQ-HIPP-REQP-004, REQ-HIPP-REQP-005, REQ-HIPP-REQP-006, REQ-HIPP-REQP-007, REQ-HIPP-RANGE-001, REQ-HIPP-RANGE-002 | Value constraints |
| SREQ-HIPP-016 | _(No REQ sections; satisfied by SYS-HIPP-015, SYS-HIPP-026)_ | Immutable history (architectural) |
| SREQ-HIPP-017 | _(No REQ sections)_ | Localization support (future) |
| SREQ-HIPP-018 | REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-002, REQ-HIPP-UNITS-003, REQ-HIPP-UNITS-004, REQ-HIPP-UNITS-005, REQ-HIPP-MED-001, REQ-HIPP-MED-002, REQ-HIPP-MED-003, REQ-HIPP-CORE-001, REQ-HIPP-CORE-002, REQ-HIPP-CORE-003, REQ-HIPP-CORE-005, REQ-HIPP-DTIME-001, REQ-HIPP-DTIME-002 | Medical domain awareness |
| SREQ-HIPP-019 | _(No REQ sections; satisfied by SYS-HIPP-015, SYS-HIPP-026)_ | Automatic persistence (architectural) |
| SREQ-HIPP-020 | REQ-HIPP-PLAN-001, REQ-HIPP-PLAN-002 | Plan autonomy |
| SREQ-HIPP-030 | REQ-HIPP-LANG-001, REQ-HIPP-LANG-002, REQ-HIPP-LANG-003, REQ-HIPP-LANG-004 | No comparison operators |
| SREQ-HIPP-031 | REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-002, REQ-HIPP-UNITS-003, REQ-HIPP-UNITS-004, REQ-HIPP-UNITS-005, REQ-HIPP-CORE-001, REQ-HIPP-CORE-002, REQ-HIPP-CORE-003, REQ-HIPP-CORE-005 | Unit discipline |
| SREQ-HIPP-032 | REQ-HIPP-COVER-001, REQ-HIPP-COVER-002, REQ-HIPP-COVER-003, REQ-HIPP-COVER-004, REQ-HIPP-COVER-005, REQ-HIPP-COVER-006, REQ-HIPP-COVER-007, REQ-HIPP-COVER-008, REQ-HIPP-COVER-009, REQ-HIPP-COVER-010, REQ-HIPP-COVER-011, REQ-HIPP-COVER-012, REQ-HIPP-COVER-013, REQ-HIPP-COVER-014 | Exhaustive assessment coverage |
| SREQ-HIPP-033 | REQ-HIPP-FLOW-001, REQ-HIPP-FLOW-002, REQ-HIPP-FLOW-003, REQ-HIPP-FLOW-004, REQ-HIPP-FLOW-005 | Data flow validation |
| SREQ-HIPP-034 | REQ-HIPP-INPUT-001 | Input precision validation |
| SREQ-HIPP-035 | REQ-HIPP-EXPR-001, REQ-HIPP-EXPR-002, REQ-HIPP-EXPR-003, REQ-HIPP-EXPR-004, REQ-HIPP-EXPR-005, REQ-HIPP-EXPR-006, REQ-HIPP-EXPR-007, REQ-HIPP-SUFF-001, REQ-HIPP-SUFF-002, REQ-HIPP-SUFF-003, REQ-HIPP-SUFF-004 | Data sufficiency handling |
| SREQ-HIPP-036 | REQ-HIPP-ACT-001, REQ-HIPP-ACT-002, REQ-HIPP-ACT-003, REQ-HIPP-ACT-004, REQ-HIPP-ACT-005, REQ-HIPP-ACT-006, REQ-HIPP-ACT-007, REQ-HIPP-ACT-008, REQ-HIPP-ACT-009, REQ-HIPP-ACT-010 | Plan completion actions (`after plan:` block) |
| SREQ-HIPP-037 | REQ-HIPP-CORE-001, REQ-HIPP-CORE-002, REQ-HIPP-CORE-003, REQ-HIPP-CORE-005, REQ-HIPP-VALID-001, REQ-HIPP-VALID-002, REQ-HIPP-VALID-003 | LLM-correctable error diagnostics |
| SREQ-HIPP-040 | _(Meta-requirement)_ | Requirements traceability |
| SREQ-HIPP-041 | _(Meta-requirement)_ | Reproducible verification |
| SREQ-HIPP-042 | _(Meta-requirement)_ | Class II documentation readiness |

---

## 3. System Requirements to System Design

This table maps REQ specification sections to DES-* design elements, derived from the traceability section of `02-system-design.md`.

| REQ Section | DES IDs | Description |
|-------------|---------|-------------|
| REQ-HIPP-LANG-001, REQ-HIPP-LANG-002, REQ-HIPP-LANG-003, REQ-HIPP-LANG-004 | SYS-HIPP-010 | Parser enforces syntax rules (angle brackets, no comparisons, blocks) |
| REQ-HIPP-BASIC-001, REQ-HIPP-BASIC-002, REQ-HIPP-BASIC-003, REQ-HIPP-BASIC-004, REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-002, REQ-HIPP-UNITS-003, REQ-HIPP-UNITS-004, REQ-HIPP-UNITS-005, REQ-HIPP-PROG-001 | SYS-HIPP-010, SYS-HIPP-011 | Parser and AST representation |
| REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-002, REQ-HIPP-UNITS-003, REQ-HIPP-UNITS-004, REQ-HIPP-UNITS-005 | SYS-HIPP-015, SYS-HIPP-021 | Environment unit map; Chrono for time units |
| REQ-HIPP-VALUE-001, REQ-HIPP-VALUE-002, REQ-HIPP-VALUE-003, REQ-HIPP-VALUE-004, REQ-HIPP-VALUE-005, REQ-HIPP-VALUE-006, REQ-HIPP-VALUE-007, REQ-HIPP-VALUE-008, REQ-HIPP-VALUE-009, REQ-HIPP-VALUE-010, REQ-HIPP-VALUE-011, REQ-HIPP-VALUE-012, REQ-HIPP-VALUE-013 | SYS-HIPP-015, SYS-HIPP-016 | Environment state store; Evaluator for meaning resolution |
| REQ-HIPP-PLAN-001, REQ-HIPP-PLAN-002 | SYS-HIPP-013, SYS-HIPP-014 | Executor event loop; Scheduler period calculations |
| REQ-HIPP-STMT-001, REQ-HIPP-STMT-002, REQ-HIPP-STMT-003, REQ-HIPP-STMT-004, REQ-HIPP-STMT-005 | SYS-HIPP-010, SYS-HIPP-012 | Parser grammar; Validator completeness checks |
| REQ-HIPP-ACT-001, REQ-HIPP-ACT-002, REQ-HIPP-ACT-003, REQ-HIPP-ACT-004, REQ-HIPP-ACT-005, REQ-HIPP-ACT-006, REQ-HIPP-ACT-007, REQ-HIPP-ACT-008, REQ-HIPP-ACT-009, REQ-HIPP-ACT-010 | SYS-HIPP-013, SYS-HIPP-018 | Executor action handling; FFI callbacks |
| REQ-HIPP-EVT-001, REQ-HIPP-EVT-002, REQ-HIPP-EVT-003, REQ-HIPP-EVT-004, REQ-HIPP-EVT-005, REQ-HIPP-EVT-006, REQ-HIPP-EVT-007 | SYS-HIPP-013, SYS-HIPP-014, SYS-HIPP-021 | Executor, Scheduler, Chrono |
| REQ-HIPP-COMM-001, REQ-HIPP-COMM-002 | SYS-HIPP-018, SYS-HIPP-031 | FFI callback model |
| REQ-HIPP-MED-001, REQ-HIPP-MED-002, REQ-HIPP-MED-003 | SYS-HIPP-010, SYS-HIPP-012 | Parser drug definitions; Validator drug checks |
| REQ-HIPP-CTX-001, REQ-HIPP-CTX-002, REQ-HIPP-CTX-003 | SYS-HIPP-015, SYS-HIPP-016 | Environment context stack; Evaluator scoping |
| REQ-HIPP-EXPR-001, REQ-HIPP-EXPR-002, REQ-HIPP-EXPR-003, REQ-HIPP-EXPR-004, REQ-HIPP-EXPR-005, REQ-HIPP-EXPR-006, REQ-HIPP-EXPR-007 | SYS-HIPP-016 | Evaluator statistical functions |
| REQ-HIPP-CORE-001, REQ-HIPP-CORE-002, REQ-HIPP-CORE-003, REQ-HIPP-CORE-005 | SYS-HIPP-012, SYS-HIPP-015 | Validator unit checks; Environment unit map |
| REQ-HIPP-REQP-001, REQ-HIPP-REQP-002, REQ-HIPP-REQP-003, REQ-HIPP-REQP-004, REQ-HIPP-REQP-005, REQ-HIPP-REQP-006, REQ-HIPP-REQP-007 | SYS-HIPP-012 | Validator semantic checks |
| REQ-HIPP-FLOW-001, REQ-HIPP-FLOW-002, REQ-HIPP-FLOW-003, REQ-HIPP-FLOW-004, REQ-HIPP-FLOW-005 | SYS-HIPP-012 | Validator data flow analysis |
| REQ-HIPP-COVER-001, REQ-HIPP-COVER-002, REQ-HIPP-COVER-003, REQ-HIPP-COVER-004, REQ-HIPP-COVER-005, REQ-HIPP-COVER-006, REQ-HIPP-COVER-007, REQ-HIPP-COVER-008, REQ-HIPP-COVER-009, REQ-HIPP-COVER-010, REQ-HIPP-COVER-011, REQ-HIPP-COVER-012, REQ-HIPP-COVER-013, REQ-HIPP-COVER-014 | SYS-HIPP-012 | Validator coverage analysis |
| REQ-HIPP-RANGE-001, REQ-HIPP-RANGE-002 | SYS-HIPP-012 | Validator interval validation |
| REQ-HIPP-SUFF-001, REQ-HIPP-SUFF-002, REQ-HIPP-SUFF-003, REQ-HIPP-SUFF-004 | SYS-HIPP-012, SYS-HIPP-016 | Validator sufficiency checks; Evaluator NotEnoughData |
| REQ-HIPP-DTIME-001, REQ-HIPP-DTIME-002 | SYS-HIPP-016, SYS-HIPP-021 | Evaluator date handling; Chrono |
| REQ-HIPP-EXEC-001, REQ-HIPP-EXEC-002, REQ-HIPP-EXEC-003, REQ-HIPP-EXEC-004, REQ-HIPP-EXEC-005 | SYS-HIPP-013, SYS-HIPP-015 | Executor event loop; Environment state |
| REQ-HIPP-CORE-001, REQ-HIPP-CORE-002, REQ-HIPP-CORE-003, REQ-HIPP-CORE-005 | SYS-HIPP-010 | Parser human-readable error mapping |
| REQ-HIPP-VALID-001, REQ-HIPP-VALID-002, REQ-HIPP-VALID-003 | SYS-HIPP-012 | Validator pipeline |
| REQ-HIPP-VALID-001, REQ-HIPP-VALID-002, REQ-HIPP-VALID-003 | SYS-HIPP-012 | Validator undefined reference detection and suggestion generation |
| REQ-HIPP-INPUT-001 | SYS-HIPP-015, SYS-HIPP-042 | Environment input validation; Input channel |
| REQ-HIPP-MEAN-001, REQ-HIPP-MEAN-002, REQ-HIPP-MEAN-003 | SYS-HIPP-016 | Evaluator meaning resolution |

---

## 4. System Design to Detailed Design

This table maps DES-* design elements to DDR-* detailed design elements, derived from the traceability section of `03-detailed-design.md`.

| DES ID | DDR ID Range | Component |
|--------|-------------|-----------|
| SYS-HIPP-010 | DET-HIPP-PARSER-001..06 | Parser (PEG grammar, AST construction, indentation, entry point, ordinal/bare-unit sugar, error humanization) |
| SYS-HIPP-011 | DET-HIPP-DOM-001..06, DET-HIPP-PARSER-002 | AST representation and domain model types |
| SYS-HIPP-012 | DET-HIPP-VAL-001..08 | Multi-layer validator (pipeline, semantics, intervals, data flow, coverage, error reporting, undefined references, suggested fixes) |
| SYS-HIPP-013 | DET-HIPP-RT-001, DET-HIPP-RT-003, DET-HIPP-RT-008, DET-HIPP-RT-010 | Runtime executor (Engine struct, Executor, execution modes, after plan) |
| SYS-HIPP-014 | DET-HIPP-RT-005 | Scheduler |
| SYS-HIPP-015 | DET-HIPP-DOM-001..06, DET-HIPP-RT-002, DET-HIPP-RT-007 | Environment (state store, domain types, input validation) |
| SYS-HIPP-016 | DET-HIPP-RT-004 | Evaluator |
| SYS-HIPP-017 | DET-HIPP-RT-006 | Session (multi-plan coordinator) |
| SYS-HIPP-018 | DET-HIPP-FFI-001..19 | FFI layer (all C-compatible API functions) |
| SYS-HIPP-019 | DET-HIPP-FMT-001 | Formatter |

---

## 5. System Requirements to Test Cases (Adopted)

This section adopts the content from `tests/spec/TRACEABILITY.md` verbatim. All 93 REQ-to-test-function mappings are listed below, organized by specification section.

### 5.1 -- Section 2: Language Principles
- REQ-HIPP-LANG-001 -- `tests/spec/grammar.rs::spec_identifiers_require_angle_brackets` -- identifiers must use angle brackets.
- REQ-HIPP-LANG-002 -- `tests/spec/grammar.rs::spec_string_literal_rejects_angle_brackets` -- string literals must not contain angle brackets.
- REQ-HIPP-LANG-003 -- `tests/spec/grammar.rs::spec_no_comparison_operators` -- comparison operators are not supported; use ranges.
- REQ-HIPP-LANG-004 -- `tests/spec/grammar.rs::spec_block_requires_newline_after_colon` -- block openings require a newline and indented block.

### 5.2 -- Section 3.1: Basic Elements
- REQ-HIPP-BASIC-001 -- `tests/spec/contexts_expressions.rs::spec_time_indications_parsing` -- time indications parse for now, weekday, and time-of-day.
- REQ-HIPP-BASIC-002 -- `tests/spec/contexts_expressions.rs::spec_relative_time_from_now_parsing` -- relative time expressions from now parse.
- REQ-HIPP-BASIC-003 -- `tests/spec/grammar.rs::spec_inline_colon_requires_block` -- inline ':' forms are only allowed where explicitly shown.
- REQ-HIPP-BASIC-004 -- `tests/spec/contexts_expressions.rs::spec_date_time_literals_parsing` -- date/time literals parse for date and date-time forms.

### 5.3 -- Section 3.2: Units and Quantities
- REQ-HIPP-UNITS-001 -- `tests/spec/units.rs::spec_custom_unit_pluralization_is_canonical` -- custom unit pluralization canonicalizes values.
- REQ-HIPP-UNITS-002 -- `tests/spec/units.rs::spec_standard_units_still_work` -- standard units work in calculations.
- REQ-HIPP-UNITS-003 -- `tests/spec/units.rs::spec_custom_unit_abbreviation_is_canonical` -- custom unit abbreviations canonicalize values.
- REQ-HIPP-UNITS-004 -- `tests/spec/units.rs::spec_custom_unit_quantity_parsing` -- custom unit quantities parse with definitions.
- REQ-HIPP-UNITS-005 -- `tests/spec/grammar.rs::spec_unitless_numeric_literal_fails` -- numeric literals must include units.

### 5.4 -- Section 3.3: Program Structure
- REQ-HIPP-PROG-001 -- `tests/spec/fixtures.rs::spec_full_fixture_parses_core_definitions` -- multi-definition fixtures parse core definitions.

### 5.5 -- Section 3.4: Values
- REQ-HIPP-VALUE-001 -- `tests/spec/values.rs::spec_value_definition_parsing` -- value definitions parse from fixtures.
- REQ-HIPP-VALUE-002 -- `tests/spec/values.rs::spec_value_type_variants_parse` -- value type variants parse correctly.
- REQ-HIPP-VALUE-003 -- `tests/spec/values.rs::spec_unit_property_parsing` -- unit properties parse in numeric values.
- REQ-HIPP-VALUE-004 -- `tests/spec/values.rs::spec_value_timeframe_property_parsing` -- value timeframe properties parse.
- REQ-HIPP-VALUE-005 -- `tests/spec/values.rs::spec_inheritance_property_parsing` -- inheritance properties parse with overrides.
- REQ-HIPP-VALUE-006 -- `tests/spec/values.rs::spec_documentation_property_parsing` -- documentation properties parse in inline and block forms.
- REQ-HIPP-VALUE-007 -- `tests/spec/values.rs::spec_generic_property_parsing` -- custom properties parse as generic properties.
- REQ-HIPP-VALUE-008 -- `tests/spec/values.rs::spec_value_type_variants_parse` -- date/time value type parses.
- REQ-HIPP-VALUE-009 -- `tests/spec/grammar.rs::spec_meaning_assessments_not_allowed_in_plans` -- meaning assessments are only allowed in value definition blocks.
- REQ-HIPP-VALUE-010 -- `tests/spec/grammar.rs::spec_meaning_requires_target_identifier` -- meaning properties require an explicit target identifier.
- REQ-HIPP-VALUE-011 -- `tests/spec/grammar.rs::spec_meaning_requires_valid_meanings` -- meaning properties must declare valid meanings.
- REQ-HIPP-VALUE-012 -- `tests/spec/grammar.rs::spec_meaning_labels_require_identifiers` -- meaning labels must be identifiers (angle brackets).
- REQ-HIPP-VALUE-013 -- `tests/spec/validation.rs::spec_enum_valid_values_require_identifiers` -- enumeration valid values are identifiers (angle brackets).

### 5.6 -- Section 3.5: Periods and Plans
- REQ-HIPP-PLAN-001 -- `tests/spec/periods_plans.rs::spec_period_definition_parsing` -- period definitions parse by name.
- REQ-HIPP-PLAN-002 -- `tests/spec/periods_plans.rs::spec_period_parsing_structure` -- period timeframe lines parse with range selectors.

### 5.7 -- Section 3.6: Statements, Assessments, and Ranges
- REQ-HIPP-STMT-001 -- `tests/spec/statements_actions.rs::spec_timeframe_block_parsing` -- timeframe blocks parse with nested statements.
- REQ-HIPP-STMT-002 -- `tests/spec/statements_actions.rs::spec_timeframe_requires_range_selector` -- timeframe selectors require a start and end.
- REQ-HIPP-STMT-003 -- `tests/spec/validation.rs::spec_timeframe_selector_requires_period_definition` -- timeframe selector identifiers must refer to defined periods.
- REQ-HIPP-STMT-004 -- `tests/spec/grammar.rs::spec_block_statements_require_period` -- statements inside blocks must terminate with a period.
- REQ-HIPP-STMT-005 -- `tests/spec/grammar.rs::spec_blocks_require_colon` -- blocks must be introduced with a colon.

### 5.8 -- Section 3.7: Actions and Questions
- REQ-HIPP-ACT-001 -- `tests/spec/statements_actions.rs::spec_question_config_parsing_and_validation` -- question configuration parses and validates references.
- REQ-HIPP-ACT-002 -- `tests/spec/statements_actions.rs::spec_message_expiration_parsing` -- message expiration attaches to information, warning, and urgent warning actions.
- REQ-HIPP-ACT-003 -- `tests/spec/statements_actions.rs::spec_question_modifiers_parsing` -- question modifiers parse (validate/type/style/expire).
- REQ-HIPP-ACT-004 -- `tests/spec/statements_actions.rs::spec_validate_answer_within_parsing` -- validate answer within parsing attaches to ask blocks.
- REQ-HIPP-ACT-005 -- `tests/spec/statements_actions.rs::spec_listen_send_start_and_simple_command_parsing` -- listen/send/start/simple command actions parse.
- REQ-HIPP-ACT-006 -- `tests/spec/statements_actions.rs::spec_question_expiration_block_parsing` -- question expiration blocks parse with reminder statements.
- REQ-HIPP-ACT-007 -- `tests/spec/statements_actions.rs::spec_question_expiration_until_event_trigger_parsing` -- question expiration supports until event triggers.
- REQ-HIPP-ACT-008 -- `tests/spec/statements_actions.rs::spec_message_action_keyword_parsing` -- information, warning, and urgent warning are accepted as message action keywords.
- REQ-HIPP-ACT-009 -- `tests/spec/statements_actions.rs::spec_message_expiration_parsing` -- message actions accept semicolon-separated addressee lists.
- REQ-HIPP-ACT-010 -- `tests/spec/periods_plans.rs::spec_after_plan_block_parsing` -- `after plan:` block parses into `PlanBlock::AfterPlan` AST node.

### 5.9 -- Section 3.8: Events and Timing
- REQ-HIPP-EVT-001 -- `tests/spec/periods_plans.rs::spec_event_trigger_parsing` -- event triggers parse for change/start/periodic.
- REQ-HIPP-EVT-002 -- `tests/spec/periods_plans.rs::spec_event_block_parsing` -- event blocks attach statements to triggers.
- REQ-HIPP-EVT-003 -- `tests/spec/execution.rs::spec_scheduler_next_occurrence` -- scheduler computes next occurrence for periods.
- REQ-HIPP-EVT-004 -- `tests/spec/periods_plans.rs::spec_event_trigger_duration_and_offset_parsing` -- periodic triggers parse duration and offsets.
- REQ-HIPP-EVT-005 -- `tests/spec/periods_plans.rs::spec_event_trigger_time_of_day_parsing` -- periodic triggers parse `at <time>` clause.
- REQ-HIPP-EVT-006 -- `tests/integration/simulation.rs::test_period_based_repetition_within_duration` -- period-based triggers fire at every occurrence within duration window.
- REQ-HIPP-EVT-007 -- `tests/spec/periods_plans.rs::spec_bare_unit_trigger_parsing` -- bare unit triggers (`every day`) parse to interval=1.0. `tests/spec/periods_plans.rs::spec_ordinal_trigger_parsing` -- ordinal triggers (`every third day`) parse to interval=3.0.

### 5.10 -- Section 3.9: Communication and Actors
- REQ-HIPP-COMM-001 -- `tests/spec/actors_drugs.rs::spec_addressee_group_and_contact_logic_parsing` -- addressee groups and contact logic parse.
- REQ-HIPP-COMM-002 -- `tests/spec/actors_drugs.rs::spec_addressee_contact_details_and_sequence_order_parsing` -- contact details and sequence ordering parse.

### 5.11 -- Section 3.10: Medication
- REQ-HIPP-MED-001 -- `tests/spec/actors_drugs.rs::spec_drug_definition_validation` -- drug definition validation rejects undefined units.
- REQ-HIPP-MED-002 -- `tests/spec/actors_drugs.rs::spec_drug_interactions_parse` -- drug interaction properties parse.
- REQ-HIPP-MED-003 -- `tests/spec/actors_drugs.rs::spec_drug_dosage_and_admin_rules_parsing` -- dosage safety and administration rules parse.

### 5.12 -- Section 3.11: Data Contexts
- REQ-HIPP-CTX-001 -- `tests/spec/contexts_expressions.rs::spec_context_definition_parsing` -- context definitions parse timeframe/data/value filter items.
- REQ-HIPP-CTX-002 -- `tests/spec/contexts_expressions.rs::spec_context_block_items_parsing` -- context blocks parse data/value filters and nested statements.
- REQ-HIPP-CTX-003 -- `tests/spec/contexts_expressions.rs::spec_context_for_analysis_execution` -- context for analysis executes with scoped timeframe.

### 5.13 -- Section 3.12: Expressions and Statistical Analysis
- REQ-HIPP-EXPR-001 -- `tests/spec/contexts_expressions.rs::spec_statistical_functions_parsing` -- statistical function expressions parse in assignments.
- REQ-HIPP-EXPR-002 -- `tests/spec/execution.rs::spec_timeframe_filtering` -- timeframe filtering applies to statistical evaluations.
- REQ-HIPP-EXPR-003 -- `tests/spec/execution.rs::spec_timeframe_variants` -- timeframe variants resolve counts over different windows.
- REQ-HIPP-EXPR-004 -- `tests/spec/execution.rs::spec_trend_analysis_evaluates` -- trend analysis evaluates statistical trends over timeframes.
- REQ-HIPP-EXPR-005 -- `tests/spec/validation.rs::spec_statistical_functions_require_timeframe_context` -- statistical functions require an analysis timeframe context.
- REQ-HIPP-EXPR-006 -- `tests/spec/contexts_expressions.rs::spec_date_diff_parsing` -- date diff expressions parse.
- REQ-HIPP-EXPR-007 -- `tests/spec/contexts_expressions.rs::spec_meaning_of_expression_parsing` -- meaning-of expressions parse in assignments.

### 5.14 -- Section 4.1: Core Unit Groups and Conversion
- REQ-HIPP-CORE-001 -- `tests/spec/units.rs::spec_builtin_units_cannot_be_redefined` -- built-in units cannot be redefined.
- REQ-HIPP-CORE-002 -- `tests/spec/units.rs::spec_unit_conversions_within_groups` -- unit conversions are supported within compatible groups.
- REQ-HIPP-CORE-003 -- `tests/spec/units.rs::spec_assignment_requires_unit_and_precision_match` -- calculations and assignments require matching units and precision.

### 5.14a -- Section 4.1 (REQ-HIPP-CORE-005): Parse Error Humanization
- REQ-HIPP-CORE-005 -- `tests/spec/validation.rs::spec_parse_error_human_readable` -- parse errors include human-readable descriptions, not raw Rule names.

### 5.15 -- Section 4.2: Required Properties
- REQ-HIPP-REQP-001 -- `tests/spec/validation.rs::spec_unit_requirement_validation` -- numeric valid values require units.
- REQ-HIPP-REQP-002 -- `tests/spec/validation.rs::spec_unitless_assess_fails` -- assessment ranges require units.
- REQ-HIPP-REQP-003 -- `tests/spec/validation.rs::spec_unitless_definition_fails` -- numeric definitions require units.
- REQ-HIPP-REQP-004 -- `tests/spec/validation.rs::spec_ask_requires_question_property` -- ask requires a question property on the value.
- REQ-HIPP-REQP-005 -- `tests/spec/validation.rs::spec_validation_error_line_number` -- unit requirement errors report line numbers.
- REQ-HIPP-REQP-006 -- `tests/spec/validation.rs::spec_missing_valid_values_fails` -- numbers and enumerations must define valid values.
- REQ-HIPP-REQP-007 -- `tests/spec/validation.rs::spec_valid_values_ranges_do_not_overlap` -- valid value ranges must not overlap.
- REQ-HIPP-REQP-007 -- `tests/spec/validation.rs::spec_valid_values_datetime_ranges_do_not_overlap` -- date/time valid value ranges must not overlap.

### 5.16 -- Section 4.3: Data Flow and Validity
- REQ-HIPP-FLOW-001 -- `tests/spec/validation.rs::spec_data_flow_use_before_assignment_fails` -- values cannot be used before assignment.
- REQ-HIPP-FLOW-002 -- `tests/spec/validation.rs::spec_calculation_does_not_initialize_value` -- calculation properties do not seed values.
- REQ-HIPP-FLOW-003 -- `tests/spec/validation.rs::spec_statistical_functions_do_not_require_local_init` -- statistical functions do not require local initialization.
- REQ-HIPP-FLOW-004 -- `tests/spec/validation.rs::spec_listen_and_context_initialize_values` -- listen for and context data initialize values.
- REQ-HIPP-FLOW-005 -- `tests/spec/validation.rs::spec_meaning_of_requires_question_when_uninitialized` -- meaning-of expressions require an askable value when not initialized.
- REQ-HIPP-FLOW-005 -- `tests/spec/validation.rs::spec_meaning_of_allows_question_when_uninitialized` -- meaning-of is allowed when the value is askable.

### 5.17 -- Section 4.4: Assessment Coverage
- REQ-HIPP-COVER-001 -- `tests/spec/validation.rs::spec_meaning_coverage_gaps_integer` -- meaning ranges must cover valid values (integer gaps).
- REQ-HIPP-COVER-002 -- `tests/spec/validation.rs::spec_meaning_coverage_gaps_float` -- meaning ranges must cover valid values (float gaps).
- REQ-HIPP-COVER-003 -- `tests/spec/validation.rs::spec_meaning_coverage_disjoint_ranges_ok` -- disjoint valid ranges are allowed when fully covered.
- REQ-HIPP-COVER-004 -- `tests/spec/validation.rs::spec_validator_numeric_overlap` -- overlapping numeric assessment ranges are invalid.
- REQ-HIPP-COVER-005 -- `tests/spec/validation.rs::spec_validator_enum_duplicate` -- duplicate enumeration cases are invalid.
- REQ-HIPP-COVER-006 -- `tests/spec/validation.rs::spec_validator_integer_gap_message` -- gap detection reports missing integer spans.
- REQ-HIPP-COVER-007 -- `tests/spec/validation.rs::spec_validator_float_gap_message` -- gap detection reports missing float spans.
- REQ-HIPP-COVER-008 -- `tests/spec/validation.rs::spec_precision_gaps` -- coverage gaps respect precision for float and integer ranges.
- REQ-HIPP-COVER-009 -- `tests/spec/validation.rs::spec_range_overlap` -- overlapping ranges are rejected.
- REQ-HIPP-COVER-010 -- `tests/spec/validation.rs::spec_reproduce_missing_error` -- missing coverage yields a validation error.
- REQ-HIPP-COVER-011 -- `tests/spec/validation.rs::spec_trend_requires_full_coverage` -- trend assessments require full coverage.
- REQ-HIPP-COVER-012 -- `tests/spec/validation.rs::spec_precision_consistency` -- numeric valid value ranges use consistent precision across bounds and intervals.
- REQ-HIPP-COVER-013 -- `tests/spec/validation.rs::spec_meaning_valid_meanings_must_be_used` -- valid meanings must be fully used across meaning assessments.
- REQ-HIPP-COVER-014 -- `tests/spec/validation.rs::spec_meaning_invalid_label_rejected` -- meaning labels must be drawn from declared valid meanings.

### 5.18 -- Section 4.5: Range Compliance
- REQ-HIPP-RANGE-001 -- `tests/spec/validation.rs::spec_interval_creation_and_math` -- interval math supports range compliance checks.
- REQ-HIPP-RANGE-002 -- `tests/spec/validation.rs::spec_assignment_range_compliance_warning` -- assignment range compliance fails when out of bounds.

### 5.19 -- Section 4.6: Data Sufficiency
- REQ-HIPP-SUFF-001 -- `tests/spec/validation.rs::spec_validator_requires_not_enough_data_case` -- timeframe calculations require Not enough data handling.
- REQ-HIPP-SUFF-002 -- `tests/spec/validation.rs::spec_validator_passes_with_not_enough_data` -- Not enough data handling satisfies sufficiency.
- REQ-HIPP-SUFF-003 -- `tests/spec/execution.rs::spec_not_enough_data_evaluation` -- runtime evaluation returns NotEnoughData when history is insufficient.
- REQ-HIPP-SUFF-004 -- `tests/spec/validation.rs::spec_not_enough_data_requires_statistical_target` -- Not enough data is only allowed for statistical assessments.

### 5.20 -- Section 4.7: Date/Time Semantics
- REQ-HIPP-DTIME-001 -- `tests/spec/execution.rs::spec_date_time_range_evaluation` -- date/time valid value ranges evaluate using date/time and time-of-day semantics.
- REQ-HIPP-DTIME-002 -- `tests/spec/execution.rs::spec_date_diff_evaluation` -- date diff expressions evaluate to quantities in requested units.

### 5.21 -- Section 5: Execution Model
- REQ-HIPP-EXEC-001 -- `tests/spec/execution.rs::spec_runtime_execution_flow` -- runtime executes assignments and actions in order.
- REQ-HIPP-EXEC-002 -- `tests/spec/execution.rs::spec_validity_reuse_timeframe` -- reuse timeframes prevent re-asking within the validity window.
- REQ-HIPP-EXEC-003 -- `tests/spec/execution.rs::spec_message_callback_missing_warns` -- runtime emits a warning when a message action executes without a message callback.
- REQ-HIPP-EXEC-005 -- `tests/integration/simulation.rs::test_time_pinned_periodic_trigger` -- time-pinned triggers fire at specified time, not plan start time.

### 5.22 -- Section 5.1: Validation Logic
- REQ-HIPP-VALID-001 -- `tests/spec/validation.rs::spec_validate_plan_fixture_suite` -- full-plan validation passes for a complete plan.
- REQ-HIPP-VALID-002 -- `tests/spec/validation.rs::spec_undefined_reference_detection` -- undeclared addressee/variable/unit produce errors listing available definitions.
- REQ-HIPP-VALID-003 -- `tests/spec/validation.rs::spec_validation_error_suggestions` -- coverage gap error includes suggestion with exact missing range.

### 5.23 -- Section 5.2: Input Validation
- REQ-HIPP-INPUT-001 -- `tests/spec/execution.rs::spec_numeric_input_precision_rejection` -- numeric answers must respect the decimal precision implied by valid values.

### 5.24 -- Section 5.3: Meaning Evaluation
- REQ-HIPP-MEAN-001 -- `tests/spec/execution.rs::spec_meaning_of_evaluates` -- meaning evaluation returns the assessed meaning.
- REQ-HIPP-MEAN-002 -- `tests/spec/execution.rs::spec_meaning_of_missing_value` -- meaning evaluation returns Missing when the source value is unknown.
- REQ-HIPP-MEAN-003 -- `tests/spec/execution.rs::spec_meaning_of_nested_assessment` -- meaning evaluation supports nested assessments.

---

## 6. Test Plan Coverage Summary

| V-Model Level | Test Plan | Test ID Prefix | # Test Cases | Coverage |
|---|---|---|---|---|
| Stakeholder Req (STKR) | `test-plans/03-acceptance-test-plan.md` | AT-\* | 28 | 100% (all 28 STKR covered) |
| System Req (REQ) | `test-plans/02-system-test-plan.md` | ST-\* | 99 | 100% (97 unique REQ IDs, 99 test cases) |
| System Design (DES) | `test-plans/01-integration-test-plan.md` | IT-\* | 28 | 14/33 DES elements directly covered (~42%) |
| Detailed Design (DDR) | `test-plans/00-unit-test-plan.md` | UT-\* | 133 | 30/33 DDR elements covered (~91%) |

**Notes on DES coverage:** DES elements not covered by integration tests (SYS-HIPP-001, SYS-HIPP-002, SYS-HIPP-003, SYS-HIPP-019, SYS-HIPP-020..SYS-HIPP-026, SYS-HIPP-030, SYS-HIPP-033, SYS-HIPP-034, SYS-HIPP-040) are either architectural constraints verified by successful compilation, dependency declarations, or host-side concerns outside the scope of Rust integration tests. SYS-HIPP-043 (stop signal) is now covered by IT-HIPP-025.

**Notes on DDR coverage:** DET-HIPP-FFI-001..19 are partially covered by UT-HIPP-FFI-001..10 at the unit level; remaining FFI functions are tested at the integration level via Swift/C bindings. DET-HIPP-RT-002, DET-HIPP-RT-008, and DET-HIPP-FMT-001 gaps have been closed by UT-HIPP-RT-015..17 and UT-HIPP-FMT-001..02.

---

## 7. Full V-Model Matrix

The master matrix traces each stakeholder requirement through all V-Model levels.

| STKR | REQ (section) | DES | DDR | UT | IT | ST | AT | Status |
|------|--------------|-----|-----|----|----|----|----|----|
| SREQ-HIPP-001 | REQ-HIPP-ACT-001, REQ-HIPP-ACT-002, REQ-HIPP-ACT-003, REQ-HIPP-ACT-004, REQ-HIPP-ACT-005, REQ-HIPP-ACT-006, REQ-HIPP-ACT-007, REQ-HIPP-ACT-008, REQ-HIPP-ACT-009, REQ-HIPP-ACT-010, REQ-HIPP-COMM-001, REQ-HIPP-COMM-002, REQ-HIPP-MED-001, REQ-HIPP-MED-002, REQ-HIPP-MED-003, REQ-HIPP-EXEC-001, REQ-HIPP-EXEC-002, REQ-HIPP-EXEC-003, REQ-HIPP-EXEC-004, REQ-HIPP-EXEC-005 | SYS-HIPP-010, SYS-HIPP-013, SYS-HIPP-016, SYS-HIPP-018 | DET-HIPP-PARSER-001..04, DET-HIPP-RT-001, DET-HIPP-RT-003, DET-HIPP-RT-004 | UT-PARSER-\*, UT-HIPP-RT-008, UT-HIPP-RT-009, UT-ACTIONS-\*, UT-ACTORS-\* | IT-HIPP-002, IT-HIPP-007..IT-HIPP-012 | ST-3.7-\*, ST-3.9-\*, ST-3.10-\*, ST-5-\* | AT-HIPP-001 | Covered |
| SREQ-HIPP-002 | REQ-HIPP-LANG-001, REQ-HIPP-LANG-002, REQ-HIPP-LANG-003, REQ-HIPP-LANG-004, REQ-HIPP-BASIC-001, REQ-HIPP-BASIC-002, REQ-HIPP-BASIC-003, REQ-HIPP-BASIC-004, REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-002, REQ-HIPP-UNITS-003, REQ-HIPP-UNITS-004, REQ-HIPP-UNITS-005, REQ-HIPP-PROG-001, REQ-HIPP-EVT-001, REQ-HIPP-EVT-002, REQ-HIPP-EVT-003, REQ-HIPP-EVT-004, REQ-HIPP-EVT-005, REQ-HIPP-EVT-006, REQ-HIPP-EVT-007 | SYS-HIPP-010, SYS-HIPP-011, SYS-HIPP-019 | DET-HIPP-PARSER-001..05, DET-HIPP-FMT-001 | UT-PARSER-\*, UT-VALUES-\*, UT-HIPP-FIX-001, UT-HIPP-CTX-005..08, UT-HIPP-PERIODS-008..09 | IT-HIPP-003..IT-HIPP-006 | ST-2-\*, ST-3.1-\*, ST-3.2-\*, ST-HIPP-PROG-001, ST-HIPP-EVT-007 | AT-HIPP-002 | Covered |
| SREQ-HIPP-003 | _(architectural)_ | SYS-HIPP-001, SYS-HIPP-002, SYS-HIPP-003, SYS-HIPP-018, SYS-HIPP-022, SYS-HIPP-024, SYS-HIPP-030..SYS-HIPP-034 | DET-HIPP-FFI-001..19 | UT-HIPP-FFI-001..10 | IT-HIPP-023 | _(no ST)_ | AT-HIPP-003 | Covered |
| SREQ-HIPP-004 | REQ-HIPP-VALID-001, REQ-HIPP-VALID-002, REQ-HIPP-VALID-003 | SYS-HIPP-012 | DET-HIPP-VAL-001..06 | UT-HIPP-VAL-001..37 | IT-HIPP-001, IT-HIPP-002 | ST-HIPP-VALID-001 | AT-HIPP-004 | Covered |
| SREQ-HIPP-005 | REQ-HIPP-PLAN-001, REQ-HIPP-PLAN-002, REQ-HIPP-ACT-001, REQ-HIPP-ACT-002, REQ-HIPP-ACT-003, REQ-HIPP-ACT-004, REQ-HIPP-ACT-005, REQ-HIPP-ACT-006, REQ-HIPP-ACT-007, REQ-HIPP-ACT-008, REQ-HIPP-ACT-009, REQ-HIPP-ACT-010, REQ-HIPP-EVT-001, REQ-HIPP-EVT-002, REQ-HIPP-EVT-003, REQ-HIPP-EVT-004, REQ-HIPP-EVT-005, REQ-HIPP-EVT-006, REQ-HIPP-EVT-007, REQ-HIPP-EXEC-001, REQ-HIPP-EXEC-002, REQ-HIPP-EXEC-003, REQ-HIPP-EXEC-004, REQ-HIPP-EXEC-005 | SYS-HIPP-010, SYS-HIPP-013, SYS-HIPP-014, SYS-HIPP-017, SYS-HIPP-021, SYS-HIPP-031, SYS-HIPP-040..SYS-HIPP-043 | DET-HIPP-RT-001, DET-HIPP-RT-003..DET-HIPP-RT-006, DET-HIPP-RT-008, DET-HIPP-RT-009, DET-HIPP-PARSER-002, DET-HIPP-PARSER-005 | UT-PERIODS-\*, UT-HIPP-RT-008, UT-HIPP-RT-010, UT-HIPP-RT-014 | IT-HIPP-007..IT-HIPP-013, IT-HIPP-016..IT-HIPP-022 | ST-3.5-\*, ST-3.7-\*, ST-3.8-\*, ST-5-\* | AT-HIPP-005 | Covered |
| SREQ-HIPP-006 | _(architectural)_ | SYS-HIPP-041 | DET-HIPP-RT-008 | UT-HIPP-RT-017 | IT-HIPP-007..IT-HIPP-009, IT-HIPP-020, IT-HIPP-021 | ST-HIPP-EXEC-004 | AT-HIPP-006 | Covered |
| SREQ-HIPP-010 | REQ-HIPP-STMT-001, REQ-HIPP-STMT-002, REQ-HIPP-STMT-003, REQ-HIPP-STMT-004, REQ-HIPP-STMT-005, REQ-HIPP-COVER-001, REQ-HIPP-COVER-002, REQ-HIPP-COVER-003, REQ-HIPP-COVER-004, REQ-HIPP-COVER-005, REQ-HIPP-COVER-006, REQ-HIPP-COVER-007, REQ-HIPP-COVER-008, REQ-HIPP-COVER-009, REQ-HIPP-COVER-010, REQ-HIPP-COVER-011, REQ-HIPP-COVER-012, REQ-HIPP-COVER-013, REQ-HIPP-COVER-014 | SYS-HIPP-010, SYS-HIPP-012 | DET-HIPP-VAL-005, DET-HIPP-PARSER-001..02 | UT-HIPP-VAL-001..07, UT-HIPP-VAL-025, UT-HIPP-VAL-036, UT-HIPP-ACTIONS-001..02 | IT-HIPP-001, IT-HIPP-002 | ST-3.6-\*, ST-4.4-\* | AT-HIPP-010 | Covered |
| SREQ-HIPP-011 | REQ-HIPP-BASIC-001, REQ-HIPP-BASIC-002, REQ-HIPP-BASIC-003, REQ-HIPP-BASIC-004, REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-002, REQ-HIPP-UNITS-003, REQ-HIPP-UNITS-004, REQ-HIPP-UNITS-005, REQ-HIPP-PROG-001, REQ-HIPP-REQP-001, REQ-HIPP-REQP-002, REQ-HIPP-REQP-003, REQ-HIPP-REQP-004, REQ-HIPP-REQP-005, REQ-HIPP-REQP-006, REQ-HIPP-REQP-007 | SYS-HIPP-010, SYS-HIPP-012 | DET-HIPP-PARSER-001..02, DET-HIPP-VAL-002 | UT-PARSER-\*, UT-HIPP-VAL-021..27, UT-HIPP-VAL-035 | IT-HIPP-001, IT-HIPP-002 | ST-3.1-\*, ST-3.2-\*, ST-HIPP-PROG-001, ST-4.2-\* | AT-HIPP-011 | Covered |
| SREQ-HIPP-012 | REQ-HIPP-VALUE-001, REQ-HIPP-VALUE-002, REQ-HIPP-VALUE-003, REQ-HIPP-VALUE-004, REQ-HIPP-VALUE-005, REQ-HIPP-VALUE-006, REQ-HIPP-VALUE-007, REQ-HIPP-VALUE-008, REQ-HIPP-VALUE-009, REQ-HIPP-VALUE-010, REQ-HIPP-VALUE-011, REQ-HIPP-VALUE-012, REQ-HIPP-VALUE-013, REQ-HIPP-MEAN-001, REQ-HIPP-MEAN-002, REQ-HIPP-MEAN-003 | SYS-HIPP-015, SYS-HIPP-016 | DET-HIPP-RT-004, DET-HIPP-PARSER-002 | UT-VALUES-\*, UT-HIPP-RT-004..06 | IT-HIPP-010, IT-HIPP-014, IT-HIPP-015 | ST-3.4-\*, ST-5.3-\* | AT-HIPP-012 | Covered |
| SREQ-HIPP-013 | REQ-HIPP-LANG-001, REQ-HIPP-LANG-002, REQ-HIPP-LANG-003, REQ-HIPP-LANG-004, REQ-HIPP-STMT-001, REQ-HIPP-STMT-002, REQ-HIPP-STMT-003, REQ-HIPP-STMT-004, REQ-HIPP-STMT-005, REQ-HIPP-CTX-001, REQ-HIPP-CTX-002, REQ-HIPP-CTX-003 | SYS-HIPP-010, SYS-HIPP-015, SYS-HIPP-016 | DET-HIPP-PARSER-001..02, DET-HIPP-RT-004 | UT-CTX-\*, UT-HIPP-ACTIONS-001 | IT-HIPP-011 | ST-2-\*, ST-3.6-\*, ST-3.11-\* | AT-HIPP-013 | Covered |
| SREQ-HIPP-014 | REQ-HIPP-LANG-001, REQ-HIPP-LANG-002, REQ-HIPP-LANG-003, REQ-HIPP-LANG-004, REQ-HIPP-COMM-001, REQ-HIPP-COMM-002 | SYS-HIPP-001, SYS-HIPP-003, SYS-HIPP-010 | DET-HIPP-PARSER-001 | UT-PARSER-\* | _(architectural)_ | ST-2-\*, ST-3.9-\* | AT-HIPP-014 | Covered |
| SREQ-HIPP-015 | REQ-HIPP-VALUE-001, REQ-HIPP-VALUE-002, REQ-HIPP-VALUE-003, REQ-HIPP-VALUE-004, REQ-HIPP-VALUE-005, REQ-HIPP-VALUE-006, REQ-HIPP-VALUE-007, REQ-HIPP-VALUE-008, REQ-HIPP-VALUE-009, REQ-HIPP-VALUE-010, REQ-HIPP-VALUE-011, REQ-HIPP-VALUE-012, REQ-HIPP-VALUE-013, REQ-HIPP-REQP-001, REQ-HIPP-REQP-002, REQ-HIPP-REQP-003, REQ-HIPP-REQP-004, REQ-HIPP-REQP-005, REQ-HIPP-REQP-006, REQ-HIPP-REQP-007, REQ-HIPP-RANGE-001, REQ-HIPP-RANGE-002 | SYS-HIPP-012, SYS-HIPP-015, SYS-HIPP-042 | DET-HIPP-VAL-002..03, DET-HIPP-RT-007 | UT-HIPP-VAL-021..23, UT-HIPP-VAL-028, UT-HIPP-VAL-037, UT-HIPP-RT-007 | IT-HIPP-002 | ST-3.4-\*, ST-4.2-\*, ST-4.5-\* | AT-HIPP-015 | Covered |
| SREQ-HIPP-016 | _(architectural)_ | SYS-HIPP-015, SYS-HIPP-026 | DET-HIPP-RT-002 | UT-HIPP-RT-015 | IT-HIPP-010, IT-HIPP-014, IT-HIPP-015 | _(no ST)_ | AT-HIPP-016 | Covered |
| SREQ-HIPP-017 | _(future)_ | _(none)_ | _(none)_ | _(none)_ | _(none)_ | _(none)_ | AT-HIPP-017 | Gap |
| SREQ-HIPP-018 | REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-002, REQ-HIPP-UNITS-003, REQ-HIPP-UNITS-004, REQ-HIPP-UNITS-005, REQ-HIPP-MED-001, REQ-HIPP-MED-002, REQ-HIPP-MED-003, REQ-HIPP-CORE-001, REQ-HIPP-CORE-002, REQ-HIPP-CORE-003, REQ-HIPP-CORE-005, REQ-HIPP-DTIME-001, REQ-HIPP-DTIME-002 | SYS-HIPP-015, SYS-HIPP-021 | DET-HIPP-DOM-002, DET-HIPP-PARSER-001..02, DET-HIPP-VAL-002 | UT-UNITS-\*, UT-HIPP-ACTORS-001..05, UT-HIPP-RT-002..03 | IT-HIPP-007..IT-HIPP-010 | ST-3.2-\*, ST-3.10-\*, ST-4.1-\*, ST-4.7-\* | AT-HIPP-018 | Covered |
| SREQ-HIPP-019 | _(architectural)_ | SYS-HIPP-015, SYS-HIPP-026 | DET-HIPP-RT-002 | UT-HIPP-RT-016 | IT-HIPP-010, IT-HIPP-014 | _(no ST)_ | AT-HIPP-019 | Covered |
| SREQ-HIPP-020 | REQ-HIPP-PLAN-001, REQ-HIPP-PLAN-002 | SYS-HIPP-013, SYS-HIPP-014 | DET-HIPP-PARSER-002, DET-HIPP-RT-001 | UT-HIPP-PERIODS-001..02, UT-HIPP-RT-008 | IT-HIPP-002, IT-HIPP-007 | ST-3.5-\* | AT-HIPP-020 | Covered |
| SREQ-HIPP-030 | REQ-HIPP-LANG-001, REQ-HIPP-LANG-002, REQ-HIPP-LANG-003, REQ-HIPP-LANG-004 | SYS-HIPP-010, SYS-HIPP-012 | DET-HIPP-PARSER-001 | UT-HIPP-PARSER-005 | IT-HIPP-001, IT-HIPP-002 | ST-HIPP-LANG-003 | AT-HIPP-030 | Covered |
| SREQ-HIPP-031 | REQ-HIPP-UNITS-001, REQ-HIPP-UNITS-002, REQ-HIPP-UNITS-003, REQ-HIPP-UNITS-004, REQ-HIPP-UNITS-005, REQ-HIPP-CORE-001, REQ-HIPP-CORE-002, REQ-HIPP-CORE-003, REQ-HIPP-CORE-005 | SYS-HIPP-012, SYS-HIPP-015 | DET-HIPP-VAL-002, DET-HIPP-DOM-002 | UT-HIPP-PARSER-006, UT-UNITS-\*, UT-HIPP-VAL-021..27 | IT-HIPP-012, IT-HIPP-013, IT-HIPP-016, IT-HIPP-017 | ST-HIPP-UNITS-005, ST-4.1-\* | AT-HIPP-031 | Covered |
| SREQ-HIPP-032 | REQ-HIPP-COVER-001, REQ-HIPP-COVER-002, REQ-HIPP-COVER-003, REQ-HIPP-COVER-004, REQ-HIPP-COVER-005, REQ-HIPP-COVER-006, REQ-HIPP-COVER-007, REQ-HIPP-COVER-008, REQ-HIPP-COVER-009, REQ-HIPP-COVER-010, REQ-HIPP-COVER-011, REQ-HIPP-COVER-012, REQ-HIPP-COVER-013, REQ-HIPP-COVER-014 | SYS-HIPP-012 | DET-HIPP-VAL-003, DET-HIPP-VAL-005 | UT-HIPP-VAL-001..07, UT-HIPP-VAL-014..20, UT-HIPP-VAL-025, UT-HIPP-VAL-036 | IT-HIPP-001, IT-HIPP-002 | ST-4.4-\* | AT-HIPP-032 | Covered |
| SREQ-HIPP-033 | REQ-HIPP-FLOW-001, REQ-HIPP-FLOW-002, REQ-HIPP-FLOW-003, REQ-HIPP-FLOW-004, REQ-HIPP-FLOW-005 | SYS-HIPP-012 | DET-HIPP-VAL-004 | UT-HIPP-VAL-029..34 | IT-HIPP-001, IT-HIPP-002 | ST-4.3-\* | AT-HIPP-033 | Covered |
| SREQ-HIPP-034 | REQ-HIPP-INPUT-001 | SYS-HIPP-015, SYS-HIPP-042 | DET-HIPP-RT-007 | UT-HIPP-RT-007 | IT-HIPP-016, IT-HIPP-017 | ST-HIPP-INPUT-001 | AT-HIPP-034 | Covered |
| SREQ-HIPP-035 | REQ-HIPP-EXPR-001, REQ-HIPP-EXPR-002, REQ-HIPP-EXPR-003, REQ-HIPP-EXPR-004, REQ-HIPP-EXPR-005, REQ-HIPP-EXPR-006, REQ-HIPP-EXPR-007, REQ-HIPP-SUFF-001, REQ-HIPP-SUFF-002, REQ-HIPP-SUFF-003, REQ-HIPP-SUFF-004 | SYS-HIPP-012, SYS-HIPP-016 | DET-HIPP-VAL-002, DET-HIPP-RT-004 | UT-HIPP-VAL-009..12, UT-HIPP-RT-001, UT-HIPP-CTX-009 | IT-HIPP-011 | ST-HIPP-EXPR-005, ST-4.6-\* | AT-HIPP-035 | Covered |
| SREQ-HIPP-036 | REQ-HIPP-ACT-001, REQ-HIPP-ACT-002, REQ-HIPP-ACT-003, REQ-HIPP-ACT-004, REQ-HIPP-ACT-005, REQ-HIPP-ACT-006, REQ-HIPP-ACT-007, REQ-HIPP-ACT-008, REQ-HIPP-ACT-009, REQ-HIPP-ACT-010 | SYS-HIPP-013 | DET-HIPP-RT-010 | UT-HIPP-PLAN-001 | IT-HIPP-028 | ST-HIPP-ACT-010 | AT-HIPP-036 | Covered |
| SREQ-HIPP-037 | REQ-HIPP-CORE-001, REQ-HIPP-CORE-002, REQ-HIPP-CORE-003, REQ-HIPP-CORE-005, REQ-HIPP-VALID-001, REQ-HIPP-VALID-002, REQ-HIPP-VALID-003 | SYS-HIPP-010, SYS-HIPP-012 | DET-HIPP-PARSER-006, DET-HIPP-VAL-007, DET-HIPP-VAL-008 | UT-HIPP-PARSER-013, UT-HIPP-VAL-038, UT-HIPP-VAL-039 | _(none)_ | ST-HIPP-CORE-005, ST-HIPP-VALID-002, ST-HIPP-VALID-003 | AT-HIPP-037 | Covered |
| SREQ-HIPP-040 | _(meta)_ | _(this document)_ | _(this document)_ | _(N/A)_ | _(N/A)_ | _(N/A)_ | AT-HIPP-040 | Covered |
| SREQ-HIPP-041 | _(meta)_ | _(build system)_ | _(build system)_ | _(all tests)_ | _(all tests)_ | _(all tests)_ | AT-HIPP-041 | Covered |
| SREQ-HIPP-042 | _(meta)_ | _(doc set)_ | _(doc set)_ | _(N/A)_ | _(N/A)_ | _(N/A)_ | AT-HIPP-042 | Covered |

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
| DET-HIPP-FFI-001..19 | FFI C-API functions | Low | **Partially closed.** UT-HIPP-FFI-001..10 cover DET-HIPP-FFI-001, -02, -03, -07, -08, -09, -10, -11. Remaining FFI functions (DET-HIPP-FFI-004..06, -12..19) are tested at integration level via Swift/C host bindings. |
| DET-HIPP-RT-002 | Environment struct | Low | **Closed.** UT-HIPP-RT-015 (append-only history) and UT-HIPP-RT-016 (value history retrieval) provide dedicated coverage. |
| DET-HIPP-RT-008 | Execution modes (simulation vs. real-time) | Medium | **Closed.** UT-HIPP-RT-017 verifies simulation mode completes without real-time delays. |
| DET-HIPP-FMT-001 | Formatter `format_script` | Medium | **Closed.** UT-HIPP-FMT-001 (round-trip parse-format-parse) and UT-HIPP-FMT-002 (all definition types) provide coverage. |

### 8.2 DES Elements Without Integration Tests

The following DES elements are not directly covered by integration tests (per `01-integration-test-plan.md`):

| DES Element | Description | Reason |
|-------------|-------------|--------|
| SYS-HIPP-001, SYS-HIPP-002, SYS-HIPP-003 | Language selection, dual crate, C-FFI boundary | Architectural constraints verified by successful compilation and linking. |
| SYS-HIPP-019 | Formatter | **Closed at unit level.** UT-HIPP-FMT-001 provides a round-trip parse-format-parse test. No integration test needed. |
| SYS-HIPP-020..SYS-HIPP-026 | Dependencies (Pest, Chrono, Serde, etc.) | Dependency declarations; verified by compilation. |
| SYS-HIPP-030, SYS-HIPP-033, SYS-HIPP-034 | FFI lifecycle, memory management, iOS integration | Host-side concerns; verified by the SwiftUI editor integration. |
| SYS-HIPP-040 | Real-time execution mode | Not testable in automated CI without wall-clock delays. |
| SYS-HIPP-043 | Stop signal | **Closed.** IT-HIPP-025 verifies stop signal terminates a long-running simulation early. |

### 8.3 STKR Without Full Automated Evidence

| STKR | Issue | Status |
|------|-------|--------|
| SREQ-HIPP-003 | FFI/embedding tested only via Swift host; no Rust-level unit test. | **Closed.** UT-HIPP-FFI-001..10 provide Rust-side FFI unit tests covering parse, validate, engine lifecycle, load, periods, time, simulation, and stop. |
| SREQ-HIPP-006 | Simulation mode has no system-level REQ or ST; tested only at integration level. | **Closed.** ST-HIPP-EXEC-004 (system test) and UT-HIPP-RT-017 (unit test) now provide dedicated simulation mode coverage. |
| SREQ-HIPP-016 | Immutable history is architectural; no dedicated ST or UT. | **Closed.** UT-HIPP-RT-015 verifies append-only value history semantics. |
| SREQ-HIPP-017 | Localization support is not implemented. | **Accepted gap.** Future scope (Priority: May). This is the only remaining accepted gap. |
| SREQ-HIPP-019 | Automatic persistence is architectural; no dedicated ST or UT. | **Closed.** UT-HIPP-RT-016 verifies value history retrieval with timestamps after recording. |

---

## Revision History

| Version | Date | Changes |
|---|---|---|
| 1.0 | 2026-03-20 | Initial traceability matrix. Full V-Model cross-reference of all specifications, design documents, and test plans. |
| 1.1 | 2026-03-20 | Closed gaps: SREQ-HIPP-003, -06, -16, -19 changed from Partial to Covered. Added REQ-HIPP-EXEC-004 and ST-HIPP-EXEC-004. Updated test counts. SREQ-HIPP-017 remains only accepted gap. |
| 1.2 | 2026-03-23 | Added REQ-HIPP-EVT-005, REQ-HIPP-EVT-006, REQ-HIPP-EXEC-005, DET-HIPP-RT-009. Updated SREQ-HIPP-005 row. |
| 1.3 | 2026-03-23 | Added SREQ-HIPP-036 chain: REQ-HIPP-ACT-010, DET-HIPP-RT-010, UT-HIPP-PLAN-001, IT-HIPP-028, ST-HIPP-ACT-010, AT-HIPP-036. |
| 1.4 | 2026-03-23 | Added REQ-HIPP-EVT-007 (bare unit and ordinal triggers). Added DET-HIPP-PARSER-005. Updated SREQ-HIPP-002 and SREQ-HIPP-005 rows. Added UT-HIPP-PERIODS-008..09, ST-HIPP-EVT-007. |
| 1.5 | 2026-03-23 | Added SREQ-HIPP-037 chain: REQ-HIPP-CORE-005, REQ-HIPP-VALID-002, REQ-HIPP-VALID-003, DET-HIPP-PARSER-006, DET-HIPP-VAL-007, DET-HIPP-VAL-008, UT-HIPP-PARSER-013, UT-HIPP-VAL-038, UT-HIPP-VAL-039, ST-HIPP-CORE-005, ST-HIPP-VALID-002, ST-HIPP-VALID-003, AT-HIPP-037. Updated test counts. |
| 1.6 | 2026-04-19 | Replaced legacy `§N.M` section references in §2 and §7 matrix cells with explicit REQ-HIPP-\* ID lists so every cell resolves to a clickable link in `spec.html`. |

