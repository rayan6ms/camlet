import type { IpcRendererEvent } from "electron";
import { describe, expect, it, vi } from "vitest";
import type { AboutInfo } from "../../src/shared/about.js";
import type {
	AppBootstrap,
	CamletApi,
	CamletContextMenuAction,
	CamletContextMenuRequest,
} from "../../src/shared/ipc.js";
import { ipcChannels } from "../../src/shared/ipc.js";
import type { AppLanguage } from "../../src/shared/language.js";
import type {
	CamletSettings,
	OverlayAppearanceSettingsPatch,
} from "../../src/shared/settings.js";
import type {
	DisplayWorkArea,
	ScreenPoint,
	WindowState,
} from "../../src/shared/window-state.js";

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

function createCamletApi(ipcRenderer: IpcRendererLike): CamletApi {
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
		getCurrentDisplayWorkArea: () =>
			invoke<DisplayWorkArea>(ipcChannels.getCurrentDisplayWorkArea),
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

describe("preload API contract", () => {
	it("exposes a frozen API that forwards IPC calls to the expected channels", async () => {
		const invoke = vi.fn(async () => undefined);
		const on = vi.fn((_channel, _listener) => mockIpcRenderer);
		const removeListener = vi.fn((_channel, _listener) => mockIpcRenderer);
		const mockIpcRenderer = {
			invoke,
			on,
			removeListener,
		};
		const api = createCamletApi(mockIpcRenderer);
		const contextMenuPayload = {
			labels: {
				theme: "Theme",
				shape: "Shape",
				cornerRoundness: "Corner roundness",
				language: "Language",
				cameraInput: "Camera Input",
				resize: "Resize",
				advancedSettings: "Advanced Settings",
				aboutCamlet: "About Camlet",
				closeApp: "Close Camlet",
				retryCamera: "Retry Camera",
				resetAppearance: "Reset Appearance",
				fitMode: "Fit Mode",
				ringThickness: "Ring Thickness",
				systemInfo: "System",
				status: "Status",
				activeDevice: "Active Device",
				displayProtocol: "Display Protocol",
				noDevices: "No devices",
				themeOptions: {
					mint: "Mint",
					ocean: "Ocean",
					ember: "Ember",
					orchid: "Orchid",
					grove: "Grove",
					graphite: "Graphite",
				},
				shapeOptions: {
					circle: "Circle",
					roundedSquare: "Square",
					diamond: "Diamond",
					rectangleY: "Rectangle Y",
					rectangleX: "Rectangle X",
				},
				fitModeOptions: {
					cover: "Cover",
					contain: "Contain",
				},
				languageOptions: {
					system: "System default",
					en: "English",
					"pt-BR": "Português (Brasil)",
				},
			},
			selectedThemeId: "mint" as const,
			selectedShape: "circle" as const,
			selectedLanguage: "en" as const,
			selectedFitMode: "cover" as const,
			selectedRingThickness: 8,
			selectedCornerRoundness: 24,
			cameraOptions: [],
			selectedCameraDeviceId: null,
			cameraStatusLabel: "Camera preview active",
			activeCameraLabel: "None",
			displayProtocolLabel: "Wayland",
		};

		expect(Object.isFrozen(api)).toBe(true);

		await api.getBootstrap();
		await api.getSettings();
		await api.getAboutInfo();
		await api.openAboutWindow();
		await api.setLanguage("pt-BR");
		await api.setSelectedCameraDeviceId("camera-1");
		await api.updateOverlayAppearanceSettings({
			ringThickness: 8,
		});
		await api.startWindowDrag({
			screenX: 50,
			screenY: 60,
		});
		await api.updateWindowDrag({
			screenX: 70,
			screenY: 90,
		});
		await api.endWindowDrag();
		await api.setWindowResizable(true);
		await api.setWindowState({
			x: 24,
			y: 30,
			width: 280,
			height: 280,
		});
		await api.getCurrentDisplayWorkArea();
		await api.updateContextMenuState(contextMenuPayload);
		await api.showContextMenu(contextMenuPayload);

		expect(invoke.mock.calls).toEqual([
			[ipcChannels.getBootstrap],
			[ipcChannels.getSettings],
			[ipcChannels.getAboutInfo],
			[ipcChannels.openAboutWindow],
			[ipcChannels.setLanguage, "pt-BR"],
			[ipcChannels.setSelectedCameraDeviceId, "camera-1"],
			[ipcChannels.updateOverlayAppearanceSettings, { ringThickness: 8 }],
			[ipcChannels.startWindowDrag, { screenX: 50, screenY: 60 }],
			[ipcChannels.updateWindowDrag, { screenX: 70, screenY: 90 }],
			[ipcChannels.endWindowDrag],
			[ipcChannels.setWindowResizable, true],
			[
				ipcChannels.setWindowState,
				{
					x: 24,
					y: 30,
					width: 280,
					height: 280,
				},
			],
			[ipcChannels.getCurrentDisplayWorkArea],
			[ipcChannels.updateContextMenuState, contextMenuPayload],
			[ipcChannels.showContextMenu, contextMenuPayload],
		]);
	});

	it("subscribes and unsubscribes context menu request events through IPC", () => {
		const listeners = new Map<string, (...args: unknown[]) => void>();
		const mockIpcRenderer = {
			invoke: vi.fn(async () => undefined),
			on: vi.fn((channel: string, listener: (...args: unknown[]) => void) => {
				listeners.set(channel, listener);
				return mockIpcRenderer;
			}),
			removeListener: vi.fn(
				(channel: string, listener: (...args: unknown[]) => void) => {
					if (listeners.get(channel) === listener) {
						listeners.delete(channel);
					}

					return mockIpcRenderer;
				},
			),
		};
		const api = createCamletApi(mockIpcRenderer);
		const listener = vi.fn();

		const unsubscribe = api.onContextMenuRequested(listener);
		listeners.get(ipcChannels.contextMenuRequested)?.({});
		unsubscribe();

		expect(listener).toHaveBeenCalledTimes(1);
		expect(mockIpcRenderer.on).toHaveBeenCalledTimes(1);
		expect(mockIpcRenderer.removeListener).toHaveBeenCalledTimes(1);
	});

	it("subscribes and unsubscribes window state listeners through IPC", () => {
		const listeners = new Map<string, (...args: unknown[]) => void>();
		const mockIpcRenderer = {
			invoke: vi.fn(async () => undefined),
			on: vi.fn((channel: string, listener: (...args: unknown[]) => void) => {
				listeners.set(channel, listener);
				return mockIpcRenderer;
			}),
			removeListener: vi.fn(
				(channel: string, listener: (...args: unknown[]) => void) => {
					if (listeners.get(channel) === listener) {
						listeners.delete(channel);
					}

					return mockIpcRenderer;
				},
			),
		};
		const api = createCamletApi(mockIpcRenderer);
		const listener = vi.fn();

		const unsubscribe = api.onWindowStateChange(listener);
		listeners.get(ipcChannels.windowStateChanged)?.(
			{},
			{
				x: 12,
				y: 18,
				width: 360,
				height: 360,
			},
		);
		unsubscribe();

		expect(listener).toHaveBeenCalledWith({
			x: 12,
			y: 18,
			width: 360,
			height: 360,
		});
		expect(mockIpcRenderer.on).toHaveBeenCalledTimes(1);
		expect(mockIpcRenderer.removeListener).toHaveBeenCalledTimes(1);
	});

	it("subscribes and unsubscribes context menu actions through IPC", () => {
		const listeners = new Map<string, (...args: unknown[]) => void>();
		const mockIpcRenderer = {
			invoke: vi.fn(async () => undefined),
			on: vi.fn((channel: string, listener: (...args: unknown[]) => void) => {
				listeners.set(channel, listener);
				return mockIpcRenderer;
			}),
			removeListener: vi.fn(
				(channel: string, listener: (...args: unknown[]) => void) => {
					if (listeners.get(channel) === listener) {
						listeners.delete(channel);
					}

					return mockIpcRenderer;
				},
			),
		};
		const api = createCamletApi(mockIpcRenderer);
		const listener = vi.fn();

		const unsubscribe = api.onContextMenuAction(listener);
		listeners.get(ipcChannels.contextMenuAction)?.(
			{},
			{
				type: "enter-resize-mode",
			},
		);
		unsubscribe();

		expect(listener).toHaveBeenCalledWith({
			type: "enter-resize-mode",
		});
		expect(mockIpcRenderer.on).toHaveBeenCalledTimes(1);
		expect(mockIpcRenderer.removeListener).toHaveBeenCalledTimes(1);
	});
});
