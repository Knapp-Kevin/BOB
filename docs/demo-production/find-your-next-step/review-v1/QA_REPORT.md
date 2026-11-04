# QA report: "B.O.B.: Find Your Next Step", first review cut v1 (2026-11-04)

**Status: private first review cut, awaiting owner review. Not approved, not published.**

- **Branch:** `demo/find-your-next-step-v1-20261103`, from `master` at `ecce695`, the `v0.1.0` tag commit.
- **Script and storyboard:** [`../SCRIPT_AND_STORYBOARD.md`](../SCRIPT_AND_STORYBOARD.md).
- **Binaries stay outside Git:** `C:\dev\mythologiq\bob-media-handoff\bob-find-your-next-step-v1-review\first-review\`, checked against `SHA256SUMS.txt`.

## The film

| Property | Value |
| --- | --- |
| Length | 90.6 s |
| Video | 1920×1080, 30 fps, H.264 High, `yuv420p` |
| Audio | AAC 48 kHz stereo |

Story: Sam, a fictional adult, gets one Tuesday under control. The arc is overwhelm (a full Today), then capture, then classify and ask, then preview and apply, then Overwhelmed mode and Do it now, then Break it down and Save my place, then a restart that keeps everything, then the branded close.

## Application qualification (Phase 1)

| Item | Evidence |
| --- | --- |
| Release | GitHub pre-release [`v0.1.0`](https://github.com/Knapp-Kevin/BOB/releases/tag/v0.1.0), tag on `ecce695` |
| Installer | `B.O.B._0.1.0_x64-setup.exe`, SHA-256 `36015f6a0edec34cf9157959e623680f9fab02ce0e77304d4f243b9273b87b2b`. Downloaded from the release, matches its `.sha256`, and `signtool verify /pa` passes (signer MythologIQ Labs, LLC, timestamped) |
| Installed app (`source-capture/install-provenance.json`) | Silent per-user install, exit 0. `bob.exe` SHA-256 `63cd5110…8f8d64`, identical to the release job's installed-exe hash. FileVersion and ProductVersion `0.1.0`. Authenticode **Valid** |
| Earlier qualification | The same installer passed 20/20 installed-app checks on 2026-11-02 (recorded on PR #142) |
| Host | Offline Windows Sandbox: Windows 11 Enterprise 10.0.26100, no network adapters up. Clipboard, printers, audio/video input and vGPU off. The host's own B.O.B. profile was never read or written |
| Profile | A separate fictional profile under `C:\demo` inside the Sandbox. The fixture is written to its SQLite file before the take (`scripts/demo/windows/film-session.ps1`, `fixture-film-sam`) |
| Rehearsal | `source-capture/rehearsal/r01`–`r15`. Each scene's behavior was checked on the installed build before scripting (see the table in the storyboard) |

## Capture (Phase 4)

- **Tooling:** `scripts/demo/windows/film-session.ps1` runs inside the Sandbox. It moves the real mouse and sends real keystrokes (SetCursorPos/mouse_event, SendKeys) to the B.O.B. window, and records the 1920×1080 client area with ffmpeg `gdigrab` at 30 fps, real cursor included. A one-second black sync flash gives each take its zero, and every scene is marked against it.
- **Take 1 (`film-take.mkv`)** covers S01–S06. After the restart, its click on Today did not register: the cursor glided and clicked, but the app stayed on Chat. This was a **recording-input timing problem**: the same click worked in the rehearsal and in take 2. It is not a product defect.
- **Take 2 (`film-take2.mkv`)** was recorded in the same Sandbox session and profile, so it continues the same state. B.O.B. is closed and reopened, then Chat and Today are shown. The film uses take 2 for S07.
- **Cuts:** four segments, joined by three 0.4 s dissolves. The cuts remove only waiting: 2.1 s before typing starts, 1.0 s after the Do it now message, and the close-and-reopen itself (replaced by take 2's post-restart footage, with the on-screen note "B.O.B. was closed and reopened"). Picture is never retimed, frozen or altered.

## Narration (Phase 3): Sonic Rift 0.9.1, installed app, off-screen

- **Voice design:** Voices › New voice › Design with **Qwen3 voice design** (`Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign`, Apache-2.0). Four designs came from one written description: a warm, calm, clear adult narrator speaking North American English, conversational and reassuring, never an announcer. No real person was imitated. Design 2 was chosen: the slowest pace (3.6 words/s), mid pitch, and a complete transcript.
- **Narrator:** design 2 was cloned and saved as "B.O.B. narrator".
- **Lines:** Speech with **Qwen3 voice cloning** (`Qwen/Qwen3-TTS-12Hz-1.7B-Base`, Apache-2.0) produced 4 takes per line, 32 in all. S06 was then rewritten longer, so the Break it down and Save my place actions are narrated, and given 4 more takes. Multi-sentence lines were entered with paragraph breaks, so Speech puts its paragraph pause between sentences; the words are unchanged.
- **Selection:** by measurement, recorded in `audio/narration-selected/selection.csv` and `scores.json`. Each chosen take has complete wording heard by Sonic Rift's transcriber, sentence pauses, a calm pace, no clipping, and voice similarity of 0.91–0.95 to the design.
- **Spoken name:** the narration says "Bob". Captions write "B.O.B.". **Owner check needed.**

## Music (Phase 3): Sonic Rift Music, installed app

- **Engine:** **ACE-Step 1.5 (2B turbo)**, `ACE-Step/Ace-Step1.5` @ `19671f406d603126926c1b7e2adc169acbcade22`, **MIT License**.
- **Request:** Instrumental, Plan first, 115 s, 72 BPM, F major, 4/4, base seed 20261104. The brief asked for soft felt piano and restrained ambient pads, starting sparse and hesitant, then a quiet forward pulse with harmony that slowly warms, resolving calmly, and no vocals, drums or swells. The full request is in `audio/music/log_dark.json`.
- **Rejected first request:** its brief was over Sonic Rift's 512-character limit, and the app refused it before generating anything (`audio/music-rejected-too-long/`).
- **Selected: take D** (seed 20261107), for its loudness arc:
  - near-silent and sparse for about 45 s, under the crowded day;
  - restrained through the middle;
  - building steadily from about 70 s to 95 s, under the restart and resolution.

  All four takes are kept.
- **Vocals:** Sonic Rift's transcriber "hears" phrases like "Thank you" in every take, which is Whisper's known hallucination on instrumental audio. The takes were generated with Instrumental on.

## Edit (Phase 5)

`scripts/demo/film/build_film.py` builds the film with ffmpeg alone. The full decision list is in `edit-decision-list.json`.

- **Opener (0–2.6 s):** the shared MythologIQ Labs opener. It uses the same `make_opener.py` as the Sonic Rift v0.9.1 final cut and the Sojourner film (byte-identical, SHA-256 `ff75f1fb…`), the approved silver mark `mythologiq-silver-concept.webp` (SHA-256 `c0141a60…94b2`) and DM Sans (OFL). It is re-rendered only so that its dissolve lands on B.O.B.'s first frame. It is silent.
- **Narration placement:** each line starts 0.6 s after its scene mark. S06 is pinned so that "And before stepping away…" starts just before Save my place is clicked.
- **Edit notes**, in the empty middle of the sidebar, never over page content:
  - "Installed B.O.B. Windows Alpha v0.1.0. Sam is fictional; all data is demo data." (0:03–0:13)
  - "B.O.B. was closed and reopened." (1:09–1:14)
- **Music bed:** one continuous take from 0:02.6. It is never restarted, runs through a gentle bed compressor at base −11 dB, and has per-line dips. It fades out only over the end card.
- **End card:** typeset from the approved `docs/assets/bob-mascot-transparent.png` on the app's navy (`scripts/demo/film/end_card.py`). No artwork was generated.

## Technical QA (Phase 6): measured on the delivered files

| Check | Result |
| --- | --- |
| Decode, streams | **Pass**. Main: 90.62 s, H.264 High 1920×1080 30 fps `yuv420p` (tv range), AAC LC 48 kHz stereo, faststart. Silent-captioned: same picture plus burned-in captions, no audio stream |
| Loudness | **Pass**: −17.3 LUFS integrated, true peak −1.5 dBFS, LRA 11.7 LU, 0 clipped samples. Target −16 LUFS / −1.5 dBTP, within 1.5 LU |
| Narration over music, per line (RMS) | **Pass**: S01 16.7, S02 16.9, S03 13.9, S04 15.3, S05 16.8, S06 15.8, S07 13.8, S08 13.9 dB |
| Overlapping narration | **None** |
| Intelligible in the final mix | **Pass**: Sonic Rift's transcriber heard all 8 lines in the mastered mix with **no missing words** (`qa-transcription-final-mix.json`) |
| Silence below −50 dBFS for 0.75 s or more | 0–3.55 s (the silent opener into the bed's quiet start) and 4.70–5.45 s (the pause after "This is Sam's Tuesday", while take D is still near-silent). Judge whether that opening pause feels too bare |
| Music continuity | **Pass**: one take, no restarts, no fades between scenes |
| Black frames | Only 0–0.57 s, the opener's dark canvas before the mark |
| Static holds (`freezedetect` n=0.001, ≥4 s) | 3.0–13.0 (Today, S01), 33.2–42.1 (preview card, S04), 44.5–55.4 (Today and Overwhelmed mode, S05), 70.7–76.6 (Chat after restart, S07). Each is a **narrated reading hold** on a responsive app where only the cursor moves. No loading indicator or app freeze appears |
| Captions | 16 WebVTT cues taken from the placed takes' own pauses: 0 overlaps, shortest 1.45 s, longest 89 characters. The burned-in version uses the same cues |
| Claims | No AI or inference shown or implied. Chat shows "Local B.O.B. guidance active." "Suggests one next step" is Organize's deterministic proposal, which always needs Apply. No diagnosis, treatment, productivity or mental-health claims |
| Build identity | The version text in the film (sidebar note, end card) matches the installed exe (`0.1.0`) |
| Private data | None. No keys, accounts or real people. The Sandbox had no network |
| Earlier media | Untouched: the 2026-10-30 screenshots and silent walkthrough on `docs/marketing/` were not modified |

## Defects found (Phase 4)

| Finding | Type | Action |
| --- | --- | --- |
| The Save my place reply says "Current state: doing"; the handoff card says "In progress" | Product copy defect | [#146](https://github.com/Knapp-Kevin/BOB/issues/146) filed. It is visible as small reply text in S06/S07 and is not narrated or highlighted. If it is fixed before publication, those scenes can be re-recorded |
| Take 1's Today click after the restart did not register | Recording problem | Re-recorded as take 2 in the same session |
| Overwhelmed mode does not survive a restart | Expected (per-session) | Not claimed in the film |

## Watched and not watched

- **Watched, as frames:**
  - contact sheets every 2 s across the whole cut;
  - every cut point, the opener, the poster, burned-in caption frames, the sidebar notes and the end card;
  - every rehearsal and take frame used for scripting.
- **Audio:** checked by measurement and by transcribing the final mix.
- **Not done:** I cannot listen. The narrator's warmth and delivery, the pronunciation of "Bob", the music's feel and the overall mix balance need a human listen before approval.
