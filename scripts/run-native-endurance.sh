#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: scripts/run-native-endurance.sh <output-directory>" >&2
  exit 2
fi

output_directory=$1
binary=target/release/camlet
fixture=${CAMLET_ENDURANCE_FIXTURE:-fixtures/automation/endurance-30m.json}
sample_interval_seconds=${CAMLET_SAMPLE_INTERVAL_SECONDS:-60}
expected_screenshot=${CAMLET_EXPECTED_SCREENSHOT:-endurance.ppm}
maximum_pss_kib=${CAMLET_MAXIMUM_PSS_KIB:-122880}
maximum_growth_kib=${CAMLET_MAXIMUM_GROWTH_KIB:-65536}
results_directory="$output_directory/results"
samples="$output_directory/pss-kib.tsv"

if ! [[ "$sample_interval_seconds" =~ ^[1-9][0-9]*$ ]] \
  || ! [[ "$maximum_pss_kib" =~ ^[1-9][0-9]*$ ]] \
  || ! [[ "$maximum_growth_kib" =~ ^[1-9][0-9]*$ ]]; then
  echo "sample interval and memory limits must be positive integers" >&2
  exit 2
fi

if [[ -e "$output_directory" ]]; then
  echo "output directory already exists: $output_directory" >&2
  exit 2
fi

if [[ ! -x "$binary" ]] || [[ ! -f "$fixture" ]]; then
  echo "release binary or automation fixture is missing" >&2
  exit 2
fi

mkdir -p "$output_directory"
: >"$samples"

WINIT_UNIX_BACKEND="${WINIT_UNIX_BACKEND:-x11}" \
  "$binary" \
  --frame-source synthetic \
  --profile-dir "$output_directory/profile" \
  --automation-script "$fixture" \
  --automation-output "$results_directory" \
  >"$output_directory/stdout" \
  2>"$output_directory/stderr" &
camlet_pid=$!

cleanup() {
  if kill -0 "$camlet_pid" 2>/dev/null; then
    kill "$camlet_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

for _ in $(seq 1 60); do
  [[ -f "$results_directory/ready.json" ]] && break
  kill -0 "$camlet_pid"
  sleep 1
done
test -f "$results_directory/ready.json"

while kill -0 "$camlet_pid" 2>/dev/null; do
  if [[ -r "/proc/$camlet_pid/smaps_rollup" ]]; then
    pss_kib=$(awk '/^Pss:/ { print $2 }' "/proc/$camlet_pid/smaps_rollup")
    printf '%s\t%s\n' "$(date +%s)" "$pss_kib" >>"$samples"
  fi
  sleep "$sample_interval_seconds"
done

wait "$camlet_pid"
trap - EXIT INT TERM

jq -e '.status == "complete"' "$results_directory/complete.json" >/dev/null
jq -e '.camera.status == "preview"' "$results_directory/diagnostics.json" >/dev/null
test -s "$results_directory/$expected_screenshot"
test ! -s "$output_directory/stderr"

awk -v maximum_allowed="$maximum_pss_kib" -v growth_allowed="$maximum_growth_kib" '
  NR == 1 { first = $2 }
  { last = $2; if ($2 > maximum) maximum = $2 }
  END {
    if (NR == 0) exit 1
    printf "samples=%d first_pss_kib=%d last_pss_kib=%d max_pss_kib=%d delta_kib=%d\n", NR, first, last, maximum, last - first
    if (maximum > maximum_allowed || last - first > growth_allowed) exit 1
  }
' "$samples"
