# RFC-0002: Runtime Adapter Protocol

**Status:** Accepted  
**Related:** PRD-0001, PRD-0003, ADR-0001, ADR-0005, Wayfinder #34, #35, #79, #80, #81  
**Historical context:** ADR-0003 was rejected after the first-alpha inference route changed.

## Proposal

B.O.B. owns one small internal runtime contract that lets the single B.O.B. agent use different inference capabilities without changing canonical product state, user-facing identity, authority, or cost policy.

Execution tools remain behind a separate bounded tool gateway. A runtime may itself be capable of tools, shell, filesystem, MCP, or agent sessions, but those capabilities are not inherited through the inference adapter unless later authority explicitly grants them.

## Product invariant

> **The adapter selects capability, not identity. The user is always interacting with B.O.B.**

Removing any one adapter must leave B.O.B. buildable, launchable, state-safe, and usefully deterministic.

## Authority and current applicability

The first runnable alpha used Gemini Developer API Free to prove the inference, credential, privacy, billing, and failure-policy seams. That adapter remains an advanced optional capability, not B.O.B.'s permanent runtime identity.

Wayfinder #34 is accepted. B.O.B. owns orchestration, bounded context assembly, routing/policy, response and proposal validation, deterministic-service coordination, compact continuity, and failure handling.

Wayfinder #79 is the current provider-independence destination. Resolved research #80 and #81 establishes the current supported direction:

- account-backed external runtimes may be added when they expose an official machine-consumable surface and truthful auth/billing state;
- B.O.B.-native local inference should use a Rust-owned adapter seam, initially with an in-process Rust engine and GGUF as the first intentionally supported local model-package format;
- Ollama and LM Studio may be supported as optional compatibility adapters, but localhost alone does not prove local execution, privacy class, or billing class;
- provider/runtime-specific tools, sessions, model-management ecosystems, MCPs, and arbitrary sampling knobs do not belong in the common contract.

## Normalized runtime contract

Normalize only information B.O.B. needs to route safely and explain behavior truthfully.

### Runtime identity and health

Every adapter reports:

- stable runtime ID;
- runtime kind/version when observable;
- availability/health;
- normalized failure reason when unavailable.

### Authentication

Every adapter reports:

- authentication mechanism, such as `none`, `api_key`, `account_session`, or `runtime_token`;
- authentication state: `not_required`, `ready`, `missing`, `expired`, `invalid`, or `unknown`.

Authentication mechanism does **not** imply billing class.

### Billing and locality

Every adapter reports one billing class:

- `free`;
- `subscription`;
- `local`;
- `metered`;
- `unknown`.

Every adapter also reports one locality/privacy class:

- `on_device`;
- `loopback_local`;
- `lan_remote`;
- `cloud`;
- `unknown`.

Unknown billing classification fails closed. A loopback endpoint does not prove `local` billing or on-device privacy. Provider/model changes that materially affect billing, privacy, or user intent never happen silently.

### Model and capability state

Where supported, an adapter reports:

- stable model ID and user-facing display name;
- proven capabilities required by B.O.B., such as text generation, structured output, streaming, or cancellation;
- context limit when reliably observable;
- load/readiness state;
- bounded resource metadata when useful for local inference, such as approximate memory footprint or accelerator class.

Do not infer capabilities from provider brand or model name when the runtime can report them directly.

### Invocation lifecycle

The shared lifecycle is intentionally small:

```rust
trait RuntimeAdapter {
    fn identity(&self) -> RuntimeIdentity;
    async fn status(&self) -> Result<RuntimeStatus, RuntimeError>;
    async fn invoke(&self, request: InferenceRequest) -> Result<InferenceResult, RuntimeError>;
    async fn cancel(&self, operation: OperationId) -> Result<(), RuntimeError>;
}
```

Streaming may be exposed through a narrow optional capability where the adapter can support it truthfully. The exact Rust types are implementation details; the semantics above are normative.

## Inference request boundary

A request may contain only B.O.B.-owned, policy-approved inference inputs:

- operation ID;
- current B.O.B. authority mode;
- user instruction;
- bounded B.O.B. context package;
- requested structured-output schema when required;
- required inference capabilities;
- timeout/cancellation metadata;
- cost-policy snapshot;
- privacy/locality constraints.

Ordinary Assist requests do not carry arbitrary shell commands, workspace grants, filesystem paths, provider credentials, MCP/tool definitions, or vendor-session authority.

## Inference result boundary

A normalized result contains:

- operation ID;
- runtime/provider and model identity where useful;
- generated output;
- structured proposed B.O.B. actions when present;
- safe execution metadata;
- terminal status;
- normalized failure classification.

The result does not become a separate agent conversation or canonical state. B.O.B. validates it, integrates useful output into B.O.B.'s own response/continuity, and previews important proposed state changes before application.

## Proposed B.O.B. actions

Model output is untrusted. State-changing proposals use an allowlisted typed schema. The Rust core verifies schema, current state, business constraints, authority, and required confirmation before applying any proposal.

## Runtime-specific boundaries

### Gemini Developer API

The currently implemented API-key adapter remains an advanced optional path. It must preserve the accepted professional/business-use, unpaid-service data-use, Free-Tier confirmation, secret-storage, and fail-closed billing boundaries.

### Account-backed external runtimes

Claude Code, Codex, Antigravity, or another future runtime may be supported only through an official machine-consumable surface with observable enough auth, billing, failure, and cancellation behavior to satisfy this RFC.

B.O.B. does not reuse private credentials, scrape proprietary sessions, or make any one vendor client mandatory.

Owner decision (2026-11-02): B.O.B. is API-only and does not drive vendor agent CLIs. Current Anthropic and OpenAI terms do not clearly allow a third-party app to drive a user's subscription through Claude Code or Codex, and those CLIs are separate agents rather than model capabilities. See `docs/governance/AI_COST_AND_PROVIDER_POLICY.md`. Reopening this needs an explicit vendor allowance and a new owner decision.

### OpenRouter free-only mode

In free-only mode the OpenRouter adapter reports billing `free`, sends `provider.max_price` all zero, and treats a non-zero or missing `usage.cost` as a failure that revokes the session. Free-only mode is the default. For paid mode, readiness includes a `GET /api/v1/key` check that the key carries a credit limit with credit left, and a configured B.O.B. monthly spending limit with headroom.

### Spend accounting

Metered adapters reserve a worst-case cost against a B.O.B. monthly limit before invocation and settle the actual cost from the provider's reported usage afterwards (`src-tauri/src/spend.rs`, `spend_ledger` in schema v7). A provider rejection releases the reservation; an uncertain outcome keeps it.

### Direct vendor APIs

OpenAI (`chat/completions`) and Anthropic (Messages API, `x-api-key` and `anthropic-version: 2023-06-01`) adapters use API keys in the OS credential store, report `metered` billing and `cloud` locality, and share the OpenRouter transport bounds. They are reached only through fixed vendor endpoints.

### B.O.B.-native local runtime

The first-party local seam is a B.O.B.-owned `LocalRuntimeAdapter`. Current #81 research supports an in-process Rust implementation initially backed by `mistralrs`, with GGUF as the first intentionally supported package format.

The underlying engine's tool use, shell, filesystem, web search, MCP, or agent-session features remain disabled/outside the adapter. Run inference off the UI thread. Preserve a platform-neutral adapter contract so an isolated worker or different engine can replace the implementation later without changing B.O.B. product state.

### Ollama compatibility

An Ollama adapter may use the official local API for version, model inventory, running-model state, metadata, chat, streaming, and cancellation where supported.

Because Ollama localhost can also route to cloud models, B.O.B. must classify each selected model/runtime route explicitly. Cloud or ambiguous routes remain `unknown` until governed and therefore fail closed for no-surprise billing/locality policy.

### LM Studio compatibility

An LM Studio adapter may use its supported local/v1 or OpenAI-compatible API and its API-token auth when configured.

B.O.B. must distinguish on-device execution from LAN/remote/cloud routes and must not inherit LM Studio MCP/tool authority through the inference adapter.

## Error model

Normalize at least:

- unavailable;
- unauthenticated;
- allowance or quota exhausted;
- billing class unknown;
- privacy/locality policy blocked;
- timeout;
- cancelled;
- invalid response;
- unsupported capability;
- execution failure.

Do not collapse actionable failures into a generic runtime error.

## Security requirements

- sanitize logs and returned metadata;
- keep raw credentials inside provider/runtime-specific protected boundaries;
- no arbitrary shell or filesystem authority through inference adapters;
- validate structured proposals before canonical-state mutation;
- fail closed for unknown billing or materially ambiguous locality/privacy class;
- preserve B.O.B. as the only canonical conversation identity;
- do not inherit external runtime tool/MCP/session authority by convenience;
- do not silently change provider, runtime, model, billing class, or locality when that materially changes user intent, privacy, or cost.

## Deliberately excluded from the common contract

Do not normalize merely because providers expose it:

- full model-management ecosystems;
- provider-specific agent/session abstractions;
- MCP servers or tool catalogs;
- shell/filesystem/workspace authority;
- every provider sampling parameter;
- provider-specific billing products;
- mobile/cloud-sync semantics;
- peer-agent identities.

Add common fields only after at least one B.O.B. product behavior actually requires them.

## Acceptance criteria

This RFC is satisfied when each supported adapter can report truthful identity, auth, billing, locality/privacy, capabilities, readiness, invocation/cancellation behavior, and normalized failures through the B.O.B.-owned seam without owning canonical state or user identity.

Adding a new adapter must not require rewriting B.O.B.'s task/planning state, conversation identity, authority model, or UI around that provider. Unsupported or unknown billing/privacy/capability state must fail closed rather than silently selecting a materially different route.
