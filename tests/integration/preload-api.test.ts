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

		expect(Object.isFrozen(api)).toBe(true);

		await api.getBootstrap();
		await api.getSettings();
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
		await api.showContextMenu({
			labels: {
				theme: "Theme",
				shape: "Shape",
				language: "Language",
				cameraInput: "Camera Input",
				resize: "Resize",
				advancedSettings: "Advanced Settings",
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
					coral: "Coral",
					sky: "Sky",
					graphite: "Graphite",
				},
				shapeOptions: {
					circle: "Circle",
					roundedSquare: "Rounded square",
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
			selectedThemeId: "mint",
			selectedShape: "circle",
			selectedLanguage: "en",
			selectedFitMode: "cover",
			selectedRingThickness: 8,
			cameraOptions: [],
			selectedCameraDeviceId: null,
			cameraStatusLabel: "Camera preview active",
			activeCameraLabel: "None",
			displayProtocolLabel: "Wayland",
		});

		expect(invoke.mock.calls).toEqual([
			[ipcChannels.getBootstrap],
			[ipcChannels.getSettings],
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
			[
				ipcChannels.showContextMenu,
				{
					labels: {
						theme: "Theme",
						shape: "Shape",
						language: "Language",
						cameraInput: "Camera Input",
						resize: "Resize",
						advancedSettings: "Advanced Settings",
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
							coral: "Coral",
							sky: "Sky",
							graphite: "Graphite",
						},
						shapeOptions: {
							circle: "Circle",
							roundedSquare: "Rounded square",
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
					selectedThemeId: "mint",
					selectedShape: "circle",
					selectedLanguage: "en",
					selectedFitMode: "cover",
					selectedRingThickness: 8,
					cameraOptions: [],
					selectedCameraDeviceId: null,
					cameraStatusLabel: "Camera preview active",
					activeCameraLabel: "None",
					displayProtocolLabel: "Wayland",
				},
			],
		]);
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
