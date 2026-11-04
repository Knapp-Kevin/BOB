"""End card for "B.O.B.: Find Your Next Step": type on the app's own navy, with the approved transparent mascot
(docs/assets/bob-mascot-transparent.png). No artwork is generated. Rendered at 1920x1080 by headless Edge.

    python end_card.py <repo root> <out dir>
"""
import base64
import pathlib
import subprocess
import sys

ROOT, OUT = map(pathlib.Path, sys.argv[1:3])
OUT.mkdir(parents=True, exist_ok=True)
mascot = base64.b64encode((ROOT / "docs/assets/bob-mascot-transparent.png").read_bytes()).decode()
html = f"""<!doctype html><html><head><meta charset="utf-8"><style>
html,body{{margin:0;width:1920px;height:1080px;overflow:hidden}}
body{{font-family:Inter,'Segoe UI',system-ui,sans-serif;color:#fff;
  background:radial-gradient(circle at 50% 36%,#0d557c 0%,#073e5b 46%,#062f47 100%);
  display:flex;flex-direction:column;align-items:center;justify-content:center}}
img{{width:230px;height:230px;object-fit:contain;filter:drop-shadow(0 14px 30px rgba(0,0,0,.35))}}
.name{{margin-top:26px;font-weight:700;font-size:104px;letter-spacing:.04em;line-height:1}}
.sub{{margin-top:16px;font-size:32px;color:#bcd8e8;letter-spacing:.02em}}
.ver{{margin-top:56px;font-weight:600;font-size:28px}}
.url{{margin-top:10px;font-size:25px;color:#bcd8e8}}
.foot{{position:absolute;bottom:34px;font-size:20px;color:#9fc3d6}}
</style></head><body>
<img src="data:image/png;base64,{mascot}" alt="">
<div class="name">B.O.B.</div><div class="sub">Better Organized Brain</div>
<div class="ver">Windows Alpha · v0.1.0</div><div class="url">github.com/Knapp-Kevin/BOB/releases</div>
<div class="foot">MythologIQ Labs, LLC · Sam and everything on screen are fictional demo data.</div>
</body></html>"""
page = OUT / "card-end.html"
page.write_text(html, encoding="utf-8")
edge = r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
subprocess.run([edge, "--headless=new", "--disable-gpu", "--hide-scrollbars", "--force-device-scale-factor=1",
                "--window-size=1920,1080", f"--user-data-dir={OUT / 'edge-profile'}", f"--screenshot={OUT / 'card-end.png'}", page.as_uri()], check=True)
print(OUT / "card-end.png")
