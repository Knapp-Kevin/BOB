# RFC-0004: Portable B.O.B. capability and host-adapter contract

**Status:** Proposed  
**Date:** 2026-08-23  
**Related:** ADR-0006, issue #109

## Summary

Define the smallest architecture that lets B.O.B. remain a standalone Tauri/Rust product while selected harness-neutral B.O.B. capabilities can also be consumed from another agent harness through first-party adapters maintained in this repository.

DeepSeek Harness is the first proving target. QOR Agent and GG-CORE provide additional conformance pressure, but this RFC does not require changes to either repository.

## Goals

- isolate deterministic B.O.B. behavior from desktop-host plumbing where the behavior is genuinely portable;
- preserve standalone B.O.B. canonical-state, identity, authority, privacy, and cost boundaries;
- let external harness adapters depend on B.O.B.-owned capability definitions rather than duplicate B.O.B. semantics;
- keep harness-specific compatibility churn at the adapter edge;
- prove the design with one concrete second host before generalizing it;
- keep the extraction small enough that it does not become a framework rewrite in ceremonial clothing.

## Non-goals

- a public plugin marketplace;
- arbitrary runtime discovery or auto-loading;
- stable dynamic Rust ABI support;
- moving canonical B.O.B. state into a vendor harness session;
- implementing Delegate authority in this RFC;
- replacing QOR Agent, DeepSeek Harness, or GG-CORE;
- requiring source changes outside `Knapp-Kevin/BOB`;
- choosing a cross-language bridge before a tracer implementation compares the practical options.

## Proposed logical layers

### 1. Portable B.O.B. domain/capability layer

Candidate responsibilities are limited to behavior that can execute without a desktop window, SQLite implementation, OS keyring, network provider client, or target harness runtime:

- work-item/task domain operations;
- deterministic planning and replanning;
- proposal schemas and validation;
- bounded context/continuity transformations;
- provider-neutral/runtime-neutral policy and capability types;
- normalized result and failure types needed by these capabilities.

The extraction should be evidence-driven. A module does not become portable merely because moving it into a new crate makes the tree look architectural.

### 2. Ports

Portable capabilities may depend on narrow B.O.B.-owned traits/interfaces for host services that cannot remain pure. Initial candidates:

- `StatePort` or smaller domain-specific persistence ports;
- `InferencePort` for normalized inference/runtime use;
- `ObserverPort` for bounded progress/evidence events;
- `CancellationPort` or cancellation token abstraction;
- secret-reference access only where a portable capability truly needs it;
- a future `ExecutionPort` only after Delegate authority is accepted for the relevant use case.

Prefer several small role-specific ports over one `BobHost` object that quietly becomes an operating system.

### 3. Standalone desktop host

The existing B.O.B. application implements those ports with its current authorities:

- Tauri lifecycle and command boundary;
- Rust-owned SQLite canonical state;
- OS-backed secret storage;
- B.O.B. UI;
- desktop packaging/recovery;
- provider/runtime adapters such as Gemini and Ollama;
- future GG-CORE inference integration where accepted.

The desktop host remains independently buildable without DeepSeek Harness or QOR Agent dependencies.

### 4. External harness adapters

Each adapter is first-party B.O.B. code under a dedicated repository subtree, for example:

```text
integrations/
  deepseek-harness/
  qor-agent/
```

The exact path is implementation detail. The important rule is that target-harness dependencies terminate inside the adapter boundary.

## DeepSeek Harness design lesson

DeepSeek Harness currently organizes extensions around plugin-provided services and explicit dependency injection. Its package guidance separates **Service Definition**, **Service Provider**, and **Consumer**, and directs extension plugins to depend on service definitions rather than concrete providers.

B.O.B. should borrow that dependency principle without adopting Cordis as its internal architecture:

```text
B.O.B. capability definition
           ^
           |
B.O.B. implementation/provider
           ^
           |
standalone host or harness adapter consumer
```

DeepSeek-specific plugin registration, service injection, effects/lifecycle, commands, tool exposure, and version compatibility belong only in the DeepSeek adapter.

Because DeepSeek Harness is currently a developer preview and explicitly expects compatibility-breaking changes, the adapter should:

- pin the tested compatible DeepSeek package range;
- expose compatibility metadata;
- fail clearly on unsupported versions where practical;
- include contract/integration tests that exercise only the adapter boundary;
- avoid leaking Cordis service types into B.O.B. portable Rust APIs.

## DeepSeek cross-language bridge options

The authoritative B.O.B. capability implementation is expected to remain Rust-first. DeepSeek plugins are TypeScript/Cordis based, so a bridge is required.

### Option A: local stdio or local JSON-RPC sidecar

**Shape:** the DeepSeek plugin starts or connects to a B.O.B.-owned local capability process and exchanges versioned typed requests/events over stdio or a local authenticated transport.

**Advantages**

- language/runtime isolation;
- no Node native ABI packaging requirement;
- target-harness churn stays in TypeScript;
- Rust core can be tested independently;
- process lifecycle and cancellation are explicit.

**Costs**

- process packaging/startup complexity;
- protocol/versioning work;
- must prevent orphaned processes and ambiguous state ownership.

**Current recommendation:** first tracer candidate because it is replaceable and minimizes coupling while DeepSeek's API is young.

### Option B: Node native binding

**Shape:** expose the portable Rust capability layer through N-API or another maintained Node binding.

**Advantages**

- low-latency in-process calls;
- compact runtime topology.

**Costs**

- native packaging per platform/architecture;
- Node ABI/tooling complexity;
- tighter process privilege sharing;
- greater release burden before the integration is proven valuable.

**Current disposition:** compare after the sidecar tracer establishes real workload and performance requirements.

### Option C: WebAssembly/component boundary

**Shape:** compile suitable portable B.O.B. capabilities to WASM and load them from the adapter.

**Advantages**

- portable sandboxable artifact;
- clean capability boundary for pure logic.

**Costs**

- state, filesystem, secrets, async runtime, and native integration require additional host design;
- could distort current Rust code merely to satisfy a speculative deployment form.

**Current disposition:** future option, not the first proving route.

## Proposed DeepSeek capability surface

The first adapter should expose a deliberately small subset of B.O.B. capabilities rather than attempt to recreate the full desktop product.

A sensible tracer set is:

1. deterministic task decomposition/planning request;
2. proposal validation for a typed B.O.B. work mutation;
3. bounded reorientation/next-action assistance using host-supplied context;
4. optional inference only through a declared B.O.B. `InferencePort`, not through hidden provider ownership.

A plugin may surface those capabilities as DeepSeek services and/or model-facing tools according to the target harness's public conventions. The adapter must not silently grant filesystem, shell, credential, or repository authority.

## QOR Agent interaction

QOR Agent already defines small public extension roles. A B.O.B.-owned QOR adapter should map only the role required by the product use case:

- portable B.O.B. deterministic behavior as registered Tool/Capability implementations;
- B.O.B. policy narrowing as an Interceptor only when semantics align;
- observation/evidence through Observer;
- state/session adaptation only through an explicit SessionStore boundary;
- a richer B.O.B. host through ExternalRuntime if B.O.B. owns more lifecycle than a plain model request.

Do not make B.O.B. portable core depend on QOR Agent. The adapter depends on both public contracts and translates between them.

Cloudflare remains merely one possible QOR host/proving environment.

## GG-CORE interaction

GG-CORE belongs below B.O.B.'s inference/runtime port, not beside B.O.B. as another application state owner.

Supported integration shapes may include:

- direct Rust embedding through GG-CORE's secure runtime facade; or
- authenticated local IPC when process isolation is preferred.

B.O.B. remains responsible for:

- deciding whether local inference is allowed;
- selecting the runtime/model according to B.O.B. policy;
- canonical work/continuity state;
- proposal validation;
- tool/delegation authority;
- presenting results to the user/host.

GG-CORE remains responsible for contained model execution, its own runtime security boundary, and normalized inference output.

## State model in external hosts

### Stateless capability mode

Preferred first mode. The host sends a bounded request and context; B.O.B. returns typed output. No persistent B.O.B. state is created by the adapter unless explicitly requested by a later accepted contract.

### Stateful capability mode

Deferred. Before implementation it must specify:

- canonical owner;
- namespace and identity;
- storage location;
- migration/versioning;
- synchronization/conflict behavior if more than one host touches the same logical work;
- deletion/export/recovery semantics;
- credential separation;
- whether B.O.B. identity is actually being presented or only B.O.B. capabilities are embedded.

No adapter may infer these answers from the host harness's session store.

## Compatibility contract

The portable B.O.B. capability API and each external adapter version independently.

Conceptually:

```text
bob-capability-protocol: 0.x
bob-core:                 0.x
bob-deepseek-adapter:     0.x
supported-deepseek-api:   explicit tested range
```

A target harness breaking its plugin API should require an adapter release, not a portable B.O.B. core rewrite.

## Security and authority

- external-host input is untrusted;
- model output remains untrusted;
- a host's ability to call a B.O.B. capability is not equivalent to B.O.B. Delegate authority;
- portable capabilities receive the minimum data needed for the request;
- secrets do not cross the bridge unless a specific port contract requires a reference and accepted authority permits it;
- adapters clean up processes/listeners/effects on unload or cancellation;
- process bridges use bounded message sizes, version negotiation, timeouts, and explicit shutdown;
- no adapter auto-discovers and executes arbitrary plugins merely because they exist on disk.

## Proposed repository evolution

Do not perform all of this as one migration. The intended sequence is:

### Pass P0: boundary audit

Classify current Rust modules as portable domain, host service, adapter, or presentation/desktop lifecycle. No code movement yet.

### Pass P1: extract one pure vertical slice

Create the minimum Rust crate/module boundary needed to run one deterministic B.O.B. capability without Tauri. Preserve current behavior with direct tests.

### Pass P2: formalize ports and conformance fixtures

Add only the ports required by the extracted slice and define versioned request/result fixtures.

### Pass P3: keep desktop host green

Move the existing desktop path onto the same portable contract and prove there is no standalone regression.

### Pass P4: DeepSeek tracer adapter

Build the thin plugin plus selected bridge, expose one or two B.O.B. capabilities, and validate lifecycle/cancellation/error behavior.

### Pass P5: harden compatibility and packaging

Pin/test DeepSeek compatibility, document installation, add removal/failure cases, and ensure the adapter can be omitted without affecting standalone builds.

Only after these passes should additional B.O.B. capabilities or other harness adapters be considered.

## Acceptance criteria for this RFC

Before moving from Proposed to Accepted, implementation planning should confirm:

- the first portable capability slice;
- the first DeepSeek-facing capability/service surface;
- the cross-language tracer mechanism;
- portable API versioning shape;
- standalone desktop dependency direction;
- state mode for the first adapter, expected to be stateless;
- exact validation matrix and supported DeepSeek version range.

## Validation expectations after acceptance

A completed first DeepSeek integration should demonstrate:

1. portable B.O.B. tests pass without Tauri or DeepSeek installed;
2. standalone desktop tests/builds continue to pass;
3. DeepSeek adapter installs against the pinned supported harness version;
4. the adapter can be enabled/disabled cleanly;
5. one deterministic B.O.B. capability works end to end;
6. cancellation/unload terminates adapter-owned work;
7. malformed/oversized/unsupported protocol requests fail closed;
8. no B.O.B. canonical desktop state or secrets are silently imported into the host harness;
9. removing the adapter dependency leaves standalone B.O.B. unaffected;
10. no external repository write is part of the integration procedure.
