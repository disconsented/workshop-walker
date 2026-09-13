#!/usr/bin/env bash
# Runs the same three items and two prompts through llama.cpp, for comparison
# with the candle rows. Settings match the candle bench: greedy sampling,
# repeat penalty 1.1 over the last 64 tokens, 256 token cap.
#
# Usage: run-llamacpp.sh <model.gguf> <label> [extra llama-server args...]
set -eu

cd "$(dirname "$0")/.."
SCRATCH=${SCRATCH:?set SCRATCH to the scratchpad directory}
SERVER="$SCRATCH/llama.cpp/build/bin/llama-server"
MODEL=$1
LABEL=$2
shift 2
# Set IMAGE=1 to attach each item's preview; needs --mmproj in the extra args.
IMAGE=${IMAGE:-0}

OUT=bench-out-llamacpp
mkdir -p "$OUT"

PORT=${PORT:-8081}
"$SERVER" -m "$MODEL" --host 127.0.0.1 --port "$PORT" \
  -c 4096 -t 16 --jinja --no-warmup "$@" > "$SCRATCH/llama-server-$LABEL.log" 2>&1 &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null || true' EXIT

echo "waiting for llama-server (pid $SERVER_PID)"
for _ in $(seq 1 120); do
  if curl -sf "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then break; fi
  if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "server died, last lines:"; tail -15 "$SCRATCH/llama-server-$LABEL.log"; exit 1
  fi
  sleep 2
done
echo "server up"

for item in ${ITEMS:-3666883698 3121742525 3754128439}; do
  for prompt in features genres; do
    python3 - "$item" "$prompt" "$PORT" "$LABEL" "$OUT" "$MODEL" "$IMAGE" <<'PY'
import json, sys, time, urllib.request

item, prompt_key, port, label, out_dir, model_path, want_image = sys.argv[1:8]
want_image = want_image == "1"

meta = json.load(open(f"bench-cache/item_{item}.json"))
template = open(f"prompts/{prompt_key}.txt").read()
content = (
    template.replace("[TITLE]", meta["title"])
    .replace("[DESCRIPTION]", meta["description"])
    .replace("\t", "")
)

if want_image:
    import base64, mimetypes, os
    path = f"bench-cache/preview_{item}"
    if not os.path.exists(path):
        import urllib.request as _u
        _u.urlretrieve(meta["preview_url"], path)
    with open(path, "rb") as fh:
        b64 = base64.b64encode(fh.read()).decode()
    # The previews are PNG; the cache file has no extension to sniff.
    message_content = [
        {"type": "image_url", "image_url": {"url": f"data:image/png;base64,{b64}"}},
        {"type": "text", "text": content},
    ]
else:
    message_content = content

body = json.dumps({
    "messages": [{"role": "user", "content": message_content}],
    "temperature": 0.0,
    "top_k": 1,
    "repeat_penalty": 1.1,
    "repeat_last_n": 64,
    "max_tokens": 256,
    "cache_prompt": False,
    # Gemma 4 reasons before answering; unchecked it spends 895 tokens to reach
    # a 51 token answer, and the cap cuts it off mid-thought with empty content.
    "chat_template_kwargs": {"enable_thinking": False},
}).encode()

req = urllib.request.Request(
    f"http://127.0.0.1:{port}/v1/chat/completions",
    data=body,
    headers={"Content-Type": "application/json"},
)
started = time.monotonic()
with urllib.request.urlopen(req, timeout=1800) as resp:
    payload = json.load(resp)
wall = time.monotonic() - started

message = payload["choices"][0]["message"]
text = message.get("content") or ""
reasoning = message.get("reasoning_content") or ""
finish = payload["choices"][0].get("finish_reason")
t = payload.get("timings", {})


def parses(raw):
    """Accepts a bare object or one inside a markdown fence. Fencing is a
    formatting habit, not a content failure, and every model has its own."""
    body = raw.strip()
    if body.startswith("```"):
        body = body.split("\n", 1)[-1]
        body = body.rsplit("```", 1)[0]
    try:
        return isinstance(json.loads(body.strip()), dict)
    except Exception:
        return False


ok = parses(text)

report = {
    "runtime": "llama.cpp",
    "label": label,
    "model_file": model_path.rsplit("/", 1)[-1],
    "item": item,
    "title": meta["title"],
    "prompt": prompt_key,
    "with_image": want_image,
    "prefill_tokens": t.get("prompt_n"),
    "prefill_secs": (t.get("prompt_ms") or 0) / 1000.0,
    "decode_tokens": t.get("predicted_n"),
    "decode_secs": (t.get("predicted_ms") or 0) / 1000.0,
    "prefill_tokens_per_sec": t.get("prompt_per_second"),
    "decode_tokens_per_sec": t.get("predicted_per_second"),
    "wall_secs": wall,
    "finish_reason": finish,
    "parses_as_json": ok,
    "raw_output": text,
    "reasoning_output": reasoning,
}
modality = "image" if want_image else "text"
name = f"{out_dir}/llamacpp_{label}_{item}_{prompt_key}_{modality}.json"
json.dump(report, open(name, "w"), indent=2)
print(
    f"{label} {item} {prompt_key}: prefill {report['prefill_tokens']} tok at "
    f"{report['prefill_tokens_per_sec'] or 0:.1f} tok/s, decode "
    f"{report['decode_tokens']} tok at {report['decode_tokens_per_sec'] or 0:.2f} tok/s, "
    f"json={ok} -> {name}"
)
PY
  done
done

# The server holds the model for every request, so its high-water mark is the
# figure to compare against candle's per-process peak.
peak_kb=$(awk '/VmHWM/ {print $2}' "/proc/$SERVER_PID/status" 2>/dev/null || echo 0)
rss_kb=$(awk '/VmRSS/ {print $2}' "/proc/$SERVER_PID/status" 2>/dev/null || echo 0)
python3 - "$OUT" "$LABEL" "$peak_kb" "$rss_kb" "$MODEL" <<'PY'
import json, os, sys
out_dir, label, peak_kb, rss_kb, model_path = sys.argv[1:6]
json.dump({
    "label": label,
    "model_file": model_path.rsplit("/", 1)[-1],
    "weight_bytes": os.path.getsize(model_path),
    "peak_rss_bytes": int(peak_kb) * 1024,
    "final_rss_bytes": int(rss_kb) * 1024,
}, open(f"{out_dir}/_memory_{label}.json", "w"), indent=2)
print(f"{label}: peak RSS {int(peak_kb)*1024/2**30:.2f} GiB, "
      f"weights {os.path.getsize(model_path)/2**30:.2f} GiB")
PY
