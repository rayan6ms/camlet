#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: scripts/run-native-performance.sh <new-output-directory>" >&2
  exit 2
fi

output_directory=$1
binary=target/release/camlet
startup_runs=${CAMLET_STARTUP_RUNS:-20}
preview_warmup_seconds=${CAMLET_PREVIEW_WARMUP_SECONDS:-10}
preview_sample_seconds=${CAMLET_PREVIEW_SAMPLE_SECONDS:-60}
idle_warmup_seconds=${CAMLET_IDLE_WARMUP_SECONDS:-30}
idle_sample_seconds=${CAMLET_IDLE_SAMPLE_SECONDS:-10}
clang_path=${LIBCLANG_PATH:-}

if [[ -e "$output_directory" ]]; then
  echo "output directory already exists: $output_directory" >&2
  exit 2
fi
if ! [[ "$startup_runs" =~ ^[1-9][0-9]*$ ]] \
  || ! [[ "$preview_warmup_seconds" =~ ^[0-9]+$ ]] \
  || ! [[ "$preview_sample_seconds" =~ ^[1-9][0-9]*$ ]] \
  || ! [[ "$idle_warmup_seconds" =~ ^[0-9]+$ ]] \
  || ! [[ "$idle_sample_seconds" =~ ^[1-9][0-9]*$ ]]; then
  echo "performance durations and run count must be non-negative integers" >&2
  exit 2
fi
if (( preview_warmup_seconds + preview_sample_seconds > 70 )); then
  echo "preview warm-up and sample window must total at most 70 seconds" >&2
  exit 2
fi
has_camera=false
if compgen -G '/dev/video*' >/dev/null; then
  has_camera=true
fi

mkdir -p "$output_directory"
if [[ -z "$clang_path" && -d target/build-tools/clang-libs/usr/lib64 ]]; then
  clang_path=$(realpath target/build-tools/clang-libs/usr/lib64)
fi
if [[ -n "$clang_path" ]]; then
  LIBCLANG_PATH="$clang_path" cargo build --release --locked -p camlet
else
  cargo build --release --locked -p camlet
fi
test -x "$binary"

cleanup_pid=""
cleanup() {
  if [[ -n "$cleanup_pid" ]] && kill -0 "$cleanup_pid" 2>/dev/null; then
    kill "$cleanup_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

startup_samples="$output_directory/startup-ms.tsv"
: >"$startup_samples"
for run in $(seq 1 "$startup_runs"); do
  run_directory="$output_directory/startup-$run"
  start_ns=$(date +%s%N)
  env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET \
    CAMLET_X11_RELAUNCHED=1 WINIT_UNIX_BACKEND=x11 "$binary" \
    --frame-source synthetic \
    --profile-dir "$run_directory/profile" \
    --automation-script fixtures/automation/startup-preview.json \
    --automation-output "$run_directory/results" \
    >"$run_directory.stdout" 2>"$run_directory.stderr" &
  cleanup_pid=$!
  for _ in $(seq 1 150); do
    [[ -f "$run_directory/results/ready.json" ]] && break
    kill -0 "$cleanup_pid"
    sleep 0.01
  done
  test -f "$run_directory/results/ready.json"
  ready_ns=$(date +%s%N)
  wait "$cleanup_pid"
  cleanup_pid=""
  test ! -s "$run_directory.stderr"
  printf '%d\t%d\n' "$run" "$(((ready_ns - start_ns) / 1000000))" >>"$startup_samples"
done

preview_directory="$output_directory/preview"
mkdir -p "$preview_directory/profile"
cp fixtures/settings/performance-640.json "$preview_directory/profile/settings-v1.json"
env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET \
  CAMLET_X11_RELAUNCHED=1 WINIT_UNIX_BACKEND=x11 "$binary" \
  --frame-source synthetic \
  --profile-dir "$preview_directory/profile" \
  --automation-script fixtures/automation/performance-preview-75s.json \
  --automation-output "$preview_directory/results" \
  >"$preview_directory/stdout" 2>"$preview_directory/stderr" &
cleanup_pid=$!
for _ in $(seq 1 150); do
  [[ -f "$preview_directory/results/ready.json" ]] && break
  kill -0 "$cleanup_pid"
  sleep 0.01
done
test -f "$preview_directory/results/ready.json"
sleep "$preview_warmup_seconds"
clock_ticks=$(getconf CLK_TCK)
cpu_start=$(awk -v ticks="$clock_ticks" '{ print ($14 + $15) / ticks }' "/proc/$cleanup_pid/stat")
wall_start=$(date +%s%N)
preview_pss="$preview_directory/pss-kib.tsv"
: >"$preview_pss"
sample_count=$((preview_sample_seconds / 2))
if (( sample_count < 1 )); then sample_count=1; fi
for sample in $(seq 1 "$sample_count"); do
  awk -v sample="$sample" '/^Pss:/ { print sample "\t" $2 }' "/proc/$cleanup_pid/smaps_rollup" >>"$preview_pss"
  sleep 2
done
cpu_end=$(awk -v ticks="$clock_ticks" '{ print ($14 + $15) / ticks }' "/proc/$cleanup_pid/stat")
wall_end=$(date +%s%N)
preview_cpu_percent=$(awk -v first="$cpu_start" -v last="$cpu_end" -v elapsed_ns="$((wall_end - wall_start))" 'BEGIN { printf "%.2f", (last - first) / (elapsed_ns / 1000000000) * 100 }')
wait "$cleanup_pid"
cleanup_pid=""
test ! -s "$preview_directory/stderr"

idle_pss=""
if [[ "$has_camera" == false ]]; then
  idle_directory="$output_directory/idle"
  mkdir -p "$idle_directory"
  env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET \
    CAMLET_X11_RELAUNCHED=1 WINIT_UNIX_BACKEND=x11 "$binary" \
    --frame-source real \
    --profile-dir "$idle_directory/profile" \
    >"$idle_directory/stdout" 2>"$idle_directory/stderr" &
  cleanup_pid=$!
  sleep "$idle_warmup_seconds"
  idle_pss="$idle_directory/pss-kib.tsv"
  : >"$idle_pss"
  idle_count=$((idle_sample_seconds / 2))
  if (( idle_count < 1 )); then idle_count=1; fi
  for sample in $(seq 1 "$idle_count"); do
    awk -v sample="$sample" '/^Pss:/ { print sample "\t" $2 }' "/proc/$cleanup_pid/smaps_rollup" >>"$idle_pss"
    sleep 2
  done
  kill "$cleanup_pid"
  wait "$cleanup_pid" || true
  cleanup_pid=""
fi

resize_directory="$output_directory/resize"
env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET \
  CAMLET_X11_RELAUNCHED=1 WINIT_UNIX_BACKEND=x11 "$binary" \
  --frame-source synthetic \
  --profile-dir "$resize_directory/profile" \
  --automation-script fixtures/automation/performance-resize-500.json \
  --automation-output "$resize_directory/results" \
  >"$resize_directory.stdout" 2>"$resize_directory.stderr" &
cleanup_pid=$!
resize_pss="$resize_directory.pss-kib.tsv"
: >"$resize_pss"
for _ in $(seq 1 150); do
  [[ -f "$resize_directory/results/ready.json" ]] && break
  kill -0 "$cleanup_pid"
  sleep 0.01
done
test -f "$resize_directory/results/ready.json"
for sample in $(seq 1 120); do
  if [[ -r "/proc/$cleanup_pid/smaps_rollup" ]]; then
    awk -v sample="$sample" '/^Pss:/ { print sample "\t" $2 }' "/proc/$cleanup_pid/smaps_rollup" >>"$resize_pss"
  fi
  [[ -f "$resize_directory/results/complete.json" ]] && break
  sleep 0.25
done
wait "$cleanup_pid"
cleanup_pid=""
test ! -s "$resize_directory.stderr"

median_column() {
  sort -n -k2 "$1" | awk '{ values[NR] = $2 } END { if (NR % 2) print values[(NR + 1) / 2]; else printf "%.1f\n", (values[NR / 2] + values[NR / 2 + 1]) / 2 }'
}
idle_median=null
if [[ -n "$idle_pss" ]]; then
  idle_median=$(median_column "$idle_pss")
fi
startup_median=$(median_column "$startup_samples")
startup_min=$(awk 'NR == 1 || $2 < minimum { minimum = $2 } END { print minimum }' "$startup_samples")
startup_max=$(awk 'NR == 1 || $2 > maximum { maximum = $2 } END { print maximum }' "$startup_samples")
preview_median=$(median_column "$preview_pss")
resize_baseline=$(sed -n '5,19p' "$resize_pss" | median_column /dev/stdin)
resize_final=$(tail -20 "$resize_pss" | median_column /dev/stdin)
resize_delta=$(awk -v baseline="$resize_baseline" -v final="$resize_final" 'BEGIN { printf "%.1f", final - baseline }')

jq -n \
  --argjson startupRuns "$startup_runs" \
  --argjson startupMedianMs "$startup_median" \
  --argjson startupMinMs "$startup_min" \
  --argjson startupMaxMs "$startup_max" \
  --argjson previewPssMedianKib "$preview_median" \
  --argjson previewCpuPercent "$preview_cpu_percent" \
  --argjson idlePssMedianKib "$idle_median" \
  --argjson resizeBaselinePssKib "$resize_baseline" \
  --argjson resizeFinalPssKib "$resize_final" \
  --argjson resizeDeltaKib "$resize_delta" \
  --argjson binaryBytes "$(stat -c %s "$binary")" \
  '{schemaVersion: 1, startup: {runs: $startupRuns, medianMs: $startupMedianMs, minMs: $startupMinMs, maxMs: $startupMaxMs}, preview640: {pssMedianKib: $previewPssMedianKib, cpuPercentOfOneCore: $previewCpuPercent}, idleNoDevice: {pssMedianKib: $idlePssMedianKib}, resize500: {baselineMedianPssKib: $resizeBaselinePssKib, finalMedianPssKib: $resizeFinalPssKib, sustainedDeltaKib: $resizeDeltaKib}, releaseBinaryBytes: $binaryBytes}' \
  >"$output_directory/summary.json"

cat "$output_directory/summary.json"
trap - EXIT INT TERM
