# PRD-0003: Inference Runtimes and Cost Control

**Status:** Accepted, reconciled 2026-08-27  
**Related:** RFC-0002, ADR-0001, ADR-0005, rejected ADR-0003, Wayfinder #79, `docs/governance/AI_COST_AND_PROVIDER_POLICY.md`

## Summary

B.O.B. shall support replaceable inference runtimes behind one user-facing agent identity while enforcing explicit cost, privacy, locality, and provider policy. The product must not depend on a fixed provider ordering and must never silently transition to a materially different or separately billed path.

This document supersedes its original 2026-08-19 subscription-first ordering. That ordering was rejected before it became durable architecture; ADR-0003 is retained as rejected decision history.

## Product invariant

> **B.O.B. is the agent. Models, inference runtimes, provider APIs/CLIs, and tools are capabilities behind B.O.B.**

Removing any one inference adapter must leave B.O.B. buildable, launchable, state-safe, and usefully deterministic.

## Problem

The same AI vendor may expose free, subscription-backed, metered, local, account-session, API-key, CLI, and application surfaces with different billing, privacy, entitlement, and support boundaries. Authentication mechanism does not establish billing class.

B.O.B. must keep ordinary interaction simple without hiding material cost/privacy/provider changes or letting one vendor become the architectural owner of B.O.B.'s state and continuity.

## Required billing classes

Each supported runtime adapter declares one of:

- `free`
- `subscription`
- `local`
- `metered`
- `unknown`

`unknown` is blocked until safely classified.

A billing class is independent of authentication mechanism. OAuth, browser login, CLI login, API keys, and runtime tokens are not themselves evidence of billing behavior.

## Required locality/privacy classes

Where applicable, each adapter also reports one of:

- `on_device`
- `loopback_local`
- `lan_remote`
- `cloud`
- `unknown`

Loopback does not automatically prove on-device execution, local billing, or local privacy. Unknown or materially ambiguous locality/privacy state fails closed when the requested policy requires stronger guarantees.

## Selection policy

The active product boundary determines which adapters are supported. Within that boundary, B.O.B. should prefer the least surprising allowed path rather than preserving a historical provider ranking.

General preference:

1. an explicit user-selected allowed adapter;
2. an enabled `free` path appropriate to the requested capability;
3. an enabled already-included `subscription` path;
4. an enabled `local` path;
5. an explicitly enabled `metered` path;
6. otherwise continue without inference and explain the available next step.

Provider/model changes that materially affect billing, privacy, locality, or user intent never happen silently.

## Current implementation and alpha boundary

The currently implemented cloud adapter is Gemini Developer API, retained as an **advanced optional** capability behind its accepted professional/business-use, unpaid-service data-use, credential, and billing boundaries.

RFC-0002 defines the accepted provider-independent runtime contract. The Rust-owned runtime policy and a bounded non-user-facing Ollama tracer are implemented on `master`.

Wayfinder #79 and resolved research #80/#81 establish the current direction:

- account-backed runtimes may be supported only when an official machine-consumable surface exposes enough auth, billing, capability, lifecycle, and failure truth;
- B.O.B.-native local inference should use a B.O.B.-owned `LocalRuntimeAdapter`, initially favoring an in-process Rust engine and GGUF support;
- Ollama and LM Studio may remain optional compatibility paths when their actual locality/billing behavior can be classified truthfully;
- no provider/runtime is mandatory merely because it is implemented or validated first.

The immediate first-alpha frontier is native recovery/package acceptance and convergence. Provider-surface expansion is deliberately sequenced after that frontier.

## Functional requirements

### Availability and readiness

B.O.B. must detect whether a configured runtime is available and report failure without blocking deterministic application use.

### Runtime identity

Runtime/model identity must be inspectable when useful for troubleshooting, privacy, cost, locality, or explicit user choice, but B.O.B. remains the conversational identity.

### Cost visibility

Billing class must be known to policy before invocation and exposed where material to the user decision.

### No silent metered fallback

A provider/runtime failure, quota exhaustion, allowance exhaustion, authentication failure, or local-runtime failure must not cause separately billed inference unless the user has explicitly enabled that path.

### Provider/privacy/locality stability

B.O.B. must not silently switch to a provider/model/runtime that materially changes privacy, locality, capability, or user intent.

### Runtime interface

Adapters normalize only the identity, readiness, auth, billing, locality/privacy, capability, lifecycle, invocation, cancellation, and failure behavior B.O.B. actually needs. Provider-specific tool ecosystems, peer-agent/session models, or broad configuration surfaces do not enter the common contract by convenience.

### Authentication ownership

B.O.B. may use official provider/runtime-owned authentication when it satisfies the supported contract. It must not scrape private sessions, reuse unsupported credentials, or infer entitlement from login state alone.

### Local inference

Local inference is optional. B.O.B. must not require a local model to start or use deterministic features.

### Deterministic fallback

If no allowed inference capability is available, B.O.B. continues to provide capture, task lifecycle, planning, replanning, persistence, recovery, and other deterministic behavior.

## Acceptance criteria

- B.O.B. remains the single user-facing agent regardless of runtime;
- every supported adapter reports a truthful billing class before invocation;
- locality/privacy is reported where materially relevant and ambiguous state fails closed;
- metered runtimes are disabled by default unless explicitly enabled;
- no failure path silently changes provider/model/runtime when cost, privacy, locality, or user intent would materially change;
- runtime failure produces an understandable state rather than a crash or canonical-state loss;
- deterministic B.O.B. remains useful with no inference configured;
- the provider-independent runtime seam can support additional adapters without changing canonical B.O.B. state or identity;
- runtime logs and UI state do not expose protected credentials;
- unsupported account-backed or local paths are not presented as implemented product controls.

## Non-goals

- fixed permanent vendor ranking;
- cheapest-model auto-routing as a first-alpha requirement;
- token-by-token budget optimization;
- user-facing multi-agent orchestration;
- reselling inference;
- bypassing vendor terms, billing, or authentication controls;
- reverse engineering unsupported proprietary application protocols;
- requiring a second runtime before the first runnable alpha can be accepted.
