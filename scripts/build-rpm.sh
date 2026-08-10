#!/usr/bin/env bash
set -euo pipefail

repository_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
binary="$repository_root/target/release/camlet"
package_directory="$repository_root/target/packages"

command -v rpmbuild >/dev/null || {
	printf 'rpmbuild is required to build the Camlet RPM.\n' >&2
	exit 1
}

if [[ ! -x "$binary" ]]; then
	printf 'Build target/release/camlet before creating the RPM.\n' >&2
	exit 1
fi

mkdir -p "$repository_root/target" "$package_directory"
rpm_root=$(mktemp -d "$repository_root/target/rpm-build.XXXXXX")
trap 'rm -rf -- "$rpm_root"' EXIT
mkdir -p "$rpm_root/BUILD" "$rpm_root/BUILDROOT" "$rpm_root/RPMS" "$rpm_root/SOURCES" "$rpm_root/SPECS" "$rpm_root/SRPMS"

install -Dm0755 "$binary" "$rpm_root/SOURCES/camlet"
install -Dm0644 "$repository_root/packaging/linux/io.github.rayan6ms.camlet.desktop" "$rpm_root/SOURCES/io.github.rayan6ms.camlet.desktop"
install -Dm0644 "$repository_root/packaging/linux/io.github.rayan6ms.camlet.metainfo.xml" "$rpm_root/SOURCES/io.github.rayan6ms.camlet.metainfo.xml"
install -Dm0644 "$repository_root/assets/camlet-rust.svg" "$rpm_root/SOURCES/camlet-rust.svg"
install -Dm0644 "$repository_root/assets/icons/256x256.png" "$rpm_root/SOURCES/camlet-rust-256.png"
install -Dm0644 "$repository_root/LICENSE" "$rpm_root/SOURCES/LICENSE"
install -Dm0644 "$repository_root/README.md" "$rpm_root/SOURCES/README.md"
install -Dm0644 "$repository_root/packaging/linux/camlet.1" "$rpm_root/SOURCES/camlet.1"
install -Dm0644 "$repository_root/packaging/rpm/camlet.spec" "$rpm_root/SPECS/camlet.spec"

rpmbuild --define "_topdir $rpm_root" -bb "$rpm_root/SPECS/camlet.spec"
rpm_path=$(find "$rpm_root/RPMS" -type f -name '*.rpm' -print -quit)
test -n "$rpm_path"
install -Dm0644 "$rpm_path" "$package_directory/$(basename "$rpm_path")"
rpm -K "$package_directory/$(basename "$rpm_path")"
rpm -qpl "$package_directory/$(basename "$rpm_path")" >/dev/null
printf '%s\n' "$package_directory/$(basename "$rpm_path")"
