# ADR-0006: Keep B.O.B. capabilities portable across host harnesses

**Status:** Accepted  
**Date:** 2026-08-23  
**Reconciled:** 2026-08-30  
**Owner direction:** issues #109 and #118

## Context

B.O.B. is currently delivered as a Tauri 2 desktop application with a Rust privileged core. That product shape is valid, but several B.O.B. responsibilities are not inherently desktop-specific: deterministic task/work semantics, planning/replanning, proposal validation, bounded context/continuity transformations, and product-policy types can be useful independently of the Tauri host or any one inference/runtime implementation.

B.O.B. also needs to remain able to integrate with external harnesses and runtimes without making those systems architectural landlords. DeepSeek Harness is the first proving target because its plugin/service architecture separates capability definitions, providers, and consumers and lets out-of-tree plugins compose into a host profile. QOR Agent demonstrates a complementary small-harness approach with explicit public extension seams. GG-CORE demonstrates a deliberately narrow inference-execution boundary.

The repository must preserve the product invariant that B.O.B. owns its agent identity, policy, authority, and deterministic semantics. Portability must not silently turn another harness session into B.O.B. canonical state or require changes to another repository.

## Decision

B.O.B. uses a **portable capability core + first-party host adapters** architecture.

1. Harness-neutral B.O.B. behavior belongs behind B.O.B.-owned typed contracts rather than depending directly on Tauri, DeepSeek Harness, QOR Agent, Cloudflare, GG-CORE, or a specific inference provider.
2. The current Tauri desktop application is the **B.O.B. Desktop Host**, not the only possible host.
3. DeepSeek Harness is the first alternate first-party host. The governing relationship is **B.O.B. is the agent; DeepSeek is the harness.**
4. First-party host integrations live in the B.O.B. repository and adapt B.O.B.-owned contracts to the target harness's public extension surface.
5. External harness/runtime repositories remain independent. B.O.B. integration work does not require source changes in those repositories.
6. Harness-specific adapters may depend on their target harness API. The portable B.O.B. capability layer must not.
7. DeepSeek-specific churn remains isolated in `integrations/deepseek-harness/` while DeepSeek Harness is developer preview.
8. QOR Agent may be supported through its public model/tool/interceptor/session/observer/external-runtime seams when a concrete use case warrants it. Cloudflare is one possible QOR host, not the QOR Agent architecture.
9. GG-CORE may satisfy a B.O.B. inference/runtime port through supported Rust or authenticated local IPC surfaces. GG-CORE does not acquire B.O.B. state, policy, tool, or product-identity authority.
10. This is **not** a decision to create a broad plugin marketplace, arbitrary dynamic native plugin loader, or universal extension framework.

## B.O.T. anti-pattern

**B.O.T.** means **Bag of Tools**: an agent-like aggregation of models, tools, plugins, runtimes, or integrations that does not own a coherent identity, state model, continuity model, authority boundary, policy layer, and user experience.

B.O.B. must never collapse into a B.O.T. A host may expose many capabilities, but B.O.B. decides which capabilities are part of its contract and which authority is granted.

A practical test is: if removing the installed tools/providers leaves no meaningful B.O.B. behavior or identity behind, the system has become a B.O.T., not B.O.B.

## Identity and state consequences

The meaning of “B.O.B. is the agent and owns the work” is host-contextual, but the host never acquires B.O.B. authority merely by loading an adapter.

- **B.O.B. Desktop Host:** B.O.B. remains the user-facing agent and canonical state owner under ADR-0001 and ADR-0004.
- **Stateless alternate-host mode:** a host supplies bounded request values and receives typed B.O.B. results. No second B.O.B. state store is created. This is the first DeepSeek tracer mode.
- **Future stateful B.O.B. hosted mode:** B.O.B. may present a fuller hosted experience through DeepSeek or another host, but canonical ownership, namespace, migration, synchronization, recovery, credential separation, and authority must be explicitly specified first. Host session state does not silently become B.O.B. canonical state.

ADR-0001 and ADR-0004 remain fully in force for the standalone desktop product.

## Dependency direction

```text
                         B.O.B. portable core
                     planning · policy · proposals
                                ^
                                |
                 +--------------+--------------+
                 |                             |
          B.O.B. Desktop Host          B.O.B. DeepSeek Host
             Tauri / Rust              plugin / profile edge
                 |                             |
          SQLite · keyring              Cordis host services
                 |
          inference/runtime port
                 |
        Gemini · Ollama · GG-CORE · other allowed runtime
```

Adapters point inward toward B.O.B.-owned definitions. Portable B.O.B. definitions do not point outward toward a concrete host implementation.

## First proving slice

Issue #118 implements the first bounded proof:

- `crates/bob-core` owns a narrow deterministic planning request/result contract and the planning projection implementation;
- the desktop planner maps canonical work into that narrow request and uses the same portable planning source;
- `crates/bob-capability-host` exposes protocol version 1 over stdio with bounded requests, request identity, validation, and typed results;
- `integrations/deepseek-harness` registers exactly one planning capability and launches the explicit B.O.B. host executable with shell evaluation disabled;
- no persistent alternate-host state, Delegate authority, provider expansion, or external repository write is introduced.

The first planning contract intentionally excludes title, estimate, handoff text, credentials, filesystem state, and host session transcripts because deterministic planning does not need them.

## Sequencing consequence

The original decision sequenced substantial portability implementation after alpha stabilization. On 2026-08-30 the repository owner explicitly authorized the bounded #118 tracer to proceed early, provided it remains disjoint from startup recovery PR #103 and Windows evidence PR #115 and does not weaken their acceptance gates.

This is a scoped sequencing override, not a cancellation of alpha qualification. Broader multi-host work still requires evidence from the tracer and must not opportunistically consume the recovery/packaging frontier.

RFC-0004 defines the concrete host protocol, compatibility boundary, and validation expectations.

## Rejected alternatives

### Make DeepSeek Harness the new B.O.B. core

Rejected. It would transfer architectural ownership to a young external harness and make compatibility churn a B.O.B. core concern.

### Treat B.O.B. as a DeepSeek Bag of Tools

Rejected. A list of model-facing tools is not a B.O.B. agent. B.O.B. retains identity, policy, authority, state rules, continuity rules, and deterministic semantics even when DeepSeek supplies the surrounding harness.

### Duplicate B.O.B. logic inside each host plugin

Rejected. It would create divergent planning/policy/proposal semantics and make fixes dependent on whichever language a host happens to use.

### Build a universal B.O.B. plugin marketplace first

Rejected. The current need is first-party composition through a small number of explicit adapters. Marketplace infrastructure is broader scope with no demonstrated requirement.

### Require changes in QOR Agent, DeepSeek Harness, or GG-CORE

Rejected. B.O.B. adapts to their supported public seams from inside this repository. Same-organization ownership is not a license to create cross-repository coupling for convenience.

## Consequences

### Positive

- B.O.B. can remain a coherent standalone product while its useful semantics become reusable.
- DeepSeek can become an alternate first-party B.O.B. host without becoming B.O.B.'s semantic owner.
- External harness churn is isolated at adapter boundaries.
- QOR Agent, GG-CORE, and future runtimes can be integration targets without becoming mandatory B.O.B. dependencies.
- Portable capability tests pressure B.O.B. toward cleaner separation between deterministic behavior and desktop plumbing.
- The B.O.T. term gives reviews a concise test against capability-aggregation drift.

### Costs

- Some current Rust modules require incremental extraction from the Tauri application crate.
- Cross-language hosts such as DeepSeek Harness require a deliberate bridge rather than importing Rust semantics directly into JavaScript/TypeScript.
- Compatibility testing becomes a first-party adapter responsibility.
- Stateful hosted B.O.B. remains a separate design problem and cannot be inferred from a stateless tracer.

## Validation expectations

Any portability extraction or host adapter must prove:

- standalone B.O.B. behavior remains intact;
- portable contracts do not depend on Tauri or a concrete target harness;
- state/authority ownership is explicit;
- host input is validated and host authority is narrowed rather than inherited;
- cancellation, cleanup, and typed failures cross the adapter boundary correctly;
- secrets are not moved into a host session or ordinary plugin state;
- removal of an adapter does not break unrelated standalone behavior;
- no external repository write is required to build or test the B.O.B.-owned adapter.
