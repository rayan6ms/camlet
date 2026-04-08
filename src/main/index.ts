import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { BrowserWindow as BrowserWindowType } from "electron/main";
import { resolveLinuxWindowIconPath } from "./icon-path.js";
import { registerAppIpc } from "./ipc/register-app-ipc.js";
import {
	configureSessionSecurity,
	createRendererAssetPolicy,
	validateRendererUrl,
} from "./security.js";
import { SettingsStoreService } from "./services/settings-store.js";
import { createAboutWindow } from "./windows/create-about-window.js";
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
const linuxWindowIconPath = resolveLinuxWindowIconPath(currentDir);
const appVersion =
	typeof packageJson.version === "string" &&
	packageJson.version.trim().length > 0
		? packageJson.version
		: "0.0.0";
const isLinux = process.platform === "linux";

function hasExplicitOzonePlatformArgument() {
	return process.argv.some(
		(argument) =>
			argument === "--ozone-platform" ||
			argument.startsWith("--ozone-platform="),
	);
}

function shouldPreferX11Windowing() {
	if (!isLinux) {
		return false;
	}

	if (process.env.CAMLET_PREFER_WAYLAND === "1") {
		return false;
	}

	if (hasExplicitOzonePlatformArgument()) {
		return false;
	}

	return Boolean(process.env.DISPLAY?.trim());
}

function configureLinuxWindowingEnvironment() {
	if (!shouldPreferX11Windowing()) {
		return;
	}

	process.env.ELECTRON_OZONE_PLATFORM_HINT = "x11";
	process.env.OZONE_PLATFORM = "x11";
	process.env.GDK_BACKEND = "x11";
	delete process.env.WAYLAND_DISPLAY;
}

configureLinuxWindowingEnvironment();

const { app, BrowserWindow, session } = await import("electron/main");
const appWithVersionSetter = app as typeof app & {
	setVersion?: (version: string) => void;
};

function configureLinuxWindowingBackend() {
	if (
		!shouldPreferX11Windowing() ||
		app.commandLine.hasSwitch("ozone-platform")
	) {
		return;
	}

	app.commandLine.appendSwitch("ozone-platform", "x11");
}

configureLinuxWindowingBackend();

async function openMainWindow(
	settingsStore: SettingsStoreService,
	rendererPolicy = createRendererAssetPolicy(rendererHtmlPath),
) {
	const settings = settingsStore.getSettings();
	const windowState = getSafeMainWindowState(settingsStore);
	settingsStore.setWindowState(windowState);
	const window = await createMainWindow({
		appearance: {
			overlayShape: settings.overlayShape,
			cornerRoundness: settings.cornerRoundness,
		},
		...(linuxWindowIconPath !== undefined
			? { iconPath: linuxWindowIconPath }
			: {}),
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
		let aboutWindow: BrowserWindowType | null = null;
		const openAboutWindow = async () => {
			if (aboutWindow !== null && !aboutWindow.isDestroyed()) {
				aboutWindow.show();
				aboutWindow.focus();
				return;
			}

			aboutWindow = await createAboutWindow({
				...(linuxWindowIconPath !== undefined
					? { iconPath: linuxWindowIconPath }
					: {}),
				preloadPath,
				...rendererPolicy,
			});
			aboutWindow.on("closed", () => {
				aboutWindow = null;
			});
		};

		registerAppIpc(settingsStore, {
			openAboutWindow,
		});
		const mainWindow = await openMainWindow(settingsStore, rendererPolicy);

		if (rendererUrl !== undefined) {
			const { openDevelopmentDiagnosticsWindow } = await import(
				"./dev/diagnostics-window.js"
			);
			await openDevelopmentDiagnosticsWindow({
				overlayWindow: mainWindow,
			});
		}

		app.on("activate", async () => {
			if (BrowserWindow.getAllWindows().length === 0) {
				const nextMainWindow = await openMainWindow(
					settingsStore,
					rendererPolicy,
				);

				if (rendererUrl !== undefined) {
					const { openDevelopmentDiagnosticsWindow } = await import(
						"./dev/diagnostics-window.js"
					);
					await openDevelopmentDiagnosticsWindow({
						overlayWindow: nextMainWindow,
					});
				}
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
