"""Intelligibility check: Sonic Rift's own transcriber listens to each narrated line in the mastered mix (narration
and music together) and reports any expected word it did not hear. Measures only.

    python qa_transcribe.py "<Sonic Rift install>" <build dir> <spoken-lines.json>
"""
import json
import os
import re
import sys

import numpy as np
import soundfile as sf
from scipy.signal import resample_poly

APP, BUILD, SPOKEN = sys.argv[1:4]
sys.path.insert(0, APP)
from this_voice_thing.core import transcription  # noqa: E402

edl = json.load(open(os.path.join(BUILD, "edit-decision-list.json"), encoding="utf-8"))
spoken = json.load(open(SPOKEN, encoding="utf-8-sig"))
mix, rate = sf.read(os.path.join(BUILD, "stems", "mix-master.wav"), dtype="float32", always_2d=True)
mono = mix.mean(axis=1)
T = transcription.Transcriber()


def words(text):
    return re.sub(r"[^a-z0-9' ]", " ", text.lower().replace("’", "'")).split()


rows = []
for p in edl["narration"]:
    a, b = int(max(0, p["at"] - 0.2) * rate), int((p["ends"] + 0.3) * rate)
    heard = T.transcribe(resample_poly(mono[a:b], 16000, rate).astype(np.float32), language="en").text.strip()
    want = words(spoken[p["scene"]])
    got = words(heard)
    rows.append({"scene": p["scene"], "expected": spoken[p["scene"]], "heard": heard,
                 "missing_words": [w for w in want if w not in got]})
    print(p["scene"], "missing:", rows[-1]["missing_words"], "|", heard)
json.dump(rows, open(os.path.join(BUILD, "qa-transcription-final-mix.json"), "w", encoding="utf-8"), indent=1)
