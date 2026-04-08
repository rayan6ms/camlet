import { setTimeout as delay } from "node:timers/promises";
import { BrowserWindow } from "electron/main";
import type { OverlayAppearanceSettings } from "../../shared/appearance.js";
import {
	minimumWindowHeight,
	minimumWindowWidth,
	type WindowState,
} from "../../shared/window-state.js";
import { showMainWindowContextMenu } from "../ipc/context-menu.js";
import {
	isAllowedNavigationTarget,
	type RendererAssetPolicy,
} from "../security.js";
import { applyMainWindowShape } from "./window-shape.js";

export interface MainWindowAssets extends RendererAssetPolicy {
	appearance: Pick<
		OverlayAppearanceSettings,
		"overlayShape" | "cornerRoundness"
	>;
	preloadPath: string;
	windowState: WindowState;
}

function hardenWindowNavigation(
	window: BrowserWindow,
	rendererPolicy: RendererAssetPolicy,
) {
	window.webContents.setWindowOpenHandler(() => ({
		action: "deny",
	}));

	window.webContents.on("will-attach-webview", (event) => {
		event.preventDefault();
	});

	window.webContents.on("will-navigate", (event, targetUrl) => {
		if (isAllowedNavigationTarget(targetUrl, rendererPolicy)) {
			return;
		}

		event.preventDefault();
	});

	window.webContents.on("will-redirect", (event, targetUrl) => {
		if (isAllowedNavigationTarget(targetUrl, rendererPolicy)) {
			return;
		}

		event.preventDefault();
	});
}

function attachDevWindowDiagnostics(
	window: BrowserWindow,
	rendererPolicy: RendererAssetPolicy,
) {
	if (rendererPolicy.rendererUrl === undefined) {
		return;
	}

	const logPrefix = "[camlet:window]";
	const logTarget = rendererPolicy.rendererUrl;
	const logConsoleMessage = (level: number, message: string) => {
		if (level >= 2) {
			console.error(message);
			return;
		}

		if (level === 1) {
			console.warn(message);
			return;
		}

		console.log(message);
	};

	console.log(`${logPrefix} loading renderer ${logTarget}`);

	window.on("show", () => {
		console.log(`${logPrefix} shown`);
	});

	window.on("unresponsive", () => {
		console.error(`${logPrefix} renderer became unresponsive`);
	});

	window.on("responsive", () => {
		console.log(`${logPrefix} renderer recovered responsiveness`);
	});

	window.webContents.on("dom-ready", () => {
		console.log(`${logPrefix} DOM ready at ${window.webContents.getURL()}`);
	});

	window.webContents.on("did-finish-load", () => {
		console.log(`${logPrefix} finished load at ${window.webContents.getURL()}`);
	});

	window.webContents.on(
		"did-fail-load",
		(_event, errorCode, errorDescription, validatedUrl, isMainFrame) => {
			if (!isMainFrame) {
				return;
			}

			console.error(
				`${logPrefix} failed load for ${validatedUrl} (${errorCode}): ${errorDescription}`,
			);
		},
	);

	window.webContents.on("render-process-gone", (_event, details) => {
		console.error(
			`${logPrefix} renderer process exited: ${details.reason} (exitCode=${details.exitCode})`,
		);
	});

	window.webContents.on("console-message", (details) => {
		const { level, lineNumber, message, sourceId } = details;
		const levelValue = level === "error" ? 2 : level === "warning" ? 1 : 0;

		logConsoleMessage(
			levelValue,
			`[camlet:renderer] ${message} (${sourceId}:${lineNumber})`,
		);
	});
}

function maintainAlwaysOnTop(window: BrowserWindow) {
	const reassertAlwaysOnTop = () => {
		if (window.isDestroyed()) {
			return;
		}

		window.setAlwaysOnTop(true, "screen-saver", 1);
		window.moveTop();
	};

	window.on("show", reassertAlwaysOnTop);
	window.on("focus", reassertAlwaysOnTop);
	window.on("restore", reassertAlwaysOnTop);
	window.on("blur", () => {
		reassertAlwaysOnTop();
		setTimeout(reassertAlwaysOnTop, 0);
		setTimeout(reassertAlwaysOnTop, 150);
	});

	reassertAlwaysOnTop();
}

function isRetryableDevRendererLoadError(error: unknown): boolean {
	if (!(error instanceof Error)) {
		return false;
	}

	const code =
		typeof Reflect.get(error, "code") === "string"
			? Reflect.get(error, "code")
			: null;

	return code === "ERR_FAILED" || code === "ERR_CONNECTION_REFUSED";
}

async function loadDevRendererUrl(
	window: BrowserWindow,
	rendererUrl: string,
	retryDelayMs = 250,
	maxAttempts = 20,
) {
	let lastError: unknown;

	for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
		try {
			await window.loadURL(rendererUrl);
			return;
		} catch (error) {
			lastError = error;

			if (
				window.isDestroyed() ||
				attempt === maxAttempts ||
				!isRetryableDevRendererLoadError(error)
			) {
				throw error;
			}

			console.warn(
				`[camlet:window] renderer load attempt ${attempt} failed, retrying in ${retryDelayMs}ms`,
			);
			await delay(retryDelayMs);
		}
	}

	throw lastError;
}

export async function createMainWindow({
	appearance,
	preloadPath,
	rendererHtmlPath,
	rendererUrl,
	windowState,
}: MainWindowAssets): Promise<BrowserWindow> {
	const rendererPolicy: RendererAssetPolicy = {
		rendererHtmlPath,
		...(rendererUrl !== undefined ? { rendererUrl } : {}),
	};
	const window = new BrowserWindow({
		x: windowState.x,
		y: windowState.y,
		width: windowState.width,
		height: windowState.height,
		minWidth: minimumWindowWidth,
		minHeight: minimumWindowHeight,
		frame: false,
		transparent: true,
		hasShadow: false,
		alwaysOnTop: true,
		resizable: false,
		maximizable: false,
		fullscreenable: false,
		show: false,
		autoHideMenuBar: true,
		backgroundColor: "#00000000",
		title: "Camlet",
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

	window.setAspectRatio(1);
	window.setHasShadow(false);
	window.setVisibleOnAllWorkspaces(true, {
		visibleOnFullScreen: true,
	});
	window.on("system-context-menu", (event) => {
		event.preventDefault();
		showMainWindowContextMenu(window);
	});
	applyMainWindowShape(window, appearance);
	maintainAlwaysOnTop(window);
	hardenWindowNavigation(window, rendererPolicy);
	attachDevWindowDiagnostics(window, rendererPolicy);

	window.once("ready-to-show", () => {
		if (rendererPolicy.rendererUrl !== undefined) {
			console.log("[camlet:window] ready to show");
		}

		window.show();
	});

	if (rendererUrl !== undefined) {
		await loadDevRendererUrl(window, rendererUrl);
		return window;
	}

	await window.loadFile(rendererHtmlPath);
	return window;
}
