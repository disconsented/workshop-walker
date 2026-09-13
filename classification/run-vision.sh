#!/usr/bin/env bash
# Secondary run: the same items and prompts, with the item's preview image
# attached. Only the vision models take part; Mistral has no vision tower.
#
# The vision path is new code, so each model gets one probe run first. If the
# probe fails the rest of that model is skipped, and the error is recorded
# rather than repeated eleven more times.
set -u

cd "$(dirname "$0")/.."
BENCH=./target/release/bench
ITEMS=(3666883698 3121742525 3754128439)
PROMPTS=(features genres)
MODELS=(qwen3-vl gemma4)

export RAYON_NUM_THREADS=16
export OMP_NUM_THREADS=16

echo "waiting for the text matrix to finish"
while pgrep -f "release/bench" >/dev/null 2>&1; do
  sleep 30
done
echo "starting vision matrix at $(date -Is)"

mkdir -p bench-out

run() {
  local model=$1 item=$2 prompt=$3
  local out status
  out=$($BENCH --model "$model" --item "$item" --prompt "$prompt" \
        --image --label nomkl-vision 2>&1)
  status=$?
  if [ $status -eq 0 ]; then
    echo "$out" | tail -1
  else
    echo "FAILED ($status): $model $item $prompt image :: $(echo "$out" | tail -2 | tr '\n' ' ')"
  fi
  return $status
}

for model in "${MODELS[@]}"; do
  echo "--- $model probe ---"
  if ! run "$model" "${ITEMS[0]}" "${PROMPTS[0]}"; then
    echo "SKIPPING $model: the probe run failed"
    continue
  fi
  for item in "${ITEMS[@]}"; do
    for prompt in "${PROMPTS[@]}"; do
      # The probe already covered this combination.
      [ "$item" = "${ITEMS[0]}" ] && [ "$prompt" = "${PROMPTS[0]}" ] && continue
      run "$model" "$item" "$prompt"
    done
  done
done

echo "vision matrix done at $(date -Is)"
