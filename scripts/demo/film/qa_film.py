"""Technical QA of a built film: loudness and peak of the master, narration-to-music level under every line,
gaps where neither narration nor music is audible, frozen and near-blank pictures, stream identity and the
caption timing. Measures only; the report says what a listener and viewer still have to judge.

    python qa_film.py <out dir> <name>
"""
import json
import os
import re
import subprocess
import sys

import numpy as np
import soundfile as sf

OUT, NAME = sys.argv[1:3]
edl = json.load(open(os.path.join(OUT, "edit-decision-list.json"), encoding="utf-8"))
film = os.path.join(OUT, f"{NAME}.mp4")
silent = os.path.join(OUT, f"{NAME}-silent-captioned.mp4")


def ff(args):
    return subprocess.run(["ffmpeg", "-hide_banner", "-nostats", *args, "-f", "null", "-"], capture_output=True, text=True).stderr


def probe(path):
    return json.loads(subprocess.run(["ffprobe", "-v", "error", "-show_streams", "-show_format", "-of", "json", path],
                                     capture_output=True, text=True, check=True).stdout)


def lufs(path, start=None, dur=None):
    args = (["-ss", str(start), "-t", str(dur)] if start is not None else []) + ["-i", path, "-af", "ebur128=peak=true", "-vn"]
    err = ff(args)
    summary = err[err.rfind("Summary:"):]
    i = re.search(r"I:\s+(-?[0-9.]+) LUFS", summary)
    tp = re.search(r"Peak:\s+(-?[0-9.]+) dBFS", summary)
    lra = re.search(r"LRA:\s+(-?[0-9.]+) LU", summary)
    return {"integrated_lufs": float(i.group(1)) if i else None, "true_peak_dbfs": float(tp.group(1)) if tp else None,
            "lra_lu": float(lra.group(1)) if lra else None}


def rms_db(x):
    return float(20 * np.log10(np.sqrt(np.mean(x ** 2)) + 1e-9))


report = {"film": film, "probe": {}, "checks": {}}
for path in (film, silent):
    p = probe(path)
    report["probe"][os.path.basename(path)] = {
        "duration": float(p["format"]["duration"]), "bytes": int(p["format"]["size"]),
        "streams": [{k: s.get(k) for k in ("codec_name", "profile", "width", "height", "r_frame_rate", "sample_rate", "channels",
                                            "pix_fmt", "nb_frames")} for s in p["streams"]]}
report["checks"]["master_loudness"] = lufs(film)

stems = os.path.join(OUT, "stems")
vo, r = sf.read(os.path.join(stems, "stem-narration.wav"), dtype="float32", always_2d=True)
music = None
if os.path.exists(os.path.join(stems, "stem-music.wav")):
    music, _ = sf.read(os.path.join(stems, "stem-music.wav"), dtype="float32", always_2d=True)
master, _ = sf.read(os.path.join(stems, "mix-master.wav"), dtype="float32", always_2d=True)
report["checks"]["master_sample_peak_dbfs"] = round(float(20 * np.log10(np.abs(master).max() + 1e-9)), 2)
report["checks"]["master_clipped_samples"] = int((np.abs(master) >= 0.999).sum())

lines = []
for p in edl["narration"]:
    a, b = int(p["at"] * r), int(p["ends"] * r)
    row = {"scene": p["scene"], "at": p["at"], "ends": p["ends"], "narration_rms_db": round(rms_db(vo[a:b]), 1)}
    if music is not None:
        row["music_rms_db"] = round(rms_db(music[a:b]), 1)
        row["narration_over_music_db"] = round(row["narration_rms_db"] - row["music_rms_db"], 1)
    lines.append(row)
report["checks"]["lines"] = lines
for a, b in zip(edl["narration"], edl["narration"][1:]):
    if a["ends"] > b["at"]:
        report["checks"].setdefault("overlaps", []).append([a["scene"], b["scene"]])

# Silence anywhere in the master (neither narration nor music)
hop = int(r * 0.05)
m = master.mean(axis=1)
db = np.array([rms_db(m[i:i + hop]) for i in range(0, len(m) - hop, hop)])
quiet, runs, start = db < -50, [], None
for i, q in enumerate(quiet):
    if q and start is None:
        start = i
    elif not q and start is not None:
        if (i - start) * 0.05 >= 0.75:
            runs.append([round(start * 0.05, 2), round(i * 0.05, 2)])
        start = None
if start is not None and (len(quiet) - start) * 0.05 >= 0.75:
    runs.append([round(start * 0.05, 2), round(len(quiet) * 0.05, 2)])
report["checks"]["silences_over_0_75s_below_-50dbfs"] = runs

err = ff(["-i", film, "-vf", "freezedetect=n=0.001:d=4", "-an"])
report["checks"]["frozen_picture_over_4s"] = [
    [float(s), float(e)] for s, e in zip(re.findall(r"freeze_start: ([0-9.]+)", err), re.findall(r"freeze_end: ([0-9.]+)", err))]
err = ff(["-i", film, "-vf", "blackdetect=d=0.2:pix_th=0.08", "-an"])
report["checks"]["black_frames"] = re.findall(r"black_start:[0-9.]+ black_end:[0-9.]+", err)
caps = edl["captions"]
report["checks"]["captions"] = {"count": len(caps), "overlapping": sum(1 for a, b in zip(caps, caps[1:]) if a["end"] > b["start"]),
                                "shortest_s": round(min(c["end"] - c["start"] for c in caps), 2),
                                "max_chars": max(len(c["text"]) for c in caps)}
json.dump(report, open(os.path.join(OUT, "qa-measurements.json"), "w", encoding="utf-8"), indent=1)
print(json.dumps(report["checks"], indent=1))
