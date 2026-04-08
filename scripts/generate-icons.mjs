import { execFileSync } from "node:child_process";
import {
	mkdirSync,
	mkdtempSync,
	readFileSync,
	rmSync,
	writeFileSync,
} from "node:fs";
import path from "node:path";

const projectRoot = process.cwd();
const sourceIconPath = path.join(projectRoot, "assets", "icon.svg");
const outputDirectoryPath = path.join(projectRoot, "assets", "icons");
const outputIcoPath = path.join(projectRoot, "assets", "icon.ico");
const temporaryRootPath = path.join(projectRoot, ".tmp");
const pngSizes = [16, 32, 48, 64, 128, 256, 512];
const renderSize = 1024;

function resolveChromiumBinary() {
	const candidates = [
		process.env.CHROMIUM_BIN,
		"/snap/bin/chromium",
		"chromium",
		"chromium-browser",
		"google-chrome",
		"google-chrome-stable",
	].filter((value) => typeof value === "string" && value.length > 0);

	for (const candidate of candidates) {
		try {
			execFileSync(candidate, ["--version"], {
				stdio: "ignore",
			});
			return candidate;
		} catch {}
	}

	throw new Error(
		"Could not find a Chromium binary. Set CHROMIUM_BIN to an installed Chromium/Chrome executable.",
	);
}

function run(command, args) {
	execFileSync(command, args, {
		cwd: projectRoot,
		stdio: "inherit",
	});
}

const chromiumBinary = resolveChromiumBinary();
mkdirSync(temporaryRootPath, { recursive: true });

const tempDirectoryPath = mkdtempSync(
	path.join(temporaryRootPath, "camlet-icons-"),
);
const renderHtmlPath = path.join(tempDirectoryPath, "render-icon.html");
const masterPngPath = path.join(tempDirectoryPath, "icon-master.png");
const svgMarkup = readFileSync(sourceIconPath, "utf8");

try {
	mkdirSync(outputDirectoryPath, { recursive: true });

	writeFileSync(
		renderHtmlPath,
		`<!doctype html>
<html>
<head>
<meta charset="utf-8" />
<style>
html, body {
	margin: 0;
	width: ${renderSize}px;
	height: ${renderSize}px;
	overflow: hidden;
	background: transparent;
}
body > svg {
	display: block;
	width: ${renderSize}px;
	height: ${renderSize}px;
}
</style>
</head>
<body>
${svgMarkup}
</body>
</html>`,
	);

	run(chromiumBinary, [
		"--headless",
		"--disable-gpu",
		"--hide-scrollbars",
		"--default-background-color=00000000",
		`--window-size=${renderSize},${renderSize}`,
		`--screenshot=${masterPngPath}`,
		`file://${renderHtmlPath}`,
	]);

	for (const size of pngSizes) {
		const outputPath = path.join(outputDirectoryPath, `${size}x${size}.png`);
		run("magick", [
			masterPngPath,
			"-background",
			"none",
			"-resize",
			`${size}x${size}`,
			outputPath,
		]);
	}

	run("magick", [
		path.join(outputDirectoryPath, "256x256.png"),
		path.join(outputDirectoryPath, "128x128.png"),
		path.join(outputDirectoryPath, "64x64.png"),
		path.join(outputDirectoryPath, "48x48.png"),
		path.join(outputDirectoryPath, "32x32.png"),
		path.join(outputDirectoryPath, "16x16.png"),
		outputIcoPath,
	]);
} finally {
	rmSync(tempDirectoryPath, {
		force: true,
		recursive: true,
	});
}
