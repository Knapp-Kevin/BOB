# "B.O.B.: Find Your Next Step": first review cut (v1, 2026-11-04)

This is the narrated B.O.B. product film. It was recorded from the installed, signed Windows Alpha [`v0.1.0`](https://github.com/Knapp-Kevin/BOB/releases/tag/v0.1.0), with narration and music made in Sonic Rift 0.9.1. **Status: private review cut, awaiting owner review. Not approved and not published.**

- **Review page:** a private claude.ai artifact, owner only: <https://claude.ai/artifact/XRZUXThFZysfyQpaBi12Hn>. It has the narrated film, the silent captioned version, captions, the poster, the story table and a provenance summary.
- **Binaries stay outside Git** (this repository is public): `media-handoff` on the owner's workstation, at `C:\dev\mythologiq\bob-media-handoff\bob-find-your-next-step-v1-review\first-review\`. That is 149 files, checked against `SHA256SUMS.txt`, plus `MANIFEST.json`.
- **Text records here:**
  - [QA report](./QA_REPORT.md)
  - [Transcript with scene descriptions](./transcript.md)
  - [Storyboard and final script](../SCRIPT_AND_STORYBOARD.md)
- **Tooling:**
  - `scripts/demo/windows/`: the offline film Sandbox, the capture helper and the take command lists.
  - `scripts/demo/film/`: Sonic Rift narration and music drivers, take scoring, the end card, the edit, QA and packaging.
- **Defect found while recording:** [#146](https://github.com/Knapp-Kevin/BOB/issues/146).

After owner approval, the approved film goes to an assets-only media branch with a pinned commit, a full manifest and website integration notes, as was done for Sojourner. This branch does not touch the website.
