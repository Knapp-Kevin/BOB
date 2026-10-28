# B.O.B. alpha walkthrough: transcript and text alternative

Video: [`bob-alpha-walkthrough.mp4`](bob-alpha-walkthrough.mp4) (74 seconds, silent, captions in [`bob-alpha-walkthrough.en.vtt`](bob-alpha-walkthrough.en.vtt)). Poster frame: [`bob-alpha-walkthrough-poster.png`](bob-alpha-walkthrough-poster.png).

The recording has no audio. Every item and message shown is fictional demo data.

## Short text alternative

A screen recording of the B.O.B. alpha desktop app on Windows. The user captures a thought, classifies it, and accepts one suggested change to make a task the next action. They start that task, ask B.O.B. to break it down, and save their place. It ends on Settings, where optional inference providers are off by default.

## Transcript

| Time | What happens on screen |
| --- | --- |
| 0:00 | **Inbox.** Three fictional items: "Email the bike shop about the brake repair quote" (task), "Idea: color-code the pantry shelves" (idea), "Draft agenda for Thursday's volunteer meetup" (task). |
| 0:08 | The user types "Return library books before Friday" into Quick capture. It lands in the Inbox as a note, and the user changes its type to Reminder. |
| 0:17 | The user selects **Organize with B.O.B.** Chat opens with a **Preview before apply** card that proposes making the bike-shop email the next action and leaving every other item untouched. |
| 0:32 | The user selects **Apply this change.** Today shows the bike-shop email as the next action, one focus item in the remaining plan, and the volunteer-meetup agenda under "Resume where I left off". A notice confirms that only the previewed change was applied. |
| 0:41 | The user selects **Do it now**; the next action shows "In progress". |
| 0:44 | In **B.O.B. Chat**, the user selects **Break it down**. B.O.B. suggests spending five minutes finding the first concrete change. A status line confirms that local guidance is active and prompts are not sent to OpenRouter. |
| 0:56 | The user selects **Save my place**. A saved-handoff card records the objective, the current state ("In progress") and the next step, and notes that it is stored in B.O.B.'s local state. |
| 1:09 | **Settings.** The Inference section lists optional providers. OpenRouter shows no API key, no approved model, and cloud and billed use unchecked. |

## Limitations shown or implied

- This was recorded from a development (debug) build, not a signed installer.
- The Chat replies come from B.O.B.'s local, rule-based guidance; no AI model was used.
- The Chat panel in this recording still grows as messages arrive. That was fixed after recording, in `c68c86c`; still `04-chat-saved-handoff.png` shows the fixed layout.
