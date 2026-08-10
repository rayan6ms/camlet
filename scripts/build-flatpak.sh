#!/usr/bin/env bash
set -euo pipefail

repository_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
manifest="$repository_root/packaging/flatpak/io.github.rayan6ms.camlet.yml"
vendor_directory="$repository_root/packaging/flatpak/vendor"
flatpak_root="$repository_root/target/flatpak"
build_directory="$flatpak_root/build"
repository="$flatpak_root/repository"
state_directory="$flatpak_root/state"
package_directory="$repository_root/target/packages"
bundle="$package_directory/camlet-rust_0.2.1_x86_64.flatpak"

command -v flatpak >/dev/null || {
	printf 'flatpak is required to build the Camlet Flatpak.\n' >&2
	exit 1
}

if command -v flatpak-builder >/dev/null; then
	builder=(flatpak-builder)
elif flatpak info --user org.flatpak.Builder >/dev/null 2>&1; then
	builder=(flatpak run --command=flatpak-builder org.flatpak.Builder)
else
	printf 'Install flatpak-builder or the user-scoped org.flatpak.Builder application.\n' >&2
	exit 1
fi

rm -rf -- "$vendor_directory" "$build_directory" "$repository" "$state_directory"
mkdir -p "$vendor_directory" "$flatpak_root" "$package_directory"
trap 'rm -rf -- "$vendor_directory"' EXIT

cargo vendor --quiet --locked "$vendor_directory" >/dev/null
"${builder[@]}" \
	--user \
	--force-clean \
	--disable-rofiles-fuse \
	--state-dir="$state_directory" \
	--repo="$repository" \
	"$build_directory" \
	"$manifest"
flatpak build-bundle \
	--runtime-repo=https://flathub.org/repo/flathub.flatpakrepo \
	"$repository" \
	"$bundle" \
	io.github.rayan6ms.camlet

if [[ "${CAMLET_TEST_FLATPAK:-0}" == "1" ]]; then
	flatpak install --user --noninteractive -y --reinstall "$bundle"
	flatpak run io.github.rayan6ms.camlet --frame-source synthetic --automation-check
	flatpak uninstall --user --noninteractive -y io.github.rayan6ms.camlet
fi

printf '%s\n' "$bundle"
