# ADR-0006: Keep B.O.B. capabilities portable across host harnesses

**Status:** Accepted  
**Date:** 2026-08-23  
**Owner direction:** issue #109

## Context

B.O.B. is currently delivered as a Tauri 2 desktop application with a Rust privileged core. That product shape is valid, but several B.O.B. responsibilities are not inherently desktop-specific: deterministic task/work semantics, planning/replanning, proposal validation, bounded context/continuity transformations, and product-policy types can be useful independently of the Tauri host or any one inference/runtime implementation.

B.O.B. also needs to remain able to integrate with external harnesses and runtimes without making those systems architectural landlords. DeepSeek Harness is an immediate proving target because its plugin/service architecture demonstrates strong separation between capability definitions, providers, and consumers. QOR Agent demonstrates a complementary small-harness approach with explicit public extension seams. GG-CORE demonstrates a deliberately narrow inference-execution boundary.

The repository must preserve the existing product invariant that the standalone B.O.B. application owns its canonical work state, identity, authority, and deterministic behavior. Portability must not silently turn another harness session into B.O.B. canonical state or require changes to another repository.

## Decision

B.O.B. will evolve toward a **portable capability core + host adapters** architecture.

1. Harness-neutral B.O.B. behavior belongs behind B.O.B.-owned typed contracts rather than depending directly on Tauri, DeepSeek Harness, QOR Agent, Cloudflare, GG-CORE, or a specific inference provider.
2. The current Tauri desktop application remains a first-party B.O.B. host, not the only possible host.
3. First-party integrations for external harnesses live in the B.O.B. repository and adapt B.O.B.-owned contracts to the target harness's public extension surface.
4. External harness/runtime repositories remain independent. B.O.B. integration work must not require source changes in those repositories.
5. Harness-specific adapters may depend on their target harness API. The portable B.O.B. capability layer must not.
6. The first external-harness proving target is DeepSeek Harness. Because its current API is developer-preview and may break compatibility, DeepSeek-specific churn must remain isolated in the adapter boundary.
7. QOR Agent may be supported through its public model/tool/interceptor/session/observer/external-runtime seams when a concrete use case warrants it. Cloudflare is one possible QOR host, not the QOR Agent architecture.
8. GG-CORE may satisfy a B.O.B. inference/runtime port through its supported Rust or authenticated local IPC surfaces. GG-CORE does not acquire B.O.B. state, policy, tool, or product-identity authority.
9. This is **not** a decision to create a broad plugin marketplace, arbitrary dynamic native plugin loader, or universal extension framework.

## Identity and state consequences

The meaning of “B.O.B. is the agent and owns the work” is host-contextual rather than a claim that every process importing a B.O.B. capability becomes the standalone B.O.B. product.

- **Standalone B.O.B. host:** B.O.B. remains the user-facing agent and canonical state owner under ADR-0001 and ADR-0004.
- **Stateless capability embedding:** a host may consume bounded B.O.B. capability results while owning its surrounding session/lifecycle. This does not make host state B.O.B. canonical state.
- **Future stateful embedding:** state ownership, namespace, lifecycle, migration, synchronization, and authority must be explicitly specified before implementation. No adapter may silently create a second canonical B.O.B. authority.

ADR-0001 and ADR-0004 remain fully in force for the standalone product.

## Dependency direction

```text
                 B.O.B. portable capability contracts
        task/planning · proposals · context · policy types
                               ^
                               |
        +----------------------+----------------------+
        |                      |                      |
  Desktop/Tauri host     DeepSeek adapter       QOR adapter
        |                                             |
 SQLite · keyring                              public QOR seams
        |
  inference/runtime port
        |
 Gemini · Ollama · GG-CORE · other allowed runtime
```

Adapters point inward toward B.O.B.-owned definitions. Portable B.O.B. definitions do not point outward toward a concrete host implementation.

## Sequencing consequence

This decision authorizes the architecture direction but does not displace the current alpha stabilization frontier. Active native/rendered UX, recovery, packaging, exact-head validation, and provider-readiness work should converge first. Extraction should then be driven by the smallest real second-host requirement rather than a speculative rewrite.

The exact crate/package layout, cross-language bridge, compatibility policy, and DeepSeek adapter contract are defined by RFC-0004 before substantial implementation.

## Rejected alternatives

### Make DeepSeek Harness the new B.O.B. core

Rejected. It would transfer architectural ownership to a young external harness and make compatibility churn a B.O.B. core concern.

### Duplicate B.O.B. logic inside each host plugin

Rejected. It would create divergent planning/policy/proposal semantics and make fixes dependent on whichever language a host happens to use.

### Build a universal B.O.B. plugin marketplace first

Rejected. The current need is first-party composition through a small number of explicit adapters. Marketplace infrastructure is broader scope with no demonstrated requirement.

### Require changes in QOR Agent or GG-CORE

Rejected. B.O.B. should adapt to their supported public seams from inside this repository. Same-organization ownership is not a license to create cross-repository coupling for convenience.

## Consequences

### Positive

- B.O.B. can remain a coherent standalone product while its useful semantics become reusable.
- External harness churn is isolated at adapter boundaries.
- QOR Agent, GG-CORE, and future runtimes can be integration targets without becoming mandatory B.O.B. dependencies.
- Portable capability tests can pressure B.O.B. toward cleaner separation between deterministic behavior and desktop plumbing.

### Costs

- Some current Rust modules will eventually need extraction from the Tauri application crate.
- Cross-language hosts such as DeepSeek Harness require a deliberate bridge rather than importing Rust semantics directly into TypeScript.
- Compatibility testing becomes a first-party adapter responsibility.

## Validation expectations

Any portability extraction or adapter must prove:

- standalone B.O.B. behavior remains intact;
- portable contracts do not depend on Tauri or a concrete target harness;
- state/authority ownership is explicit;
- cancellation, cleanup, and typed failures cross the adapter boundary correctly;
- secrets are not moved into a host session or ordinary plugin state;
- removal of an adapter does not break unrelated standalone behavior;
- no external repository write is required to build or test the B.O.B.-owned adapter.
