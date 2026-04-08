import { pathToFileURL } from "node:url";
import { BrowserWindow } from "electron/main";
import type { RendererAssetPolicy } from "../security.js";

export interface AboutWindowAssets extends RendererAssetPolicy {
	iconPath?: string;
	preloadPath: string;
}

export async function createAboutWindow({
	iconPath,
	preloadPath,
	rendererHtmlPath,
	rendererUrl,
}: AboutWindowAssets): Promise<BrowserWindow> {
	const window = new BrowserWindow({
		width: 376,
		height: 468,
		minWidth: 344,
		minHeight: 410,
		show: false,
		autoHideMenuBar: true,
		backgroundColor: "#0B0F14",
		...(iconPath !== undefined ? { icon: iconPath } : {}),
		title: "About Camlet",
		webPreferences: {
			preload: preloadPath,
			contextIsolation: true,
			nodeIntegration: false,
			sandbox: true,
			webSecurity: true,
			spellcheck: false,
			allowRunningInsecureContent: false,
			webviewTag: false,
		},
	});

	window.once("ready-to-show", () => {
		window.show();
	});

	if (rendererUrl !== undefined) {
		await window.loadURL(`${rendererUrl}#about`);
		return window;
	}

	await window.loadURL(`${pathToFileURL(rendererHtmlPath).toString()}#about`);
	return window;
}
