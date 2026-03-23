# Stakeholder Requirements (STKR)

**Document ID:** STKR
**Version:** 1.0
**Status:** Draft

## 1. Purpose and Scope

This document captures the stakeholder-level requirements for the Hippocrates engine — a domain-specific language (DSL) runtime for authoring and executing medical care plans. These requirements describe outcomes and needs in business and user language, not implementation details.

Acceptance testing (see [`test-plans/03-acceptance-test-plan.md`](test-plans/03-acceptance-test-plan.md)) validates against these requirements.

### Stakeholders

| Role | Interest |
|---|---|
| Medical professionals | Author and review care plan scripts without programming training |
| Software integrators | Embed the engine in mobile and desktop applications |
| Regulatory bodies | Audit traceability, safety, and validation evidence |
| Patients | Receive safe, deterministic care plan behavior |

---

## 2. Purpose and Scope Requirements

### STKR-01 — Medical Care Plan Execution

The engine shall parse and execute medical care plans authored in `.hipp` scripts, producing deterministic, auditable behavior for a single patient context.

**Priority:** Must
**Acceptance criteria:**
- Given a syntactically valid `.hipp` script,
- When the engine loads and executes the script,
- Then the plan runs according to its defined logic, producing events, questions, and messages.

### STKR-02 — Readability by Medical Professionals

Scripts shall be readable and reviewable by medical professionals without programming training. The language syntax shall mimic natural English sentences.

**Priority:** Must
**Acceptance criteria:**
- Given a care plan script,
- When presented to a medical professional unfamiliar with programming,
- Then the professional can identify the script's clinical intent and decision logic.

### STKR-03 — Embeddable Runtime

The engine shall be embeddable in host applications (iOS, Android, macOS) via a C-compatible Foreign Function Interface (FFI), without requiring the host to understand engine internals.

**Priority:** Must
**Acceptance criteria:**
- Given a host application linked against the engine's static library,
- When the host calls the public C API to load and execute a plan,
- Then the engine operates correctly within the host process.

### STKR-04 — Validation Without Execution

The engine shall validate scripts for correctness without executing them, reporting all errors with source locations.

**Priority:** Must
**Acceptance criteria:**
- Given a `.hipp` script with errors,
- When the engine validates the script,
- Then all errors are reported with line and column numbers, without executing any plan logic.

### STKR-05 — Event-Driven Execution

The engine shall support event-driven execution, reacting to scheduled times, value changes, and external triggers according to the plan's defined logic. Periodic triggers shall support pinning to a specific time of day, and period-based triggers shall fire at every occurrence within a duration window.

**Priority:** Must
**Acceptance criteria:**
- Given a plan with periodic triggers, change-of-value triggers, and start-of-period triggers,
- When the engine executes the plan,
- Then each trigger fires at the correct time and invokes the corresponding actions.
- Given a periodic trigger with an `at 08:00` clause,
- When the engine executes the plan,
- Then the trigger fires at 08:00 each day, not at the plan start time.
- Given a trigger using `every <period> for <duration>`,
- When the engine executes the plan,
- Then the trigger fires at every occurrence of the period within the duration window.

### STKR-06 — Simulation Mode

The engine shall support a simulation mode that executes plans at accelerated speed for testing and visualization, without requiring real-time delays.

**Priority:** Should
**Acceptance criteria:**
- Given a plan with time-based triggers,
- When simulation mode is enabled,
- Then the plan executes to completion at maximum speed, producing the same logical results as real-time execution.

---

## 3. Design Philosophy Requirements

These requirements are derived from the foundational principles of the Hippocrates language.

### STKR-10 — Completeness

All decision branches in a care plan shall be explicitly handled. The engine shall reject scripts that contain gaps in value ranges or unhandled assessment cases.

**Priority:** Must
**Acceptance criteria:**
- Given a script with an assessment that does not cover the full range of valid values,
- When the engine validates the script,
- Then validation fails with an error identifying the coverage gap.

### STKR-11 — Readability Over Defaults

No implicit default values shall be used in the language. Every parameter and constraint shall be explicitly stated in the script so that the expression can be understood without deep knowledge of the engine.

**Priority:** Must
**Acceptance criteria:**
- Given any language construct,
- When a required parameter is omitted,
- Then the engine rejects the script rather than applying a hidden default.

### STKR-12 — Meaningful Values

Values shall carry semantic meaning beyond raw numbers. The language shall support defining meaning models (e.g., mapping numeric ranges to clinical labels like "Normal", "Elevated", "Critical") so that physicians can write logic using clinical terms.

**Priority:** Must
**Acceptance criteria:**
- Given a value definition with a meaning model,
- When a numeric value is assessed,
- Then the engine resolves the value to its clinical meaning label.

### STKR-13 — Contextual Statements

The syntax shall be designed so that a statement creates the context for subsequent statements, enabling natural reading flow without requiring forward or backward references.

**Priority:** Should
**Acceptance criteria:**
- Given a block of statements,
- When read sequentially,
- Then each statement is understandable in the context established by the preceding statements.

### STKR-14 — Separation of Concerns

The language shall contain nothing specific to any platform, technology, or deployment target. All platform integration shall happen through the embedding API, not in scripts.

**Priority:** Must
**Acceptance criteria:**
- Given a `.hipp` script,
- When inspected for platform-specific constructs,
- Then none are found; the script is purely domain logic.

### STKR-15 — Value Constraints

Every value shall have a definition of valid values (ranges, enumerations, or sets). The engine shall reject values outside defined constraints to prevent faulty or dangerous decisions.

**Priority:** Must
**Acceptance criteria:**
- Given a value with defined valid values,
- When an out-of-range value is assigned at runtime,
- Then the engine rejects the assignment.

### STKR-16 — Immutable History

Values recorded in the past shall not be changed. The engine shall maintain a complete, append-only history of all value changes with timestamps.

**Priority:** Must
**Acceptance criteria:**
- Given a value that was recorded at time T1,
- When the same value is updated at time T2,
- Then both T1 and T2 entries exist in the history; the T1 entry is not modified.

### STKR-17 — Localization Support

The language shall be designed to support automatic translation of scripts into supported local languages. All text values shall support translation metadata.

**Priority:** May
**Acceptance criteria:**
- Given a script with translatable text,
- When translation metadata is provided,
- Then the engine can resolve text in the target language.

### STKR-18 — Medical Domain Awareness

The engine shall come with built-in knowledge of medical units, measurement types, and terminology (e.g., temperature in Celsius, blood pressure in mmHg, weight in kg).

**Priority:** Must
**Acceptance criteria:**
- Given a script using standard medical units,
- When the engine parses and validates the script,
- Then units are recognized without requiring custom definitions.

### STKR-19 — Automatic Persistence

The engine's runtime model shall be built around durable value history. All changes to values shall be automatically stored without requiring explicit save logic in scripts.

**Priority:** Must
**Acceptance criteria:**
- Given a running plan that records values,
- When the engine is queried for value history after recording,
- Then all recorded values are available with their timestamps.

### STKR-20 — Plan Autonomy

A plan shall be self-contained. All definitions required to execute a plan shall be included within the plan script itself, with no external dependencies beyond the engine runtime.

**Priority:** Must
**Acceptance criteria:**
- Given a valid `.hipp` script,
- When loaded into a fresh engine instance,
- Then the engine can execute the plan without requiring additional definition files.

---

## 4. Safety Requirements

### STKR-30 — No Comparison Operators

The language shall not support comparison operators (`<`, `>`, `<=`, `>=`). All threshold logic shall use explicit ranges (`min ... max`) to prevent ambiguity in medical decision boundaries.

**Priority:** Must
**Acceptance criteria:**
- Given a script containing a comparison operator,
- When the engine parses the script,
- Then parsing fails with an error.

### STKR-31 — Unit Discipline

All numeric values in the language shall carry explicit units. The engine shall reject unitless numeric literals and enforce unit compatibility in calculations.

**Priority:** Must
**Acceptance criteria:**
- Given a numeric literal without a unit,
- When the engine parses or validates the script,
- Then the script is rejected with a unit requirement error.

### STKR-32 — Exhaustive Assessment Coverage

Assessment and meaning definitions shall cover the full range of valid values. The engine shall reject definitions with gaps, overlaps, or missing cases.

**Priority:** Must
**Acceptance criteria:**
- Given an assessment with a gap between 5 and 8 in a range of 0–10,
- When the engine validates the script,
- Then validation fails identifying the missing span.

### STKR-33 — Data Flow Validation

The engine shall detect use of values before assignment and circular dependencies during static validation, before any plan execution begins.

**Priority:** Must
**Acceptance criteria:**
- Given a script that uses a value before it has been assigned or asked,
- When the engine validates the script,
- Then validation fails with a data flow error.

### STKR-34 — Input Precision Validation

The engine shall validate that runtime input values respect the decimal precision implied by the value's definition (integer vs. float ranges).

**Priority:** Must
**Acceptance criteria:**
- Given a value defined with integer precision (e.g., 0 ... 10),
- When a float answer (e.g., 3.5) is provided at runtime,
- Then the engine rejects the answer.

### STKR-35 — Data Sufficiency Handling

The engine shall require explicit handling of insufficient data scenarios in statistical calculations, preventing silent failures when history is too short.

**Priority:** Must
**Acceptance criteria:**
- Given a statistical calculation over a timeframe with no data,
- When the script does not handle the "Not enough data" case,
- Then validation fails requiring explicit handling.

### STKR-36 — Plan Completion Actions

Plans shall support defining actions that execute when the plan reaches its natural end, so that patients and care providers receive completion notifications without relying on timing hacks.

**Priority:** Must
**Acceptance criteria:**
- Given a plan with an `after plan:` block,
- When the plan's event loop finishes (all triggers exhausted),
- Then the statements in the `after plan:` block execute exactly once.

---

## 6. AI Integration Requirements

### STKR-37 — LLM-Correctable Error Diagnostics

Error messages produced by the engine shall be detailed and specific enough for a large language model (LLM) to automatically correct Hippocrates scripts without human intervention. This requires human-readable descriptions (not raw parser internals), detection of all reference errors, and actionable suggested fixes.

**Priority:** Must
**Acceptance criteria:**
- Given a script with a syntax error, when validated, then the error message describes what's wrong in plain English with a suggested fix.
- Given a script referencing an undeclared variable, addressee, or unit, when validated, then the error identifies the undefined reference and lists available definitions.
- Given a script with a coverage gap, when validated, then the error includes the exact range to add.

---

## 5. Regulatory Requirements

### STKR-40 — Requirements Traceability

Every system requirement shall trace to a stakeholder requirement, and every test shall trace to a requirement. The traceability matrix shall be maintained as a living document.

**Priority:** Must
**Acceptance criteria:**
- Given the traceability matrix,
- When audited,
- Then every system requirement maps to at least one stakeholder requirement, and every requirement has at least one associated test.

### STKR-41 — Reproducible Verification

All verification evidence shall be reproducible. Running the test suite shall produce deterministic results that can be independently verified.

**Priority:** Must
**Acceptance criteria:**
- Given the engine test suite,
- When run on a clean checkout,
- Then all tests produce the same pass/fail results.

### STKR-42 — Class II Documentation Readiness

The engine documentation shall be structured to support Class II medical device regulatory submissions per IEC 62304 and ISO 14971, including requirements specification, design documentation, test plans, and risk analysis.

**Priority:** Should
**Acceptance criteria:**
- Given the V-Model documentation set,
- When reviewed against IEC 62304 required artifacts,
- Then all required document types are present with appropriate content.

---

## Revision History

| Version | Date | Changes |
|---|---|---|
| 1.0 | 2026-03-20 | Initial V-Model adoption from existing project documentation |
| 1.1 | 2026-03-23 | Extended STKR-05 acceptance criteria for time-of-day pinning and period-based repetition |
| 1.2 | 2026-03-23 | Added STKR-36 (Plan Completion Actions) for `after plan:` block |
| 1.3 | 2026-03-23 | Natural language trigger syntax (bare units, ordinals) enhances STKR-02 readability. No new STKR required. |
| 1.4 | 2026-03-23 | Added STKR-37 (LLM-Correctable Error Diagnostics) in new AI Integration Requirements section. |
