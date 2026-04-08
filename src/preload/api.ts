import type { IpcRendererEvent } from "electron";
import type { AboutInfo } from "../shared/about.js";
import type {
	AppBootstrap,
	CamletApi,
	CamletContextMenuAction,
	CamletContextMenuRequest,
} from "../shared/ipc.js";
import { ipcChannels } from "../shared/ipc.js";
import type { AppLanguage } from "../shared/language.js";
import type {
	CamletSettings,
	OverlayAppearanceSettingsPatch,
} from "../shared/settings.js";
import type { ScreenPoint, WindowState } from "../shared/window-state.js";

interface IpcRendererLike {
	invoke(channel: string, ...args: unknown[]): Promise<unknown>;
	on(
		channel: string,
		listener: (event: IpcRendererEvent, ...args: unknown[]) => void,
	): void;
	removeListener(
		channel: string,
		listener: (event: IpcRendererEvent, ...args: unknown[]) => void,
	): void;
}

export function createCamletApi(ipcRenderer: IpcRendererLike): CamletApi {
	const invoke = <T>(channel: string, ...args: unknown[]) =>
		ipcRenderer.invoke(channel, ...args) as Promise<T>;

	return Object.freeze({
		getBootstrap: () => invoke<AppBootstrap>(ipcChannels.getBootstrap),
		getSettings: () => invoke<CamletSettings>(ipcChannels.getSettings),
		setLanguage: (language: AppLanguage) =>
			invoke<CamletSettings>(ipcChannels.setLanguage, language),
		setSelectedCameraDeviceId: (deviceId: string | null) =>
			invoke<CamletSettings>(ipcChannels.setSelectedCameraDeviceId, deviceId),
		updateOverlayAppearanceSettings: (patch: OverlayAppearanceSettingsPatch) =>
			invoke<CamletSettings>(
				ipcChannels.updateOverlayAppearanceSettings,
				patch,
			),
		startWindowDrag: (pointer: ScreenPoint) =>
			invoke<void>(ipcChannels.startWindowDrag, pointer),
		updateWindowDrag: (pointer: ScreenPoint) =>
			invoke<void>(ipcChannels.updateWindowDrag, pointer),
		endWindowDrag: () => invoke<void>(ipcChannels.endWindowDrag),
		setWindowResizable: (resizable: boolean) =>
			invoke<void>(ipcChannels.setWindowResizable, resizable),
		setWindowState: (windowState: WindowState) =>
			invoke<WindowState>(ipcChannels.setWindowState, windowState),
		getAboutInfo: () => invoke<AboutInfo>(ipcChannels.getAboutInfo),
		openAboutWindow: () => invoke<void>(ipcChannels.openAboutWindow),
		updateContextMenuState: (request: CamletContextMenuRequest) =>
			invoke<void>(ipcChannels.updateContextMenuState, request),
		showContextMenu: (request: CamletContextMenuRequest) =>
			invoke<void>(ipcChannels.showContextMenu, request),
		onContextMenuRequested: (listener: () => void) => {
			const subscription = () => {
				listener();
			};

			ipcRenderer.on(ipcChannels.contextMenuRequested, subscription);

			return () => {
				ipcRenderer.removeListener(
					ipcChannels.contextMenuRequested,
					subscription,
				);
			};
		},
		onContextMenuAction: (
			listener: (action: CamletContextMenuAction) => void,
		) => {
			const subscription = (_event: IpcRendererEvent, ...args: unknown[]) => {
				const [action] = args;
				listener(action as CamletContextMenuAction);
			};

			ipcRenderer.on(ipcChannels.contextMenuAction, subscription);

			return () => {
				ipcRenderer.removeListener(ipcChannels.contextMenuAction, subscription);
			};
		},
		onWindowStateChange: (listener: (windowState: WindowState) => void) => {
			const subscription = (_event: IpcRendererEvent, ...args: unknown[]) => {
				const [windowState] = args;
				listener(windowState as WindowState);
			};

			ipcRenderer.on(ipcChannels.windowStateChanged, subscription);

			return () => {
				ipcRenderer.removeListener(
					ipcChannels.windowStateChanged,
					subscription,
				);
			};
		},
	});
}
