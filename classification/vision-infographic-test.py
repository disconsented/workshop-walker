#!/usr/bin/env python3
"""Tests extraction when the content lives in the description's images.

Item 3799737423 is the case that matters: its prose is 2,674 characters of
marketing, while 18 embedded infographics carry the actual feature list. The
markers below are research project names that appear in those images and
nowhere in the text, so anything the model returns from them came from vision.

Run A sends the prose alone. Run B sends the prose plus the infographics.
"""
import base64
import json
import pathlib
import re
import sys
import urllib.request

PORT = sys.argv[1] if len(sys.argv) > 1 else "8081"
ITEM = "3799737423"
PROMPT_KEY = sys.argv[2] if len(sys.argv) > 2 else "features"
OUT = f"bench-out-llamacpp/_infographic_{PROMPT_KEY}.json"

IMAGE_ONLY_TERMS = [
    "basic gravtech", "gravship power", "oxygen network", "orbital tech",
    "gravship living", "standard gravtech", "gravship weaponry",
    "compact workspaces", "astrofuel refining", "heat dissipation",
    "advanced gravtech", "gravship defenses", "gravdata",
]

meta = json.load(open(f"bench-cache/item_{ITEM}.json"))
template = open(f"prompts/{PROMPT_KEY}.txt").read()
content = (
    template.replace("[TITLE]", meta["title"])
    .replace("[DESCRIPTION]", meta["description"])
    .replace("\t", "")
)

shots = sorted(
    pathlib.Path("bench-cache/infographics").glob("InfographicsBattle_*.png"),
    key=lambda p: int(re.search(r"_(\d+)", p.name).group(1)),
)


def ask(with_images):
    if with_images:
        parts = []
        for p in shots:
            b64 = base64.b64encode(p.read_bytes()).decode()
            parts.append({"type": "image_url",
                          "image_url": {"url": f"data:image/png;base64,{b64}"}})
        parts.append({"type": "text", "text": content})
        body_content = parts
    else:
        body_content = content

    body = json.dumps({
        "messages": [{"role": "user", "content": body_content}],
        "temperature": 0.0,
        "max_tokens": 512,
        "chat_template_kwargs": {"enable_thinking": False},
    }).encode()
    req = urllib.request.Request(
        f"http://127.0.0.1:{PORT}/v1/chat/completions",
        data=body, headers={"Content-Type": "application/json"})
    payload = json.load(urllib.request.urlopen(req, timeout=3600))
    msg = payload["choices"][0]["message"]
    return {
        "answer": msg.get("content") or "",
        "finish": payload["choices"][0].get("finish_reason"),
        "timings": payload.get("timings", {}),
    }


results = {"item": ITEM, "title": meta["title"], "prompt": PROMPT_KEY,
           "n_images": len(shots), "markers": IMAGE_ONLY_TERMS}
for mode, with_images in (("text_only", False), ("with_infographics", True)):
    r = ask(with_images)
    hits = [t for t in IMAGE_ONLY_TERMS if t in r["answer"].lower()]
    r["hits"] = hits
    r["hit_count"] = len(hits)
    results[mode] = r
    t = r["timings"]
    print(f"=== {mode}")
    print(f"    prompt tokens : {t.get('prompt_n')}")
    print(f"    image-only terms found: {len(hits)}/{len(IMAGE_ONLY_TERMS)}  {hits}")
    print(f"    finish: {r['finish']}")
    print("    " + r["answer"].strip().replace("\n", "\n    ")[:1400])
    print()

json.dump(results, open(OUT, "w"), indent=2, ensure_ascii=False)
print(f"wrote {OUT}")
