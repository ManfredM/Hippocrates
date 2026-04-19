# Hippocrates Engine — V-Model Specification

This directory contains the complete V-Model documentation for the Hippocrates engine, providing bidirectional traceability from stakeholder needs through implementation to verification.

## V-Model Structure

```
Stakeholder Requirements (SREQ)    <──────────>   Acceptance Testing (AT)
   System Requirements (REQ)       <────────>      System Testing (ST)
      System Design (SYS)          <──────>        Integration Testing (IT)
         Detailed Design (DET)     <────>          Unit Testing (UT)
                  Implementation
```

Every requirement on the left side has a corresponding verification on the right side. Nothing is implemented without a spec; nothing is verified without traceability back to a requirement.

## ID Conventions

| V-Model Level | Prefix | Format | Example |
|---|---|---|---|
| Stakeholder Requirements | `SREQ-` | `SREQ-HIPP-nnn` | `SREQ-HIPP-010` |
| System Requirements | `REQ-` | `REQ-HIPP-{section}-nnn` | `REQ-HIPP-VALUE-005` |
| System Design | `SYS-` | `SYS-HIPP-nnn` | `SYS-HIPP-012` |
| Detailed Design | `DET-` | `DET-HIPP-{module}-nnn` | `DET-HIPP-FFI-003` |
| Unit Tests | `UT-` | `UT-HIPP-{module}-nnn` | `UT-HIPP-PARSER-001` |
| Integration Tests | `IT-` | `IT-HIPP-nnn` | `IT-HIPP-005` |
| System Tests | `ST-` | `ST-HIPP-{section}-nnn` | `ST-HIPP-VALUE-005` |
| Acceptance Tests | `AT-` | `AT-HIPP-nnn` | `AT-HIPP-010` |

**Numbering policy**: SREQ IDs use gapped ranges to allow insertion without renumbering:
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
| 0 | [`test-plans/00-unit-test-plan.md`](test-plans/00-unit-test-plan.md) | Right (bottom) | Detailed Design (DET) |
| 1 | [`test-plans/01-integration-test-plan.md`](test-plans/01-integration-test-plan.md) | Right | System Design (SYS) |
| 2 | [`test-plans/02-system-test-plan.md`](test-plans/02-system-test-plan.md) | Right | System Requirements (REQ) |
| 3 | [`test-plans/03-acceptance-test-plan.md`](test-plans/03-acceptance-test-plan.md) | Right (top) | Stakeholder Requirements (SREQ) |

## Traceability Flow

```
SREQ-HIPP-010 ──> REQ-HIPP-LANG-001, REQ-HIPP-LANG-002  ──> SYS-HIPP-010       ──> DET-HIPP-PARSER-001  ──> UT-HIPP-PARSER-001
                                                                          │
            REQ-HIPP-LANG-001             <── ST-HIPP-LANG-001       <── IT-HIPP-005          <───┘
SREQ-HIPP-010                         <── AT-HIPP-010
```

Every row in the [traceability matrix](04-traceability.md) links a stakeholder requirement through system requirements, design, detailed design, and all four test levels. Gaps in this matrix signal incomplete work.

## Regulatory Context

This documentation structure supports Class II medical device readiness per IEC 62304 (software lifecycle) and ISO 14971 (risk management). The traceability matrix provides the audit trail required for regulatory submissions.

## Revision History

| Version | Date | Changes |
|---|---|---|
| 1.0 | 2026-03-20 | Initial V-Model README describing spec layout and ID conventions. |
| 1.1 | 2026-04-19 | Updated ID-convention table, V-Model diagram, and test-plan "Verifies" column to canonical SREQ/REQ/SYS/DET prefixes (was STKR/DES/DDR). |
