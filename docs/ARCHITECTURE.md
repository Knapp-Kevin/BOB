# Target Architecture

**Status:** Accepted  
**Related RFCs:** RFC-0001, RFC-0002, RFC-0003; RFC-0004 (Proposed)  
**Related decisions:** ADR-0006

## Architectural objective

Build the smallest standalone desktop architecture that presents one coherent B.O.B. agent, safely owns local personal state, and can draw on multiple inference runtimes and tools over time without exposing backend complexity as the user's interaction model. Keep genuinely harness-neutral B.O.B. behavior portable behind B.O.B.-owned typed contracts so the desktop host is a first-party product surface rather than an accidental permanent container for every B.O.B. capability.

The current desktop architecture is a Tauri 2 shell with a Rust application core and a framework-free TypeScript + Vite frontend. Windows 11 x64 is the primary supported platform for the current runnable waypoint.

The first-alpha Gemini Developer API Free path proved the inference, credential, privacy, billing, and failure-policy seams. It remains an advanced optional adapter. Current Wayfinder direction is provider-independent: future account-backed and local paths must fit behind B.O.B.-owned routing, state, continuity, authority, and cost policy rather than redefining the application around any provider.

## Core invariant

> **B.O.B. is the agent. Models, inference runtimes, provider APIs/CLIs, and tools are capabilities behind B.O.B.**

The user has one point of contact in the standalone product: B.O.B. Removing any single inference adapter must leave B.O.B. buildable, launchable, state-safe, and usefully deterministic.

ADR-0006 clarifies the deployment consequence: the standalone product invariant does not require harness-neutral B.O.B. capabilities to remain physically coupled to the Tauri desktop host. Another host may consume bounded B.O.B. capabilities without silently becoming the canonical B.O.B. product or state owner.

## Host portability boundary

B.O.B. uses a **portable capability core + host adapters** direction.

```mermaid
flowchart TB
    PORTABLE[B.O.B. portable capability contracts\nplanning · proposals · context · policy types]

    DESKTOP[B.O.B. Desktop Host\nTauri + Rust]
    DEEPSEEK[Future B.O.B.-owned\nDeepSeek Harness adapter]
    QOR[Future B.O.B.-owned\nQOR Agent adapter]

    DESKTOP --> PORTABLE
    DEEPSEEK --> PORTABLE
    QOR --> PORTABLE

    DESKTOP --> STATE[(SQLite canonical state)]
    DESKTOP --> SECRET[OS secret store]
    DESKTOP --> INFER[Inference/runtime port]

    INFER --> GEMINI[Gemini API]
    INFER --> OLLAMA[Ollama compatibility]
    INFER --> GG[GG-CORE\nRust or authenticated local IPC]
```

Dependency rules:

- harness-neutral B.O.B. definitions do not depend on Tauri, DeepSeek Harness, QOR Agent, Cloudflare, GG-CORE, or a concrete inference provider;
- the desktop host and external-harness adapters depend inward on B.O.B.-owned definitions;
- external-harness dependencies terminate inside their B.O.B.-owned adapter boundary;
- the first DeepSeek integration is a proving adapter, not a decision to make DeepSeek Harness part of B.O.B. core;
- QOR Agent integration uses its public extension seams when a concrete use case warrants it; Cloudflare is one possible QOR host/proving surface, not the definition of the QOR Agent harness;
- GG-CORE sits below B.O.B.'s inference/runtime port and retains only its inference-execution responsibilities; B.O.B. retains state, authority, proposal, tool, privacy, and cost decisions;
- portability work for these integrations is implemented in this repository and does not require source changes in the target repositories.

The first external-harness mode should be stateless unless a later accepted contract explicitly defines canonical owner, namespace, migration, synchronization, recovery, deletion/export, and credential behavior. A host harness session does not become B.O.B. canonical state by implication.

The exact portable crate/package boundary and DeepSeek cross-language bridge remain governed by Proposed RFC-0004 and must not be invented ahead of acceptance.

## System context

```mermaid
flowchart TB
    USER[User] <--> BOB[B.O.B.\nSingle user-facing agent]

    BOB <--> STATE[(Canonical local state)]
    BOB --> POLICY[Context · Cost · Authority · Routing]

    POLICY --> GEMINI[Gemini API adapter\nadvanced optional]
    POLICY --> ACCOUNT[Future supported\naccount-backed adapters]
    POLICY --> LOCAL[Future supported\nlocal adapters]
    POLICY --> TOOLS[Approved tools\nlater bounded execution]

    GEMINI --> BOB
    ACCOUNT --> BOB
    LOCAL --> BOB
    TOOLS --> BOB
```

B.O.B. is the product identity and system of record. Inference/runtime adapters provide replaceable capabilities.

## Logical architecture

```mermaid
flowchart TB
    subgraph UI[Presentation Boundary]
        TODAY[Today]
        INBOX[Inbox]
        CHAT[B.O.B. Chat]
        SETTINGS[Settings]
    end

    subgraph CORE[Rust Application Core]
        CMD[Command API]
        BOB[B.O.B. Agent Core]
        TASK[Task Service]
        PLAN[Planner Service]
        CONTEXT[Context Broker]
        ROUTER[Inference Router]
        POLICY[Authority + Cost Policy]
        TOOLS[Tool Gateway\nlater capability]
        PREF[Preference Service]
    end

    subgraph DATA[Local Persistence]
        STORE[(SQLite Canonical Store)]
        HISTORY[(Action + Continuity History)]
        SECRET[Protected Credential References]
    end

    subgraph RUNTIMES[Inference / Runtime Adapter Boundary]
        GEMINI[GeminiAdapter\nadvanced optional]
        OTHER[Future supported adapters]
    end

    UI --> CMD
    CMD --> BOB
    CMD --> TASK
    CMD --> PLAN
    CMD --> PREF

    BOB --> CONTEXT
    BOB --> ROUTER
    BOB --> POLICY

    TASK --> STORE
    PLAN --> STORE
    PREF --> STORE
    BOB --> HISTORY

    ROUTER --> POLICY
    ROUTER --> RUNTIMES
    CONTEXT --> RUNTIMES
    RUNTIMES --> BOB
    POLICY --> SECRET
    POLICY -. later .-> TOOLS
```

## Layer responsibilities

### Presentation boundary

The frontend renders B.O.B. state and sends typed commands. It does not receive unrestricted filesystem, process, shell, database, or credential access.

Provider/runtime detail is exposed only when useful for explicit configuration, cost, privacy, capability, or troubleshooting. Ordinary interaction remains with B.O.B. Settings must not make provider plumbing the product identity.

### B.O.B. Agent Core

The B.O.B. Agent Core is the single user-facing orchestration boundary. The accepted #34 contract gives it responsibility for:

- conversational identity and response assembly;
- intent interpretation;
- bounded context assembly;
- inference routing and policy coordination;
- proposal validation;
- deterministic-service coordination;
- compact continuity and failure handling;
- result/evidence presentation.

It does not surrender canonical state ownership to an underlying runtime. Model/runtime output is untrusted until B.O.B. validates it. Important state-changing proposals are previewed before application.

### Deterministic application services

The Rust core owns deterministic business behavior, including item lifecycle, planning, persistence orchestration, export/migration, proposal validation, preference handling, and authority/cost enforcement. These services remain useful when no inference runtime is available.

ADR-0006 allows suitable deterministic services to move behind portable B.O.B.-owned contracts when a real second host requires them. Extraction does not change their semantics or transfer their authority to the consuming harness.

### Inference router

The inference router selects only allowed capabilities according to supported configuration, explicit user choice where applicable, availability, auth state, billing classification, privacy constraints, and required capability.

Authentication method does not imply billing class. Unknown billing classification fails closed. Provider/model changes that materially affect cost, privacy, or user intent never happen silently.

The router must not become a speculative universal provider framework. Normalize only the fields and lifecycle behavior required by supported adapters.

### Inference/runtime adapters

Adapters normalize supported inference backends to narrow internal contracts. They do not become B.O.B.'s user-facing identity and do not own product state.

An adapter may report, where supported:

- availability;
- authentication state;
- runtime/model identity;
- capabilities;
- billing class;
- invocation status;
- structured result;
- cancellation.

Gemini Developer API is the currently implemented cloud adapter and remains subject to its accepted professional/business-use, unpaid-service data-use, secret, and billing boundaries. Account-backed and local adapters remain governed by Wayfinder #79 and must not be invented ahead of accepted authority.

GG-CORE, when integrated, implements this inference/runtime role through a supported B.O.B.-owned adapter. It does not become an application-state or tool-authority boundary.

### Tool gateway

Tools are separate capabilities from inference/runtime identity. Future Delegate behavior may add bounded execution authority through explicit user grants. Ordinary Assist requests do not inherit shell, filesystem, repository, or broad external permissions.

An external harness exposing a B.O.B. capability does not receive Delegate authority merely because it can invoke that capability. Any executable authority crossing a host-adapter boundary requires its own accepted contract.

### Persistence

Persistence is local-first and single-user. The accepted contract is:

- one SQLite database owned exclusively by the Rust core as canonical ordinary application state;
- every logical state change commits transactionally or not at all;
- Rust owns immutable monotonic migrations and startup compatibility checks;
- schema-changing migrations create SQLite-consistent safety copies and fail closed on open/migration failure;
- B.O.B. retains bounded known-good recovery snapshots;
- ordinary crash consistency relies on SQLite rather than custom shadow-file logic;
- backup/restore uses SQLite-consistent snapshots with fail-closed rollback;
- portable export is a documented versioned JSON package of user-owned non-secret state;
- credentials remain in the OS secret store, with SQLite limited to non-secret references/status;
- corruption or migration failure never silently resets user data.

The SQLite schema owns ordinary product state, not an open-ended advanced memory subsystem. Richer governed-memory behavior should preferentially integrate later with `MythologIQ-Labs-LLC/agent-memory` through an explicit contract.

Stateless external-harness capability use does not create another B.O.B. database. Stateful embedding is deferred until a separate accepted state-ownership contract exists.

See ADR-0004, ADR-0006, RFC-0003, and Proposed RFC-0004.

## Request flow

```mermaid
sequenceDiagram
    actor U as User
    participant UI as B.O.B. UI
    participant B as B.O.B. Agent Core
    participant P as Policy
    participant X as Context Broker
    participant R as Inference Router
    participant A as Selected Adapter
    participant S as Canonical State

    U->>UI: Ask B.O.B.
    UI->>B: Typed request
    B->>P: Check authority + cost + privacy
    P-->>B: Allowed constraints
    B->>X: Build bounded context
    X-->>B: Context package
    B->>R: Request supported capability
    R->>A: Invoke selected allowed adapter
    A-->>R: Normalized result
    R-->>B: Inference result
    B->>B: Validate response/proposals
    B-->>UI: Response + previewed actions
    U->>UI: Confirm or reject important action
    UI->>B: Disposition
    B->>S: Apply validated confirmed change
```

The user never changes conversational identity during this flow. Inference failure leaves deterministic B.O.B. behavior available.

## Canonical state boundary

```text
+------------------------------------------------------------------+
|                              B.O.B.                              |
|                                                                  |
|  Identity  Tasks  Plans  Inbox  Preferences  Working Continuity |
|      \       |      |      |        |             /             |
|       +------+------+-+----+--------+------------+              |
|                         |                                        |
|                  Canonical Local State                          |
|                         |                                        |
|             B.O.B. Context + Policy Boundary                    |
+-------------------------|----------------------------------------+
                          |
                selected capability, not identity
                          |
          +---------------+----------------+----------------+
          |                                |                |
   Gemini API adapter            Account-backed paths   Local paths
   advanced optional             when supported         when accepted
          |                                |                |
   non-canonical backend          non-canonical          non-canonical
```

## State domains

### Item

Minimum conceptual fields include `id`, `type`, `title`, `notes`, lifecycle `status`, `priority`, estimate, due time, optional energy, tags, and creation/update timestamps.

### DayPlan

A day plan owns its date, up to three default focus items, ordered blocks, optional day bounds/capacity, and enough generated/replanned metadata to explain the current plan.

### ConversationContinuity

B.O.B. stores compact continuity needed to preserve the one-agent experience across restart and runtime changes: conversation/thread identity, current intent, relevant work associations, compact summaries, runtime identity when useful, and returned proposals/outcomes. Vendor/runtime transcripts are not canonical state by default.

## Authority model

Authority belongs to B.O.B., not whichever runtime supplied inference. Assist may reason, summarize, organize, transform, and propose using an allowed adapter. Important state changes remain validated and previewed before application. Future Delegate behavior requires a separate bounded grant.

For an external harness consuming a stateless B.O.B. capability, the host owns its surrounding session/lifecycle while B.O.B. capability code owns only the semantics of its typed request/result boundary. Host authority is not imported into B.O.B. by default.

## Cost-policy boundary

Every inference/runtime adapter declares one billing class: `free`, `subscription`, `local`, `metered`, or `unknown`.

Unknown is blocked until classified. Metered inference is disabled by default and requires explicit user enablement. Authentication mechanism alone does not determine billing class.

The implemented Gemini API path is an advanced optional adapter with its own accepted provider-purpose/data-use and Free-Tier confirmation boundary. Future account-backed/local paths must preserve the same no-surprise-billing invariant.

See `docs/governance/AI_COST_AND_PROVIDER_POLICY.md`.

## Failure behavior

B.O.B. degrades safely:

- inference unavailable: deterministic B.O.B. remains operational;
- quota/auth/provider/network failure: no silent paid or different-provider fallback;
- invalid model proposal: reject without changing canonical state;
- persistence interruption: preserve user data and follow ADR-0004/RFC-0003 recovery semantics;
- runtime/provider failure: isolate failure from canonical state;
- external-harness adapter failure: isolate the adapter from standalone B.O.B. state and fail through a bounded typed error;
- future tool failure: return evidence/status without widening authority.

## Technology boundaries

Current direction:

- Tauri 2 desktop shell as the current first-party standalone host;
- Rust privileged application core with a future portable capability boundary defined by ADR-0006;
- framework-free TypeScript + Vite frontend;
- Windows 11 x64 primary platform;
- one B.O.B. agent identity in the standalone product;
- Rust-owned SQLite canonical ordinary-state store;
- OS-backed secret-store boundary;
- provider-independent inference/runtime seams;
- Gemini Developer API as an advanced optional current adapter;
- future first-party external-harness adapters remain optional and live in this repository;
- no required local HTTP inference server, Python runtime, vector database, peer-agent UX, plugin marketplace, or mandatory provider/harness client;
- no direct UI access to native secrets, database, shell, or arbitrary filesystem;
- no competing advanced-memory subsystem inside ordinary SQLite state.

Future account-backed adapters, local inference, governed-memory integration, bounded tools, and external-harness adapters may extend these seams only under accepted product/architecture authority. Proposed RFC-0004 must be accepted before substantial portable-host implementation proceeds.
