# Hippocrates Project Summary

**Date:** February 8, 2026

## 1. Executive Summary

Hippocrates is a domain-specific language (DSL) and runtime stack for authoring and executing medical care plans in a readable, structured, and verifiable way. The project combines:

- A formally specified language for plan logic (`.hipp` scripts).
- A Rust engine for parsing, validation, scheduling, and execution.
- A native macOS SwiftUI editor for authoring and visualizing plan behavior.

The central idea is to let medical-domain logic be authored in near-natural language while preserving strict machine validation. In practical terms, Hippocrates aims to reduce ambiguity in care workflows by making units explicit, decision ranges exhaustive, and plan behavior event-driven and traceable.

## 2. Concept of the Language

At its core, Hippocrates is designed around medical decision safety and readability.

The language model emphasizes:

- Natural-language-like expressions so domain experts can read and review scripts.
- Strong typing around quantities, units, and value meaning.
- Explicit ranges for decision logic instead of free-form comparison operators.
- Event- and time-driven execution so plans can react to scheduled moments and incoming values.
- Intended-use metadata so scripts carry explicit usage context.

### 2.1 Design Principles

The documentation and specification consistently focus on these principles:

- **Completeness of decisions**: the language encourages exhaustive handling of value ranges and “other cases,” especially in medical assessments.
- **Unit discipline**: values are expected to carry units, and conversion behavior is explicit.
- **Semantic clarity**: value definitions can include meaning models, not just raw numerical ranges.
- **Context-driven behavior**: actions are tied to periods, events, and conditions over time.
- **Persistence and history**: the runtime model is built around durable value history and reproducible execution.

### 2.2 Grammar and Safety Constraints

The current formal specification and repository rules enforce several important constraints:

- Identifiers are angle-bracketed (for example, `<heart rate>`).
- Comparison operators like `<`, `>`, `<=`, `>=` are intentionally not supported in plan grammar.
- Ranges (`min ... max`) are the intended mechanism for threshold logic.
- Block structure is indentation-based, with explicit statement boundaries.

This combination is important for medical plan authoring because it narrows syntactic freedom in favor of clarity and deterministic behavior.

## 3. System Architecture and Components

Hippocrates is organized as a two-part product: runtime engine and editor.

### 3.1 Engine (Rust)

The engine implements parsing, validation, execution, and embedding interfaces. It exposes a C-compatible FFI boundary so host applications can integrate it across platforms.

Core functional areas include:

- Parser and AST construction (`pest`-based grammar).
- Validation and semantic checks (including interval/range correctness and data-flow/coverage style checks).
- Runtime execution and scheduling.
- Environment/session handling for plan state.
- FFI layer for external app integration.

The engine is configured as both Rust library and static library output, supporting embedding into native app stacks.

### 3.2 Editor (SwiftUI)

The macOS editor provides:

- Script editing with syntax-aware rendering.
- Timeline and execution visualizations.
- Integration with the engine library for validation/execution loops.

This editor acts as a developer and domain-authoring workbench for iterating on plan scripts.

## 4. Delivery Timeline: Start to First Running Version

Using repository history, the project’s early delivery cadence is clear:

- **Start point**: `Initial commit` on **2026-01-17 12:51:08 +0100**.
- **First running milestone**: `Refactor: Convert to native macOS App with Rust Engine integration` on **2026-01-18 15:10:57 +0100**.
- **Elapsed time**: **1 day, 2:19:49** (94,789 seconds).
- **Commits from start to first running version**: **3 commits**.

Interpretation:

- The first two commits established repository scaffolding and formal documentation.
- The third commit introduced the integrated app-engine architecture, which is the first defensible “running version” milestone.

As of the current repository state, total history depth is **54 commits**, indicating sustained iteration after the initial runnable baseline.

## 5. Current Codebase Size

Current tracked source LOC (code-oriented extensions) is:

- **Total source LOC: 21,064**

Breakdown by major area:

| Area | LOC |
|---|---:|
| Engine source (`hippocrates_engine/src`) | 11,633 |
| Engine tests (`hippocrates_engine/tests`) | 5,415 |
| Editor source (`hippocrates_editor/Sources`) | 3,176 |
| Engine examples (`hippocrates_engine/examples`) | 373 |
| Editor tests (`hippocrates_editor/Tests`) | 98 |
| Other source files | 369 |

Notes on counting:

- Count is based on git-tracked files with source-like extensions: `.rs`, `.swift`, `.c`, `.h`, `.pest`, `.toml`, `.sh`, `.yml`, `.yaml`, `.hipp`.
- Binary assets and generated artifacts are excluded from this source LOC metric.

## 6. Quality and Engineering Posture

The project structure reflects a quality-first posture, especially around language behavior:

- A dedicated specification file and traceability-oriented spec tests.
- Broad engine tests across grammar, execution, validation, units, periods/plans, and integration paths.
- Separation between language engine concerns and editor concerns.
- Explicit repository rules emphasizing parity during refactors and avoidance of hidden grammar shortcuts.

This is consistent with systems that need deterministic, auditable behavior.

## 7. Regulatory Trajectory (Including Class II Positioning)

Hippocrates documentation already treats “intended use” as a first-class concern in script definition, including explicit medical/non-medical intent categories and constraints tied to use context. This creates a foundation for regulated software development practices.

In addition, the Hippocrates Engine is documented to become a **Class II medical device** as part of the project trajectory. In practical terms, this implies the engine and associated toolchain should continue moving toward:

- formalized requirements traceability,
- risk-driven validation rigor,
- reproducible verification evidence,
- controlled release and change-management workflows.

This trajectory aligns with the existing direction of strict language constraints, explicit semantics, and substantial automated test coverage.

## 8. Current State, in One Paragraph

Hippocrates has moved beyond concept into an implemented stack with a formal language specification, a Rust execution engine, and a native editor for authoring and visualization. The first integrated runnable version was reached quickly (in just over one day and three commits), and the codebase has since grown to 21k+ source lines with a significant proportion dedicated to engine correctness and tests. The architecture and documentation already emphasize intended-use governance and safety-oriented language design, while the stated path toward Class II medical-device readiness frames the next stage of engineering maturity.

## 9. Source References

- `/Users/manfred/Development/Projects/Hippocrates/README.md`
- `/Users/manfred/Development/Projects/Hippocrates/specification/hippocrates_specification.md`
- `/Users/manfred/Development/Projects/Hippocrates/specification/runtime_architecture.md`
- `/Users/manfred/Development/Projects/Hippocrates/documents/hippocrates.md`
- `/Users/manfred/Development/Projects/Hippocrates/hippocrates_engine/Cargo.toml`
- `/Users/manfred/Development/Projects/Hippocrates/.git` history (`git log --reverse`)
