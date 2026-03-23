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

### 4.1 Purpose and Scope (STKR-01 through STKR-06)

#### AT-01 — Medical Care Plan Execution

| Field | Value |
|---|---|
| **STKR Reference** | STKR-01 |
| **Description** | Confirm the engine can load and execute a complete medical care plan from a `.hipp` script, producing the expected events, questions, and messages. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a syntactically valid `.hipp` script (`treating_copd.hipp`), when the engine loads and executes the script, then the plan runs according to its defined logic, producing events, questions, and messages. |
| **Status** | Not Started |
| **Evidence** | Run `treating_copd.hipp` end-to-end via simulation; cross-reference ST-\* execution tests. |

#### AT-02 — Readability by Medical Professionals

| Field | Value |
|---|---|
| **STKR Reference** | STKR-02 |
| **Description** | Confirm that example care plan scripts are readable and reviewable by medical professionals without programming training. |
| **Verification Method** | Manual |
| **Pass Criteria** | Given a care plan script, when presented to a medical professional unfamiliar with programming, then the professional can identify the script's clinical intent and decision logic. |
| **Status** | Not Started |
| **Evidence** | Expert review session of example scripts (e.g., `treating_copd.hipp`); documented reviewer feedback. Natural language trigger syntax (`every day`, `every other day`, `every third week`) enhances readability per REQ-3.8-07; verified by ST-3.8-07, UT-PERIODS-08, UT-PERIODS-09. |

#### AT-03 — Embeddable Runtime

| Field | Value |
|---|---|
| **STKR Reference** | STKR-03 |
| **Description** | Confirm the engine can be embedded in a host application via its C-compatible FFI, operating correctly within the host process. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a host application linked against the engine's static library, when the host calls the public C API to load and execute a plan, then the engine operates correctly within the host process. |
| **Status** | Evidence Available |
| **Evidence** | UT-FFI-01..10 (Rust-side FFI unit tests covering parse, validate, engine lifecycle, load, periods, time, simulation, stop); IT-23 (FFI JSON parsing logic); successful integration in the SwiftUI editor (`hippocrates_editor`). |

#### AT-04 — Validation Without Execution

| Field | Value |
|---|---|
| **STKR Reference** | STKR-04 |
| **Description** | Confirm the engine can validate scripts for correctness without executing them, reporting all errors with source locations. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a `.hipp` script with errors, when the engine validates the script, then all errors are reported with line and column numbers, without executing any plan logic. |
| **Status** | Not Started |
| **Evidence** | Run `hippocrates_validate_file` on scripts with known errors; cross-reference ST-\* validation tests. |

#### AT-05 — Event-Driven Execution

| Field | Value |
|---|---|
| **STKR Reference** | STKR-05 |
| **Description** | Confirm the engine reacts to scheduled times, value changes, and external triggers according to the plan's defined logic. |
| **Verification Method** | Automated |
| **Pass Criteria** | (1) Given a plan with periodic triggers, change-of-value triggers, and start-of-period triggers, when the engine executes the plan, then each trigger fires at the correct time and invokes the corresponding actions. (2) Given a periodic trigger with `at 08:00`, events fire at 08:00 each day. (3) Given `every <period> for <duration>`, events fire at every period occurrence within the window. |
| **Status** | Evidence Available |
| **Evidence** | ST-3.8-01..06, ST-5-05; IT-26 (time-pinned), IT-27 (period repetition); UT-PERIODS-06, UT-PERIODS-07. |

#### AT-06 — Simulation Mode

| Field | Value |
|---|---|
| **STKR Reference** | STKR-06 |
| **Description** | Confirm the engine supports accelerated simulation execution for testing and visualization without real-time delays. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a plan with time-based triggers, when simulation mode is enabled, then the plan executes to completion at maximum speed, producing the same logical results as real-time execution. |
| **Status** | Evidence Available |
| **Evidence** | ST-5-04 (simulation mode system test); UT-RT-17 (simulation mode unit test); IT-07..IT-09, IT-20, IT-21 (simulation integration tests). |

---

### 4.2 Design Philosophy (STKR-10 through STKR-20)

#### AT-10 — Completeness

| Field | Value |
|---|---|
| **STKR Reference** | STKR-10 |
| **Description** | Confirm the engine rejects scripts with decision branches that are not exhaustively handled, such as gaps in value ranges or unhandled assessment cases. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a script with an assessment that does not cover the full range of valid values, when the engine validates the script, then validation fails with an error identifying the coverage gap. |
| **Status** | Not Started |
| **Evidence** | REQ-4.4-\* test suite (coverage validation tests). |

#### AT-11 — Readability Over Defaults

| Field | Value |
|---|---|
| **STKR Reference** | STKR-11 |
| **Description** | Confirm the language uses no implicit default values; every parameter must be explicitly stated. |
| **Verification Method** | Automated + Manual |
| **Pass Criteria** | Given any language construct, when a required parameter is omitted, then the engine rejects the script rather than applying a hidden default. |
| **Status** | Not Started |
| **Evidence** | REQ-4.2-\* tests; manual review of language grammar for hidden defaults. |

#### AT-12 — Meaningful Values

| Field | Value |
|---|---|
| **STKR Reference** | STKR-12 |
| **Description** | Confirm values carry semantic meaning beyond raw numbers, supporting meaning models that map numeric ranges to clinical labels. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a value definition with a meaning model, when a numeric value is assessed, then the engine resolves the value to its clinical meaning label. |
| **Status** | Not Started |
| **Evidence** | REQ-5.3-\* tests (meaning evaluation). |

#### AT-13 — Contextual Statements

| Field | Value |
|---|---|
| **STKR Reference** | STKR-13 |
| **Description** | Confirm the syntax creates natural reading flow where each statement is understandable in the context of preceding statements. |
| **Verification Method** | Manual |
| **Pass Criteria** | Given a block of statements, when read sequentially, then each statement is understandable in the context established by the preceding statements. |
| **Status** | Not Started |
| **Evidence** | Manual review of example scripts by domain experts. |

#### AT-14 — Separation of Concerns

| Field | Value |
|---|---|
| **STKR Reference** | STKR-14 |
| **Description** | Confirm that `.hipp` scripts contain no platform-specific constructs; all platform integration happens through the embedding API. |
| **Verification Method** | Manual |
| **Pass Criteria** | Given a `.hipp` script, when inspected for platform-specific constructs, then none are found; the script is purely domain logic. |
| **Status** | Not Started |
| **Evidence** | Manual audit of `.hipp` grammar definition and example scripts for platform-specific constructs. |

#### AT-15 — Value Constraints

| Field | Value |
|---|---|
| **STKR Reference** | STKR-15 |
| **Description** | Confirm every value has a definition of valid values and the engine rejects values outside defined constraints. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a value with defined valid values, when an out-of-range value is assigned at runtime, then the engine rejects the assignment. |
| **Status** | Not Started |
| **Evidence** | REQ-4.2-06, REQ-4.5-\* tests. |

#### AT-16 — Immutable History

| Field | Value |
|---|---|
| **STKR Reference** | STKR-16 |
| **Description** | Confirm that values recorded in the past cannot be changed and the engine maintains a complete, append-only history with timestamps. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a value recorded at time T1, when the same value is updated at time T2, then both T1 and T2 entries exist in the history; the T1 entry is not modified. |
| **Status** | Evidence Available |
| **Evidence** | UT-RT-15 (append-only value history verification); IT-10, IT-14, IT-15 (integration tests exercising value history). |

#### AT-17 — Localization Support

| Field | Value |
|---|---|
| **STKR Reference** | STKR-17 |
| **Description** | Confirm the language supports translation metadata for automatic translation of scripts into local languages. |
| **Verification Method** | Manual |
| **Pass Criteria** | Given a script with translatable text, when translation metadata is provided, then the engine can resolve text in the target language. |
| **Status** | Not Started |
| **Evidence** | Manual review confirming the language supports translation metadata. |

#### AT-18 — Medical Domain Awareness

| Field | Value |
|---|---|
| **STKR Reference** | STKR-18 |
| **Description** | Confirm the engine recognizes standard medical units, measurement types, and terminology without requiring custom definitions. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a script using standard medical units, when the engine parses and validates the script, then units are recognized without requiring custom definitions. |
| **Status** | Not Started |
| **Evidence** | REQ-3.2-02, REQ-4.1-\* tests (built-in units). |

#### AT-19 — Automatic Persistence

| Field | Value |
|---|---|
| **STKR Reference** | STKR-19 |
| **Description** | Confirm all value changes are automatically stored without explicit save logic, and value history is retrievable after recording. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a running plan that records values, when the engine is queried for value history after recording, then all recorded values are available with their timestamps. |
| **Status** | Evidence Available |
| **Evidence** | UT-RT-16 (value history retrieval with timestamps); IT-10, IT-14 (integration tests exercising value retrieval). |

#### AT-20 — Plan Autonomy

| Field | Value |
|---|---|
| **STKR Reference** | STKR-20 |
| **Description** | Confirm a plan is self-contained and can be loaded into a fresh engine instance without additional definition files. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a valid `.hipp` script, when loaded into a fresh engine instance, then the engine can execute the plan without requiring additional definition files. |
| **Status** | Not Started |
| **Evidence** | Loading standalone `.hipp` files in fresh engine instances; cross-reference IT-\* integration tests. |

---

### 4.3 Safety (STKR-30 through STKR-35)

#### AT-30 — No Comparison Operators

| Field | Value |
|---|---|
| **STKR Reference** | STKR-30 |
| **Description** | Confirm the language does not support comparison operators and that scripts containing them are rejected. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a script containing a comparison operator, when the engine parses the script, then parsing fails with an error. |
| **Status** | Not Started |
| **Evidence** | REQ-2-03 test. |

#### AT-31 — Unit Discipline

| Field | Value |
|---|---|
| **STKR Reference** | STKR-31 |
| **Description** | Confirm all numeric values must carry explicit units and the engine rejects unitless numeric literals. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a numeric literal without a unit, when the engine parses or validates the script, then the script is rejected with a unit requirement error. |
| **Status** | Not Started |
| **Evidence** | REQ-3.2-05, REQ-4.2-01, REQ-4.2-02, REQ-4.2-03 tests. |

#### AT-32 — Exhaustive Assessment Coverage

| Field | Value |
|---|---|
| **STKR Reference** | STKR-32 |
| **Description** | Confirm assessment and meaning definitions cover the full range of valid values, and the engine rejects definitions with gaps, overlaps, or missing cases. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given an assessment with a gap between 5 and 8 in a range of 0 to 10, when the engine validates the script, then validation fails identifying the missing span. |
| **Status** | Not Started |
| **Evidence** | REQ-4.4-\* test suite (14 tests). |

#### AT-33 — Data Flow Validation

| Field | Value |
|---|---|
| **STKR Reference** | STKR-33 |
| **Description** | Confirm the engine detects use of values before assignment and circular dependencies during static validation. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a script that uses a value before it has been assigned or asked, when the engine validates the script, then validation fails with a data flow error. |
| **Status** | Not Started |
| **Evidence** | REQ-4.3-\* tests. |

#### AT-34 — Input Precision Validation

| Field | Value |
|---|---|
| **STKR Reference** | STKR-34 |
| **Description** | Confirm the engine validates that runtime input values respect the decimal precision implied by the value's definition. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a value defined with integer precision (e.g., 0 ... 10), when a float answer (e.g., 3.5) is provided at runtime, then the engine rejects the answer. |
| **Status** | Not Started |
| **Evidence** | REQ-5.2-01 test. |

#### AT-35 — Data Sufficiency Handling

| Field | Value |
|---|---|
| **STKR Reference** | STKR-35 |
| **Description** | Confirm the engine requires explicit handling of insufficient data scenarios in statistical calculations. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a statistical calculation over a timeframe with no data, when the script does not handle the "Not enough data" case, then validation fails requiring explicit handling. |
| **Status** | Not Started |
| **Evidence** | REQ-4.6-\* tests. |

#### AT-36 — Plan Completion Actions

| Field | Value |
|---|---|
| **STKR Reference** | STKR-36 |
| **Description** | Confirm that plans support defining actions that execute when the plan reaches its natural end, so that patients and care providers receive completion notifications. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given a plan with an `after plan:` block, when the plan's event loop finishes (all triggers exhausted), then the statements in the `after plan:` block execute exactly once. |
| **Status** | Not Started |
| **Evidence** | ST-3.7-10 (system test); UT-PLAN-01 (unit test); IT-28 (integration test). |

---

### 4.4 Regulatory (STKR-40 through STKR-42)

#### AT-40 — Requirements Traceability

| Field | Value |
|---|---|
| **STKR Reference** | STKR-40 |
| **Description** | Confirm every system requirement traces to a stakeholder requirement, and every requirement has at least one associated test. |
| **Verification Method** | Documentation |
| **Pass Criteria** | Given the traceability matrix, when audited, then every system requirement maps to at least one stakeholder requirement, and every requirement has at least one associated test. |
| **Status** | Not Started |
| **Evidence** | Documentation audit of `04-traceability.md`. |

#### AT-41 — Reproducible Verification

| Field | Value |
|---|---|
| **STKR Reference** | STKR-41 |
| **Description** | Confirm all verification evidence is reproducible, producing deterministic results on independent runs. |
| **Verification Method** | Automated |
| **Pass Criteria** | Given the engine test suite, when run on a clean checkout, then all tests produce the same pass/fail results. |
| **Status** | Not Started |
| **Evidence** | Run `cargo test` on a clean checkout twice and compare results. |

#### AT-42 — Class II Documentation Readiness

| Field | Value |
|---|---|
| **STKR Reference** | STKR-42 |
| **Description** | Confirm the documentation set is structured to support Class II medical device regulatory submissions per IEC 62304 and ISO 14971. |
| **Verification Method** | Documentation |
| **Pass Criteria** | Given the V-Model documentation set, when reviewed against IEC 62304 required artifacts, then all required document types are present with appropriate content. |
| **Status** | Not Started |
| **Evidence** | Documentation audit against IEC 62304 checklist. |

---

## 5. Coverage Summary

| STKR ID | AT ID | Verification Method | Status |
|---|---|---|---|
| STKR-01 | AT-01 | Automated | Not Started |
| STKR-02 | AT-02 | Manual | Not Started |
| STKR-03 | AT-03 | Automated | Evidence Available |
| STKR-04 | AT-04 | Automated | Not Started |
| STKR-05 | AT-05 | Automated | Not Started |
| STKR-06 | AT-06 | Automated | Evidence Available |
| STKR-10 | AT-10 | Automated | Not Started |
| STKR-11 | AT-11 | Automated + Manual | Not Started |
| STKR-12 | AT-12 | Automated | Not Started |
| STKR-13 | AT-13 | Manual | Not Started |
| STKR-14 | AT-14 | Manual | Not Started |
| STKR-15 | AT-15 | Automated | Not Started |
| STKR-16 | AT-16 | Automated | Evidence Available |
| STKR-17 | AT-17 | Manual | Not Started |
| STKR-18 | AT-18 | Automated | Not Started |
| STKR-19 | AT-19 | Automated | Evidence Available |
| STKR-20 | AT-20 | Automated | Not Started |
| STKR-30 | AT-30 | Automated | Not Started |
| STKR-31 | AT-31 | Automated | Not Started |
| STKR-32 | AT-32 | Automated | Not Started |
| STKR-33 | AT-33 | Automated | Not Started |
| STKR-34 | AT-34 | Automated | Not Started |
| STKR-35 | AT-35 | Automated | Not Started |
| STKR-36 | AT-36 | Automated | Not Started |
| STKR-40 | AT-40 | Documentation | Not Started |
| STKR-41 | AT-41 | Automated | Not Started |
| STKR-42 | AT-42 | Documentation | Not Started |

All 27 stakeholder requirements are covered by acceptance test cases.

---

## Revision History

| Version | Date | Changes |
|---|---|---|
| 1.0 | 2026-03-20 | Initial acceptance test plan |
| 1.1 | 2026-03-20 | Updated AT-03, AT-06, AT-16, AT-19 with automated evidence references. |
| 1.2 | 2026-03-23 | Updated AT-05 with time-of-day and period repetition acceptance criteria and evidence. |
| 1.3 | 2026-03-23 | Added AT-36 (Plan Completion Actions) for STKR-36. |
| 1.4 | 2026-03-23 | Updated AT-02 evidence with natural language trigger syntax references (REQ-3.8-07). |
