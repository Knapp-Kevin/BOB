# Live Inference Validation

**Authority:** `docs/governance/AI_COST_AND_PROVIDER_POLICY.md`, PRD-0003, RFC-0002
**Who runs it:** the owner, in the installed app, with their own accounts. Agents never type API keys or make paid calls on the owner's behalf.

This checklist confirms each optional cloud inference adapter end to end. Every adapter is off by default, and B.O.B. stays fully usable without any of them. Run only the sections for the providers you intend to use.

## Before you start

1. Install the build under test and launch it. Confirm Today, Inbox, and Chat work with no provider configured. Chat should say "Local B.O.B. guidance active."
2. Only one cloud provider can answer Chat at a time. Turn one off before you enable the next.
3. Use a throwaway prompt with no personal data, for example `Give me one tiny first step for tidying a desk.`
4. Expect small charges on metered providers. Each request is capped at 4,000 prompt bytes and 512 output tokens.

## 1. Spending limits

Every paid provider needs two limits:

- **B.O.B.'s monthly limit (required).** You set it in each provider card in Settings. It is pre-filled at $20 and B.O.B. will not allow paid use without it. For OpenAI and Anthropic you also enter the model's price per million input and output tokens. Use **Look up model prices ↗** on the card to open the vendor's pricing page.
- **A vendor-side limit (strongly recommended).** It catches anything B.O.B. cannot see, such as a wrong price entered by hand or the same key used elsewhere. Use the card's buttons to open the right page:

| Provider | Vendor-side limit |
| --- | --- |
| OpenRouter | **Get an API key ↗** opens openrouter.ai Keys. Set a credit limit on the key; B.O.B. refuses paid OpenRouter use without one. Free-only mode needs no limit. |
| OpenAI | **Also set a limit in the console ↗** opens platform.openai.com Limits. Prefer a dedicated project for B.O.B., a monthly budget, and prepaid credit with auto-recharge off. |
| Anthropic | **Also set a limit in the console ↗** opens the Claude Console Limits page. Prefer a dedicated workspace for B.O.B., a spend limit, and prepaid credits with auto-reload off. |

Record both limits in the PR or issue that tracks this validation.

## 2. OpenAI API

1. On the **OpenAI API** card, use **Get an API key ↗** to open platform.openai.com API keys, and create a key in the project from step 1. Copy a chat-capable model ID from the project's model list (a small, non-reasoning model keeps cost and empty-reply risk low).
2. In B.O.B.: Settings → Inference → **OpenAI API**.
3. Paste the key into **OpenAI API key** → **Store key securely**. Expect "API key configured". The key field clears.
4. Enter the model ID. Under **Spending limit (required)** keep or change the $20 monthly limit, and enter the model's input and output prices per million tokens (**Look up model prices ↗**). Check **Allow cloud transmission** and **Allow separately billed inference**, then **Save model authorization**. Confirm the dialog. Saving with billed use checked but a price missing is refused.
5. Check **Use OpenAI API in Chat for this session** and confirm the dialog.
6. Chat → send the test prompt. Expect: the status line names OpenAI and the model, and B.O.B. replies with model text.
7. Confirm usage appears in the OpenAI dashboard for that project, and that the card's **Spent this month** went up by roughly that amount.
8. Negative checks:
   - Uncheck **Allow separately billed inference** and save. The Chat toggle turns off and cannot be turned back on. A Chat message gets local guidance.
   - Quit and relaunch B.O.B. The Chat toggle is off again (session grants never survive a restart).
   - Set the monthly limit to $0.01 and save, then re-enable and send. Expect "reached your monthly spending limit. Nothing was sent." and local guidance. Restore your real limit afterwards.
   - Store a deliberately wrong key (any 20+ character string), re-enable, and send. Expect "OpenAI API rejected the saved API key" and local guidance, not another provider.
9. Turn the Chat toggle off. Optionally **Remove OpenAI API key**.

## 3. Anthropic API

1. On the **Anthropic API** card, use **Get an API key ↗** to open the Claude Console keys page, and create a key in the workspace from step 1. Pick a model ID from the Console's model list (for example a Haiku-class model).
2. In B.O.B.: Settings → Inference → **Anthropic API**. Repeat steps 3–9 from the OpenAI section with the Anthropic key and model.
3. Confirm usage appears in the Anthropic Console for that workspace.

## 4. OpenRouter

### 4a. Free models only (the default, no spending)

1. On the **OpenRouter** card, use **Get an API key ↗** to open openrouter.ai Keys and create a key. A credit limit is not needed for this part, but setting a small one is still wise.
2. Use **Browse free models ↗** to open OpenRouter's free model list and pick a model whose ID ends in `:free`.
3. In B.O.B.: Settings → Inference → **OpenRouter**. Store the key, enter the `:free` model ID, confirm **Free models only** is checked (it is the default), check **Allow cloud transmission** (leave the metered box unchecked), and save. The card header reads "CLOUD · FREE MODELS ONLY".
4. Enable **Use OpenRouter in Chat for this session** and send the test prompt. Expect a reply, and in OpenRouter's Activity page a cost of $0 for that request.
5. Negative checks:
   - Change the model ID to one without `:free` and save. Enabling Chat is refused.
   - If the reply fails with "No OpenRouter host for this model meets B.O.B.'s price and privacy limits", that free model has no host that refuses prompt logging. Try another `:free` model and record which ones failed.
6. **Check key limits** shows free-model requests left today.

### 4b. Paid models with a capped key

1. Uncheck **Free models only**, enter a paid `vendor/model` ID, keep or change the $20 monthly limit, check both permissions, and save. Clearing the monthly limit and saving is refused.
2. With a key that has **no** credit limit: enabling Chat is refused with "paid use needs a key with a credit limit". **Check key limits** says the key has no limit.
3. Set a small credit limit on that key at openrouter.ai → Keys. **Check key limits** now shows the limit and remaining credit. Enable Chat and send the test prompt.
4. Remove the limit on openrouter.ai, then send another prompt. Expect "the key has no credit limit or no credit left" and local guidance, with Chat turned off.
5. Restore the key limit. Confirm **Spent this month** on the card matches OpenRouter's Activity page for these requests.
6. Confirm that with OpenRouter Chat on, enabling OpenAI or Anthropic Chat is refused until OpenRouter is turned off.

## 5. Record the evidence

For each adapter you ran, record in the tracking issue or PR:

- the build or commit under test and the date;
- the model ID, and the spending cap that was in place;
- pass or fail for each numbered step, including the negative checks;
- the dashboard usage you observed (amount only, no keys or account IDs);
- anything that did not match this checklist.

Never paste API keys, emails, or organization IDs into the record.

## Owner decisions this checklist cannot make

- The monthly limit for each paid provider (B.O.B. pre-fills $20) and the matching vendor-side limit.
