# Hippocrates Engine — V-Model Specification

This directory contains the complete V-Model documentation for the Hippocrates engine, providing bidirectional traceability from stakeholder needs through implementation to verification.

## V-Model Structure

```
Stakeholder Requirements (STKR)    <──────────>   Acceptance Testing (AT)
   System Requirements (REQ)       <────────>      System Testing (ST)
      System Design (DES)          <──────>        Integration Testing (IT)
         Detailed Design (DDR)     <────>          Unit Testing (UT)
                  Implementation
```

Every requirement on the left side has a corresponding verification on the right side. Nothing is implemented without a spec; nothing is verified without traceability back to a requirement.

## ID Conventions

| V-Model Level | Prefix | Format | Example |
|---|---|---|---|
| Stakeholder Requirements | `STKR-` | `STKR-nn` | `STKR-10` |
| System Requirements | `REQ-` | `REQ-{section}-{nn}` | `REQ-3.4-05` |
| System Design | `DES-` | `DES-nn` | `DES-12` |
| Detailed Design | `DDR-` | `DDR-{module}-nn` | `DDR-FFI-03` |
| Unit Tests | `UT-` | `UT-{module}-nn` | `UT-PARSER-01` |
| Integration Tests | `IT-` | `IT-nn` | `IT-05` |
| System Tests | `ST-` | `ST-{section}-nn` | `ST-3.4-05` |
| Acceptance Tests | `AT-` | `AT-nn` | `AT-10` |

**Numbering policy**: STKR IDs use gapped ranges to allow insertion without renumbering:
- 01–09: Purpose and scope
- 10–29: Design philosophy
- 30–39: Safety
- 40–49: Regulatory

REQ IDs are inherited from the existing language specification and must not be renumbered.

## Documents

| # | File | V-Model Level | Description |
|---|---|---|---|
| 0 | [`00-stakeholder-requirements.md`](00-stakeholder-requirements.md) | Left (top) | What users and the business need |
| 1 | [`01-system-requirements.md`](01-system-requirements.md) | Left | Testable technical requirements (adopted from language specification) |
| 2 | [`02-system-design.md`](02-system-design.md) | Left | Component architecture and technology decisions |
| 3 | [`03-detailed-design.md`](03-detailed-design.md) | Left (bottom) | Module-level API, data models, and behavior |
| 4 | [`04-traceability.md`](04-traceability.md) | Cross-cutting | Full bidirectional traceability matrix |

### Test Plans

| # | File | V-Model Level | Verifies |
|---|---|---|---|
| 0 | [`test-plans/00-unit-test-plan.md`](test-plans/00-unit-test-plan.md) | Right (bottom) | Detailed Design (DDR) |
| 1 | [`test-plans/01-integration-test-plan.md`](test-plans/01-integration-test-plan.md) | Right | System Design (DES) |
| 2 | [`test-plans/02-system-test-plan.md`](test-plans/02-system-test-plan.md) | Right | System Requirements (REQ) |
| 3 | [`test-plans/03-acceptance-test-plan.md`](test-plans/03-acceptance-test-plan.md) | Right (top) | Stakeholder Requirements (STKR) |

## Traceability Flow

```
STKR-10 ──> REQ-2-01, REQ-2-02  ──> DES-10       ──> DDR-PARSER-01  ──> UT-PARSER-01
                                                                          │
            REQ-2-01             <── ST-2-01       <── IT-05          <───┘
STKR-10                         <── AT-10
```

Every row in the [traceability matrix](04-traceability.md) links a stakeholder requirement through system requirements, design, detailed design, and all four test levels. Gaps in this matrix signal incomplete work.

## Regulatory Context

This documentation structure supports Class II medical device readiness per IEC 62304 (software lifecycle) and ISO 14971 (risk management). The traceability matrix provides the audit trail required for regulatory submissions.
