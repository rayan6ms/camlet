#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
	echo "usage: scripts/generate-native-inventories.sh <output-directory>" >&2
	exit 2
fi

output_directory=$1
if [[ -e "$output_directory" ]]; then
	echo "inventory output directory already exists: $output_directory" >&2
	exit 2
fi

mkdir -p "$output_directory"

SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-0} cargo cyclonedx \
	--manifest-path Cargo.toml \
	--format json \
	--all-features \
	--spec-version 1.5 \
	--override-filename camlet.cdx

mv crates/camlet-core/camlet.cdx.json "$output_directory/camlet-core.cdx.json"
mv crates/camlet-camera/camlet.cdx.json "$output_directory/camlet-camera.cdx.json"
mv crates/camlet/camlet.cdx.json "$output_directory/camlet.cdx.json"

for sbom in "$output_directory"/*.cdx.json; do
	jq -e '.bomFormat == "CycloneDX" and .specVersion == "1.5"' "$sbom" >/dev/null
done

cargo deny list --format json >"$output_directory/licenses.json"
jq -e 'length > 0' "$output_directory/licenses.json" >/dev/null

