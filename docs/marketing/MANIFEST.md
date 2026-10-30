# Marketing media manifest

Status: **draft — owner review required before any public use.** Tracks issue #128. Not a release, and not approval to publish.

## Provenance

| Field | Value |
| --- | --- |
| Repository | `Knapp-Kevin/BOB` |
| Captured from | The beta candidate on `integration/beta-candidate-20261028` (PR #132). |
| Build commit | `487f9fd`. Its product code (`src/`, `src-tauri/`) is identical to `a815ae1`, the last product change on that branch; later commits changed only scripts and docs. |
| Product / build | B.O.B. `0.0.1`, release build installed from the NSIS package `B.O.B._0.0.1_x64-setup.exe` (SHA-256 `c0fd1b1d1657a78350b4411b47b8c0d7fcb05dc4226ebc2d5880c20ef02cafb0`, unsigned). Installed `bob.exe` SHA-256 `bc5f9d9fea85dff1b0dced95b88bdd68eaf85109432dc1cdf23a3a9f522392be`. Built with `npm run package:windows` (Rust 1.88.0, Node 22.23.4, MSVC 14.44, Windows SDK 10.0.26100) in a disposable Windows Sandbox (`scripts/new-windows-sandbox-build.ps1`). |
| Capture environment | A separate disposable Windows Sandbox with networking, clipboard, printers, audio/video input and vGPU off: Windows 11 Enterprise 10.0.26100, WebView2 156.0.4314.50. The package was installed silently (`/S`) per user. The same package passed the installed-app qualification in `scripts/qualify-installed-windows.ps1` (PR #132). |
| Capture date | 2026-10-30 (UTC) |
| Viewport | Native window client area of exactly 1600×900. Captured inside the Sandbox with `ffmpeg` `gdigrab` from the client-area region only; every frame was reviewed to contain only B.O.B. Mouse and keyboard input was real input sent to the B.O.B. window inside the Sandbox. |
| Demo fixture | `fixture-128-b`: a fresh profile on its own profile root, no API keys. Three fictional items were written to its SQLite file before the take: "Email the bike shop about the brake repair quote" (task, high), "Idea: color-code the pantry shelves" (idea), "Draft agenda for Thursday's volunteer meetup" (task, Thursday). "Return library books before Friday" was typed in during the take. All Chat text is B.O.B.'s local, rule-based output for these items. |
| Inference | None. OpenRouter and Gemini were not configured, the Sandbox had no network, and no cloud request was possible. |
| Data classification | Synthetic or fictional only. No personal data, credentials, private URLs or real schedules. |
| Rights | Screenshots and video of the MIT-licensed B.O.B. interface, captured for the project owner; no third-party media, fonts beyond the app's own, or audio. |

## Assets

| Path | SHA-256 | Bytes | Alt text | Caption |
| --- | --- | --- | --- | --- |
| `screenshots/01-today.png` | `052a7558e75cbac433f35aaaf0a55320f4df10396ec0a53f4878efef6f1ce0ec` | 168830 | B.O.B. Today screen showing one next action, "Email the bike shop about the brake repair quote", with Do it now and Not now buttons, a quick-capture bar, a one-item remaining plan, and a resume card for a volunteer-meetup agenda. | Today keeps one next action in front, with the rest of the plan and a way back into unfinished work. |
| `screenshots/02-inbox-classified.png` | `67ced7a2f1f4a61851ca98c076730746646c6f80a802c3b7e8bba79047416f6f` | 144743 | B.O.B. Inbox listing four captured items labelled Reminder, Task, Idea and Task, each with a Type selector, beside an Inbox snapshot card and an Organize with B.O.B. button. | Capture first and decide what it is later; the Inbox makes sorting optional. |
| `screenshots/03-chat-preview-before-apply.png` | `d2ef082fa3f6e3b089e8befb1ba5dae03250fd0d6e4fb60a342ba8f7e085588e` | 170643 | B.O.B. Chat showing a "Preview before apply" card that proposes making one Inbox item the next action, with "Apply this change" and "Keep things as they are" buttons. | B.O.B. shows a proposed change before anything changes, and nothing happens until you accept it. |
| `screenshots/04-chat-saved-handoff.png` | `20fd726196d10ab7df7fac26538d2ccc336b7e87bdd0ee4b357c84cf88c31a6c` | 192941 | B.O.B. Chat with a short conversation and a Saved handoff card listing the objective, its In progress state and the next step; a Current context panel on the right offers Reduce this, Break it down, Reorient me, Help me decide and Save my place. | Save my place stores where you were and what comes next, so you can pick it back up later. |
| `screenshots/05-today-overwhelmed-mode.png` | `f2059a3ed96bd23d4eef1a5fdd6e98c01c20c672f615af9351220264129171ec` | 118741 | B.O.B. Today in Overwhelmed mode showing only the next action, marked In progress, and quick capture, with a notice that reduced-information mode is on and a Show full view button. | Overwhelmed mode cuts the screen down to the next action and a place to capture. Compare with the Today overview. |
| `screenshots/06-settings-inference.png` | `67965d09732154f14b63fcab8be5c80216200a3e85e15411d1ee1d9bd8a501cb` | 146522 | B.O.B. Settings with an Inference section; the OpenRouter card shows no API key, an empty model field, cloud-transmission and billed-inference permissions unchecked, and the per-session Use OpenRouter in Chat switch off. | Optional inference providers are listed separately and stay off until you configure them. |
| `screenshots/07-settings-accessibility.png` | `cc44608d98be8973e329ad4f3c266a1a38e484d2a878c542889b300f9e83e1c1` | 161683 | Lower part of B.O.B. Settings: an unconnected Gemini API card, an Accessibility card with Larger interface text and Reduced motion checkboxes, and a Data ownership card with an Export my data button. | Readability settings and a full data export sit next to the optional providers. |
| `video/bob-alpha-walkthrough.mp4` | `00f01e6207bb4d457ca2da44c71496c1e3bb8aea5ca695504099e9249b3a8e99` | 1454795 | See `video/TRANSCRIPT.md`. | A 60-second walkthrough: capture, classify, preview and apply, start, break it down, save your place, restart and resume. |
| `video/bob-alpha-walkthrough-poster.png` | `522ea1a526887d7b953e770f94632e2a95036e93d546795856ef2af3fcc9aee1` | 383671 | Poster frame: B.O.B. Chat after a restart, still showing the conversation and the Saved handoff card. | — |
| `video/bob-alpha-walkthrough.en.vtt` | see `git log` | — | English captions describing on-screen actions (the video is silent). | — |
| `video/TRANSCRIPT.md` | see `git log` | — | Transcript and short text alternative. | — |

Text files are listed without a checksum because Git may convert their line endings on checkout.

Video details: H.264 High profile, yuv420p, 1600×900, 15 fps, 60.5 s, no audio track, `faststart` for web playback. It joins two continuous takes with one cut at 0:43.9, where B.O.B. was closed and reopened to show that the task and handoff survive a restart. Nothing else was edited.

## Superseded media

The 2026-10-28 set captured from a debug build (`npx tauri dev`) on commits `6ea9e2b` and `c68c86c` is replaced in full; it remains in Git history. The new set comes from the installed release package, shows the fixed-height Chat panel, and includes the Settings accessibility controls.

## Logo and wordmark

No new artwork was made. Use the approved assets in [`docs/assets/`](../assets/README.md): `bob-app-icon.png` (framed application mark) and `bob-mascot-transparent.png` (transparent mascot), following the role rules there. **No SVG logo or standalone wordmark exists yet**; the "B.O.B. / Better Organized Brain" lockup in the sidebar is live text, not an asset. A vector wordmark needs owner-approved design work.

## Known limitations of this media

- The installer is unsigned and the product is an alpha. Do not offer a download link until signing and release are approved.
- No AI inference appears; Chat output is B.O.B.'s deterministic local guidance.
- The Sandbox desktop has no personalization, so the window frame and cursor use Windows defaults.
- Accessibility, security and performance are not claimed by this media.

## Checks performed

- Each PNG was opened and inspected at full size; `ffprobe` confirms 1600×900.
- The MP4 decodes end to end (`ffmpeg -f null`). Contact sheets of both raw takes (one frame every 2 s) showed only B.O.B. content.
- Caption timings were set from `ffmpeg` scene-change detection, and a frame at each caption's midpoint was checked against its text.
- SHA-256 values above were computed from the committed binary files.
- Relative links in this file, `README.md` here, and the repository README resolve to files in the repository.
