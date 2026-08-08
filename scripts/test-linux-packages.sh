#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
	echo "usage: scripts/test-linux-packages.sh <appimage> <deb> <output-directory>" >&2
	exit 2
fi

appimage=$(realpath "$1")
deb=$(realpath "$2")
output_directory=$3
fixture=$(realpath fixtures/automation/full-smoke.json)

if [[ ! -x "$appimage" ]] || [[ ! -f "$deb" ]] || [[ -e "$output_directory" ]]; then
	echo "packages must exist and the output directory must be new" >&2
	exit 2
fi

mkdir -p "$output_directory/appimage" "$output_directory/installed" "$output_directory/profile"

APPIMAGE_EXTRACT_AND_RUN=1 xvfb-run --auto-servernum "$appimage" \
	--frame-source synthetic \
	--profile-dir "$output_directory/appimage/profile" \
	--automation-script "$fixture" \
	--automation-output "$output_directory/appimage/results"

jq -e '.status == "complete"' "$output_directory/appimage/results/complete.json" >/dev/null
jq -e '.camera.status == "preview" and .camera.deviceCount == 1' \
	"$output_directory/appimage/results/diagnostics.json" >/dev/null
for capture in original circle rounded-square diamond rectangle-y rectangle-x overlay; do
	test -s "$output_directory/appimage/results/$capture.ppm"
done

sudo apt-get install --yes "$deb"
test -x /usr/bin/camlet

for run in first reinstall; do
	xvfb-run --auto-servernum /usr/bin/camlet \
		--frame-source synthetic \
		--profile-dir "$output_directory/profile" \
		--automation-script "$fixture" \
		--automation-output "$output_directory/installed/$run"
	jq -e '.status == "complete"' "$output_directory/installed/$run/complete.json" >/dev/null
	test -s "$output_directory/installed/$run/overlay.ppm"

	if [[ "$run" == "first" ]]; then
		native_settings="$output_directory/profile/settings-v1.json"
		test -s "$native_settings"
		sudo apt-get remove --yes camlet
		test ! -e /usr/bin/camlet
		test -s "$native_settings"
		sudo apt-get install --yes "$deb"
	fi
done

sudo apt-get remove --yes camlet
test ! -e /usr/bin/camlet
test -s "$native_settings"
appimage_directory=$(dirname "$appimage")
appimage_name=$(basename "$appimage")
deb_directory=$(dirname "$deb")
deb_name=$(basename "$deb")
{
	(cd "$appimage_directory" && sha256sum "$appimage_name")
	(cd "$deb_directory" && sha256sum "$deb_name")
} >"$output_directory/SHA256SUMS"
