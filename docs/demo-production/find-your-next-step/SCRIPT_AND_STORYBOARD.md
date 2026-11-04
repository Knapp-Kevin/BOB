# "B.O.B.: Find Your Next Step": storyboard and final script (v1)

The film is recorded from the installed, signed B.O.B. Windows Alpha **v0.1.0**. It uses an offline Windows Sandbox, a 1920×1080 client area and fictional fixture `fixture-film-sam`. Narration and music are made in Sonic Rift 0.9.1.

**Arc:** overwhelm → clarity → choice → action → confidence. The film makes one claim: B.O.B. makes the next decision easier. It does not solve Sam's day, and it does not suggest B.O.B. uses AI.

## What the rehearsal showed (installed v0.1.0, 2026-11-04)

Each scene uses only behavior seen in the rehearsal (`source-capture/rehearsal/r01`–`r15`):

| Behavior | Seen |
| --- | --- |
| Today shows a next action, "What matters today" (up to three), a remaining plan and "Resume where I left off" | r01 |
| Quick capture puts text in the Inbox as a **Note**. B.O.B. does not classify it | r02, r03 |
| The user sets the type with the Type selector (Note → Reminder) | r04, r05 |
| **Organize with B.O.B.** opens Chat with a **Preview before apply** card. It proposes moving the highest-priority inbox task ("Send the budget summary to Dana by 3 pm") into planned work as the next action. Nothing changes until **Apply this change** | r06 |
| Apply → Today, toast "Applied the previewed change. Everything else stayed put." The budget summary is the next action | r07, r08 |
| **Overwhelmed mode** leaves only the next action and quick capture, with "Reduced-information mode is on." | r09 |
| **Do it now** → "In progress", toast "Started. Everything else can wait for now." | r10 |
| In Overwhelmed mode, Chat hides the Current context panel, so the film shows full view again before Chat | r11 |
| **Break it down** → "Start smaller: reopen … and spend five minutes identifying the first concrete change." | r12 |
| **Save my place** → Saved handoff card (Objective, State In progress, Next) | r13 |
| After closing and reopening B.O.B., the conversation, the handoff and the In progress task are all still there | r14, r15 |
| Chat status line: "Local B.O.B. guidance active. Chat prompts are not sent to any cloud provider." | r06 |

**Defect found:** the Save my place reply says "Current state: doing" while the card says "In progress" ([#146](https://github.com/Knapp-Kevin/BOB/issues/146)). The film shows the card and does not draw attention to that reply line. The defect is disclosed in the QA report.

**Not shown, because v0.1.0 doesn't do it:**
- automatic classification;
- inference or AI suggestions;
- clock-time scheduling;
- Overwhelmed mode surviving a restart (it is per session).

## Fictional fixture (`fixture-film-sam`)

Sam, an adult with a full Tuesday. All items are fictional:

- **Planned:**
  - "Prep slides for Thursday's team meeting" (45 min, Thursday)
  - "Pay the electric bill" (10 min, Friday)
  - "Reply to Priya about Saturday plans" (5 min)
- **Inbox:**
  - "Send the budget summary to Dana by 3 pm" (task, high, 20 min, Today)
  - "Sign the school field-trip form" (task, Tomorrow)
  - "Landlord: the kitchen faucet is still dripping" (note)
  - "Idea: a picnic for Grandma's birthday" (idea)
- **Typed live during the take:** "Renew the car registration before Friday"

## Storyboard and narration

The spoken text says "Bob" so that Sonic Rift pronounces the name as a word. Captions and transcript write "B.O.B.". **Owner check:** confirm the spoken pronunciation.

| # | Picture (real input in the installed app) | Narration (captions) |
| --- | --- | --- |
| Opener | Shared MythologIQ Labs opener (silver mark, DM Sans), silent, dissolving into Today | — |
| S01 | Today, full view: three planned tasks compete, with a resume list. The cursor drifts across them | "This is Sam's Tuesday. Some days, everything seems important. And deciding what to do first can feel like another task." |
| S02 | Click Quick capture, type "Renew the car registration before Friday", Enter, then open Inbox: the item arrives as a Note | "B.O.B. gives those thoughts somewhere to go, so they don't all have to stay in your head." |
| S03 | Set the new item's Type to **Reminder**. Click **Organize with B.O.B.** | "Sam decides what each thing is. And when asked, B.O.B. suggests one next step, without taking the decision away." |
| S04 | Chat: the **Preview before apply** card. The cursor rests, then **Apply this change** → Today with the budget summary as next action, toast "Everything else stayed put." | "Nothing changes until Sam says so. Instead of handling everything at once, there's one useful next step." |
| S05 | **Overwhelmed mode** → only the next action and capture. **Do it now** → In progress, toast "Started. Everything else can wait for now." | "And when there's simply too much, the screen can get quieter: just the next step, and a place for new thoughts." |
| S06 | **Show full view** → Chat → **Break it down** → **Save my place** → Saved handoff card | "Sam starts small, and saves a place to come back to." |
| S07 | Close and reopen B.O.B. (one cut at the restart). Chat still shows the handoff; then Today shows the budget summary In progress | "Even after closing B.O.B., it's still there. The day isn't suddenly empty. But the next step is clearer, and the progress doesn't have to disappear." |
| S08 | End card: B.O.B. mascot, "B.O.B. · Better Organized Brain · Windows Alpha v0.1.0", release link, fictional-data note | "B.O.B. A little less noise. A clearer next step." |

An on-screen note appears at the bottom of the sidebar over S01: "Installed B.O.B. Windows Alpha v0.1.0. Sam is fictional; all data is demo data."

## Claims guardrails

- No diagnosis, treatment, medical or productivity guarantees. No "AI" wording.
- "Suggests one next step" describes Organize's deterministic proposal of the highest-priority inbox task. It always needs the user's Apply.
- "It's still there" describes what was shown: the conversation, the saved handoff and the In progress task after a full restart.
