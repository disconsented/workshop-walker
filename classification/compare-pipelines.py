#!/usr/bin/env python3
"""One-go against transcribe-then-extract, across every test case.

one_go              a single call: preview + every description image, plus the
                    extraction prompt, all in one message
transcribe_extract  one transcription call per image, then the ordinary text
                    extraction over description + transcripts, no image attached

Only Gemma 4 E4B q4_0 on llama.cpp. Transcripts are produced once per item and
reused for both prompts, which is what a real pipeline would do.
"""
import base64
import json
import pathlib
import re
import sys
import time
import urllib.request

PORT = sys.argv[1] if len(sys.argv) > 1 else "8082"
OUT_DIR = pathlib.Path("bench-out-pipelines")
OUT_DIR.mkdir(exist_ok=True)
CACHE = pathlib.Path("bench-cache/images")
CACHE.mkdir(parents=True, exist_ok=True)

ITEMS = ["3666883698", "3121742525", "3754128439", "3799737423", "3761824516"]
PROMPTS = ["features", "genres"]
SKIP = re.compile(r"patreon|discord|ko-fi|steamcommunity", re.I)
# Bare JSON comes from a grammar, not from the prompt. Any trailing instruction
# about output format also makes the model terse: asking for "the JSON object
# only" cut one item from 16 features to 2. The schema constrains decoding
# instead, so the prompt is untouched and the content is unchanged.
SCHEMAS = {
    "features": {"types": "array", "features": "array"},
    "genres": {"genres": "array", "themes": "array"},
}


def response_format(prompt_key):
    fields = SCHEMAS[prompt_key]
    schema = {
        "type": "object",
        "properties": {f: {"type": "array", "items": {"type": "string"}} for f in fields},
        "required": list(fields),
        "additionalProperties": False,
    }
    return {"type": "json_schema",
            "json_schema": {"name": prompt_key, "schema": schema, "strict": True}}
TRANSCRIBE = ("Transcribe every label and heading in this image, one per line. "
              "Be exhaustive and literal.")


def call(parts, max_tokens=512, prompt_key=None):
    payload = {
        "messages": [{"role": "user", "content": parts}],
        "temperature": 0.0, "max_tokens": max_tokens,
        # Caching the shared prompt prefix would flatter whichever call runs
        # second, so every call here reprocesses its prompt in full.
        "cache_prompt": False,
        "chat_template_kwargs": {"enable_thinking": False},
    }
    if prompt_key:
        payload["response_format"] = response_format(prompt_key)
    body = json.dumps(payload).encode()
    req = urllib.request.Request(
        f"http://127.0.0.1:{PORT}/v1/chat/completions",
        data=body, headers={"Content-Type": "application/json"})
    started = time.monotonic()
    p = json.load(urllib.request.urlopen(req, timeout=3600))
    t = p.get("timings", {})
    return {
        "answer": p["choices"][0]["message"].get("content") or "",
        "finish": p["choices"][0].get("finish_reason"),
        "prompt_tokens": t.get("prompt_n") or 0,
        "generated_tokens": t.get("predicted_n") or 0,
        "wall_secs": round(time.monotonic() - started, 2),
    }


def image_part(path):
    b64 = base64.b64encode(path.read_bytes()).decode()
    return {"type": "image_url", "image_url": {"url": f"data:image/png;base64,{b64}"}}


def images_for(item, meta):
    """Preview first, then every description image that is not a donate badge."""
    urls = [meta["preview_url"]] + [
        u for u in re.findall(r'<img src="([^"]+)"', meta["description"]) if not SKIP.search(u)
    ]
    paths = []
    for n, url in enumerate(urls):
        path = CACHE / f"{item}_{n}.png"
        if not path.exists():
            try:
                urllib.request.urlretrieve(url, path)
            except Exception as exc:
                print(f"    skip {url[:60]}: {exc}")
                continue
        paths.append(path)
    return paths


def as_object(raw):
    body = raw.strip()
    if body.startswith("```"):
        body = body.split("\n", 1)[-1].rsplit("```", 1)[0]
    try:
        value = json.loads(body.strip())
        return value if isinstance(value, dict) else None
    except Exception:
        return None


for item in ITEMS:
    meta = json.load(open(f"bench-cache/item_{item}.json"))
    shots = images_for(item, meta)
    record = {"item": item, "title": meta["title"], "n_images": len(shots), "runs": {}}
    print(f"=== {item} {meta['title']}  ({len(shots)} images)")

    # Transcribe once, reuse for both prompts. Cached on disk too: the
    # transcription prompt is independent of the extraction prompt, so
    # comparing extraction changes must not pay for it again.
    cache_path = OUT_DIR / f"transcript_{item}.json"
    cached = json.load(open(cache_path)) if cache_path.exists() else None
    if cached and cached.get("n_calls") == len(shots):
        transcripts = cached["parts"]
        t_prompt, t_gen, t_wall = (cached["prompt_tokens"], cached["generated_tokens"],
                                   cached["wall_secs"])
        print(f"    reusing cached transcript ({len(shots)} images)")
    else:
        transcripts, t_prompt, t_gen, t_wall = [], 0, 0, 0.0
        for path in shots:
            r = call([image_part(path), {"type": "text", "text": TRANSCRIBE}], 500)
            transcripts.append(f"[{path.name}]\n{r['answer'].strip()}")
            t_prompt += r["prompt_tokens"]
            t_gen += r["generated_tokens"]
            t_wall += r["wall_secs"]
        json.dump({"n_calls": len(shots), "parts": transcripts, "prompt_tokens": t_prompt,
                   "generated_tokens": t_gen, "wall_secs": round(t_wall, 2)},
                  open(cache_path, "w"), indent=2, ensure_ascii=False)
    record["transcription"] = {
        "n_calls": len(shots), "prompt_tokens": t_prompt,
        "generated_tokens": t_gen, "wall_secs": round(t_wall, 2),
        "text": "\n\n".join(transcripts),
    }
    print(f"    transcribed {len(shots)} images: {t_gen} tok out, {t_wall:.1f}s")

    for prompt_key in PROMPTS:
        template = open(f"prompts/{prompt_key}.txt").read()

        def build(description):
            return (template.replace("[TITLE]", meta["title"])
                    .replace("[DESCRIPTION]", description).replace("\t", ""))

        one = call([image_part(p) for p in shots]
                   + [{"type": "text", "text": build(meta["description"])}],
                   prompt_key=prompt_key)
        one["object"] = as_object(one["answer"])

        enriched = (meta["description"]
                    + "\n\nText extracted from the mod's images:\n"
                    + record["transcription"]["text"])
        two = call(build(enriched), prompt_key=prompt_key)
        two["object"] = as_object(two["answer"])
        # Charge the transcription cost to the pipeline that needs it.
        two["total_prompt_tokens"] = two["prompt_tokens"] + t_prompt
        two["total_generated_tokens"] = two["generated_tokens"] + t_gen
        two["total_wall_secs"] = round(two["wall_secs"] + t_wall, 2)

        record["runs"][prompt_key] = {"one_go": one, "transcribe_extract": two}
        def count(r):
            o = r["object"] or {}
            return sum(len(v) for v in o.values() if isinstance(v, list))
        print(f"    {prompt_key:8} one_go {count(one):2} values, {one['wall_secs']:6.1f}s "
              f"| transcribe+extract {count(two):2} values, {two['wall_secs']:6.1f}s extract")

    # Per item, both prompts. Transcription is paid once and reused, so it is
    # counted once here rather than charged to each prompt.
    one_wall = sum(record["runs"][k]["one_go"]["wall_secs"] for k in PROMPTS)
    one_prompt = sum(record["runs"][k]["one_go"]["prompt_tokens"] for k in PROMPTS)
    two_wall = t_wall + sum(record["runs"][k]["transcribe_extract"]["wall_secs"] for k in PROMPTS)
    two_prompt = t_prompt + sum(
        record["runs"][k]["transcribe_extract"]["prompt_tokens"] for k in PROMPTS)
    record["per_item"] = {
        "one_go": {"calls": len(PROMPTS), "wall_secs": round(one_wall, 2),
                   "prompt_tokens": one_prompt},
        "transcribe_extract": {"calls": len(shots) + len(PROMPTS),
                               "wall_secs": round(two_wall, 2),
                               "prompt_tokens": two_prompt,
                               "transcription_wall_secs": round(t_wall, 2)},
    }
    ratio = two_wall / one_wall if one_wall else 0
    print(f"    per item: one_go {one_wall:6.1f}s / {len(PROMPTS)} calls  |  "
          f"transcribe+extract {two_wall:6.1f}s / {len(shots) + len(PROMPTS)} calls  "
          f"= {ratio:.1f}x the time")

    json.dump(record, open(OUT_DIR / f"pipelines_{item}.json", "w"),
              indent=2, ensure_ascii=False)

print(f"\nwrote {OUT_DIR}/pipelines_*.json")
