#!/usr/bin/env bash
# Runs the full text matrix once the machine is quiet: every model against
# every item and both prompts. MKL is off, so this uses candle's own backend.
set -u

cd "$(dirname "$0")/.."
BENCH=./target/release/bench
ITEMS=(3666883698 3121742525 3754128439)
PROMPTS=(features genres)
MODELS=(mistral qwen3-vl gemma4)

export RAYON_NUM_THREADS=16
export OMP_NUM_THREADS=16

echo "waiting for downloads and any running bench to finish"
while pgrep -f "release/bench" >/dev/null 2>&1 || \
      find ~/.cache/huggingface/hub -name "*.sync.part" 2>/dev/null | grep -q .; do
  sleep 30
done
echo "machine is quiet, starting matrix at $(date -Is)"

# Keep the contended MKL results rather than overwrite them.
if [ -d bench-out ] && [ -z "${KEEP_OUT:-}" ]; then
  mv bench-out "bench-out-mkl-$(date +%H%M%S)"
fi
mkdir -p bench-out

for model in "${MODELS[@]}"; do
  repo_arg=()
  [ "$model" = "mistral" ] && repo_arg=(--repo mistralai/Mistral-7B-Instruct-v0.2)
  for item in "${ITEMS[@]}"; do
    for prompt in "${PROMPTS[@]}"; do
      out=$($BENCH --model "$model" "${repo_arg[@]}" --item "$item" \
            --prompt "$prompt" --label nomkl 2>&1)
      status=$?
      if [ $status -eq 0 ]; then
        echo "$out" | tail -1
      else
        echo "FAILED ($status): $model $item $prompt :: $(echo "$out" | tail -2 | tr '\n' ' ')"
      fi
    done
  done
done

echo "matrix done at $(date -Is)"
