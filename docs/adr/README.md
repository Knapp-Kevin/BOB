# Architecture Decision Records

ADRs capture durable architectural choices and their consequences.

## Index

| ID | Decision | Status |
| --- | --- | --- |
| ADR-0001 | B.O.B. is the agent and owns the work | Accepted |
| ADR-0002 | Use Tauri + Rust for the revived desktop architecture | Accepted |
| ADR-0003 | Subscription-first inference ordering | **Rejected** |
| ADR-0004 | B.O.B. owns local canonical state and continuity | Accepted |
| ADR-0005 | Separate B.O.B. Assist authority from explicit Delegate authority | Accepted |
| ADR-0006 | Keep B.O.B. capabilities portable across host harnesses | Accepted |

ADR-0003 is retained as decision history only. Its fixed subscription-first ordering was rejected before acceptance and is not current implementation authority. Current cost/provider behavior is governed by `docs/governance/AI_COST_AND_PROVIDER_POLICY.md`, reconciled PRD-0003, RFC-0002, and Wayfinder #79.

Accepted ADRs are immutable except for status metadata and clarifying links. A changed accepted decision requires a new ADR that explicitly supersedes the old one.
