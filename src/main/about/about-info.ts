import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { AboutInfo } from "../../shared/about.js";
import { aboutMetadata } from "./about-metadata.js";

const currentDir = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(currentDir, "../../../");
const avatarPath = path.join(projectRoot, "assets/about/rayan6ms.png");

export async function getAboutInfo(): Promise<AboutInfo> {
	const avatar = await readFile(avatarPath);

	return {
		...aboutMetadata,
		avatarDataUrl: `data:image/jpeg;base64,${avatar.toString("base64")}`,
	};
}
