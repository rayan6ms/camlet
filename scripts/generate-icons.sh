#!/usr/bin/env bash
set -euo pipefail

repository_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
source_icon="$repository_root/assets/camlet-rust.svg"
icon_directory="$repository_root/assets/icons"

command -v rsvg-convert >/dev/null || {
	printf 'rsvg-convert is required to generate Camlet icons.\n' >&2
	exit 1
}
command -v magick >/dev/null || {
	printf 'ImageMagick is required to generate the Windows icon.\n' >&2
	exit 1
}

mkdir -p "$icon_directory"
for size in 16 32 48 64 128 256 512; do
	rsvg-convert \
		--width "$size" \
		--height "$size" \
		--output "$icon_directory/${size}x${size}.png" \
		"$source_icon"
done

magick \
	"$icon_directory/256x256.png" \
	"$icon_directory/128x128.png" \
	"$icon_directory/64x64.png" \
	"$icon_directory/48x48.png" \
	"$icon_directory/32x32.png" \
	"$icon_directory/16x16.png" \
	"$repository_root/assets/icon.ico"
