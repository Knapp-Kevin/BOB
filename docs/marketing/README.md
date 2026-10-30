# B.O.B. marketing media handoff

Draft material for the MythologIQ Labs site (issue #128). **Owner review is required before any public use.** Nothing here is published, deployed or announced.

- Asset list, checksums, provenance, alt text and captions: [`MANIFEST.md`](MANIFEST.md)
- Walkthrough video, captions, poster and transcript: [`video/`](video/)
- Screenshots: [`screenshots/`](screenshots/)
- Approved logo and mascot: [`docs/assets/`](../assets/README.md) (no SVG wordmark exists yet)

## Suggested product description (43 words)

> B.O.B. (Better Organized Brain) is a local-first desktop workbench for people whose brains don't do backlogs. Capture first, sort later, and see one next action at a time. B.O.B. previews changes before applying them and stays useful with no AI configured. Alpha.

This avoids clinical or efficacy claims. If the site wants to address neurodivergent users directly, an alternative is: "designed with ADHD and other neurodivergent brains in mind". That describes design intent, not outcomes.

## Proposed placement

| Site placement | Asset | Notes |
| --- | --- | --- |
| Product hero | `screenshots/01-today.png` | Overview of the main screen. Pair with the product description. |
| "Capture first" feature row | `screenshots/02-inbox-classified.png` | |
| "You stay in charge" feature row | `screenshots/03-chat-preview-before-apply.png` | Shows preview before apply. |
| "Pick up where you left off" feature row | `screenshots/04-chat-saved-handoff.png` | |
| "When it's too much" feature row | `screenshots/05-today-overwhelmed-mode.png` | Show next to `01-today.png` as an off/on comparison. |
| Privacy / cost note | `screenshots/06-settings-inference.png` | Optional providers stay off until configured. |
| Accessibility / data ownership note | `screenshots/07-settings-accessibility.png` | Larger text, reduced motion and data export. |
| Demo section | `video/bob-alpha-walkthrough.mp4` + poster + `.vtt` | Use `controls` and the poster; do not autoplay. Link `video/TRANSCRIPT.md` as the text alternative. |
| Logo / social | `docs/assets/bob-app-icon.png`, `docs/assets/bob-mascot-transparent.png` | Follow the role rules in `docs/assets/README.md`. |

Label everything as **Alpha**. Do not describe the product as released, signed or generally available.

## Current development status

- Alpha, Windows 11 x64 only. The media shows the installed beta-candidate package from PR #132, which passed installed-app qualification in a disposable Windows Sandbox (install, launch, single instance, restart, forced termination, corrupt-database start, uninstall and reinstall).
- The installer is unsigned and nothing has been released. Chat uses B.O.B.'s local guidance unless an optional provider is configured.

## Outstanding gates before public use

1. Owner review of every asset, caption and the product description.
2. PR #132 (and the OpenRouter work it carries) merged, so the site shows what `master` ships.
3. A signed, approved release before any download link is offered.
