import { app, BrowserWindow, ipcMain } from "electron/main";
import {
	type CamletContextMenuRequest,
	ipcChannels,
} from "../../shared/ipc.js";
import { appLanguageSchema } from "../../shared/language.js";
import { overlayAppearanceSettingsPatchSchema } from "../../shared/settings.js";
import {
	type DragOffset,
	screenPointSchema,
	windowStateSchema,
} from "../../shared/window-state.js";
import { getAboutInfo } from "../about/about-info.js";
import { createAppBootstrap, resolveDisplayProtocol } from "../bootstrap.js";
import type { SettingsStoreService } from "../services/settings-store.js";
import { applyMainWindowShape } from "../windows/window-shape.js";
import {
	createWindowDragOffset,
	getCurrentDisplayWorkArea,
	moveMainWindowWithPointer,
	persistWindowStateNow,
	setMainWindowState,
} from "../windows/window-state.js";
import {
	showMainWindowContextMenu,
	updateContextMenuRequest,
} from "./context-menu.js";

function resolveWindow(webContents: Electron.WebContents): BrowserWindow {
	const window = BrowserWindow.fromWebContents(webContents);

	if (window === null) {
		throw new Error(
			"Camlet IPC request did not originate from a BrowserWindow",
		);
	}

	return window;
}

export function registerAppIpc(
	settingsStore: SettingsStoreService,
	options: {
		openAboutWindow: () => Promise<void>;
	},
) {
	const activeDrags = new WeakMap<BrowserWindow, DragOffset>();

	ipcMain.handle(ipcChannels.getBootstrap, () => {
		const settings = settingsStore.getSettings();
		return createAppBootstrap({
			app: {
				name: app.getName(),
				version: app.getVersion(),
				platform: process.platform,
				arch: process.arch,
				mode: app.isPackaged ? "production" : "development",
				packaged: app.isPackaged,
				displayProtocol: resolveDisplayProtocol({
					platform: process.platform,
					ozonePlatform: app.commandLine.getSwitchValue("ozone-platform"),
					sessionType: process.env.XDG_SESSION_TYPE,
					waylandDisplay: process.env.WAYLAND_DISPLAY,
					display: process.env.DISPLAY,
				}),
				electronVersion: process.versions.electron,
				chromeVersion: process.versions.chrome,
			},
			settings,
			systemLocale: app.getLocale(),
		});
	});

	ipcMain.handle(ipcChannels.getAboutInfo, () => getAboutInfo());

	ipcMain.handle(ipcChannels.openAboutWindow, () => options.openAboutWindow());

	ipcMain.handle(ipcChannels.getSettings, () => settingsStore.getSettings());

	ipcMain.handle(ipcChannels.setLanguage, (_event, language: unknown) => {
		const nextLanguage = appLanguageSchema.parse(language);
		return settingsStore.setLanguage(nextLanguage);
	});

	ipcMain.handle(
		ipcChannels.setSelectedCameraDeviceId,
		(_event, deviceId: unknown) => {
			if (deviceId !== null && typeof deviceId !== "string") {
				throw new Error("Selected camera device id must be a string or null");
			}

			return settingsStore.setSelectedCameraDeviceId(deviceId);
		},
	);

	ipcMain.handle(
		ipcChannels.updateOverlayAppearanceSettings,
		(event, patch: unknown) => {
			const window = resolveWindow(event.sender);
			const nextSettings = settingsStore.updateOverlayAppearanceSettings(
				overlayAppearanceSettingsPatchSchema.parse(patch),
			);

			applyMainWindowShape(window, {
				overlayShape: nextSettings.overlayShape,
				cornerRoundness: nextSettings.cornerRoundness,
			});
			return nextSettings;
		},
	);

	ipcMain.handle(
		ipcChannels.updateContextMenuState,
		(event, request: CamletContextMenuRequest) => {
			const window = resolveWindow(event.sender);
			updateContextMenuRequest(window, request);
		},
	);

	ipcMain.handle(ipcChannels.startWindowDrag, (event, pointer: unknown) => {
		const window = resolveWindow(event.sender);
		const nextPointer = screenPointSchema.parse(pointer);
		activeDrags.set(window, createWindowDragOffset(window, nextPointer));
	});

	ipcMain.handle(ipcChannels.updateWindowDrag, (event, pointer: unknown) => {
		const window = resolveWindow(event.sender);
		const dragOffset = activeDrags.get(window);

		if (dragOffset === undefined) {
			return;
		}

		moveMainWindowWithPointer(
			window,
			screenPointSchema.parse(pointer),
			dragOffset,
		);
	});

	ipcMain.handle(ipcChannels.endWindowDrag, (event) => {
		const window = resolveWindow(event.sender);
		activeDrags.delete(window);
		persistWindowStateNow(window, settingsStore);
	});

	ipcMain.handle(
		ipcChannels.setWindowResizable,
		(event, resizable: unknown) => {
			if (typeof resizable !== "boolean") {
				throw new Error("Window resizable flag must be a boolean");
			}

			const window = resolveWindow(event.sender);
			window.setResizable(resizable);
		},
	);

	ipcMain.handle(ipcChannels.setWindowState, (event, windowState: unknown) => {
		const window = resolveWindow(event.sender);
		return setMainWindowState(window, windowStateSchema.parse(windowState));
	});

	ipcMain.handle(ipcChannels.getCurrentDisplayWorkArea, (event) => {
		const window = resolveWindow(event.sender);
		return getCurrentDisplayWorkArea(window);
	});

	ipcMain.handle(
		ipcChannels.showContextMenu,
		(event, request: CamletContextMenuRequest) => {
			const window = resolveWindow(event.sender);
			updateContextMenuRequest(window, request);
			showMainWindowContextMenu(window);
		},
	);
}
