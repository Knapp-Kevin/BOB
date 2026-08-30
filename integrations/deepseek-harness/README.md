# B.O.B. DeepSeek Harness host tracer

This integration proves the first alternate first-party host boundary for B.O.B.

> **B.O.B. is the agent. DeepSeek is the harness.**

The plugin is intentionally narrow. It registers one deterministic B.O.B. planning capability and delegates the planning semantics to the Rust-owned `bob-capability-host` over a bounded newline-delimited JSON protocol. It does not reimplement B.O.B. planning in JavaScript.

## Current compatibility target

This tracer is grounded against DeepSeek Harness:

- version: `0.1.2-alpha.2`
- source commit: `0a53fb55bea101816fa226bb964ae2bed71c343b`
- plugin model: Cordis function plugin with `tools` injection

DeepSeek Harness is currently developer preview and explicitly permits compatibility-breaking changes. Compatibility churn belongs here, not in `bob-core`.

## What this proves

- the DeepSeek adapter can invoke a B.O.B.-owned portable capability without making DeepSeek the B.O.B. semantic core;
- the JavaScript edge and real Rust sidecar round-trip through the same bounded protocol exercised in CI;
- the capability host can run without Tauri, SQLite, a provider client, credentials, or B.O.B. desktop state;
- the plugin sends only planning-relevant values to the sidecar;
- the sidecar starts with an explicit empty process environment rather than inheriting DeepSeek Harness environment variables;
- DeepSeek shell, filesystem, credential, job, subagent, and other tool authority is not inherited by B.O.B.;
- removing this integration leaves the B.O.B. desktop source and portable capability implementation intact.

This does **not** yet prove installation/unload compatibility inside a live DeepSeek Harness checkout. That remains a separate compatibility-hardening gate.

## Configuration

Build `crates/bob-capability-host`, then configure the plugin with an **absolute** path to that executable:

```yaml
- insert:
    - id: bob-host
      name: '/absolute/path/to/BOB/integrations/deepseek-harness/src/index.js'
      config:
        hostPath: '/absolute/path/to/bob-capability-host'
```

The adapter never evaluates `hostPath` through a shell. It launches exactly that executable with no command arguments and `shell: false`. The child process receives an empty environment, requests and responses are capped at 64 KiB, successful planning responses are type/bounds checked, and a planning invocation fails closed if the sidecar does not complete within 10 seconds.

For a local DeepSeek Harness checkout, load the overlay with the Harness-supported `--patch` mechanism. This tracer is not published to npm and does not claim general DeepSeek-version compatibility yet.

## Exposed capability

`bob_plan_remaining_work` accepts:

- optional current `activeId`;
- an array of planning items containing only `id`, `kind`, `priority`, optional `due`, and `status`.

The B.O.B. host returns `nextId` plus up to three ordered `focusIds`.

Titles, notes, estimates, handoff text, provider credentials, filesystem state, shell authority, DeepSeek environment variables, and the DeepSeek session transcript do not cross into the B.O.B. sidecar.

## B.O.T. guardrail

**B.O.T.** means **Bag of Tools**: an agent-like aggregation of capabilities without coherent identity, state, continuity, authority, policy, and user experience.

This plugin is not permission to expose every DeepSeek capability through B.O.B. The first tracer deliberately registers one capability. Future host work must preserve B.O.B.-owned authority and policy rather than turning B.O.B. into a B.O.T.

## Validation

From the B.O.B. repository:

```sh
cargo test --manifest-path crates/bob-core/Cargo.toml
cargo test --manifest-path crates/bob-capability-host/Cargo.toml
node --test integrations/deepseek-harness/tests/*.test.mjs
```

The Rust capability-host integration test launches the real sidecar process, writes one protocol request on stdin, and asserts the typed response from stdout. The Node edge test then invokes that built sidecar through `runPlanningRequest`, exercising the environment-isolated JavaScript-to-Rust process boundary directly.

## Deferred work

- package/install discovery for the Rust sidecar;
- live DeepSeek Harness install/load/unload compatibility evidence;
- stateful B.O.B. hosted mode and explicit cross-host state ownership;
- full B.O.B. identity/preset/UI composition in a DeepSeek profile;
- DeepSeek compatibility matrix beyond the pinned developer-preview target;
- N-API or WASM bridge alternatives, unless profiling demonstrates the stdio boundary is insufficient;
- Delegate/tool execution authority.
