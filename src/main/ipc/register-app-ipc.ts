import {
	app,
	BrowserWindow,
	ipcMain,
	Menu,
	type MenuItemConstructorOptions,
} from "electron/main";
import {
	type CamletContextMenuAction,
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
import { createAppBootstrap, resolveDisplayProtocol } from "../bootstrap.js";
import type { SettingsStoreService } from "../services/settings-store.js";
import { applyMainWindowShape } from "../windows/window-shape.js";
import {
	createWindowDragOffset,
	moveMainWindowWithPointer,
	persistWindowStateNow,
	setMainWindowState,
} from "../windows/window-state.js";

const ringThicknessOptions = [4, 8, 12, 16];

function sendContextMenuAction(
	window: BrowserWindow,
	action: CamletContextMenuAction,
) {
	if (window.isDestroyed() || window.webContents.isDestroyed()) {
		return;
	}

	window.webContents.send(ipcChannels.contextMenuAction, action);
}

function createContextMenuTemplate(
	window: BrowserWindow,
	request: CamletContextMenuRequest,
): MenuItemConstructorOptions[] {
	const cameraSubmenu =
		request.cameraOptions.length > 0
			? request.cameraOptions.map<MenuItemConstructorOptions>((device) => ({
					label: device.label,
					type: "radio",
					checked: device.deviceId === request.selectedCameraDeviceId,
					click: () => {
						sendContextMenuAction(window, {
							type: "set-camera",
							deviceId: device.deviceId,
						});
					},
				}))
			: [
					{
						label: request.labels.noDevices,
						enabled: false,
					},
				];

	return [
		{
			label: request.labels.theme,
			submenu: [
				{
					label: request.labels.themeOptions.mint,
					type: "radio",
					checked: request.selectedThemeId === "mint",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-theme",
							themeId: "mint",
						});
					},
				},
				{
					label: request.labels.themeOptions.coral,
					type: "radio",
					checked: request.selectedThemeId === "coral",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-theme",
							themeId: "coral",
						});
					},
				},
				{
					label: request.labels.themeOptions.sky,
					type: "radio",
					checked: request.selectedThemeId === "sky",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-theme",
							themeId: "sky",
						});
					},
				},
				{
					label: request.labels.themeOptions.graphite,
					type: "radio",
					checked: request.selectedThemeId === "graphite",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-theme",
							themeId: "graphite",
						});
					},
				},
			],
		},
		{
			label: request.labels.shape,
			submenu: [
				{
					label: request.labels.shapeOptions.circle,
					type: "radio",
					checked: request.selectedShape === "circle",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-shape",
							shape: "circle",
						});
					},
				},
				{
					label: request.labels.shapeOptions.roundedSquare,
					type: "radio",
					checked: request.selectedShape === "rounded-square",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-shape",
							shape: "rounded-square",
						});
					},
				},
			],
		},
		{
			label: request.labels.language,
			submenu: Object.entries(request.labels.languageOptions).map(
				([language, label]) => ({
					label,
					type: "radio",
					checked: language === request.selectedLanguage,
					click: () => {
						sendContextMenuAction(window, {
							type: "set-language",
							language:
								language as CamletContextMenuRequest["selectedLanguage"],
						});
					},
				}),
			),
		},
		{
			label: request.labels.cameraInput,
			submenu: cameraSubmenu,
		},
		{
			type: "separator",
		},
		{
			label: request.labels.resize,
			click: () => {
				sendContextMenuAction(window, {
					type: "enter-resize-mode",
				});
			},
		},
		{
			label: request.labels.advancedSettings,
			submenu: [
				{
					label: `${request.labels.status}: ${request.cameraStatusLabel}`,
					enabled: false,
				},
				{
					label: `${request.labels.activeDevice}: ${request.activeCameraLabel}`,
					enabled: false,
				},
				{
					label: `${request.labels.displayProtocol}: ${request.displayProtocolLabel}`,
					enabled: false,
				},
				{
					type: "separator",
				},
				{
					label: request.labels.fitMode,
					submenu: [
						{
							label: request.labels.fitModeOptions.cover,
							type: "radio",
							checked: request.selectedFitMode === "cover",
							click: () => {
								sendContextMenuAction(window, {
									type: "set-fit-mode",
									fitMode: "cover",
								});
							},
						},
						{
							label: request.labels.fitModeOptions.contain,
							type: "radio",
							checked: request.selectedFitMode === "contain",
							click: () => {
								sendContextMenuAction(window, {
									type: "set-fit-mode",
									fitMode: "contain",
								});
							},
						},
					],
				},
				{
					label: request.labels.ringThickness,
					submenu: ringThicknessOptions.map<MenuItemConstructorOptions>(
						(ringThickness) => ({
							label: `${ringThickness}px`,
							type: "radio",
							checked: request.selectedRingThickness === ringThickness,
							click: () => {
								sendContextMenuAction(window, {
									type: "set-ring-thickness",
									ringThickness,
								});
							},
						}),
					),
				},
				{
					type: "separator",
				},
				{
					label: request.labels.retryCamera,
					click: () => {
						sendContextMenuAction(window, {
							type: "retry-camera",
						});
					},
				},
				{
					label: request.labels.resetAppearance,
					click: () => {
						sendContextMenuAction(window, {
							type: "reset-appearance",
						});
					},
				},
			],
		},
		{
			type: "separator",
		},
		{
			label: request.labels.closeApp,
			click: () => {
				app.quit();
			},
		},
	];
}

function resolveWindow(webContents: Electron.WebContents): BrowserWindow {
	const window = BrowserWindow.fromWebContents(webContents);

	if (window === null) {
		throw new Error(
			"Camlet IPC request did not originate from a BrowserWindow",
		);
	}

	return window;
}

export function registerAppIpc(settingsStore: SettingsStoreService) {
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
					sessionType: process.env.XDG_SESSION_TYPE,
					waylandDisplay: process.env.WAYLAND_DISPLAY,
					display: process.env.DISPLAY,
				}),
				electronVersion: process.versions.electron,
				chromeVersion: process.versions.chrome,
			},
			settings,
			systemLocale: app.getLocale(),
			issues: settingsStore.getBootstrapIssues(),
		});
	});

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

			applyMainWindowShape(window, nextSettings.overlayShape);
			return nextSettings;
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

	ipcMain.handle(
		ipcChannels.showContextMenu,
		(event, request: CamletContextMenuRequest) => {
			const window = resolveWindow(event.sender);
			const menu = Menu.buildFromTemplate(
				createContextMenuTemplate(window, request),
			);
			menu.popup({
				window,
			});
		},
	);
}
