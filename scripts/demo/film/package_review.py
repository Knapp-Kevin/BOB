"""Assembles the first-review package for "B.O.B.: Find Your Next Step" from the build and the source folder and
writes a SHA-256 manifest of every delivered file. Copies only; nothing is re-encoded here.

    python package_review.py <source dir> <repo root> <package dir> <name>
"""
import hashlib
import json
import os
import shutil
import subprocess
import sys

SOURCE, REPO, PKG, NAME = sys.argv[1:5]
BUILD = os.path.join(SOURCE, "build-v1")
os.makedirs(PKG, exist_ok=True)


def copy(src, rel):
    dst = os.path.join(PKG, rel)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    shutil.copy2(src, dst)


for f in (f"{NAME}.mp4", f"{NAME}-silent-captioned.mp4", f"{NAME}.en.vtt", f"{NAME}-poster.png", "edit-decision-list.json",
          "qa-measurements.json", "qa-transcription-final-mix.json", "captions-burnin.ass"):
    copy(os.path.join(BUILD, f), f)
for f in ("stem-narration.wav", "stem-music.wav", "mix-master.wav"):
    copy(os.path.join(BUILD, "stems", f), os.path.join("stems", f))
audio = os.path.join(SOURCE, "audio")
for sub in ("narration-design", "narration-takes", "narration-takes-s06", "narration-selected", "music", "music-rejected-too-long"):
    for f in sorted(os.listdir(os.path.join(audio, sub))):
        if f.endswith((".wav", ".json", ".png", ".txt", ".csv")):
            copy(os.path.join(audio, sub, f), os.path.join("audio", sub, f))
for f in ("lines-spoken.json", "lines-paragraphs.json"):
    copy(os.path.join(audio, f), os.path.join("audio", f))
cap = os.path.join(SOURCE, "source-capture")
for f in sorted(os.listdir(cap)):
    if os.path.isfile(os.path.join(cap, f)):
        copy(os.path.join(cap, f), os.path.join("source-capture", f))
for f in sorted(os.listdir(os.path.join(cap, "rehearsal"))):
    copy(os.path.join(cap, "rehearsal", f), os.path.join("source-capture", "rehearsal", f))
for f in ("card-end.png", "card-end.html", "first-frame.jpg"):
    copy(os.path.join(SOURCE, "brand", f), os.path.join("brand", f))
for f in sorted(os.listdir(os.path.join(SOURCE, "brand", "opener-source"))):
    copy(os.path.join(SOURCE, "brand", "opener-source", f), os.path.join("brand", "opener-source", f))
copy(os.path.join(SOURCE, "brand", "opener", "opener.json"), os.path.join("brand", "opener", "opener.json"))
copy(os.path.join(REPO, "docs", "assets", "bob-mascot-transparent.png"), os.path.join("brand", "bob-mascot-transparent.png"))
copy(os.path.join(SOURCE, "build-config.json"), "build-config.json")
docs = os.path.join(REPO, "docs", "demo-production", "find-your-next-step")
copy(os.path.join(docs, "SCRIPT_AND_STORYBOARD.md"), "SCRIPT_AND_STORYBOARD.md")
for f in ("QA_REPORT.md", "transcript.md"):
    copy(os.path.join(docs, "review-v1", f), f)
for f in ("film-session.ps1", "run-film.ps1", "relaunch-film-sandbox.ps1", "sbx.sh"):
    copy(os.path.join(REPO, "scripts", "demo", "windows", f), os.path.join("tooling", "windows", f))
for f in sorted(os.listdir(os.path.join(REPO, "scripts", "demo", "film"))):
    if f.endswith(".py"):
        copy(os.path.join(REPO, "scripts", "demo", "film", f), os.path.join("tooling", "film", f))

commit = subprocess.run(["git", "-C", REPO, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
rows = []
for dirpath, _, files in os.walk(PKG):
    for f in files:
        full = os.path.join(dirpath, f)
        rel = os.path.relpath(full, PKG).replace("\\", "/")
        if rel in ("SHA256SUMS.txt", "MANIFEST.json"):
            continue
        rows.append((rel, os.path.getsize(full), hashlib.sha256(open(full, "rb").read()).hexdigest()))
rows.sort()
with open(os.path.join(PKG, "SHA256SUMS.txt"), "w", encoding="utf-8", newline="\n") as f:
    for rel, _, h in rows:
        f.write(f"{h}  {rel}\n")
prov = json.load(open(os.path.join(cap, "install-provenance.json"), encoding="utf-8-sig"))
manifest = {
    "film": "B.O.B.: Find Your Next Step", "package": f"{NAME} (first review cut v1)",
    "status": "PRIVATE FIRST REVIEW CUT, awaiting owner review; not approved, not published",
    "application": {"build": "B.O.B. Windows Alpha v0.1.0 (installed, signed by MythologIQ Labs, LLC)", "release_tag": "v0.1.0",
                    "tag_commit": "ecce695e7a6652c5b749a94227ec13c7b27f0cee",
                    "installer_sha256": prov["installer_sha256"].lower(), "installed_exe_sha256": prov["exe_sha256"].lower(),
                    "file_version": prov["file_version"], "signature": prov["signature"], "host": prov["os"] + " (offline Windows Sandbox)"},
    "production_branch": "demo/find-your-next-step-v1-20261103", "production_commit": commit,
    "data": "fictional demo data (Sam; fixture-film-sam in scripts/demo/windows/film-session.ps1)",
    "audio": {"narration": "Sonic Rift 0.9.1: Qwen3 voice design (Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign) then Qwen3 voice cloning "
                           "(Qwen/Qwen3-TTS-12Hz-1.7B-Base), Apache-2.0",
              "music": "Sonic Rift 0.9.1 Music: ACE-Step 1.5 (ACE-Step/Ace-Step1.5 @ 19671f406d603126926c1b7e2adc169acbcade22), MIT, take D seed 20261107",
              "master": "-17.3 LUFS integrated, true peak -1.5 dBFS"},
    "opener": "Shared MythologIQ Labs opener (owner-approved silver mark, DM Sans OFL), make_opener.py from the Sonic Rift v0.9.1 final cut",
    "rights": "First-party capture of MythologIQ Labs software with fictional data; narration and music generated by MythologIQ Labs in "
              "Sonic Rift with permissively licensed models; no stock footage, third-party audio or imagery.",
    "files": [{"path": rel, "bytes": size, "sha256": h} for rel, size, h in rows],
}
json.dump(manifest, open(os.path.join(PKG, "MANIFEST.json"), "w", encoding="utf-8"), indent=1)
print(len(rows), "files")
