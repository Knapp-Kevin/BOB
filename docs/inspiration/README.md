# UI Inspiration Artifacts

**Status:** Historical / directional references

This directory preserves visual artifacts used during B.O.B.'s pre-alpha interaction exploration. They are not normative specifications and do not describe the current implementation exhaustively.

The product has since converged through accepted design authority and merged implementation work. When an image conflicts with current product, architecture, design, Wayfinder decisions, or the landed application, current authority wins.

## Durable lessons

These artifacts helped establish principles that remain current:

- B.O.B. is the user-facing agent; models, runtimes, and tools remain capabilities behind B.O.B.
- Today is the obvious starting surface.
- One useful next action should dominate secondary choices.
- Capture should be cheaper than organization.
- Reduced-information behavior should remove cognitive load rather than create another workflow.
- Chat should remain connected to current work rather than becoming a provider switchboard.
- Accessibility, readable hierarchy, and user-controllable information density are first-class requirements.

## Current disposition

Several details in the generated concepts are now superseded:

- ordinary Settings no longer carries persistent provider-specific status or development-governance exposition;
- Gemini API remains an advanced optional adapter rather than B.O.B.'s normal product identity;
- Today, Inbox, Chat, and Settings hierarchy converged through completed Wayfinder #86 and merged PRs #89/#93/#106;
- provider-independent runtime policy and a bounded Ollama tracer now exist behind B.O.B.-owned routing/policy;
- startup recovery is now represented by the restricted recovery surface in draft PR #103 and its current rendered/native acceptance contract.

Do not copy incidental generated labels, timestamps, provider names, task examples, or controls into current implementation without validating them against current authority.

## Governing references

Start with:

- [`../PRODUCT.md`](../PRODUCT.md)
- [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- [`../DESIGN.md`](../DESIGN.md)
- [`../IMPLEMENTATION_PLAN.md`](../IMPLEMENTATION_PLAN.md)
- [`../ROADMAP.md`](../ROADMAP.md)
- active Wayfinder maps and current issues/pull requests

These files are preserved because design provenance is useful. They should not be kept artificially synchronized with every later UI revision.
