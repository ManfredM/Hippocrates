# Acceptance Test Plan

**Document ID:** AT
**V-Model Level:** Acceptance Testing (Right side, top)
**Validates:** Stakeholder Requirements (STKR-\*)
**Version:** 1.0
**Status:** Draft

---

## 1. Purpose

This plan validates that the Hippocrates engine meets stakeholder needs as defined in the Stakeholder Requirements document (`00-stakeholder-requirements.md`). Tests are written in user and business language so that medical professionals, integrators, and regulatory reviewers can understand them without software development expertise.

---

## 2. Test Types

| Type | Description |
|---|---|
| **Automated** | Verified by running the existing test suite (`cargo test`). Results are deterministic and reproducible. |
| **Manual** | Requires human review or domain expert evaluation. A qualified reviewer inspects artifacts and records a judgment. |
| **Documentation** | Requires auditing documentation completeness and traceability. A reviewer checks that required documents exist and contain appropriate content. |

---

## 3. Entry / Exit Criteria

### Entry Criteria

- All system tests (ST-\*) pass.
- Documentation set is complete and current.
- The engine builds without errors on all target platforms.

### Exit Criteria

- All AT-\* test cases pass or are explicitly accepted by stakeholders with documented rationale.
- Any deviations are recorded and risk-assessed.

---

## 4. Test Cases

### 4.1 Purpose and Scope (SREQ-HIPP-001 through SREQ-HIPP-006)

#### AT-HIPP-001 — Medical Care Plan Execution

**Traces to:** SREQ-HIPP-001

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-001 |
| **Description** | Confirm the engine can load and execute a complete medical care plan from a `.hipp` script, producing the expected events, questions, and messages. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a syntactically valid `.hipp` script (`treating_copd.hipp`), when the engine loads and executes the script, then the plan runs according to its defined logic, producing events, questions, and messages. |
| **Status** | Not Started |
| **Evidence** | Run `treating_copd.hipp` end-to-end via simulation; cross-reference ST-\* execution tests. |

#### AT-HIPP-002 — Readability by Medical Professionals

**Traces to:** SREQ-HIPP-002

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-002 |
| **Description** | Confirm that example care plan scripts are readable and reviewable by medical professionals without programming training. |
| **Verification Method** | Manual |
| **Pass Criteria** | Given a care plan script, when presented to a medical professional unfamiliar with programming, then the professional can identify the script's clinical intent and decision logic. |
| **Status** | Not Started |
| **Evidence** | Expert review session of example scripts (e.g., `treating_copd.hipp`); documented reviewer feedback. Natural language trigger syntax (`every day`, `every other day`, `every third week`) enhances readability per REQ-HIPP-EVT-007; verified by ST-HIPP-EVT-007, UT-HIPP-PERIODS-008, UT-HIPP-PERIODS-009. |

#### AT-HIPP-003 — Embeddable Runtime

**Traces to:** SREQ-HIPP-003

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-003 |
| **Description** | Confirm the engine can be embedded in a host application via its C-compatible FFI, operating correctly within the host process. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a host application linked against the engine's static library, when the host calls the public C API to load and execute a plan, then the engine operates correctly within the host process. |
| **Status** | Evidence Available |
| **Evidence** | UT-HIPP-FFI-001..10 (Rust-side FFI unit tests covering parse, validate, engine lifecycle, load, periods, time, simulation, stop); IT-HIPP-023 (FFI JSON parsing logic); successful integration in the SwiftUI editor (`hippocrates_editor`). |

#### AT-HIPP-004 — Validation Without Execution

**Traces to:** SREQ-HIPP-004

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-004 |
| **Description** | Confirm the engine can validate scripts for correctness without executing them, reporting all errors with source locations. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a `.hipp` script with errors, when the engine validates the script, then all errors are reported with line and column numbers, without executing any plan logic. |
| **Status** | Not Started |
| **Evidence** | Run `hippocrates_validate_file` on scripts with known errors; cross-reference ST-\* validation tests. |

#### AT-HIPP-005 — Event-Driven Execution

**Traces to:** SREQ-HIPP-005

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-005 |
| **Description** | Confirm the engine reacts to scheduled times, value changes, and external triggers according to the plan's defined logic. |
| **Verification Method** | Automated |
| **Pass Criteria** | (1) Given a plan with periodic triggers, change-of-value triggers, and start-of-period triggers, when the engine executes the plan, then each trigger fires at the correct time and invokes the corresponding actions. (2) Given a periodic trigger with `at 08:00`, events fire at 08:00 each day. (3) Given `every <period> for <duration>`, events fire at every period occurrence within the window. |
| **Status** | Evidence Available |
| **Evidence** | ST-HIPP-EVT-001..06, ST-HIPP-EXEC-005; IT-HIPP-026 (time-pinned), IT-HIPP-027 (period repetition); UT-HIPP-PERIODS-006, UT-HIPP-PERIODS-007. |

#### AT-HIPP-006 — Simulation Mode

**Traces to:** SREQ-HIPP-006

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-006 |
| **Description** | Confirm the engine supports accelerated simulation execution for testing and visualization without real-time delays. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a plan with time-based triggers, when simulation mode is enabled, then the plan executes to completion at maximum speed, producing the same logical results as real-time execution. |
| **Status** | Evidence Available |
| **Evidence** | ST-HIPP-EXEC-004 (simulation mode system test); UT-HIPP-RT-017 (simulation mode unit test); IT-HIPP-007..IT-HIPP-009, IT-HIPP-020, IT-HIPP-021 (simulation integration tests). |

---

### 4.2 Design Philosophy (SREQ-HIPP-010 through SREQ-HIPP-020)

#### AT-HIPP-010 — Completeness

**Traces to:** SREQ-HIPP-010

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-010 |
| **Description** | Confirm the engine rejects scripts with decision branches that are not exhaustively handled, such as gaps in value ranges or unhandled assessment cases. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a script with an assessment that does not cover the full range of valid values, when the engine validates the script, then validation fails with an error identifying the coverage gap. |
| **Status** | Not Started |
| **Evidence** | REQ-4.4-\* test suite (coverage validation tests). |

#### AT-HIPP-011 — Readability Over Defaults

**Traces to:** SREQ-HIPP-011

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-011 |
| **Description** | Confirm the language uses no implicit default values; every parameter must be explicitly stated. |
| **Verification Method** | Automated + Manual |
| **Pass Criteria** | Given any language construct, when a required parameter is omitted, then the engine rejects the script rather than applying a hidden default. |
| **Status** | Not Started |
| **Evidence** | REQ-4.2-\* tests; manual review of language grammar for hidden defaults. |

#### AT-HIPP-012 — Meaningful Values

**Traces to:** SREQ-HIPP-012

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-012 |
| **Description** | Confirm values carry semantic meaning beyond raw numbers, supporting meaning models that map numeric ranges to clinical labels. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a value definition with a meaning model, when a numeric value is assessed, then the engine resolves the value to its clinical meaning label. |
| **Status** | Not Started |
| **Evidence** | REQ-5.3-\* tests (meaning evaluation). |

#### AT-HIPP-013 — Contextual Statements

**Traces to:** SREQ-HIPP-013

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-013 |
| **Description** | Confirm the syntax creates natural reading flow where each statement is understandable in the context of preceding statements. |
| **Verification Method** | Manual |
| **Pass Criteria** | Given a block of statements, when read sequentially, then each statement is understandable in the context established by the preceding statements. |
| **Status** | Not Started |
| **Evidence** | Manual review of example scripts by domain experts. |

#### AT-HIPP-014 — Separation of Concerns

**Traces to:** SREQ-HIPP-014

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-014 |
| **Description** | Confirm that `.hipp` scripts contain no platform-specific constructs; all platform integration happens through the embedding API. |
| **Verification Method** | Manual |
| **Pass Criteria** | Given a `.hipp` script, when inspected for platform-specific constructs, then none are found; the script is purely domain logic. |
| **Status** | Not Started |
| **Evidence** | Manual audit of `.hipp` grammar definition and example scripts for platform-specific constructs. |

#### AT-HIPP-015 — Value Constraints

**Traces to:** SREQ-HIPP-015

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-015 |
| **Description** | Confirm every value has a definition of valid values and the engine rejects values outside defined constraints. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a value with defined valid values, when an out-of-range value is assigned at runtime, then the engine rejects the assignment. |
| **Status** | Not Started |
| **Evidence** | REQ-HIPP-REQP-006, REQ-4.5-\* tests. |

#### AT-HIPP-016 — Immutable History

**Traces to:** SREQ-HIPP-016

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-016 |
| **Description** | Confirm that values recorded in the past cannot be changed and the engine maintains a complete, append-only history with timestamps. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a value recorded at time T1, when the same value is updated at time T2, then both T1 and T2 entries exist in the history; the T1 entry is not modified. |
| **Status** | Evidence Available |
| **Evidence** | UT-HIPP-RT-015 (append-only value history verification); IT-HIPP-010, IT-HIPP-014, IT-HIPP-015 (integration tests exercising value history). |

#### AT-HIPP-017 — Localization Support

**Traces to:** SREQ-HIPP-017

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-017 |
| **Description** | Confirm the language supports translation metadata for automatic translation of scripts into local languages. |
| **Verification Method** | Manual |
| **Pass Criteria** | Given a script with translatable text, when translation metadata is provided, then the engine can resolve text in the target language. |
| **Status** | Not Started |
| **Evidence** | Manual review confirming the language supports translation metadata. |

#### AT-HIPP-018 — Medical Domain Awareness

**Traces to:** SREQ-HIPP-018

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-018 |
| **Description** | Confirm the engine recognizes standard medical units, measurement types, and terminology without requiring custom definitions. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a script using standard medical units, when the engine parses and validates the script, then units are recognized without requiring custom definitions. |
| **Status** | Not Started |
| **Evidence** | REQ-HIPP-UNITS-002, REQ-4.1-\* tests (built-in units). |

#### AT-HIPP-019 — Automatic Persistence

**Traces to:** SREQ-HIPP-019

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-019 |
| **Description** | Confirm all value changes are automatically stored without explicit save logic, and value history is retrievable after recording. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a running plan that records values, when the engine is queried for value history after recording, then all recorded values are available with their timestamps. |
| **Status** | Evidence Available |
| **Evidence** | UT-HIPP-RT-016 (value history retrieval with timestamps); IT-HIPP-010, IT-HIPP-014 (integration tests exercising value retrieval). |

#### AT-HIPP-020 — Plan Autonomy

**Traces to:** SREQ-HIPP-020

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-020 |
| **Description** | Confirm a plan is self-contained and can be loaded into a fresh engine instance without additional definition files. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a valid `.hipp` script, when loaded into a fresh engine instance, then the engine can execute the plan without requiring additional definition files. |
| **Status** | Not Started |
| **Evidence** | Loading standalone `.hipp` files in fresh engine instances; cross-reference IT-\* integration tests. |

---

### 4.3 Safety (SREQ-HIPP-030 through SREQ-HIPP-035)

#### AT-HIPP-030 — No Comparison Operators

**Traces to:** SREQ-HIPP-030

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-030 |
| **Description** | Confirm the language does not support comparison operators and that scripts containing them are rejected. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a script containing a comparison operator, when the engine parses the script, then parsing fails with an error. |
| **Status** | Not Started |
| **Evidence** | REQ-HIPP-LANG-003 test. |

#### AT-HIPP-031 — Unit Discipline

**Traces to:** SREQ-HIPP-031

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-031 |
| **Description** | Confirm all numeric values must carry explicit units and the engine rejects unitless numeric literals. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a numeric literal without a unit, when the engine parses or validates the script, then the script is rejected with a unit requirement error. |
| **Status** | Not Started |
| **Evidence** | REQ-HIPP-UNITS-005, REQ-HIPP-REQP-001, REQ-HIPP-REQP-002, REQ-HIPP-REQP-003 tests. |

#### AT-HIPP-032 — Exhaustive Assessment Coverage

**Traces to:** SREQ-HIPP-032

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-032 |
| **Description** | Confirm assessment and meaning definitions cover the full range of valid values, and the engine rejects definitions with gaps, overlaps, or missing cases. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given an assessment with a gap between 5 and 8 in a range of 0 to 10, when the engine validates the script, then validation fails identifying the missing span. |
| **Status** | Not Started |
| **Evidence** | REQ-4.4-\* test suite (14 tests). |

#### AT-HIPP-033 — Data Flow Validation

**Traces to:** SREQ-HIPP-033

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-033 |
| **Description** | Confirm the engine detects use of values before assignment and circular dependencies during static validation. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a script that uses a value before it has been assigned or asked, when the engine validates the script, then validation fails with a data flow error. |
| **Status** | Not Started |
| **Evidence** | REQ-4.3-\* tests. |

#### AT-HIPP-034 — Input Precision Validation

**Traces to:** SREQ-HIPP-034

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-034 |
| **Description** | Confirm the engine validates that runtime input values respect the decimal precision implied by the value's definition. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a value defined with integer precision (e.g., 0 ... 10), when a float answer (e.g., 3.5) is provided at runtime, then the engine rejects the answer. |
| **Status** | Not Started |
| **Evidence** | REQ-HIPP-INPUT-001 test. |

#### AT-HIPP-035 — Data Sufficiency Handling

**Traces to:** SREQ-HIPP-035

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-035 |
| **Description** | Confirm the engine requires explicit handling of insufficient data scenarios in statistical calculations. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a statistical calculation over a timeframe with no data, when the script does not handle the "Not enough data" case, then validation fails requiring explicit handling. |
| **Status** | Not Started |
| **Evidence** | REQ-4.6-\* tests. |

#### AT-HIPP-036 — Plan Completion Actions

**Traces to:** SREQ-HIPP-036

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-036 |
| **Description** | Confirm that plans support defining actions that execute when the plan reaches its natural end, so that patients and care providers receive completion notifications. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a plan with an `after plan:` block, when the plan's event loop finishes (all triggers exhausted), then the statements in the `after plan:` block execute exactly once. |
| **Status** | Not Started |
| **Evidence** | ST-HIPP-ACT-010 (system test); UT-HIPP-PLAN-001 (unit test); IT-HIPP-028 (integration test). |

#### AT-HIPP-037 — LLM-Correctable Error Diagnostics

**Traces to:** SREQ-HIPP-037

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-037 |
| **Description** | Confirm that error messages produced by the engine are detailed and specific enough for an LLM to automatically correct Hippocrates scripts without human intervention. |
| **Verification Method** | Automated |
| **Pass Criteria** | (1) Given a script with a syntax error, when validated, then the error message describes what's wrong in plain English with a suggested fix. (2) Given a script referencing an undeclared variable, addressee, or unit, when validated, then the error identifies the undefined reference and lists available definitions. (3) Given a script with a coverage gap, when validated, then the error includes the exact range to add. |
| **Status** | Not Started |
| **Evidence** | ST-HIPP-CORE-005 (human-readable parse errors); ST-HIPP-VALID-002 (undefined reference detection); ST-HIPP-VALID-003 (validation error suggestions); UT-HIPP-PARSER-013, UT-HIPP-VAL-038, UT-HIPP-VAL-039 (unit tests). |

---

### 4.4 Regulatory (SREQ-HIPP-040 through SREQ-HIPP-042)

#### AT-HIPP-040 — Requirements Traceability

**Traces to:** SREQ-HIPP-040

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-040 |
| **Description** | Confirm every system requirement traces to a stakeholder requirement, and every requirement has at least one associated test. |
| **Verification Method** | Documentation |
| **Pass Criteria** | Given the traceability matrix, when audited, then every system requirement maps to at least one stakeholder requirement, and every requirement has at least one associated test. |
| **Status** | Not Started |
| **Evidence** | Documentation audit of `04-traceability.md`. |

#### AT-HIPP-041 — Reproducible Verification

**Traces to:** SREQ-HIPP-041

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-041 |
| **Description** | Confirm all verification evidence is reproducible, producing deterministic results on independent runs. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given the engine test suite, when run on a clean checkout, then all tests produce the same pass/fail results. |
| **Status** | Not Started |
| **Evidence** | Run `cargo test` on a clean checkout twice and compare results. |

#### AT-HIPP-042 — Class II Documentation Readiness

**Traces to:** SREQ-HIPP-042

| Field | Value |
|---|---|
| **STKR Reference** | SREQ-HIPP-042 |
| **Description** | Confirm the documentation set is structured to support Class II medical device regulatory submissions per IEC 62304 and ISO 14971. |
| **Verification Method** | Documentation |
| **Pass Criteria** | Given the V-Model documentation set, when reviewed against IEC 62304 required artifacts, then all required document types are present with appropriate content. |
| **Status** | Not Started |
| **Evidence** | Documentation audit against IEC 62304 checklist. |

---

## 5. Coverage Summary

| STKR ID | AT ID | Verification Method | Status |
|---|---|---|---|
| SREQ-HIPP-001 | AT-HIPP-001 | Automated | Not Started |
| SREQ-HIPP-002 | AT-HIPP-002 | Manual | Not Started |
| SREQ-HIPP-003 | AT-HIPP-003 | Automated | Evidence Available |
| SREQ-HIPP-004 | AT-HIPP-004 | Automated | Not Started |
| SREQ-HIPP-005 | AT-HIPP-005 | Automated | Not Started |
| SREQ-HIPP-006 | AT-HIPP-006 | Automated | Evidence Available |
| SREQ-HIPP-010 | AT-HIPP-010 | Automated | Not Started |
| SREQ-HIPP-011 | AT-HIPP-011 | Automated + Manual | Not Started |
| SREQ-HIPP-012 | AT-HIPP-012 | Automated | Not Started |
| SREQ-HIPP-013 | AT-HIPP-013 | Manual | Not Started |
| SREQ-HIPP-014 | AT-HIPP-014 | Manual | Not Started |
| SREQ-HIPP-015 | AT-HIPP-015 | Automated | Not Started |
| SREQ-HIPP-016 | AT-HIPP-016 | Automated | Evidence Available |
| SREQ-HIPP-017 | AT-HIPP-017 | Manual | Not Started |
| SREQ-HIPP-018 | AT-HIPP-018 | Automated | Not Started |
| SREQ-HIPP-019 | AT-HIPP-019 | Automated | Evidence Available |
| SREQ-HIPP-020 | AT-HIPP-020 | Automated | Not Started |
| SREQ-HIPP-030 | AT-HIPP-030 | Automated | Not Started |
| SREQ-HIPP-031 | AT-HIPP-031 | Automated | Not Started |
| SREQ-HIPP-032 | AT-HIPP-032 | Automated | Not Started |
| SREQ-HIPP-033 | AT-HIPP-033 | Automated | Not Started |
| SREQ-HIPP-034 | AT-HIPP-034 | Automated | Not Started |
| SREQ-HIPP-035 | AT-HIPP-035 | Automated | Not Started |
| SREQ-HIPP-036 | AT-HIPP-036 | Automated | Not Started |
| SREQ-HIPP-037 | AT-HIPP-037 | Automated | Not Started |
| SREQ-HIPP-040 | AT-HIPP-040 | Documentation | Not Started |
| SREQ-HIPP-041 | AT-HIPP-041 | Automated | Not Started |
| SREQ-HIPP-042 | AT-HIPP-042 | Documentation | Not Started |

All 28 stakeholder requirements are covered by acceptance test cases.

---

## Revision History

| Version | Date | Changes |
|---|---|---|
| 1.0 | 2026-03-20 | Initial acceptance test plan |
| 1.1 | 2026-03-20 | Updated AT-HIPP-003, AT-HIPP-006, AT-HIPP-016, AT-HIPP-019 with automated evidence references. |
| 1.2 | 2026-03-23 | Updated AT-HIPP-005 with time-of-day and period repetition acceptance criteria and evidence. |
| 1.3 | 2026-03-23 | Added AT-HIPP-036 (Plan Completion Actions) for SREQ-HIPP-036. |
| 1.4 | 2026-03-23 | Updated AT-HIPP-002 evidence with natural language trigger syntax references (REQ-HIPP-EVT-007). |
| 1.5 | 2026-03-23 | Added AT-HIPP-037 (LLM-Correctable Error Diagnostics) for SREQ-HIPP-037. |
