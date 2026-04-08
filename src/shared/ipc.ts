import type { AboutInfo } from "./about.js";
import type { OverlayShape, PreviewFitMode } from "./appearance.js";
import type {
	AppBootstrap,
	AppInfo,
	AppReleaseChannel,
	AppRuntimeMode,
} from "./bootstrap.js";
import type { AppLanguage } from "./language.js";
import type {
	CamletSettings,
	OverlayAppearanceSettingsPatch,
} from "./settings.js";
import type {
	DisplayWorkArea,
	ScreenPoint,
	WindowState,
} from "./window-state.js";

export interface CameraMenuOption {
	deviceId: string;
	label: string;
}

export interface CamletContextMenuLabels {
	theme: string;
	shape: string;
	cornerRoundness: string;
	language: string;
	cameraInput: string;
	resize: string;
	advancedSettings: string;
	aboutCamlet: string;
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
		ocean: string;
		ember: string;
		orchid: string;
		grove: string;
		graphite: string;
	};
	shapeOptions: {
		circle: string;
		roundedSquare: string;
		diamond: string;
		rectangleY: string;
		rectangleX: string;
	};
	fitModeOptions: Record<PreviewFitMode, string>;
	languageOptions: Record<AppLanguage, string>;
}

export interface CamletContextMenuRequest {
	labels: CamletContextMenuLabels;
	selectedThemeId:
		| "mint"
		| "ocean"
		| "ember"
		| "orchid"
		| "grove"
		| "graphite"
		| null;
	selectedShape: OverlayShape;
	selectedLanguage: AppLanguage;
	selectedFitMode: PreviewFitMode;
	selectedRingThickness: number;
	selectedCornerRoundness: number;
	cameraOptions: CameraMenuOption[];
	selectedCameraDeviceId: string | null;
	cameraStatusLabel: string;
	activeCameraLabel: string;
	displayProtocolLabel: string;
}

export type CamletContextMenuAction =
	| {
			type: "set-theme";
			themeId: "mint" | "ocean" | "ember" | "orchid" | "grove" | "graphite";
	  }
	| {
			type: "set-shape";
			shape: OverlayShape;
	  }
	| {
			type: "set-corner-roundness";
			cornerRoundness: number;
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
			type: "open-about-window";
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
	getCurrentDisplayWorkArea: "window:get-current-display-work-area",
	getAboutInfo: "app:get-about-info",
	openAboutWindow: "app:open-about-window",
	updateContextMenuState: "app:update-context-menu-state",
	showContextMenu: "app:show-context-menu",
	contextMenuRequested: "app:context-menu-requested",
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
	getCurrentDisplayWorkArea(): Promise<DisplayWorkArea>;
	getAboutInfo(): Promise<AboutInfo>;
	openAboutWindow(): Promise<void>;
	updateContextMenuState(request: CamletContextMenuRequest): Promise<void>;
	showContextMenu(request: CamletContextMenuRequest): Promise<void>;
	onContextMenuRequested(listener: () => void): () => void;
	onContextMenuAction(
		listener: (action: CamletContextMenuAction) => void,
	): () => void;
	onWindowStateChange(listener: (windowState: WindowState) => void): () => void;
}

export type {
	AppBootstrap,
	AppInfo,
	AppReleaseChannel,
	AppRuntimeMode,
};
