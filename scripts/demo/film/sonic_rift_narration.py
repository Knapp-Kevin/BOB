"""Narration for "B.O.B.: Find Your Next Step", made in the installed Sonic Rift 0.9.1 (its own Python, models and
UI code; the window stays off-screen). Nothing here synthesizes audio itself: every WAV is a take the app
saved.

    PHASE=design  Voices > New voice > Design with Qwen3 voice design: several narrator voices from one
                  written description (each take is a different voice).
    PHASE=lines   Voices > New voice > Clone the picked design (clip as is, Says = the design passage) and
                  save it as the narrator, then Speech with Qwen3 voice cloning: TAKES takes of each line.

    QT_SCALE_FACTOR=1.5 MEDIA_PROFILE=<fresh profile> PHASE=... [DESIGN=<picked design wav>] [TAKES=4] \
        "<install>\\.venv\\Scripts\\python.exe" sonic_rift_narration.py "<install>" <out dir> dark 1600x900

Adapted from the Sonic Rift review film tooling (demo-v091/tools/gen_guide.py), which drives the same app.
"""
import glob
import json
import os
import shutil
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sonic_rift_harness import *  # noqa: E402,F401,F403

PHASE = os.environ["PHASE"]
TAKES = int(os.environ.get("TAKES", "4"))
DESCRIPTION = os.environ.get("DESCRIPTION", (
    "A warm, thoughtful adult narrator speaking natural North American English. Calm, quietly confident and "
    "conversational, like a storyteller talking to one person, never a commercial announcer. Unhurried pace, "
    "gentle warmth, clear articulation, a relaxed mid-range voice."))
# The final script's lines, {"S01": "...", ...}, from LINES_FILE (UTF-8 JSON).
LINES = json.load(open(os.environ["LINES_FILE"], encoding="utf-8-sig")) if os.environ.get("LINES_FILE") else {}
NARRATOR = os.environ.get("NARRATOR", "B.O.B. narrator")
ONLY = [s for s in os.environ.get("ONLY", "").split(",") if s]
# Same words, laid out in paragraphs: Speech's finishing puts its paragraph pause between them.
PARAGRAPHS = json.loads(os.environ.get("PARAGRAPHS", "{}"))
TAKE_PREFIX = os.environ.get("TAKE_PREFIX", "")


def pick_engine(combo, label):
    for index in range(combo.count()):
        entry = next((e for e in w.model_entries if w.entry_key(e) == combo.itemData(index)), None)
        if entry and entry.get("label") == label:
            combo.setCurrentIndex(index)
            return entry
    raise RuntimeError(f"no {label} entry in the Studio engine list")


def design():
    from this_voice_thing.core import voice_studio
    w.studio_play_take = lambda *a, **k: None
    w.show_page(w.PAGE_STUDIO)
    w.set_studio_mode("design")
    entry = pick_engine(w.studio_engine_combo, "Qwen3 voice design")
    w.studio_description_input.setText(DESCRIPTION)
    w.studio_count_spin.setValue(TAKES)
    w.studio_make_takes()
    done = wait_for(lambda: sum(1 for t in w.studio_takes if t.origin == "design") >= TAKES
                    and getattr(w, "studio_thread", None) is not None and not w.studio_thread.isRunning(), 3600, 0.5)
    if done is None:
        raise RuntimeError("design takes did not finish")
    takes = []
    for n, take in enumerate(t for t in w.studio_takes if t.origin == "design"):
        dest = os.path.join(OUT, f"narrator-design-{n + 1}.wav")
        shutil.copyfile(take.path, dest)
        takes.append({"file": dest, "app_output": take.path, "label": take.label, "says": take.text,
                      "seconds": round(take.seconds, 2)})
    shot("narrator_design_page")
    LOG["design"] = {"engine": entry["label"], "repo_id": entry["repo_id"], "license": entry.get("license"),
                     "description": DESCRIPTION, "passage": voice_studio.DESIGN_PASSAGE, "takes": takes}


def lines():
    from this_voice_thing.core import voice_studio
    from this_voice_thing.ui.pages import studio as studio_page
    picked = os.environ["DESIGN"]
    w.load_model(next(e for e in w.model_entries if e.get("label") == "Qwen3 voice cloning"))
    wait_for(lambda: w.model is not None and not w.model_is_loading, 1800, 0.3)

    class _SaveVoice:
        def __init__(self, _title, voice, *_a, **_k):
            self.voice = voice

        def exec(self):
            return 1

        def apply(self):
            self.voice.name = NARRATOR
            self.voice.tags = ["Find Your Next Step", "synthetic", "designed"]
    studio_page.VoiceDetailsDialog = _SaveVoice
    w.studio_play_take = lambda *a, **k: None
    w.show_page(w.PAGE_STUDIO)
    w.set_studio_mode("clone")
    w.studio_set_source(picked, voice_studio.DESIGN_PASSAGE, f"{NARRATOR} (designed)")
    row = max(i for i, take in enumerate(w.studio_takes) if take.origin == "source")
    w.studio_take_list.setCurrentRow(row)
    w.studio_save_voice()
    narrator = next(v for v in w.voice_library.voices if v.name == NARRATOR)
    w.take_audio_out.setVolume(0.0)
    w.show_page(w.PAGE_GENERATE)
    w.use_voice(narrator)
    results = {}
    for scene, line in LINES.items():
        if ONLY and scene not in ONLY:
            continue
        text = PARAGRAPHS.get(scene, line)
        if " ".join(text.split()) != line:
            raise RuntimeError(f"{scene}: paragraph layout changes the words")
        w.text_input.setPlainText(text)
        takes = []
        for n in range(TAKES):
            before = w.current_audio_file
            w.start_generation()
            if wait_for(lambda: not w.is_generating and w.current_audio_file != before, 1800, 0.1) is None:
                raise RuntimeError(f"{scene} take {n + 1} did not finish")
            dest = os.path.join(OUT, f"VO_{scene}_{TAKE_PREFIX}take{n + 1}.wav")
            shutil.copyfile(w.current_audio_file, dest)
            takes.append({"file": dest, "app_output": w.current_audio_file})
            w.stop_audio()
        results[scene] = {"line": line, "text_as_entered": text, "takes": takes}
        LOG["lines"] = results
        save_log()
    shot("narrator_speech_page")
    LOG["lines"] = results
    LOG["narrator_voice"] = {"name": narrator.name, "kind": narrator.kind, "origin": narrator.origin,
                             "clip": narrator.clip, "source_design": picked, "model": "Qwen3 voice cloning"}


try:
    from this_voice_thing import __version__
    LOG["app"] = {"version": __version__, "root": ROOT, "python": sys.executable, "profile": os.environ.get("MEDIA_PROFILE")}
    LOG["phase"] = PHASE
    design() if PHASE == "design" else lines()
except Exception:
    LOG["exception"] = traceback.format_exc()
finish()
