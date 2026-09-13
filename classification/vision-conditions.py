#!/usr/bin/env python3
"""Runs the four vision conditions and keeps every raw answer.

A  text only                     the current pipeline
B  text + images attached        one call, images in the same message
C  transcribe                    one call per image, no extraction prompt
D  transcribe then extract       C's plain text appended to the description

Writes one JSON per item holding the verbatim output of every call.
"""
import base64
import json
import pathlib
import re
import sys
import urllib.request

PORT = sys.argv[1] if len(sys.argv) > 1 else "8082"
OUT_DIR = pathlib.Path("bench-out-vision")
OUT_DIR.mkdir(exist_ok=True)

# item -> (image directory, strings that appear in the images but not the prose)
CASES = {
    "3761824516": ("bench-cache/rustic", [
        "crafting spot", "butchering spot", "fueled smithy", "fueled stove",
        "stonecutter", "hand tailor", "butcher table", "brewery", "drug lab",
        "simple research bench", "art bench", "styling station",
        "caravan hitching", "fermenting barrel", "industrial module"]),
    "3799737423": ("bench-cache/infographics", [
        "basic gravtech", "gravship power", "oxygen network", "orbital tech",
        "gravship living", "standard gravtech", "gravship weaponry",
        "compact workspaces", "astrofuel refining", "heat dissipation",
        "advanced gravtech", "gravship defenses", "gravdata"]),
}

TRANSCRIBE = ("Transcribe every label and heading in this image, one per line. "
              "Be exhaustive and literal.")


def call(parts, max_tokens=512):
    body = json.dumps({
        "messages": [{"role": "user", "content": parts}],
        "temperature": 0.0,
        "max_tokens": max_tokens,
        "chat_template_kwargs": {"enable_thinking": False},
    }).encode()
    req = urllib.request.Request(
        f"http://127.0.0.1:{PORT}/v1/chat/completions",
        data=body, headers={"Content-Type": "application/json"})
    p = json.load(urllib.request.urlopen(req, timeout=3600))
    t = p.get("timings", {})
    return {
        "answer": p["choices"][0]["message"].get("content") or "",
        "finish": p["choices"][0].get("finish_reason"),
        "prompt_tokens": t.get("prompt_n"),
        "generated_tokens": t.get("predicted_n"),
        "decode_tokens_per_sec": t.get("predicted_per_second"),
    }


def image_part(path):
    b64 = base64.b64encode(pathlib.Path(path).read_bytes()).decode()
    return {"type": "image_url", "image_url": {"url": f"data:image/png;base64,{b64}"}}


for item, (img_dir, markers) in CASES.items():
    meta = json.load(open(f"bench-cache/item_{item}.json"))
    template = open("prompts/features.txt").read()

    def extract_prompt(description):
        return (template.replace("[TITLE]", meta["title"])
                .replace("[DESCRIPTION]", description).replace("\t", ""))

    shots = sorted(pathlib.Path(img_dir).glob("*.png"),
                   key=lambda p: (len(p.name), p.name))
    record = {"item": item, "title": meta["title"],
              "prose_chars": len(re.sub(r"<[^>]+>", "", meta["description"])),
              "n_images": len(shots), "markers": markers, "conditions": {}}

    def score(answer):
        return [m for m in markers if m in answer.lower()]

    a = call(extract_prompt(meta["description"]))
    a["hits"] = score(a["answer"])
    record["conditions"]["A_text_only"] = a

    b = call([image_part(p) for p in shots]
             + [{"type": "text", "text": extract_prompt(meta["description"])}])
    b["hits"] = score(b["answer"])
    record["conditions"]["B_text_plus_images"] = b

    transcripts = []
    per_image = []
    for p in shots:
        c = call([image_part(p), {"type": "text", "text": TRANSCRIBE}], 500)
        c["image"] = p.name
        c["hits"] = score(c["answer"])
        per_image.append(c)
        transcripts.append(f"[{p.name}]\n{c['answer'].strip()}")
    record["conditions"]["C_transcribe"] = per_image

    enriched = (meta["description"]
                + "\n\nText extracted from the mod's images:\n"
                + "\n\n".join(transcripts))
    d = call(extract_prompt(enriched))
    d["hits"] = score(d["answer"])
    record["conditions"]["D_transcribe_then_extract"] = d

    path = OUT_DIR / f"conditions_{item}.json"
    json.dump(record, open(path, "w"), indent=2, ensure_ascii=False)
    print(f"{item} {meta['title']}: "
          f"A {len(a['hits'])}/{len(markers)}, B {len(b['hits'])}/{len(markers)}, "
          f"C {len(set(h for c in per_image for h in c['hits']))}/{len(markers)}, "
          f"D {len(d['hits'])}/{len(markers)} -> {path}")
