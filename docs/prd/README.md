# Product Requirements Documents

PRDs define user problems, required behavior, acceptance criteria, and explicit non-goals. They describe what the product must accomplish without prescribing unnecessary implementation detail.

## Index

| ID | Title | Status |
| --- | --- | --- |
| PRD-0001 | Single-Agent, Multi-LLM Personal Workbench | Accepted |
| PRD-0002 | ADHD-Friendly Daily Planning | Accepted |
| PRD-0003 | Inference Runtimes and Cost Control | Accepted, reconciled 2026-08-27 |

PRD-0003 no longer carries its original subscription-first provider ordering. Current authority is provider-independent and aligned with RFC-0002, Wayfinder #79, and `docs/governance/AI_COST_AND_PROVIDER_POLICY.md`. Rejected ADR-0003 remains historical evidence only.

The core product invariant across these requirements is:

> **B.O.B. is the agent. Models, inference runtimes, provider APIs/CLIs, and tools are capabilities behind B.O.B.**

A PRD becomes authoritative when accepted through repository review under [`../governance/GOVERNANCE.md`](../governance/GOVERNANCE.md). When owner-approved product direction materially changes an accepted requirement, reconcile the PRD explicitly rather than allowing stale requirements to compete with current authority.
