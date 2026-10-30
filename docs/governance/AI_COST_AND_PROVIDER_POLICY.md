# AI Cost and Provider Policy

## Objective

Make B.O.B. safe for sustained personal use without surprise usage charges or hidden provider changes.

## Billing classes

Every inference/runtime adapter must expose a billing class before invocation:

| Class | Meaning | Default policy |
| --- | --- | --- |
| `free` | Provider-supported use is available without direct metered billing for the configured path, subject to provider quota/terms | Allowed when configured |
| `subscription` | Invocation is expected to consume an included vendor subscription allowance rather than direct per-token API billing | Allowed when configured |
| `local` | Inference runs on user-controlled local compute without vendor inference fees | Allowed when configured |
| `metered` | Invocation may generate separately billed usage | Disabled by default |
| `unknown` | Billing behavior cannot be classified with confidence | Blocked |

OAuth, API keys, CLI login, and browser login are authentication mechanisms. They do not determine billing class by themselves.

A `free` classification is not a promise of permanent availability. Provider quota, eligibility, data-use terms, and product policy must be verified for the supported integration surface and surfaced where materially relevant.

## Default ordering

The active product boundary decides which configured adapters exist. Within that boundary, B.O.B. should prefer the least surprising allowed path rather than preserving a historical provider ordering for its own sake.

General default preference:

1. an explicit user-selected allowed adapter;
2. an enabled `free` path appropriate to the requested capability;
3. an enabled already-included `subscription` path;
4. an enabled `local` path;
5. an explicitly enabled `metered` path;
6. otherwise continue without inference and explain the available next step.

For the first runnable alpha, the resolved Wayfinder route selects **Gemini Developer API Free** as the sole required inference backend. Subscription-backed and local adapters are later expansion paths, not alpha prerequisites.

## Gemini Developer API Free waypoint boundary

The first-alpha Gemini Free path is allowed only inside the provider boundary currently stated by Google's Gemini API Additional Terms.

For context-bearing Gemini Free inference B.O.B. must:

- treat the capability as intended for professional or business use, not general consumer use;
- require a clear disclosure before the first context-bearing request that unpaid Gemini API content and responses may be used by Google to provide, improve, and develop products and machine-learning technologies and may be reviewed by humans;
- tell the user not to send sensitive, confidential, or personal information through the unpaid inference path;
- keep context-bearing Gemini Free inference disabled until the user affirmatively acknowledges that boundary;
- continue deterministic Today, Inbox, planning, persistence, and other non-inference behavior when the boundary is not accepted or the provider is unavailable;
- fail closed rather than silently switching to a paid or different provider;
- re-verify the boundary if Google changes the applicable terms, regional availability, billing classification, or data-use policy.

This restriction applies to the Gemini Free inference capability, not to B.O.B.'s local deterministic product as a whole. It does not convert B.O.B. into a business-only application. Broader personal/consumer context-bearing inference requires a later provider path or authoritative terms change that actually permits that use.

The current provider authority is the Gemini API Additional Terms effective March 23, 2026. Issue #57 records the reconciliation that established this waypoint boundary.

## No silent fallback

A provider failure, authentication failure, rate limit, free-quota exhaustion, subscription allowance exhaustion, local-runtime failure, or provider outage must not silently cause metered API traffic.

Valid outcomes include:

- continue without AI while deterministic B.O.B. planning remains available;
- use another enabled non-metered path only when the active product boundary and user policy permit it;
- wait for allowance or quota reset;
- explicitly enable a metered provider.

Backend changes that materially affect privacy, billing, or user intent must not occur silently.

## Metered enablement

Metered inference requires an explicit settings action. The UI must identify that the provider is metered. Later implementations may add budget ceilings, but absence of a budget feature does not weaken the default-disabled rule.

## OpenRouter metered boundary

OpenRouter is classified `metered` and `cloud`. It is disabled by default and requires, all at once:

- one exact user-entered `vendor/model` ID (OpenRouter routers such as `openrouter/auto` are rejected because they choose a different model per request);
- separate, persisted cloud-transmission and metered-billing permissions;
- an API key held only in the operating-system credential store;
- a Rust-held, in-memory Chat session grant that starts empty on every launch, is bound to the approved model, and is revoked by any model, permission, or credential change.

Choosing an OpenRouter model does not fix the upstream host that serves it, nor that host's data policy. B.O.B. never sends alternate models, and every request sets `provider.data_collection = "deny"` so OpenRouter routes only to hosts that do not collect prompt data. Requests are bounded to 4,000 prompt bytes and 512 output tokens. A failed OpenRouter request falls back to B.O.B.'s deterministic local guidance, never to another provider.

## Provider verification

Adapter documentation must identify the supported vendor surface being used and the basis for its billing classification. When vendor terms, quota, eligibility, privacy/data-use policy, or product behavior changes, the classification must be re-verified before documentation continues to claim free, subscription-backed, or otherwise included use.

## Terms and support boundaries

B.O.B. should integrate supported programmatic, CLI, or runtime interfaces. Do not rely on reverse-engineered private app protocols, credential scraping, or mechanisms intended to bypass vendor billing or usage controls.

## Local inference

Local inference is considered no-vendor-inference-fee, not zero-cost. Compute, power, hardware, and setup costs still exist. The UI need not monetize those costs but should describe them accurately.

## Logging

Do not log secrets, raw authentication tokens, API keys, or unnecessary billing metadata. Operational status should use non-sensitive adapter identifiers and normalized error classes.
