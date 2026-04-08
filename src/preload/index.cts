import type { AboutInfo } from "../shared/about.js";
import type {
	CamletApi,
	CamletContextMenuAction,
	CamletContextMenuRequest,
} from "../shared/ipc.js";
import type { AppLanguage } from "../shared/language.js";
import type { OverlayAppearanceSettingsPatch } from "../shared/settings.js";
import type { ScreenPoint, WindowState } from "../shared/window-state.js";

const { contextBridge, ipcRenderer } =
	require("electron") as typeof import("electron");

const channels = {
	getBootstrap: "app:get-bootstrap",
	getSettings: "settings:get",
	setLanguage: "settings:set-language",
	setSelectedCameraDeviceId: "settings:set-selected-camera-device-id",
	updateOverlayAppearanceSettings:
		"settings:update-overlay-appearance-settings",
	startWindowDrag: "window:start-drag",
	updateWindowDrag: "window:update-drag",
	endWindowDrag: "window:end-drag",
	setWindowResizable: "window:set-resizable",
	setWindowState: "window:set-state",
	getAboutInfo: "app:get-about-info",
	openAboutWindow: "app:open-about-window",
	updateContextMenuState: "app:update-context-menu-state",
	showContextMenu: "app:show-context-menu",
	contextMenuRequested: "app:context-menu-requested",
	contextMenuAction: "app:context-menu-action",
	windowStateChanged: "window:state-changed",
} as const;

const camletApi: CamletApi = Object.freeze({
	getBootstrap: () => ipcRenderer.invoke(channels.getBootstrap),
	getSettings: () => ipcRenderer.invoke(channels.getSettings),
	setLanguage: (language: AppLanguage) =>
		ipcRenderer.invoke(channels.setLanguage, language),
	setSelectedCameraDeviceId: (deviceId: string | null) =>
		ipcRenderer.invoke(channels.setSelectedCameraDeviceId, deviceId),
	updateOverlayAppearanceSettings: (patch: OverlayAppearanceSettingsPatch) =>
		ipcRenderer.invoke(channels.updateOverlayAppearanceSettings, patch),
	startWindowDrag: (pointer: ScreenPoint) =>
		ipcRenderer.invoke(channels.startWindowDrag, pointer),
	updateWindowDrag: (pointer: ScreenPoint) =>
		ipcRenderer.invoke(channels.updateWindowDrag, pointer),
	endWindowDrag: () => ipcRenderer.invoke(channels.endWindowDrag),
	setWindowResizable: (resizable: boolean) =>
		ipcRenderer.invoke(channels.setWindowResizable, resizable),
	setWindowState: (windowState: WindowState) =>
		ipcRenderer.invoke(channels.setWindowState, windowState),
	getAboutInfo: () =>
		ipcRenderer.invoke(channels.getAboutInfo) as Promise<AboutInfo>,
	openAboutWindow: () => ipcRenderer.invoke(channels.openAboutWindow),
	updateContextMenuState: (request: CamletContextMenuRequest) =>
		ipcRenderer.invoke(channels.updateContextMenuState, request),
	showContextMenu: (request: CamletContextMenuRequest) =>
		ipcRenderer.invoke(channels.showContextMenu, request),
	onContextMenuRequested: (listener: () => void) => {
		const subscription = () => {
			listener();
		};

		ipcRenderer.on(channels.contextMenuRequested, subscription);

		return () => {
			ipcRenderer.removeListener(channels.contextMenuRequested, subscription);
		};
	},
	onContextMenuAction: (
		listener: (action: CamletContextMenuAction) => void,
	) => {
		const subscription = (
			_event: Electron.IpcRendererEvent,
			action: CamletContextMenuAction,
		) => {
			listener(action);
		};

		ipcRenderer.on(channels.contextMenuAction, subscription);

		return () => {
			ipcRenderer.removeListener(channels.contextMenuAction, subscription);
		};
	},
	onWindowStateChange: (listener: (windowState: WindowState) => void) => {
		const subscription = (
			_event: Electron.IpcRendererEvent,
			windowState: WindowState,
		) => {
			listener(windowState);
		};

		ipcRenderer.on(channels.windowStateChanged, subscription);

		return () => {
			ipcRenderer.removeListener(channels.windowStateChanged, subscription);
		};
	},
});

contextBridge.exposeInMainWorld("camlet", camletApi);
