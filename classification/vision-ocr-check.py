#!/usr/bin/env python3
"""Checks that the vision path actually reads the image.

Asks the model to transcribe the text in each preview, once with the image
attached and once without. The control matters: every item's description is in
the prompt either way, so only strings that appear nowhere in the text prove the
model read the picture.
"""
import base64
import json
import sys
import urllib.request

PORT = sys.argv[1] if len(sys.argv) > 1 else "8081"
OUT = sys.argv[2] if len(sys.argv) > 2 else "bench-out-llamacpp/_ocr_check.json"

# Strings visible in each preview that appear nowhere in its title or body.
IMAGE_ONLY = {
    "3666883698": ["Aelianus", "Xanthus", "Pollux", "Hadrianus", "Antoninus",
                   "Tatius", "Liviana", "Lucia", "Vita", "1.6"],
    "3121742525": ["失能", "機関", "1.5"],
    "3754128439": [],
    # The badge and version corner are metadata the description never states;
    # the API confirms two real dependencies, so "ADD-ON" is checkable.
    "3799737423": ["ADD-ON", "1.6"],
}

PROMPT = (
    "Transcribe every piece of text you can see in the image, including titles, "
    "labels, version numbers and any small captions. List each distinct piece of "
    "text on its own line. If you cannot see an image, reply exactly: NO IMAGE."
)


def ask(item, with_image):
    meta = json.load(open(f"bench-cache/item_{item}.json"))
    # The description goes in either way, so the control has every chance to
    # answer from the text alone.
    text = f"{PROMPT}\n\nFor context, this mod is titled '{meta['title']}'."
    if with_image:
        with open(f"bench-cache/preview_{item}", "rb") as fh:
            b64 = base64.b64encode(fh.read()).decode()
        content = [
            {"type": "image_url",
             "image_url": {"url": f"data:image/png;base64,{b64}"}},
            {"type": "text", "text": text},
        ]
    else:
        content = text

    body = json.dumps({
        "messages": [{"role": "user", "content": content}],
        "temperature": 0.0,
        "max_tokens": 300,
        "chat_template_kwargs": {"enable_thinking": False},
    }).encode()
    req = urllib.request.Request(
        f"http://127.0.0.1:{PORT}/v1/chat/completions",
        data=body, headers={"Content-Type": "application/json"})
    payload = json.load(urllib.request.urlopen(req, timeout=900))
    return payload["choices"][0]["message"].get("content") or ""


results = []
for item, markers in IMAGE_ONLY.items():
    row = {"item": item, "markers": markers}
    for mode in ("image", "control"):
        answer = ask(item, mode == "image")
        hits = [m for m in markers if m.lower() in answer.lower()]
        row[mode] = {"answer": answer, "hits": hits, "hit_count": len(hits)}
    results.append(row)
    title = json.load(open(f"bench-cache/item_{item}.json"))["title"]
    print(f"=== {item}  {title}")
    print(f"    image-only markers: {len(markers)}")
    print(f"    with image : {row['image']['hit_count']}/{len(markers)}  {row['image']['hits']}")
    print(f"    control    : {row['control']['hit_count']}/{len(markers)}  {row['control']['hits']}")
    print("    --- with image ---")
    for line in row["image"]["answer"].strip().splitlines()[:14]:
        print(f"      {line}")
    print("    --- control (no image) ---")
    for line in row["control"]["answer"].strip().splitlines()[:6]:
        print(f"      {line}")
    print()

json.dump(results, open(OUT, "w"), indent=2, ensure_ascii=False)
print(f"wrote {OUT}")
