import path from "node:path";

export function resolveLinuxWindowIconPath(
	currentDir: string,
): string | undefined {
	if (process.platform !== "linux") {
		return undefined;
	}

	if (process.env.VITE_DEV_SERVER_URL !== undefined) {
		return path.join(currentDir, "../../assets/icons/512x512.png");
	}

	return path.join(process.resourcesPath, "icons", "512x512.png");
}
