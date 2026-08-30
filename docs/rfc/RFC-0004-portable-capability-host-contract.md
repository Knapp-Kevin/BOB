# RFC-0004: Portable B.O.B. capability and host-adapter contract

**Status:** Accepted  
**Proposed:** 2026-08-23  
**Accepted:** 2026-08-30  
**Related:** ADR-0006, issues #109 and #118

## Summary

Define the smallest architecture that lets B.O.B. remain a standalone Tauri/Rust product while the same B.O.B.-owned portable semantics can also run behind alternate first-party hosts.

DeepSeek Harness is the first proving host. The target relationship is explicit:

> **B.O.B. is the agent. DeepSeek is the harness.**

DeepSeek does not become the B.O.B. semantic core, and B.O.B. does not become a collection of DeepSeek tools. A thin DeepSeek/Cordis edge adapts the host to B.O.B.-owned portable contracts. QOR Agent and GG-CORE provide additional conformance pressure, but this RFC does not require changes to either repository.

## Goals

- isolate deterministic B.O.B. behavior from desktop-host plumbing where the behavior is genuinely portable;
- preserve B.O.B.-owned identity, state, authority, privacy, continuity, and cost-policy boundaries across hosts;
- let host adapters depend on B.O.B.-owned capability definitions rather than duplicate B.O.B. semantics;
- keep harness-specific compatibility churn at the adapter edge;
- prove the design with one concrete alternate host before generalizing it;
- keep the extraction small enough that it does not become a framework rewrite in ceremonial clothing.

## Non-goals

- a public plugin marketplace;
- arbitrary runtime discovery or auto-loading;
- stable dynamic Rust ABI support;
- moving canonical B.O.B. state into a vendor harness session;
- implementing Delegate authority in this RFC;
- replacing QOR Agent, DeepSeek Harness, or GG-CORE;
- requiring source changes outside `Knapp-Kevin/BOB`;
- treating every capability exposed by a host as B.O.B. authority;
- choosing N-API or WASM before the stdio tracer creates evidence that they are needed.

## B.O.T. anti-pattern

**B.O.T.** means **Bag of Tools**: an agent-like aggregation of models, tools, plugins, runtimes, or integrations without coherent ownership of identity, state, continuity, authority, policy, and user experience.

B.O.B. must not collapse into a B.O.T. A host may offer many capabilities, but B.O.B. remains responsible for deciding which capabilities belong inside its contract and which authority is actually granted.

A useful architectural test is:

> If removing the installed tools/providers leaves no meaningful B.O.B. behavior or identity behind, the system has become a B.O.T., not B.O.B.

## Logical layers

### 1. Portable B.O.B. domain/capability layer

Responsibilities are limited to behavior that can execute without a desktop window, SQLite implementation, OS keyring, network provider client, or target harness runtime:

- work-item/task domain operations where genuinely host-neutral;
- deterministic planning and replanning;
- proposal schemas and validation;
- bounded context/continuity transformations;
- provider-neutral/runtime-neutral policy and capability types;
- normalized result and failure types required by those capabilities.

The extraction is evidence-driven. A module does not become portable merely because moving it into a new crate makes the tree look architectural.

The first implemented slice is narrower than canonical work state: deterministic planning consumes only `activeId` plus planning items containing `id`, `kind`, `priority`, `due`, and `status`. Titles, estimates, handoff text, credentials, and persistence metadata are deliberately excluded.

### 2. Ports

Portable capabilities may depend on narrow B.O.B.-owned traits/interfaces for host services that cannot remain pure. Candidate ports include:

- `StatePort` or smaller domain-specific persistence ports;
- `InferencePort` for normalized inference/runtime use;
- `ObserverPort` for bounded progress/evidence events;
- `CancellationPort` or cancellation token abstraction;
- secret-reference access only where a portable capability truly needs it;
- a future `ExecutionPort` only after Delegate authority is accepted for the relevant use case.

Prefer several small role-specific ports over one `BobHost` object that quietly becomes an operating system.

### 3. B.O.B. Desktop Host

The existing Tauri application is the first-party desktop host and retains its current authorities:

- Tauri lifecycle and command boundary;
- Rust-owned SQLite canonical state;
- OS-backed secret storage;
- B.O.B. UI;
- desktop packaging/recovery;
- provider/runtime adapters such as Gemini and Ollama;
- future GG-CORE inference integration where accepted.

The desktop host remains independently buildable without DeepSeek Harness or QOR Agent dependencies.

### 4. B.O.B. DeepSeek Host

DeepSeek Harness is the first alternate first-party host. DeepSeek-specific code lives under `integrations/deepseek-harness/` and terminates all Cordis/DeepSeek types at that boundary.

The first tracer is deliberately stateless and exposes one deterministic B.O.B. planning capability. It proves the dependency direction before introducing hosted state, richer identity composition, or execution authority.

A later **stateful B.O.B. hosted mode** may present a fuller B.O.B. experience through DeepSeek, but it must keep B.O.B.-owned canonical state and explicitly define synchronization, migration, identity, recovery, and authority. DeepSeek session state does not become canonical B.O.B. state by implication.

### 5. Other external host adapters

Additional first-party adapters may live in dedicated repository subtrees, for example:

```text
integrations/
  deepseek-harness/
  qor-agent/
```

The important rule is that target-harness dependencies terminate inside the adapter boundary.

## DeepSeek Harness design lesson

DeepSeek Harness organizes extensions around plugin-provided services and explicit dependency injection. Its package guidance separates **Service Definition**, **Service Provider**, and **Consumer**, and directs extension plugins to depend on service definitions rather than concrete providers.

B.O.B. borrows that dependency principle without adopting Cordis as its internal architecture:

```text
B.O.B. capability definition
           ^
           |
B.O.B. implementation/provider
           ^
           |
B.O.B. desktop host or alternate host adapter
```

DeepSeek-specific plugin registration, service injection, effects/lifecycle, commands, tool exposure, and version compatibility belong only in the DeepSeek adapter.

Because DeepSeek Harness is currently developer preview and explicitly expects compatibility-breaking changes, the adapter must:

- pin/document the tested compatible DeepSeek target;
- expose compatibility metadata;
- fail clearly on unsupported configuration where practical;
- include contract/integration tests that exercise the adapter boundary;
- avoid leaking Cordis service types into B.O.B. portable Rust APIs.

The first tracer is grounded against DeepSeek Harness `0.1.2-alpha.2` at source commit `0a53fb55bea101816fa226bb964ae2bed71c343b`. That is a compatibility target, not a promise of future-version compatibility.

## DeepSeek cross-language bridge

The authoritative B.O.B. capability implementation remains Rust-first. DeepSeek plugins are TypeScript/JavaScript/Cordis based, so a bridge is required.

### Accepted first tracer: local stdio JSON protocol

The DeepSeek plugin starts a B.O.B.-owned local capability process and exchanges one versioned JSON request/response per invocation over stdio.

Advantages:

- language/runtime isolation;
- no Node native ABI packaging requirement;
- target-harness churn stays at the JavaScript edge;
- Rust capability code tests independently;
- process lifecycle and cancellation are explicit;
- the transport can be removed later without changing B.O.B. semantics.

Costs:

- process packaging/startup complexity;
- protocol/versioning work;
- process cleanup must be explicit;
- state ownership must remain unambiguous.

The first plugin accepts an explicit absolute executable path and launches it directly with shell evaluation disabled. It does not accept an arbitrary shell command or auto-discover executables.

### Deferred option: Node native binding

N-API or another maintained binding may be evaluated only after the stdio tracer establishes real workload or latency requirements. Native packaging per platform/architecture and tighter process privilege sharing are not justified yet.

### Deferred option: WebAssembly/component boundary

WASM remains a future option for suitable pure capabilities. It must not distort Rust code or invent state/filesystem/secret host semantics merely to satisfy a speculative deployment form.

## First DeepSeek capability surface

The accepted tracer exposes exactly one model-facing capability:

1. `bob_plan_remaining_work` maps bounded DeepSeek tool input to B.O.B.'s portable deterministic planning request and returns `nextId` plus the bounded `focusIds` set.

The adapter does not duplicate planning rules in JavaScript. It does not expose or inherit filesystem, shell, credential, job, subagent, repository, or arbitrary host-tool authority.

Additional B.O.B. capabilities require their own evidence and must preserve the B.O.T. guardrail.

## QOR Agent interaction

QOR Agent defines small public extension roles. A B.O.B.-owned QOR adapter should map only the role required by the product use case:

- portable B.O.B. deterministic behavior as registered Tool/Capability implementations;
- B.O.B. policy narrowing as an Interceptor only when semantics align;
- observation/evidence through Observer;
- state/session adaptation only through an explicit SessionStore boundary;
- a richer B.O.B. host through ExternalRuntime if B.O.B. owns more lifecycle than a plain model request.

Do not make B.O.B. portable core depend on QOR Agent. The adapter depends on both public contracts and translates between them.

Cloudflare remains merely one possible QOR host/proving environment.

## GG-CORE interaction

GG-CORE belongs below B.O.B.'s inference/runtime port, not beside B.O.B. as another application state owner.

Supported integration shapes may include direct Rust embedding through GG-CORE's secure runtime facade or authenticated local IPC when process isolation is preferred.

B.O.B. remains responsible for deciding whether local inference is allowed, selecting the runtime/model, canonical work/continuity state, proposal validation, tool/delegation authority, and presenting results to the user/host.

GG-CORE remains responsible for contained model execution, its own runtime security boundary, and normalized inference output.

## State model across hosts

### Stateless capability mode

This is the accepted first mode. The host sends a bounded request; B.O.B. returns typed output. No persistent B.O.B. state is created by the adapter.

### Stateful B.O.B. hosted mode

Deferred. Before implementation it must specify:

- canonical owner;
- namespace and identity;
- storage location;
- migration/versioning;
- synchronization/conflict behavior if more than one host touches the same logical work;
- deletion/export/recovery semantics;
- credential separation;
- how the B.O.B. identity and product policy are presented through the host.

No adapter may infer these answers from the host harness's session store.

## Compatibility contract

The portable B.O.B. capability API and each alternate-host adapter version independently.

Conceptually:

```text
bob-capability-protocol: 1.x
bob-core:                 0.x
bob-deepseek-host:        0.x
supported-deepseek-api:   explicit tested target/range
```

A target harness breaking its plugin API should require an adapter release, not a portable B.O.B. core rewrite.

## Security and authority

- external-host input is untrusted;
- model output remains untrusted;
- a host's ability to call a B.O.B. capability is not equivalent to B.O.B. Delegate authority;
- portable capabilities receive the minimum data needed for the request;
- secrets do not cross the bridge unless a specific port contract requires a reference and accepted authority permits it;
- adapters clean up processes/listeners/effects on unload or cancellation;
- process bridges use bounded message sizes, protocol-version checks, request identity, and explicit process shutdown;
- adapter executable paths are explicit and are never evaluated through a shell;
- no adapter auto-discovers and executes arbitrary plugins merely because they exist on disk.

## Repository evolution

The migration remains incremental:

### P0: boundary audit — complete for the first tracer

The deterministic planner was selected because its ordering logic is pure and depends on a very small planning projection of canonical work.

### P1: extract one pure vertical slice — implemented in #118

`crates/bob-core` owns the planning request/result types and deterministic projection function. It has no Tauri, SQLite, credential, network-provider, or DeepSeek dependency.

### P2: protocol/conformance fixture — implemented in #118

`crates/bob-capability-host` owns protocol version 1 and a process-level stdio test. External input is validated before planning.

### P3: keep desktop host on the same semantic source — implemented in #118

The Tauri planner maps canonical work to the narrow portable planning request and compiles the same portable planning source rather than carrying a duplicate algorithm.

The tracer uses a source-module inclusion from the portable crate to avoid modifying the already-locked desktop dependency graph during active alpha qualification. A later packaging-hardening pass may convert this to a normal Cargo path dependency once the workspace/lockfile change can be validated without broadening the tracer.

### P4: DeepSeek tracer adapter — implemented in #118

`integrations/deepseek-harness` registers one planning tool and delegates to the Rust capability host. Compatibility is intentionally narrow and non-published while the Harness API remains developer preview.

### P5: compatibility and packaging hardening — pending

Before calling the integration generally installable, validate installation/unload against the pinned DeepSeek target, package/discover the Rust sidecar safely, add negative lifecycle cases, and ensure the adapter can be omitted without affecting standalone builds.

Additional B.O.B. capabilities or host adapters should wait until this tracer creates evidence that they are needed.

## Validation expectations

The first DeepSeek-host integration must demonstrate:

1. portable B.O.B. tests pass without Tauri or DeepSeek installed;
2. standalone desktop tests/builds continue to pass;
3. the capability host executes one deterministic B.O.B. planning request end to end over stdio;
4. malformed/oversized/unsupported protocol requests fail closed;
5. the DeepSeek edge builds only bounded planning envelopes and validates response identity/version;
6. cancellation terminates adapter-owned process work;
7. no B.O.B. canonical desktop state or secrets are silently imported into the host harness;
8. removing the adapter leaves standalone B.O.B. unaffected;
9. no external repository write is part of the integration procedure.

Pinned DeepSeek installation/unload validation is P5 release-hardening evidence and must not be claimed until actually executed.
