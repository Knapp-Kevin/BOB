# Live Inference Validation

**Authority:** `docs/governance/AI_COST_AND_PROVIDER_POLICY.md`, PRD-0003, RFC-0002
**Who runs it:** the owner, in the installed app, with their own accounts. Agents never type API keys, sign in to CLIs, or make paid calls on the owner's behalf.

This checklist confirms each optional cloud inference adapter end to end. Every adapter is off by default, and B.O.B. stays fully usable without any of them. Run only the sections for the providers you intend to use.

## Before you start

1. Install the build under test and launch it. Confirm Today, Inbox, and Chat work with no provider configured. Chat should say "Local B.O.B. guidance active."
2. Only one cloud provider can answer Chat at a time. Turn one off before you enable the next.
3. Use a throwaway prompt with no personal data, for example `Give me one tiny first step for tidying a desk.`
4. Expect small charges on metered providers. Each request is capped at 4,000 prompt bytes and 512 output tokens.

## 1. Set a spending cap first (metered providers)

B.O.B. has no budget ceiling of its own, so set the cap in each vendor console **before** you add a key there. Console layouts change, so if a menu has moved, search the console for "limits" or "budget".

| Provider | Where to cap spending |
| --- | --- |
| OpenRouter | openrouter.ai → Keys → create or edit the key → set a credit limit on the key. B.O.B. refuses paid OpenRouter use without one. Free-only mode needs no limit. |
| OpenAI | platform.openai.com → create a dedicated project for B.O.B. → Settings → Limits → set a monthly budget. Create the key inside that project. Prefer a prepaid credit balance with auto-recharge off. |
| Anthropic | platform.claude.com (Console) → create a dedicated workspace for B.O.B. → Settings → Limits → set a workspace spend limit. Create the key in that workspace. Prefer prepaid credits with auto-reload off. |

Record the cap you set in the PR or issue that tracks this validation.

## 2. OpenAI API

1. In the OpenAI project from step 1, create an API key. Copy a chat-capable model ID from the project's model list (a small, non-reasoning model keeps cost and empty-reply risk low).
2. In B.O.B.: Settings → Inference → **OpenAI API**.
3. Paste the key into **OpenAI API key** → **Store key securely**. Expect "API key configured". The key field clears.
4. Enter the model ID, check **Allow cloud transmission** and **Allow separately billed inference**, then **Save model authorization**. Confirm the dialog.
5. Check **Use OpenAI API in Chat for this session** and confirm the dialog.
6. Chat → send the test prompt. Expect: the status line names OpenAI and the model, and B.O.B. replies with model text.
7. Confirm usage appears in the OpenAI dashboard for that project.
8. Negative checks:
   - Uncheck **Allow separately billed inference** and save. The Chat toggle turns off and cannot be turned back on. A Chat message gets local guidance.
   - Quit and relaunch B.O.B. The Chat toggle is off again (session grants never survive a restart).
   - Store a deliberately wrong key (any 20+ character string), re-enable, and send. Expect "OpenAI API rejected the saved API key" and local guidance, not another provider.
9. Turn the Chat toggle off. Optionally **Remove OpenAI API key**.

## 3. Anthropic API

1. In the Anthropic workspace from step 1, create an API key. Pick a model ID from the Console's model list (for example a Haiku-class model).
2. In B.O.B.: Settings → Inference → **Anthropic API**. Repeat steps 3–9 from the OpenAI section with the Anthropic key and model.
3. Confirm usage appears in the Anthropic Console for that workspace.

## 4. OpenRouter

### 4a. Free models only (no spending)

1. Create an OpenRouter key. A credit limit is not needed for this part, but setting a small one is still wise.
2. Pick a model whose ID ends in `:free` from openrouter.ai/models (filter by price: free).
3. In B.O.B.: Settings → Inference → **OpenRouter**. Store the key, enter the `:free` model ID, check **Free models only** and **Allow cloud transmission** (leave the metered box unchecked), and save. The card header reads "CLOUD · FREE MODELS ONLY".
4. Enable **Use OpenRouter in Chat for this session** and send the test prompt. Expect a reply, and in OpenRouter's Activity page a cost of $0 for that request.
5. Negative checks:
   - Change the model ID to one without `:free` and save. Enabling Chat is refused.
   - If the reply fails with "No OpenRouter host for this model meets B.O.B.'s price and privacy limits", that free model has no host that refuses prompt logging. Try another `:free` model and record which ones failed.
6. **Check key limits** shows free-model requests left today.

### 4b. Paid models with a capped key

1. Uncheck **Free models only**, enter a paid `vendor/model` ID, check both permissions, and save.
2. With a key that has **no** credit limit: enabling Chat is refused with "paid use needs a key with a credit limit". **Check key limits** says the key has no limit.
3. Set a small credit limit on that key at openrouter.ai → Keys. **Check key limits** now shows the limit and remaining credit. Enable Chat and send the test prompt.
4. Remove the limit on openrouter.ai, then send another prompt. Expect "the key has no credit limit or no credit left" and local guidance, with Chat turned off.
5. Confirm that with OpenRouter Chat on, enabling OpenAI or Anthropic Chat is refused until OpenRouter is turned off.

## 5. Claude CLI

**Current state: blocked by policy.** Its billing class is `unknown` (see the cost policy for the terms citation), so B.O.B. refuses to start it. You can still check detection now. Generation works only after you record a billing decision and the classification constant is changed in a reviewed PR.

1. Install Claude Code and sign in yourself, in a terminal, with the account you want B.O.B. to use:

   ```powershell
   claude auth login
   ```

   (Or set up an Anthropic Console API key in Claude Code. That is metered usage.)
2. Confirm the CLI works outside B.O.B.:

   ```powershell
   claude auth status --text
   ```

3. In B.O.B.: Settings → Inference → **Claude CLI** → **Check CLI**. Expect "Installed, version …, signed in with an account (… plan)". The card must not show your email or organization.
4. Enter a model alias (for example `sonnet` or `haiku`), check **Allow cloud transmission**, and save.
5. Check **Use Claude CLI in Chat for this session**. While billing is `unknown`, expect it to be refused with "Billing for this CLI is unclassified…". That refusal **is** the expected result today.
6. After an owner decision reclassifies the CLI (separate PR), repeat step 5 and send the test prompt. Then confirm:
   - The reply arrives and the status line names Claude CLI.
   - **Stop waiting** in the Chat status line stops a slow reply. Expect "request was stopped" and local guidance.
   - Run `claude auth logout`, then send again. Expect "Claude CLI is not signed in" and local guidance.
   - Task Manager shows no leftover `claude` process after a reply, a stop, or a timeout.
   - If you are on a subscription with extra usage turned on, check your Claude usage page to see whether the request counted against the plan or against extra usage.

## 6. Codex CLI

**Current state: blocked by policy** (billing `unknown`), same as Claude CLI.

1. Install the Codex CLI (`npm install -g @openai/codex`), then sign in yourself:

   ```powershell
   codex login
   ```

2. Confirm sign-in:

   ```powershell
   codex login status
   ```

3. In B.O.B.: Settings → Inference → **Codex CLI** → **Check CLI**. Expect "Installed, version …, signed in with an account" (or "with an API key (metered)").
4. Enter a model ID that your Codex install supports, check **Allow cloud transmission**, and save. Enabling Chat is refused while billing is `unknown`. That is the expected result today.
5. After reclassification, repeat the Claude CLI step 6 checks with `codex logout` for the sign-in check. Also confirm that a prompt asking Codex to "list the files in this folder" ends with "Codex CLI tried to use a tool and was stopped", not a file listing.

## 7. Record the evidence

For each adapter you ran, record in the tracking issue or PR:

- the build or commit under test and the date;
- the model ID, and the spending cap that was in place;
- pass or fail for each numbered step, including the negative checks;
- the dashboard usage you observed (amount only, no keys or account IDs);
- anything that did not match this checklist.

Never paste API keys, CLI tokens, emails, or organization IDs into the record.

## Owner decisions this checklist cannot make

- Whether the Claude CLI should be classified `subscription`, `metered`, or kept `unknown`. Anthropic's Claude Code legal page does not clearly permit a third-party app to drive a user's subscription, and running Claude Code inside a product may require the Commercial Terms.
- Whether the Codex CLI should be classified `subscription`, `metered`, or kept `unknown`. OpenAI's Codex docs point programmatic use at API keys.
- The spending cap for each metered key.
