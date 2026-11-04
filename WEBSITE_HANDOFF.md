# B.O.B. product page: narrated film handoff

**Film:** "B.O.B.: Find Your Next Step" (narrated, 90.6 s)

**Status:** the owner approved this cut on 2026-11-04 ("Approved as cut"). Checksums, provenance and alt text are in `media-manifest.json`.

## What it shows

The film was recorded in the **installed, signed B.O.B. Windows Alpha v0.1.0**. It follows one fictional person, Sam, through a crowded Tuesday. All data is fictional demo data.

1. **Overwhelm:** Today shows three planned tasks competing for attention.
2. **Capture:** a stray thought goes into Quick capture and lands in the Inbox as a Note.
3. **Choice:** Sam sets its type (Reminder), then asks Organize with B.O.B. for help. B.O.B. shows a **Preview before apply** card for one next action, and nothing changes until Sam applies it.
4. **Action:** Overwhelmed mode leaves only the next step and capture. Sam starts the task.
5. **Confidence:** Break it down and Save my place, then B.O.B. is closed and reopened. The conversation, the saved handoff and the in-progress task are all still there.

**Audio:**
- The narration was made in Sonic Rift 0.9.1 with Qwen3 voice design and cloning (Apache-2.0).
- The original music was made in Sonic Rift Music with ACE-Step 1.5 (MIT).
- The opener is the shared MythologIQ Labs opener (silver mark).

## Placement

| Page slot | File | Notes |
| --- | --- | --- |
| Product video | `video/bob-find-your-next-step.mp4` | H.264 High plus AAC, 1920×1080, 30 fps, faststart. Ship with controls and the captions track. Do not autoplay with sound. |
| Captions | `video/bob-find-your-next-step.vtt` | WebVTT, 16 cues, timed to the narration. |
| Poster | `video/bob-find-your-next-step-poster.png` | Frame at 0:37.6: the Preview before apply card, with Apply this change and Keep things as they are. |
| Muted / autoplay loop (optional) | `video/bob-find-your-next-step-silent-captioned.mp4` | The same picture with captions burned in and no audio track. Use it only where a muted preview is wanted; link to the narrated film. |
| Accessible alternative | `video/bob-find-your-next-step-transcript.md` | Narration plus a description of each scene. Link it next to the video. |
| Identity | `brand/bob-mascot-transparent.png` | The approved transparent mascot, unchanged. |

```html
<video controls preload="metadata" width="1920" height="1080"
       poster="bob-find-your-next-step-poster.png">
  <source src="bob-find-your-next-step.mp4" type="video/mp4">
  <track kind="captions" srclang="en" label="English" src="bob-find-your-next-step.vtt">
</video>
<p><a href="bob-find-your-next-step-transcript.html">Read the transcript</a></p>
```

The film says "Bob" aloud. Captions and on-screen text write the name as "B.O.B.".

## Claims the page must not make

- **No AI claims:** in v0.1.0, Chat's guidance is local and deterministic. Optional API providers exist but are off by default and do not appear in the film. Organize with B.O.B. proposes the highest-priority inbox task, and the user must apply it.
- **No automatic sorting:** the user sets each item's type. B.O.B. does not classify captured items.
- **No clinical or outcome claims:** no diagnosis, treatment, or guaranteed productivity or mental-health improvement. The film describes design intent: fewer things competing for attention, one next step.
- **No persistent Overwhelmed mode:** it lasts for the session. What the film shows surviving a restart is the saved handoff, the conversation and the in-progress task.
- **Windows Alpha is a pre-release:** describe it as an alpha. Download: https://github.com/Knapp-Kevin/BOB/releases/tag/v0.1.0 (direct installer: https://github.com/Knapp-Kevin/BOB/releases/download/v0.1.0/B.O.B._0.1.0_x64-setup.exe).

## Known product copy issue visible in the film

In the last two scenes, a small Chat reply says "Current state: doing" while the card beside it says "In progress" ([#146](https://github.com/Knapp-Kevin/BOB/issues/146)). It is not narrated. The owner approved the cut with it visible.

## Retrieve

Use the pinned commit given in the repository's `docs/marketing/BOB_FILM_HANDOFF.md` on the production branch:

```
https://raw.githubusercontent.com/Knapp-Kevin/BOB/<commit>/<path>
```
