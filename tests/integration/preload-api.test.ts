import { describe, expect, it, vi } from "vitest";
import { createCamletApi } from "../../src/preload/api.js";
import { ipcChannels } from "../../src/shared/ipc.js";

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
					rectangle: "Rectangle",
				},
				fitModeOptions: {
					cover: "Cover",
					contain: "Contain",
				},
				languageOptions: {
					system: "System default",
					en: "English",
					"pt-BR": "Português (Brasil)",
					es: "Español",
					fr: "Français",
					de: "Deutsch",
					it: "Italiano",
					ja: "日本語",
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
		await api.setLanguage("ja");
		await api.setSelectedCameraDeviceId("camera-1");
		await api.updateOverlayAppearanceSettings({
			ringThickness: 12,
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
		await api.updateContextMenuState(contextMenuPayload);
		await api.showContextMenu(contextMenuPayload);

		expect(invoke.mock.calls).toEqual([
			[ipcChannels.getBootstrap],
			[ipcChannels.getSettings],
			[ipcChannels.getAboutInfo],
			[ipcChannels.openAboutWindow],
			[ipcChannels.setLanguage, "ja"],
			[ipcChannels.setSelectedCameraDeviceId, "camera-1"],
			[ipcChannels.updateOverlayAppearanceSettings, { ringThickness: 12 }],
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
