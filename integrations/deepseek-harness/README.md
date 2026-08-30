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

- a DeepSeek plugin can call a B.O.B.-owned portable capability without making DeepSeek the B.O.B. semantic core;
- the capability host can run without Tauri, SQLite, a provider client, credentials, or B.O.B. desktop state;
- the plugin receives only planning-relevant values;
- DeepSeek shell, filesystem, credential, job, subagent, and other tool authority is not inherited by B.O.B.;
- removing this integration leaves the B.O.B. desktop source and portable capability implementation intact.

## Configuration

Build `crates/bob-capability-host`, then configure the plugin with an **absolute** path to that executable:

```yaml
- insert:
    - id: bob-host
      name: '/absolute/path/to/BOB/integrations/deepseek-harness/src/index.js'
      config:
        hostPath: '/absolute/path/to/bob-capability-host'
```

The adapter never evaluates `hostPath` through a shell. It launches exactly that executable with no command arguments and `shell: false`.

For a local DeepSeek Harness checkout, load the overlay with the Harness-supported `--patch` mechanism. This tracer is not published to npm and does not claim general DeepSeek-version compatibility yet.

## Exposed capability

`bob_plan_remaining_work` accepts:

- optional current `activeId`;
- an array of planning items containing only `id`, `kind`, `priority`, optional `due`, and `status`.

The B.O.B. host returns `nextId` plus up to three ordered `focusIds`.

Titles, notes, estimates, handoff text, provider credentials, filesystem state, shell authority, and the DeepSeek session transcript do not cross this tracer boundary.

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

The Rust capability-host integration test launches the real sidecar process, writes one protocol request on stdin, and asserts the typed response from stdout.

## Deferred work

- package/install discovery for the Rust sidecar;
- stateful B.O.B. hosted mode and explicit cross-host state ownership;
- full B.O.B. identity/preset/UI composition in a DeepSeek profile;
- DeepSeek compatibility matrix beyond the pinned developer-preview target;
- N-API or WASM bridge alternatives, unless profiling demonstrates the stdio boundary is insufficient;
- Delegate/tool execution authority.
