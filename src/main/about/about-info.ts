import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { AboutInfo } from "../../shared/about.js";

const currentDir = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(currentDir, "../../../");
const readmePath = path.join(projectRoot, "README.md");
const avatarPath = path.join(projectRoot, "assets/about/rayan6ms.png");
const projectUrl = "https://github.com/rayan6ms/camlet";
const projectIssuesUrl = `${projectUrl}/issues`;

function extractDescription(readme: string) {
	const lines = readme.split(/\r?\n/);
	return (
		lines.find(
			(line, index) =>
				index > 0 && line.trim().length > 0 && !line.trim().startsWith("#"),
		) ?? "Camlet is a lightweight Electron floating camera app for desktops."
	);
}

function extractScope(readme: string) {
	const lines = readme.split(/\r?\n/);
	const startIndex = lines.findIndex((line) =>
		line.includes("The current scope includes:"),
	);

	if (startIndex === -1) {
		return [];
	}

	const scope: string[] = [];

	for (const line of lines.slice(startIndex + 1)) {
		if (!line.startsWith("- ")) {
			if (scope.length > 0) {
				break;
			}

			continue;
		}

		scope.push(line.slice(2).trim());
	}

	return scope.slice(0, 8);
}

export async function getAboutInfo(): Promise<AboutInfo> {
	const [readme, avatar] = await Promise.all([
		readFile(readmePath, "utf8"),
		readFile(avatarPath),
	]);

	return {
		description: extractDescription(readme),
		scope: extractScope(readme),
		license: "GPL-3.0-only",
		githubHandle: "rayan6ms",
		githubUrl: "https://github.com/rayan6ms",
		projectUrl,
		projectIssuesUrl,
		avatarDataUrl: `data:image/jpeg;base64,${avatar.toString("base64")}`,
	};
}
