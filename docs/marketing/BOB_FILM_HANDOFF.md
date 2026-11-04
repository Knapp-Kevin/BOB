# B.O.B. narrated film handoff: "Find Your Next Step"

The owner approved the film as cut on 2026-11-04. The approved files are on the assets-only branch `media/bob-find-your-next-step-2026-11-04`. That branch is never merged.

| Item | Value |
| --- | --- |
| Pinned commit | `09f2bf9349aa07aa251ee6d7f48f639591fdda72` |
| Manifest | `media-manifest.json` on that commit: SHA-256, size, dimensions, duration, provenance and alt text for each file |
| Website instructions | `WEBSITE_HANDOFF.md` on that commit: placement, markup and the claims the page must not make |
| Film | `video/bob-find-your-next-step.mp4`: 90.6 s, 1920×1080, 30 fps, H.264 High plus AAC, SHA-256 `e7d9b2e6839166b3cddc847d1683133e753b5766c0dea4f8e2738945543ec0b5` |
| Captions | `video/bob-find-your-next-step.vtt` (16 cues) |
| Silent captioned version | `video/bob-find-your-next-step-silent-captioned.mp4` |
| Poster | `video/bob-find-your-next-step-poster.png` (0:37.6) |
| Transcript | `video/bob-find-your-next-step-transcript.md` |
| Application | Installed, signed B.O.B. Windows Alpha [`v0.1.0`](https://github.com/Knapp-Kevin/BOB/releases/tag/v0.1.0) |

Retrieve a file at the pinned commit:

```
https://raw.githubusercontent.com/Knapp-Kevin/BOB/09f2bf9349aa07aa251ee6d7f48f639591fdda72/<path>
```

`raw.githubusercontent.com` serves the MP4 as `application/octet-stream` with `nosniff`, so it is for downloading, not playback. The website should copy the files into its own hosting and check them against the manifest.

Production records are in [`../demo-production/find-your-next-step/`](../demo-production/find-your-next-step/): the storyboard, QA report and transcript. The full 149-file review package (stems, every take, the edit decision list) stays on the owner's workstation.
