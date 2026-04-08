import type { OverlayShape, PreviewFitMode } from "./appearance.js";
import type {
	AppBootstrap,
	AppBootstrapIssue,
	AppInfo,
	AppReleaseChannel,
	AppRuntimeMode,
} from "./bootstrap.js";
import type { AppLanguage } from "./language.js";
import type {
	CamletSettings,
	OverlayAppearanceSettingsPatch,
} from "./settings.js";
import type { ScreenPoint, WindowState } from "./window-state.js";

export interface CameraMenuOption {
	deviceId: string;
	label: string;
}

export interface CamletContextMenuLabels {
	theme: string;
	shape: string;
	language: string;
	cameraInput: string;
	resize: string;
	advancedSettings: string;
	closeApp: string;
	retryCamera: string;
	resetAppearance: string;
	fitMode: string;
	ringThickness: string;
	systemInfo: string;
	status: string;
	activeDevice: string;
	displayProtocol: string;
	noDevices: string;
	themeOptions: {
		mint: string;
		coral: string;
		sky: string;
		graphite: string;
	};
	shapeOptions: {
		circle: string;
		roundedSquare: string;
	};
	fitModeOptions: Record<PreviewFitMode, string>;
	languageOptions: Record<AppLanguage, string>;
}

export interface CamletContextMenuRequest {
	labels: CamletContextMenuLabels;
	selectedThemeId: "mint" | "coral" | "sky" | "graphite" | null;
	selectedShape: OverlayShape;
	selectedLanguage: AppLanguage;
	selectedFitMode: PreviewFitMode;
	selectedRingThickness: number;
	cameraOptions: CameraMenuOption[];
	selectedCameraDeviceId: string | null;
	cameraStatusLabel: string;
	activeCameraLabel: string;
	displayProtocolLabel: string;
}

export type CamletContextMenuAction =
	| {
			type: "set-theme";
			themeId: "mint" | "coral" | "sky" | "graphite";
	  }
	| {
			type: "set-shape";
			shape: OverlayShape;
	  }
	| {
			type: "set-language";
			language: AppLanguage;
	  }
	| {
			type: "set-camera";
			deviceId: string;
	  }
	| {
			type: "set-fit-mode";
			fitMode: PreviewFitMode;
	  }
	| {
			type: "set-ring-thickness";
			ringThickness: number;
	  }
	| {
			type: "retry-camera";
	  }
	| {
			type: "reset-appearance";
	  }
	| {
			type: "enter-resize-mode";
	  }
	| {
			type: "close-app";
	  };

export const ipcChannels = {
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
	showContextMenu: "app:show-context-menu",
	contextMenuAction: "app:context-menu-action",
	windowStateChanged: "window:state-changed",
} as const;

export interface CamletApi {
	getBootstrap(): Promise<AppBootstrap>;
	getSettings(): Promise<CamletSettings>;
	setLanguage(language: AppLanguage): Promise<CamletSettings>;
	setSelectedCameraDeviceId(deviceId: string | null): Promise<CamletSettings>;
	updateOverlayAppearanceSettings(
		patch: OverlayAppearanceSettingsPatch,
	): Promise<CamletSettings>;
	startWindowDrag(pointer: ScreenPoint): Promise<void>;
	updateWindowDrag(pointer: ScreenPoint): Promise<void>;
	endWindowDrag(): Promise<void>;
	setWindowResizable(resizable: boolean): Promise<void>;
	setWindowState(windowState: WindowState): Promise<WindowState>;
	showContextMenu(request: CamletContextMenuRequest): Promise<void>;
	onContextMenuAction(
		listener: (action: CamletContextMenuAction) => void,
	): () => void;
	onWindowStateChange(listener: (windowState: WindowState) => void): () => void;
}

export type {
	AppBootstrap,
	AppBootstrapIssue,
	AppInfo,
	AppReleaseChannel,
	AppRuntimeMode,
};
