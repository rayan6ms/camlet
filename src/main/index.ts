import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { app, BrowserWindow, session } from "electron/main";
import { registerAppIpc } from "./ipc/register-app-ipc.js";
import {
	configureSessionSecurity,
	createRendererAssetPolicy,
	validateRendererUrl,
} from "./security.js";
import { SettingsStoreService } from "./services/settings-store.js";
import { createMainWindow } from "./windows/create-main-window.js";
import {
	bindMainWindowState,
	getSafeMainWindowState,
} from "./windows/window-state.js";

const require = createRequire(import.meta.url);
const currentDir = path.dirname(fileURLToPath(import.meta.url));
const packageJson = require("../../package.json") as {
	version?: string;
};
const preloadPath = path.join(currentDir, "../preload/index.cjs");
const rendererHtmlPath = path.join(
	currentDir,
	"../../dist/renderer/index.html",
);
const appVersion =
	typeof packageJson.version === "string" &&
	packageJson.version.trim().length > 0
		? packageJson.version
		: "0.0.0";
const appWithVersionSetter = app as typeof app & {
	setVersion?: (version: string) => void;
};

async function openMainWindow(
	settingsStore: SettingsStoreService,
	rendererPolicy = createRendererAssetPolicy(rendererHtmlPath),
) {
	const settings = settingsStore.getSettings();
	const windowState = getSafeMainWindowState(settingsStore);
	settingsStore.setWindowState(windowState);
	const window = await createMainWindow({
		overlayShape: settings.overlayShape,
		preloadPath,
		windowState,
		...rendererPolicy,
	});

	bindMainWindowState(window, settingsStore);
	return window;
}

app.setName("Camlet");
appWithVersionSetter.setVersion?.(appVersion);

app
	.whenReady()
	.then(async () => {
		const rendererUrl = validateRendererUrl(process.env.VITE_DEV_SERVER_URL);
		const rendererPolicy = createRendererAssetPolicy(
			rendererHtmlPath,
			rendererUrl,
		);

		configureSessionSecurity(session.defaultSession, rendererPolicy);

		const settingsStore = new SettingsStoreService();
		registerAppIpc(settingsStore);
		await openMainWindow(settingsStore, rendererPolicy);

		app.on("activate", async () => {
			if (BrowserWindow.getAllWindows().length === 0) {
				await openMainWindow(settingsStore, rendererPolicy);
			}
		});
	})
	.catch((error) => {
		console.error("Failed to bootstrap Camlet", error);
		app.exit(1);
	});

app.on("window-all-closed", () => {
	if (process.platform !== "darwin") {
		app.quit();
	}
});
