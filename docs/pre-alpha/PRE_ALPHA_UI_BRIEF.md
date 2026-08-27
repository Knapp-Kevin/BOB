# Pre-Alpha UI Test Launch Brief

**Status:** Historical / completed prototype reference  
**Original purpose:** define the interaction surface used to validate B.O.B.'s first-alpha UX before production-oriented implementation hardened the architecture

## Current disposition

This document records an earlier Wayfinder prototype phase. It is preserved for provenance and design history, but it is **not current implementation authority**.

The work this brief was created to explore has moved substantially beyond the prototype state it describes:

- the Tauri 2 + Rust desktop application is implemented on `master`;
- Rust-owned SQLite persistence, migrations, backup/restore/export, and protected credentials are implemented;
- live Gemini API inference exists as an advanced optional adapter behind accepted privacy/cost policy;
- provider-independent runtime policy and a bounded Ollama tracer are implemented;
- Windows packaging exists through the accepted locked targeted-clean NSIS build path;
- the calm Settings, Today, Inbox, and Chat interaction frontier converged through completed Wayfinder #86 and merged PRs #89/#93/#106;
- startup recovery is implemented in draft PR #103 and awaits native Windows acceptance;
- native NSIS lifecycle acceptance remains open under issue #84 / draft PR #115.

Statements below from the original prototype phase such as “persistence remains gated,” “live inference remains gated,” or “credential storage is not implemented” are therefore historical and must not be used as current repository truth.

## Governing sources now

Use current authority in this order:

1. explicit repository-owner direction recorded in the current governing change;
2. active Wayfinder maps and settled decision tickets;
3. accepted PRDs, RFCs, and ADRs;
4. [`../PRODUCT.md`](../PRODUCT.md);
5. [`../ARCHITECTURE.md`](../ARCHITECTURE.md);
6. [`../DESIGN.md`](../DESIGN.md);
7. [`../IMPLEMENTATION_PLAN.md`](../IMPLEMENTATION_PLAN.md) and [`../ROADMAP.md`](../ROADMAP.md);
8. current issues and pull requests for exact-head implementation/evidence state.

The generated images in [`../inspiration/`](../inspiration/) remain directional historical references only.

## What this prototype phase established

The prototype work helped establish durable principles that remain current:

- **B.O.B. is the agent. Models, runtimes, and tools are capabilities.**
- Today is the obvious starting surface.
- One useful next action should dominate secondary choices.
- Capture should be cheaper than organization.
- Inbox should feel recoverable rather than punitive.
- B.O.B. Chat should feel like one assistant, not a provider switchboard.
- Reduced-information behavior should remove cognitive load rather than create another workflow.
- Provider/runtime detail should remain secondary except where cost, privacy, locality, capability, or troubleshooting makes it material.
- Accessibility and low cognitive load are product requirements.
- Deterministic planning remains useful when inference is unavailable.
- No silent transition into paid or materially different inference.

Those principles have since been reconciled into the current accepted design and implementation documents.

## Historical phase sequence

```text
accepted product/design questions
            |
            v
     prototype UI brief
            |
            v
   generated inspiration set
            |
            v
  runnable prototype evaluation
            |
            v
 resolved Wayfinder decisions
            |
            v
 current Tauri/Rust product
```

## Preservation rule

Do not update this file every time the product changes. Its purpose is now historical. If current implementation or product guidance is needed, update the governing documents instead.

When this historical brief conflicts with accepted current authority or the landed product, current authority wins.
