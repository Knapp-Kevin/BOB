# Marketing media manifest

Status: **draft — owner review required before any public use.** Tracks issue #128. Not a release, and not approval to publish.

## Provenance

| Field | Value |
| --- | --- |
| Repository | `Knapp-Kevin/BOB` |
| Captured from | `feature/inference-model-authorization-20261028` (draft PR #127, stacked on integration PR #126). That branch contains everything on `master` and the integration branch. |
| Commits | `6ea9e2b` for the video and stills 01, 02, 03 and 06. `c68c86c` (the Chat panel fix, captured from an identical working tree just before it was committed) for stills 04 and 05. |
| Product / build | B.O.B. `0.0.1`. Native Tauri 2 debug build run with `npx tauri dev` (Rust `dev` profile, Vite dev server, WebView2). Not a signed or NSIS-packaged build. |
| OS | Windows 11 Pro 10.0.26300, 3440×1440 primary display |
| Capture date | 2026-10-28 |
| Viewport | Native window client area of exactly 1600×900. Captured with `ffmpeg` `gdigrab` from the desktop region while the window was pinned on top; every frame was reviewed to contain only B.O.B. |
| Demo fixture | `fixture-128-a`: a fresh local profile with no API keys and four fictional items: "Draft agenda for Thursday's volunteer meetup", "Email the bike shop about the brake repair quote", "Idea: color-code the pantry shelves", "Return library books before Friday". All Chat text is B.O.B.'s local, rule-based output for these items. |
| Inference | None. OpenRouter and Gemini were not configured, and no cloud request was made. |
| Data classification | Synthetic or fictional only. No personal data, credentials, private URLs or real schedules. |
| Rights | Screenshots and video of the MIT-licensed B.O.B. interface, captured for the project owner; no third-party media, fonts beyond the app's own, or audio. |

## Assets

| Path | SHA-256 | Bytes | Alt text | Caption |
| --- | --- | --- | --- | --- |
| `screenshots/01-today.png` | `b21956e80ed614095daa45fc8a7cb8f3f760761ef7e9bfdef7abd4f545e9a565` | 166768 | B.O.B. Today screen showing one next action, "Email the bike shop about the brake repair quote", with Do it now and Not now buttons, a quick-capture bar, a one-item remaining plan, and a resume card for a volunteer-meetup agenda. | Today keeps one next action in front, with the rest of the plan and a way back into unfinished work. |
| `screenshots/02-inbox-classified.png` | `a20e9ef44aea9a57f1671e06231c2bc5173b09e6e2a3adfcf53537d59056dda1` | 137830 | B.O.B. Inbox listing three captured items labelled Task, Idea and Task, each with a Type selector, beside an Inbox snapshot card and an Organize with B.O.B. button. | Capture first and decide what it is later; the Inbox makes sorting optional. |
| `screenshots/03-chat-preview-before-apply.png` | `0a8d0a57f26f6702ac10275405803efcb91970d3932d76e5fd3156f30bdf550f` | 166705 | B.O.B. Chat showing a "Preview before apply" card that proposes making one Inbox item the next action, with "Apply this change" and "Keep things as they are" buttons. | B.O.B. shows a proposed change before anything changes, and nothing happens until you accept it. |
| `screenshots/04-chat-saved-handoff.png` | `f2182909b246baecb11f490ef0c084eb8e7fe54ac1424f0d4b759dde5d61ad3e` | 192968 | B.O.B. Chat with a short conversation and a Saved handoff card listing the objective, its In progress state and the next step; a Current context panel on the right offers Reduce this, Break it down, Reorient me, Help me decide and Save my place. | Save my place stores where you were and what comes next, so you can pick it back up later. |
| `screenshots/05-today-overwhelmed-mode.png` | `9a0cad580e85f8e63e7a5c63cada18e38bb5d14d223fd122578cc504cdf56146` | 120197 | B.O.B. Today in Overwhelmed mode showing only the next action and quick capture, with a notice that reduced-information mode is on and a Show full view button. | Overwhelmed mode cuts the screen down to the next action and a place to capture. Compare with the Today overview. |
| `screenshots/06-settings-inference.png` | `28b7c3cced2fd984e108abcc993f9afc5d9c00461a46e93626e59e1f3df38b28` | 146923 | B.O.B. Settings with an Inference section; the OpenRouter card shows no API key, an empty model field, and cloud-transmission and billed-inference permissions unchecked. | Optional inference providers are listed separately and stay off until you configure them. |
| `video/bob-alpha-walkthrough.mp4` | `9da9fc83e4c0e8c6046071f25d3a45d0252956b68721f7cf8cb790c8118cbb05` | 1088430 | See `video/TRANSCRIPT.md`. | A 74-second walkthrough: capture, classify, preview and apply, start, break it down, save your place. |
| `video/bob-alpha-walkthrough-poster.png` | `53763e745b9c57c7286ca086d7720dcff2627fc3c323c41d5b18bd8ece077d4e` | 286926 | Poster frame of the Today screen right after a previewed change was applied. | — |
| `video/bob-alpha-walkthrough.en.vtt` | see `git log` | — | English captions describing on-screen actions (the video is silent). | — |
| `video/TRANSCRIPT.md` | see `git log` | — | Transcript and short text alternative. | — |

Video details: H.264 High profile, 1600×900, 15 fps, 74.3 s, no audio track, `faststart` for web playback. It was edited from a continuous 83 s take with one cut (38–47 s, idle time on Today); nothing else was altered.

## Logo and wordmark

No new artwork was made. Use the approved assets in [`docs/assets/`](../assets/README.md): `bob-app-icon.png` (framed application mark) and `bob-mascot-transparent.png` (transparent mascot), following the role rules there. **No SVG logo or standalone wordmark exists yet**; the "B.O.B. / Better Organized Brain" lockup in the sidebar is live text, not an asset. A vector wordmark needs owner-approved design work.

## Known limitations of this media

- Debug build. The signed or packaged installer has not passed native release qualification (NSIS install, relaunch and uninstall; startup recovery).
- The Settings screenshot and the OpenRouter controls come from unmerged draft PR #127.
- No AI inference appears; Chat output is B.O.B.'s deterministic local guidance.
- The video predates the Chat panel height fix (`c68c86c`), so the panel grows as messages arrive.
- Accessibility, security and performance are not claimed by this media.

## Checks performed

- Each PNG opened and was inspected at full size; `ffprobe` confirms 1600×900.
- The MP4 decodes end to end (`ffmpeg -f null`). Frames around the edit were inspected. Contact sheets of the full raw take (one frame every 2 s) showed only B.O.B. content.
- Caption timings were set from `ffmpeg` scene-change detection and match the on-screen action.
- Relative links in this file, `README.md` here, and the repository README resolve to files in the repository.
