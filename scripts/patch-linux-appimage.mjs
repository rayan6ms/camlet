import { execFileSync } from "node:child_process";
import {
	chmodSync,
	closeSync,
	mkdtempSync,
	openSync,
	readdirSync,
	readFileSync,
	rmSync,
	statSync,
	writeFileSync,
	writeSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";

const projectRoot = process.cwd();
const releaseDir = path.join(projectRoot, "release");
const appImageFiles = readdirSync(releaseDir)
	.filter((name) => name.endsWith(".AppImage"))
	.map((name) => path.join(releaseDir, name));

if (appImageFiles.length !== 1) {
	throw new Error(
		`Expected exactly one AppImage in ${releaseDir}, found ${appImageFiles.length}.`,
	);
}

const appImagePath = appImageFiles[0];
const cacheRoot = path.join(
	process.env.XDG_CACHE_HOME || path.join(os.homedir(), ".cache"),
	"electron-builder",
	"appimage",
);
const runtimeBundle = readdirSync(cacheRoot)
	.filter((name) => name.startsWith("appimage-"))
	.sort()
	.at(-1);

if (runtimeBundle === undefined) {
	throw new Error(
		`Unable to locate the electron-builder AppImage cache in ${cacheRoot}.`,
	);
}

const runtimeRoot = path.join(cacheRoot, runtimeBundle);
const runtimePath = path.join(runtimeRoot, "runtime-x64");
const mksquashfsPath = path.join(runtimeRoot, "linux-x64", "mksquashfs");
const offset = statSync(runtimePath).size;
const tempDir = mkdtempSync(path.join(os.tmpdir(), "camlet-appimage-"));
const extractedAppDir = path.join(tempDir, "AppDir");

const patchedAppRun = `#!/bin/bash
set -e

if [ ! -z "$DEBUG" ] ; then
  env
  set -x
fi

THIS="$0"
args=("$@")
NUMBER_OF_ARGS="$#"

if [ -z "$APPDIR" ] ; then
  path="$(dirname "$(readlink -f "\${THIS}")")"
  while [[ "$path" != "" && ! -e "$path/AppRun" ]]; do
    path=\${path%/*}
  done
  APPDIR="$path"
fi

export PATH="\${APPDIR}:\${APPDIR}/usr/sbin:\${PATH}"
export XDG_DATA_DIRS="./share/:/usr/share/gnome:/usr/local/share/:/usr/share/:\${XDG_DATA_DIRS}"
export LD_LIBRARY_PATH="\${APPDIR}/usr/lib:\${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="\${APPDIR}"/usr/share/:"\${XDG_DATA_DIRS}":/usr/share/gnome/:/usr/local/share/:/usr/share/
export GSETTINGS_SCHEMA_DIR="\${APPDIR}/usr/share/glib-2.0/schemas:\${GSETTINGS_SCHEMA_DIR}"

BIN="\${APPDIR}/camlet"

if [ -z "$APPIMAGE_EXIT_AFTER_INSTALL" ] ; then
  trap atexit EXIT
fi

isEulaAccepted=1

has_argument()
{
  for arg in "\${args[@]}"; do
    if [ "$arg" = "$1" ] || [[ "$arg" == "$1="* ]] ; then
      return 0
    fi
  done

  return 1
}

atexit()
{
  if [ $isEulaAccepted == 1 ] ; then
    extra_args=()

    if ! has_argument "--no-sandbox" && ! has_argument "--enable-sandbox" ; then
      extra_args+=("--no-sandbox")
    fi

    if [ "$CAMLET_PREFER_WAYLAND" != "1" ] && [ -n "$DISPLAY" ] && ! has_argument "--ozone-platform" ; then
      extra_args+=("--ozone-platform=x11")
    fi

    if [ $NUMBER_OF_ARGS -eq 0 ] ; then
      if [ \${#extra_args[@]} -eq 0 ] ; then
        exec "$BIN"
      else
        exec "$BIN" "\${extra_args[@]}"
      fi
    elif [ \${#extra_args[@]} -eq 0 ] ; then
      exec "$BIN"
    else
      exec "$BIN" "\${extra_args[@]}" "\${args[@]}"
    fi
  fi
}

error()
{
  if [ -x /usr/bin/zenity ] ; then
    LD_LIBRARY_PATH="" zenity --error --text "\${1}" 2>/dev/null
  elif [ -x /usr/bin/kdialog ] ; then
    LD_LIBRARY_PATH="" kdialog --msgbox "\${1}" 2>/dev/null
  elif [ -x /usr/bin/Xdialog ] ; then
    LD_LIBRARY_PATH="" Xdialog --msgbox "\${1}" 2>/dev/null
  else
    echo "\${1}"
  fi
  exit 1
}

yesno()
{
  TITLE=$1
  TEXT=$2
  if [ -x /usr/bin/zenity ] ; then
    LD_LIBRARY_PATH="" zenity --question --title="$TITLE" --text="$TEXT" 2>/dev/null || exit 0
  elif [ -x /usr/bin/kdialog ] ; then
    LD_LIBRARY_PATH="" kdialog --title "$TITLE" --yesno "$TEXT" || exit 0
  elif [ -x /usr/bin/Xdialog ] ; then
    LD_LIBRARY_PATH="" Xdialog --title "$TITLE" --clear --yesno "$TEXT" 10 80 || exit 0
  else
    echo "zenity, kdialog, Xdialog missing. Skipping \${THIS}."
    exit 0
  fi
}

check_dep()
{
  DEP=$1
  if [ -z $(which "$DEP") ] ; then
    echo "$DEP is missing. Skipping \${THIS}."
    exit 0
  fi
}

if [ -z "$APPIMAGE" ] ; then
  APPIMAGE="$APPDIR/AppRun"
fi
`;

try {
	execFileSync(
		"unsquashfs",
		["-offset", String(offset), "-d", extractedAppDir, appImagePath],
		{ stdio: "inherit" },
	);

	const appRunPath = path.join(extractedAppDir, "AppRun");
	const existingAppRun = readFileSync(appRunPath, "utf8");

	if (!existingAppRun.includes('exec "$BIN"')) {
		throw new Error(`Unexpected AppRun format in ${appRunPath}.`);
	}

	writeFileSync(appRunPath, patchedAppRun);
	chmodSync(appRunPath, 0o755);

	execFileSync(
		mksquashfsPath,
		[
			extractedAppDir,
			appImagePath,
			"-offset",
			String(offset),
			"-all-root",
			"-noappend",
			"-no-progress",
			"-quiet",
			"-no-xattrs",
			"-no-fragments",
		],
		{ stdio: "inherit" },
	);

	const runtimeData = readFileSync(runtimePath);
	const outputDescriptor = openSync(appImagePath, "r+");

	try {
		writeSync(outputDescriptor, runtimeData, 0, runtimeData.length, 0);
	} finally {
		closeSync(outputDescriptor);
	}

	chmodSync(appImagePath, 0o755);
} finally {
	rmSync(tempDir, { recursive: true, force: true });
}
