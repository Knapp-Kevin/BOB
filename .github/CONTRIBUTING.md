# Contributing to B.O.B.

B.O.B. is an MIT-licensed open-source project in active pre-alpha development. Contributions should make the product clearer, smaller, safer, and easier to use under real executive-function load.

## Start with the contract

Before implementation, read:

1. [`README.md`](../README.md)
2. [`docs/governance/GOVERNANCE.md`](../docs/governance/GOVERNANCE.md)
3. [`AGENTS.md`](../AGENTS.md)
4. [`docs/PRODUCT.md`](../docs/PRODUCT.md)
5. [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md)
6. [`docs/DESIGN.md`](../docs/DESIGN.md)
7. the relevant current PRD, RFC, ADR, Wayfinder map, and implementation-plan sections

Do not infer current requirements from the archived Electron/Ollama implementation, the historical pre-alpha brief, or generated inspiration images.

## Before opening work

Search existing issues and pull requests first. For a meaningful feature, prefer opening or joining an issue before investing heavily in implementation. Small documentation fixes and narrowly obvious bug fixes do not need a ceremony committee.

Never include credentials, tokens, private files, personal data, or vulnerability details in issues, pull requests, screenshots, fixtures, or logs. Security reports follow [`SECURITY.md`](SECURITY.md).

Participation is governed by [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## Contribution standard

A good change has one coherent reason to exist. It identifies the user problem, keeps scope bounded, respects canonical-state and authority boundaries, includes the smallest useful test surface, and updates documentation when behavior changes.

Material pull requests should make the following reviewable without archaeology:

- **Intent:** what problem is solved and for whom;
- **Scope:** what changes and what explicitly does not;
- **Authority:** what the application, B.O.B., user, runtime, or external process may now do;
- **Data:** what is read, written, persisted, exported, migrated, or recovered;
- **Cost/privacy:** whether billing class, locality, provider behavior, or data-use policy changes;
- **Accessibility:** whether cognitive load, keyboard use, motion, contrast, density, or readable hierarchy changes;
- **Evidence:** tests, native/manual validation, screenshots where appropriate, and failure-path verification;
- **Traceability:** PRD, RFC, ADR, Wayfinder issue, or explicit explanation when none is required.

## Current first-alpha sequencing

The current product frontier is intentionally narrow:

1. startup-recovery native acceptance under issue #85 / draft PR #103;
2. Windows 11 x64 NSIS native lifecycle acceptance under issue #84 / draft PR #115;
3. first-alpha convergence under Wayfinder #30;
4. then promotion of one already-authorized inference path with native evidence.

Do not widen provider/runtime scope or reopen completed primary-workflow UX merely because unrelated native acceptance is waiting. Safe disjoint fixes, validation, documentation reconciliation, and bounded hardening remain appropriate.

## Validation and CI

B.O.B. intentionally keeps GitHub Actions and required CI gates small. The implementing developer or coding agent owns the strongest relevant validation before requesting review.

For current source changes, start with the repository-provided commands where applicable:

```powershell
npm ci
npm run validate
```

For the accepted Windows NSIS package path:

```powershell
npm ci
npm run package:windows
```

Run additional native, recovery, credential, provider, rendered/accessibility, or package lifecycle checks when the changed boundary requires them. The pull request must state exactly what ran, what passed, what did not run, and why.

Hosted CI is a safety net, not a substitute for native Windows/product acceptance. A green job on the wrong head or the wrong execution surface is not equivalent evidence.

## Pull request expectations

Keep pull requests focused enough that a reviewer can understand the change without reconstructing several unrelated intentions. Use the repository pull-request template. Include screenshots for meaningful UI changes and concrete validation for behavior, persistence, recovery, authority, cost/privacy, or provider changes.

Maintainers may close superseded, duplicate, out-of-scope, or abandoned contributions. Closing a proposal is a scope decision, not a judgment on the person who submitted it.

## Definition of done

A change is complete when its behavior satisfies governing acceptance criteria, deterministic logic has appropriate tests, security and authority boundaries remain explicit, provider/cost/privacy behavior cannot change unexpectedly, accessibility has not regressed, documentation matches the shipped behavior, and no dead parallel path is left behind without a documented reason.

Passing CI alone does not establish the intended user or production outcome when native/manual evidence is part of the acceptance contract.

## Design bias

Prefer a smaller product with strong boundaries over a larger product with impressive nouns. B.O.B. should consume specialized inference runtimes and tools behind narrow B.O.B.-owned contracts rather than reimplementing entire vendor products, own personal work state rather than vendor sessions, and remove obsolete paths rather than preserve them indefinitely in the active tree.

B.O.B. is the user-facing agent. Do not introduce a peer-agent product model by terminology or convenience.

## Support and community

Read [`SUPPORT.md`](SUPPORT.md) for bug, question, and support routing. Read [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) for participation expectations.

## License

By contributing to B.O.B., you agree that your contribution may be distributed under the repository's [MIT License](../LICENSE).

## Historical code

The pre-revival implementation remains historical evidence in Git history and the named archive branch. The pre-alpha UI brief and inspiration artifacts are also historical/reference material. None are supported release lines or current architectural authority.
