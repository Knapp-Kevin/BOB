# Product Definition

**Status:** Accepted  
**Product:** B.O.B. | Better Organized Brain  
**Product stage:** **Alpha Product**

## Product statement

B.O.B. is a local-first personal AI workbench with **one user-facing agent identity: B.O.B.** Behind that single point of contact, B.O.B. can use multiple LLMs, inference runtimes, and tools while keeping the user's tasks, plans, preferences, and continuity independent of any one vendor or model.

B.O.B. is currently classified as an **Alpha Product**. The implemented product exists and is usable as an alpha-stage desktop application; remaining native Windows recovery and installer obligations are release-qualification evidence, not a reason to classify the product as pre-alpha.

B.O.B. adds value in two places that vendor applications do not share:

1. a unified B.O.B. agent that can draw on different intelligence and execution backends without fragmenting the user experience;
2. an ADHD-friendly executive-function interface designed around low-friction capture, realistic planning, reduced cognitive load, and one useful next action.

The governing principle is:

> **B.O.B. is the agent. Models, runtimes, and tools are capabilities.**

## Problem

AI vendors provide increasingly capable chat, coding, and agentic applications, but each vendor owns a separate session model, interface, cost structure, and execution surface. Requiring the user to manage those same distinctions inside B.O.B. would simply reproduce the complexity the product exists to remove.

Generic productivity software stores tasks but lacks an intelligent interaction layer. Generic AI chat reasons well but does not provide durable personal planning and executive-function structure.

B.O.B. occupies the layer between them while presenting only one coherent assistant to the user.

## Product invariant

The user talks to **B.O.B.**

B.O.B. may use Gemini-backed inference, Claude-backed inference, Codex-backed inference, local models, future LLMs, or approved tools. Those systems are internal capabilities, not peer user-facing agents.

Provider/runtime identity may be visible when it matters for cost, privacy, capability, troubleshooting, or explicit user choice, but it must not become the primary interaction model.

## Target user experience

A user opens one application and sees what matters today, not an empty prompt and not a roster of AI providers.

They can capture a thought without deciding its final category, ask B.O.B. to organize it, plan a realistic day, replan after disruption, or ask B.O.B. to perform bounded external work in a later capability slice. B.O.B. can change the underlying model/runtime when supported without moving canonical tasks or forcing the user into a different conversational identity.

The user should be able to ask:

- What should I do next?
- Plan my day.
- I have 25 minutes. What can I finish?
- Break this task down.
- I am overwhelmed. Show me one thing.
- Turn this brain dump into tasks.
- Move unfinished work to tomorrow.
- Use another supported runtime for this task when available.
- Keep this local when a local capability is configured.

Backend preferences are optional controls, not separate agents the user must manage.

## Product surfaces

### Today

Today is the default surface. It contains:

- up to three focus items;
- a simple time-oriented plan;
- the next recommended action;
- fixed commitments and flexible work blocks;
- quick capture;
- start, complete, defer, and replan actions;
- a low-stimulation overwhelmed state that hides nonessential backlog information.

### Inbox

Inbox is the single capture queue for unprocessed material. Items may be tasks, ideas, notes, reminders, or undetermined brain dumps. Categorization can happen later.

### B.O.B. Chat

Chat is the conversational surface for the B.O.B. agent. It is not a vendor-session selector and is not the canonical store itself.

B.O.B. Chat can:

- explain the current plan;
- organize captured material;
- propose priorities;
- break work into smaller steps;
- answer questions using bounded B.O.B. context;
- propose state changes;
- honor explicit inference preferences when relevant and supported;
- later perform bounded external work when the user intentionally delegates authority to B.O.B.

### Settings

Settings owns user-facing configuration, including:

- availability and status for inference/runtime capabilities that actually exist;
- cost, billing-class, privacy, and provider-use policy controls where relevant;
- credential and authentication state through provider-neutral product boundaries;
- local inference configuration when local adapters are supported;
- accessibility and visual preferences;
- local data location, export, backup, and reset;
- optional advanced provider controls.

Settings must not turn provider plumbing, product governance, or development-status exposition into B.O.B.'s ordinary product identity. API-key integrations may remain available as advanced options, but unresolved or unimplemented provider paths must not appear as fake controls.

## ADHD-friendly interaction requirements

ADHD-friendly means the product reduces executive-function friction through concrete interaction patterns. It does not infer or score neurological traits.

Required patterns include:

- one obvious next action;
- low-friction capture before categorization;
- progressive disclosure instead of showing the entire system at once;
- explicit time estimates where useful;
- support for re-entry after interruption;
- easy deferral without losing the item;
- realistic daily capacity rather than unlimited scheduling;
- direct language and short decision sets;
- visible distinction between suggestion and committed state;
- accessible typography, contrast, reduced motion, and keyboard operation.

## AI and inference role

Inference is a capability of B.O.B., not the application backbone and not the user-facing identity.

Without an available LLM/runtime, B.O.B. must still support capture, task state, manual planning, scheduling, completion, deferral, and local persistence.

Model output may inform or propose application changes, but application state changes are executed by B.O.B. after validation and according to the user's authority settings.

Provider-specific inference is available only when its current terms, privacy/data-use boundary, regional availability, and billing classification fit the requested use. A provider restriction on one inference path does not redefine B.O.B.'s local deterministic product as a whole.

## Authority modes

### Assist

B.O.B. reasons, summarizes, organizes, transforms, and proposes using an allowed inference runtime. Assist does not implicitly grant filesystem, shell, repository, or external-workspace authority.

### Delegate

The user explicitly grants **B.O.B.** bounded authority for a defined task. B.O.B. may then use an execution-capable runtime or approved tool inside that grant. The user is not delegating to a separate peer agent.

Delegate/tool execution is a future capability relative to the current alpha product and does not block its Assist-mode usefulness or alpha-stage classification.

## Cost and provider model

The governing policy is zero-surprise inference cost and provider independence:

1. prefer configured zero-cost, already-included/account-backed, or intentionally local inference paths whose current terms and entitlement fit the requested use;
2. classify billing independently from authentication method;
3. use metered API inference only when explicitly enabled by the user;
4. never silently fail over into separately billed inference or a materially different provider/runtime;
5. keep deterministic B.O.B. useful when no allowed inference capability is available.

The first runnable alpha waypoint used **Gemini Developer API Free** as the sole required inference backend to prove the inference, policy, credential, and fail-closed seams. That waypoint decision is historical/current implementation evidence, not B.O.B.'s long-term provider destination.

Current accepted direction is provider-independent: simple account-backed or local inference should become the normal onboarding path where an official, third-party-compatible route exists; existing API-key integrations such as Gemini Developer API may remain available as advanced optional adapters; and no provider is allowed to become B.O.B.'s architectural landlord. Claude/Codex account-backed paths and local inference remain separately governed expansion work until their concrete supported contracts are accepted and implemented.

The existing Gemini Free capability remains narrower than B.O.B.'s overall product boundary. Under the current Gemini API Additional Terms, context-bearing Gemini Free inference is enabled only for professional/business use after the user acknowledges the applicable unpaid-service data-use boundary, including that submitted content/responses may be used for product/model improvement and may be reviewed by humans, and that sensitive, confidential, or personal information must not be submitted. If that boundary is not accepted, B.O.B. continues deterministically without sending context to Gemini.

## Canonical state

B.O.B. owns:

- tasks and inbox items;
- day plans and schedule blocks;
- user preferences;
- runtime and cost-policy configuration;
- compact working continuity;
- B.O.B. conversation continuity;
- action history required for understandable behavior.

Vendor/runtime session state may be referenced as an implementation detail but is not the canonical product state.

## Explicit non-goals

The current alpha product does not include:

- a visible multi-agent swarm, agent roster, or peer-agent orchestration model;
- a required second inference backend;
- local inference as a first-run requirement;
- Delegate/tool execution as an alpha requirement;
- cognitive trait profiling;
- diagnosis, treatment, or mental-health assessment;
- ambient autonomous execution making open-ended changes;
- vector databases or general RAG knowledge centers;
- document-management platforms;
- cloud sync or multi-user collaboration;
- opaque automatic model scoring as a required first-release feature;
- general plugin marketplaces;
- vendor-specific clones of ChatGPT, Claude, or Codex;
- silent metered API fallback;
- gamified productivity scoring or shame-oriented analytics.

## Alpha qualification success criteria

B.O.B. is already classified as an **Alpha Product**. The current integrated alpha build is release-qualified when a Windows 11 x64 user can:

- launch B.O.B. into the Today-first interaction shell;
- capture and manage work through Today and Inbox;
- plan and replan a realistic day and recover after interruption;
- preserve canonical local state across restart using the resolved persistence/recovery contract;
- complete the guided Gemini Developer API Free credential flow;
- when using the accepted professional/business Gemini Free boundary, acknowledge the provider/data-use disclosure before any context-bearing request and converse with B.O.B. through that real inference path;
- decline that provider boundary and still retain deterministic B.O.B. planning, capture, persistence, and recovery behavior without context-bearing Gemini calls;
- receive organization, breakdown, reorientation, resume/handoff, and lightweight decision-facilitation help through B.O.B. Chat when an allowed inference path is active;
- preview important proposed state changes before they are applied;
- continue using deterministic planning behavior when inference is unavailable;
- understand provider/privacy/cost state without being forced to manage a provider dashboard;
- verify the alpha through the resolved local validation, recovery, accessibility, and Windows packaging evidence.

A second runtime is intentionally deferred from the current alpha qualification bar. Continued development after native qualification follows the provider-independent and local-runtime Wayfinder destinations rather than preserving Gemini as the default product identity.
