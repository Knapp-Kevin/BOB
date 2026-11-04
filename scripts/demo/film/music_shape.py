"""Shape of each music take (measures only): loudness every 5 s, the opening and the last seconds, whether the
take ends by decaying or is cut off, and whether Sonic Rift's transcriber hears any words (vocals).
    python music_shape.py "<install>" <out.json> <wav> [...]"""
import json, sys
import numpy as np, soundfile as sf
from scipy.signal import resample_poly
APP, OUT, *WAVS = sys.argv[1:]
sys.path.insert(0, APP)
from this_voice_thing.core import transcription
T = transcription.Transcriber()
rows = []
for path in WAVS:
    a, r = sf.read(path, dtype="float32", always_2d=True); m = a.mean(axis=1)
    db = lambda x: round(float(20 * np.log10(np.sqrt(np.mean(x ** 2)) + 1e-9)), 1)
    blocks = [db(m[i:i + 5 * r]) for i in range(0, len(m) - r, 5 * r)]
    last = [db(m[len(m) - (k + 1) * r // 2: len(m) - k * r // 2]) for k in range(8)][::-1]
    heard = T.transcribe(resample_poly(m, 16000, r).astype(np.float32), language="en").text.strip()
    rows.append({"file": path.replace("\\", "/").split("/")[-1], "rms_db_every_5s": blocks,
                 "last_4s_rms_db_per_half_second": last, "final_half_second_db": last[-1], "transcriber_heard": heard})
    print(rows[-1]["file"], "5s:", blocks, "| tail:", last, "| heard:", repr(heard[:80]))
json.dump(rows, open(OUT, "w", encoding="utf-8"), indent=1)
