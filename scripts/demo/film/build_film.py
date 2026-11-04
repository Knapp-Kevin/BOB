"""Edits "B.O.B.: Find Your Next Step" from authentic parts only: the installed-app takes, the picked Sonic Rift
narration takes, the Sonic Rift music take, the approved MythologIQ Labs opener and the end card. ffmpeg does every
cut, fade and mix; this script only computes the timeline from the takes' own marks and writes the edit decision list.
Adapted from the Sojourner "The Right Next Step" build (scripts/demo/film/build_film.py there).

    python build_film.py <config.json>

config: {"takes": [{"file": mkv, "marks": json}, ...], "segments": [{"take": 0, "in": s, "out": s, "why": ""}, ...],
         "vo": {"S01": wav, ...}, "lines": {"S01": caption text, ...}, "scene_take": {"S01": 0, ...},
         "align": {"S06": {"sentence": 2, "mark": "S06-saved", "before": 1.6}},
         "overlays": [{"take": 0, "from": s, "to": s, "lines": [...]}], "poster": {"take": 0, "at": s},
         "music": wav, "music_gain_db": -11, "cards": dir, "opener_frames": dir, "out": dir, "name": "..."}
Times in "segments", "overlays" and "poster" are take times: seconds after that take's one-second black sync flash.
"""
import json
import os
import re
import subprocess
import sys

import numpy as np
import soundfile as sf

CFG = json.load(open(sys.argv[1], encoding="utf-8-sig"))
OUT = CFG["out"]
NAME = CFG["name"]
STEMS = os.path.join(OUT, "stems")
os.makedirs(STEMS, exist_ok=True)
FONT = "C\\:/Windows/Fonts/segoeui.ttf"
FONT_B = "C\\:/Windows/Fonts/segoeuib.ttf"
RATE = 48000
W, H, FPS = 1920, 1080, 30

OPENER = 2.6          # the approved MythologIQ Labs opener (make_opener.py, 30 fps); its dissolve lands on frame 1
XF = 0.6              # picture cross-fade app > end card
END_HOLD = 4.6        # end card on screen after its cross-fade
TAGLINE_AFTER = 0.35  # S08 starts this long after the end card is fully up
CUT_XF = 0.4          # dissolve at each editorial cut
VO_LEAD = 0.6         # a line starts this long after its scene mark
MUSIC_RAMP = 0.45


def run(args):
    print(" ".join(a if " " not in a else f'"{a}"' for a in args)[:300])
    subprocess.run(args, check=True)


def sync_end(capture):
    """End of the take's one-second black sync frame: time zero of its marks."""
    err = subprocess.run(["ffmpeg", "-hide_banner", "-i", capture, "-vf", "blackdetect=d=0.4:pix_th=0.05", "-an",
                          "-f", "null", "-"], capture_output=True, text=True).stderr
    ends = [float(m) for m in re.findall(r"black_end:([0-9.]+)", err)]
    if not ends:
        raise SystemExit(f"no sync flash found in {capture}")
    return ends[0]


takes = [{"file": t["file"], "marks": json.load(open(t["marks"], encoding="utf-8-sig")), "zero": sync_end(t["file"])}
         for t in CFG["takes"]]
segments = CFG["segments"]
seg_film = []
t = OPENER
for i, seg in enumerate(segments):
    if i:
        t -= CUT_XF
    seg_film.append(t)
    t += seg["out"] - seg["in"]
cap_end_film = t
end_card_full = cap_end_film
total = end_card_full + END_HOLD


def film_time(take, c):
    for seg, f0 in zip(segments, seg_film):
        if seg["take"] == take and seg["in"] <= c <= seg["out"]:
            return f0 + (c - seg["in"])
    raise SystemExit(f"take {take} time {c:.2f} is in a cut")


def mark(take, name):
    return takes[take]["marks"][name]


def voiced_frames(path):
    a, r = sf.read(path, dtype="float32", always_2d=True)
    x = a.mean(axis=1)
    hop = int(r * 0.02)
    db = np.array([20 * np.log10(np.sqrt(np.mean(x[i:i + hop] ** 2)) + 1e-9) for i in range(0, len(x) - hop, hop)])
    return db > (db.max() - 32)


def pauses_of(voiced):
    idx = np.nonzero(voiced)[0]
    out, start = [], None
    for i, v in enumerate(voiced):
        if not v and start is None:
            start = i
        elif v and start is not None:
            if (i - start) * 0.02 >= 0.18 and start > idx[0]:
                out.append(((i - start) * 0.02, start * 0.02, i * 0.02))
            start = None
    return idx, out


def sentences(text):
    # "B.O.B." ends in a full stop but does not end a sentence.
    return [s.strip() for s in re.split(r"(?<!B\.O\.B\.)(?<=[.!?])\s+", text) if s.strip()]


def sentence_starts(path, n):
    idx, pauses = pauses_of(voiced_frames(path))
    if len(pauses) < n - 1:
        return None
    return [idx[0] * 0.02] + sorted(p[2] for p in sorted(pauses, reverse=True)[:n - 1])


# --- narration placement ------------------------------------------------------------------------------
vo_files, lines, scene_take, align = CFG["vo"], CFG["lines"], CFG["scene_take"], CFG.get("align", {})
placements = []
for scene in ["S01", "S02", "S03", "S04", "S05", "S06", "S07"]:
    tk = scene_take[scene]
    if scene in align:
        rule = align[scene]
        starts = sentence_starts(vo_files[scene], len(sentences(lines[scene])))
        if starts is None:
            raise SystemExit(f"{scene}: could not find the sentence pauses in the take")
        offset = starts[rule["sentence"] - 1]
        cap_at = mark(tk, rule["mark"]) - rule["before"] - offset
        placements.append({"scene": scene, "file": vo_files[scene], "at": round(film_time(tk, cap_at), 3),
                           "aligned": {**rule, "sentence_offset_s": round(offset, 3), "take_at": round(cap_at, 3)}})
    else:
        placements.append({"scene": scene, "file": vo_files[scene], "at": round(film_time(tk, mark(tk, scene) + VO_LEAD), 3)})
placements.append({"scene": "S08", "file": vo_files["S08"], "at": round(end_card_full + TAGLINE_AFTER, 3)})
for p in placements:
    p["seconds"] = round(sf.info(p["file"]).duration, 3)
    p["ends"] = round(p["at"] + p["seconds"], 3)
for a, b in zip(placements, placements[1:]):
    if a["ends"] > b["at"] - 0.25:
        raise SystemExit(f"narration overlap: {a['scene']} ends {a['ends']} but {b['scene']} starts {b['at']}")
total = max(total, placements[-1]["ends"] + 1.4)

# --- picture ------------------------------------------------------------------------------------------
overlays = [(film_time(o["take"], o["from"]), film_time(o["take"], o["to"]), o["lines"]) for o in CFG.get("overlays", [])]
NOTE_X, NOTE_W, LINE_H, NOTE_Y0 = 12, 220, 25, 560   # the empty middle of B.O.B.'s sidebar, never over page content


def sidebar_note(start, end, note_lines):
    on = f"enable='between(t,{start:.3f},{end:.3f})'"
    h = 22 + LINE_H * len(note_lines)
    out = [f"drawbox=x={NOTE_X}:y={NOTE_Y0}:w={NOTE_W}:h={h}:color=0x0b2f44@0.92:t=fill:{on}",
           f"drawbox=x={NOTE_X}:y={NOTE_Y0}:w=4:h={h}:color=0x2fb7a8@1:t=fill:{on}"]
    for i, line in enumerate(note_lines):
        safe = line.replace(":", "\\:").replace("'", "’")
        out.append(f"drawtext=fontfile='{FONT_B if i == 0 else FONT}':text='{safe}':fontsize=17:fontcolor=0xffffff:"
                   f"x={NOTE_X + 16}:y={NOTE_Y0 + 13 + i * LINE_H}:{on}")
    return ",".join(out)


inputs = ["-framerate", "30", "-i", os.path.join(CFG["opener_frames"], "o%03d.jpg")]
for tk in takes:
    inputs += ["-i", tk["file"]]
card_input = 1 + len(takes)
inputs += ["-loop", "1", "-framerate", str(FPS), "-t", str(END_HOLD + XF + 3), "-i", os.path.join(CFG["cards"], "card-end.png")]

parts = [f"[0:v]scale={W}:{H}:flags=lanczos,setsar=1,fps={FPS},format=yuv420p,trim=duration={OPENER},setpts=PTS-STARTPTS,settb=1/{FPS}[op]"]
uses = {}
for seg in segments:
    uses[seg["take"]] = uses.get(seg["take"], 0) + 1
labels = {k: [f"t{k}s{i}" for i in range(n)] for k, n in uses.items()}
for k, names in labels.items():
    parts.append(f"[{k + 1}:v]fps={FPS},format=yuv420p,split={len(names)}" + "".join(f"[{n}]" for n in names))
used = {k: 0 for k in labels}
for i, seg in enumerate(segments):
    k = seg["take"]
    src = labels[k][used[k]]
    used[k] += 1
    z = takes[k]["zero"]
    parts.append(f"[{src}]trim=start={z + seg['in']}:end={z + seg['out']},setpts=PTS-STARTPTS,settb=1/{FPS}[seg{i}]")
chain = "seg0"
for i in range(1, len(segments)):
    parts.append(f"[{chain}][seg{i}]xfade=transition=fade:duration={CUT_XF}:offset={seg_film[i] - OPENER:.3f}[cx{i}]")
    chain = f"cx{i}"
parts.append(f"[{card_input}:v]scale={W}:{H},setsar=1,fps={FPS},format=yuv420p,trim=duration={END_HOLD + XF + 3},setpts=PTS-STARTPTS,settb=1/{FPS}[end]")
parts.append(f"[op][{chain}]concat=n=2:v=1:a=0,settb=1/{FPS}[a]")
parts.append(f"[a][end]xfade=transition=fade:duration={XF}:offset={cap_end_film - XF:.3f}[b]")
notes = ",".join(sidebar_note(*o) for o in overlays) or "null"
parts.append(f"[b]{notes},trim=duration={total},scale=out_range=tv,format=yuv420p[v]")
picture = os.path.join(OUT, "picture.mp4")
run(["ffmpeg", "-hide_banner", "-y", *inputs, "-filter_complex", ";".join(parts), "-map", "[v]", "-c:v", "libx264",
     "-preset", "slow", "-crf", "17", "-pix_fmt", "yuv420p", "-color_range", "tv", "-r", str(FPS), picture])

# --- sound --------------------------------------------------------------------------------------------
vo_inputs, vo_chains = [], []
for i, p in enumerate(placements):
    vo_inputs += ["-i", p["file"]]
    ms = int(round(p["at"] * 1000))
    vo_chains.append(f"[{i}:a]aresample={RATE},aformat=channel_layouts=stereo,adelay={ms}|{ms}[vo{i}]")
vo_stem = os.path.join(STEMS, "stem-narration.wav")
run(["ffmpeg", "-hide_banner", "-y", *vo_inputs, "-filter_complex",
     ";".join(vo_chains) + ";" + "".join(f"[vo{i}]" for i in range(len(placements))) +
     f"amix=inputs={len(placements)}:normalize=0,apad,atrim=duration={total}[out]",
     "-map", "[out]", "-c:a", "pcm_s24le", "-ar", str(RATE), vo_stem])

VO_OVER_MUSIC_DB = 14.0
BED_COMPRESSOR = "acompressor=threshold=0.008:ratio=3:attack=300:release=2500:makeup=3"


def rms_db(x):
    return float(20 * np.log10(np.sqrt(np.mean(x ** 2)) + 1e-9))


def line_dips(flat_stem):
    """Per-line dip (dB) holding the bed VO_OVER_MUSIC_DB under the narrator: a fader ride measured on the flat bed."""
    m, r = sf.read(flat_stem, dtype="float32", always_2d=True)
    vo, vr = sf.read(vo_stem, dtype="float32", always_2d=True)
    m, vo = m.mean(axis=1), vo.mean(axis=1)
    dips = {}
    for p in placements:
        bed_db = rms_db(m[int(p["at"] * r):int(p["ends"] * r)])
        vo_db = rms_db(vo[int(p["at"] * vr):int(p["ends"] * vr)])
        dips[p["scene"]] = round(max(-14.0, min(0.0, (vo_db - VO_OVER_MUSIC_DB) - bed_db)), 1)
    return dips


def music_gain_expr(dips):
    pts = [(0.0, 0.0)]
    for p in placements:
        g = dips.get(p["scene"], -6.0)
        a, b = p["at"] - MUSIC_RAMP, p["ends"] + 0.15
        pts += [(a, 0.0), (a + MUSIC_RAMP, g), (b, g), (b + MUSIC_RAMP, 0.0)]
    pts.sort()
    expr = "0"
    for (t0, g0), (t1, g1) in reversed(list(zip(pts, pts[1:]))):
        if t1 <= t0:
            continue
        expr = f"if(between(t,{t0},{t1}),({g0}+({g1}-({g0}))*(t-{t0})/({t1}-{t0})),{expr})"
    return expr


music_stem = os.path.join(STEMS, "stem-music.wav")
mlen = sf.info(CFG["music"]).duration
music_at = CFG.get("music_at", OPENER)   # the opener is silent; the bed starts with the first app frame
fade_out = 2.5
used_s = min(mlen, total - music_at)
ms = int(round(music_at * 1000))
base = CFG.get("music_gain_db", -11)


def render_bed(dips, out):
    run(["ffmpeg", "-hide_banner", "-y", "-i", CFG["music"], "-filter_complex",
         f"[0:a]aresample={RATE},aformat=channel_layouts=stereo,atrim=duration={used_s},"
         f"{BED_COMPRESSOR},afade=t=in:d=1.2,afade=t=out:st={used_s - fade_out}:d={fade_out},adelay={ms}|{ms},"
         f"volume=volume='pow(10,({music_gain_expr(dips)})/20)':eval=frame,volume={base}dB,"
         f"apad,atrim=duration={total}[out]",
         "-map", "[out]", "-c:a", "pcm_s24le", "-ar", str(RATE), out])


flat = os.path.join(STEMS, "stem-music-flat-for-measuring.wav")
render_bed({p["scene"]: 0.0 for p in placements}, flat)
dips = line_dips(flat)
os.remove(flat)
render_bed(dips, music_stem)
music_info = {"file": CFG["music"], "seconds": round(mlen, 2), "starts_film_s": music_at, "used_seconds": round(used_s, 2),
              "base_gain_db": base, "dip_under_each_line_db": dips, "target_narration_over_music_db": VO_OVER_MUSIC_DB,
              "fade_in": 1.2, "fade_out": fade_out, "compressor": BED_COMPRESSOR,
              "note": "one continuous take, never restarted; faded only at its start and at the very end"}

mix = os.path.join(STEMS, "mix-premaster.wav")
run(["ffmpeg", "-hide_banner", "-y", "-i", vo_stem, "-i", music_stem, "-filter_complex",
     "[0:a][1:a]amix=inputs=2:normalize=0[out]", "-map", "[out]", "-c:a", "pcm_s24le", mix])
master = os.path.join(STEMS, "mix-master.wav")
run(["ffmpeg", "-hide_banner", "-y", "-i", mix, "-af", "loudnorm=I=-16:TP=-1.5:LRA=11:linear=true:print_format=summary",
     "-ar", str(RATE), "-c:a", "pcm_s24le", master])
final = os.path.join(OUT, f"{NAME}.mp4")
run(["ffmpeg", "-hide_banner", "-y", "-i", picture, "-i", master, "-map", "0:v", "-map", "1:a", "-c:v", "copy",
     "-c:a", "aac", "-b:a", "192k", "-ar", str(RATE), "-movflags", "+faststart", "-shortest", final])


# --- captions, from the placed takes: sentence boundaries are the take's own pauses -----------------------
def sentence_cues(scene, at, path):
    sents = sentences(lines[scene])
    idx, pauses = pauses_of(voiced_frames(path))
    first, last = idx[0] * 0.02, (idx[-1] + 1) * 0.02
    need = len(sents) - 1
    bounds = sorted(sorted(pauses, reverse=True)[:need], key=lambda p: p[1]) if len(pauses) >= need else None
    if bounds is None:
        total_chars = sum(len(s) for s in sents)
        cues, t0 = [], first
        for s in sents:
            d = (last - first) * len(s) / total_chars
            cues.append((at + t0, at + t0 + d, s))
            t0 += d
    else:
        starts = [first] + [b[2] for b in bounds]
        ends = [b[1] for b in bounds] + [last]
        cues = [(at + s0, at + e0, s) for s0, e0, s in zip(starts, ends, sents)]
    merged = []
    for c in cues:
        if merged and (c[1] - c[0] < 1.1 or merged[-1][1] - merged[-1][0] < 1.1) and len(merged[-1][2]) + len(c[2]) < 90:
            merged[-1] = (merged[-1][0], c[1], merged[-1][2] + " " + c[2])
        else:
            merged.append(c)
    out = []
    for s0, e0, text in merged:
        if len(text) > 95:
            mid = len(text) // 2
            cut = min((i for i in (text.find(": "), text.rfind(", ", 0, mid + 20)) if i > 0), key=lambda i: abs(i - mid),
                      default=text.rfind(" ", 0, mid))
            a1, a2 = text[:cut + 1].strip(), text[cut + 1:].strip()
            split_t = s0 + (e0 - s0) * len(a1) / len(text)
            out += [(s0, split_t, a1), (split_t, e0, a2)]
        else:
            out.append((s0, e0, text))
    return [(s, max(e, s + 1.2), text) for s, e, text in out]


cues = []
for p in placements:
    cues += sentence_cues(p["scene"], p["at"], p["file"])
for i in range(len(cues) - 1):
    s, e, text = cues[i]
    cues[i] = (s, min(e + 0.25, cues[i + 1][0] - 0.05), text)


def ts(t):
    h, rem = divmod(t, 3600)
    m, s = divmod(rem, 60)
    return f"{int(h):02d}:{int(m):02d}:{s:06.3f}"


vtt = os.path.join(OUT, f"{NAME}.en.vtt")
with open(vtt, "w", encoding="utf-8", newline="\n") as f:
    f.write("WEBVTT\n\n")
    for n, (s, e, text) in enumerate(cues, 1):
        f.write(f"{n}\n{ts(s)} --> {ts(e)}\n{text}\n\n")
ass = os.path.join(OUT, "captions-burnin.ass")
with open(ass, "w", encoding="utf-8", newline="\n") as f:
    f.write(f"[Script Info]\nScriptType: v4.00+\nPlayResX: {W}\nPlayResY: {H}\nWrapStyle: 0\n\n[V4+ Styles]\n"
            "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, "
            "Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, "
            "MarginR, MarginV, Encoding\n"
            "Style: Cap,Segoe UI,40,&H00FFFFFF,&H00FFFFFF,&H00000000,&HC0301E0B,0,0,0,0,100,100,0,0,3,16,0,2,330,90,40,1\n\n"
            "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n")
    for s, e, text in cues:
        f.write(f"Dialogue: 0,{ts(s)[1:-1]},{ts(e)[1:-1]},Cap,,0,0,0,,{text}\n")
silent = os.path.join(OUT, f"{NAME}-silent-captioned.mp4")
ass_arg = ass.replace("\\", "/").replace(":", "\\:")
run(["ffmpeg", "-hide_banner", "-y", "-i", picture, "-vf", f"subtitles='{ass_arg}'", "-c:v", "libx264", "-preset", "slow",
     "-crf", "18", "-pix_fmt", "yuv420p", "-an", "-movflags", "+faststart", silent])

poster = os.path.join(OUT, f"{NAME}-poster.png")
poster_t = film_time(CFG["poster"]["take"], CFG["poster"]["at"])
run(["ffmpeg", "-hide_banner", "-y", "-ss", f"{poster_t:.3f}", "-i", picture, "-frames:v", "1", poster])

edl = {
    "name": NAME, "fps": FPS, "size": [W, H], "total_seconds": round(total, 3),
    "picture": [
        {"film": [0, OPENER], "source": "opener frames o000-o077.jpg (make_opener.py)",
         "note": "approved MythologIQ Labs opener; dissolves into the first take's first frame"},
        *[{"film": [round(f0, 3), round(f0 + (s["out"] - s["in"]), 3)], "source": os.path.basename(takes[s["take"]]["file"]),
           "take_in": s["in"], "take_out": s["out"], "source_in": round(takes[s["take"]]["zero"] + s["in"], 3),
           "source_out": round(takes[s["take"]]["zero"] + s["out"], 3), "why_cut": s.get("why", ""),
           "transition_in": "straight from the opener's dissolve" if i == 0 else f"dissolve {CUT_XF}s"}
          for i, (s, f0) in enumerate(zip(segments, seg_film))],
        {"film": [round(end_card_full - XF, 3), round(total, 3)], "source": "card-end.png", "transition_in": f"fade {XF}s"},
    ],
    "capture_note": "installed signed app; picture never retimed, frozen or altered; cuts only remove waiting and the restart",
    "overlays": [{"film": [round(s, 3), round(e, 3)], "text": " ".join(t), "placement": "sidebar middle"} for s, e, t in overlays],
    "narration": placements, "music": music_info,
    "takes": [{"file": tk["file"], "sync_zero_seconds": tk["zero"], "marks": tk["marks"]} for tk in takes], "vo_lead": VO_LEAD,
    "loudness_target": {"integrated_lufs": -16, "true_peak_dbtp": -1.5},
    "poster_film_seconds": round(poster_t, 3),
    "captions": [{"start": round(s, 3), "end": round(e, 3), "text": text} for s, e, text in cues],
}
json.dump(edl, open(os.path.join(OUT, "edit-decision-list.json"), "w", encoding="utf-8"), indent=1)
print(json.dumps({"total": round(total, 2), "zeros": [tk["zero"] for tk in takes], "cues": len(cues)}))
