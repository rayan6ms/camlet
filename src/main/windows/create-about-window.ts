import { pathToFileURL } from "node:url";
import { BrowserWindow } from "electron/main";
import type { RendererAssetPolicy } from "../security.js";

export interface AboutWindowAssets extends RendererAssetPolicy {
	preloadPath: string;
}

export async function createAboutWindow({
	preloadPath,
	rendererHtmlPath,
	rendererUrl,
}: AboutWindowAssets): Promise<BrowserWindow> {
	const window = new BrowserWindow({
		width: 560,
		height: 620,
		minWidth: 460,
		minHeight: 520,
		show: false,
		autoHideMenuBar: true,
		backgroundColor: "#0B0F14",
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
