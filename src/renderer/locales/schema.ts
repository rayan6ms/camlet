import type {
	AppBootstrapIssue,
	AppDisplayProtocol,
	AppReleaseChannel,
	AppRuntimeMode,
} from "../../shared/bootstrap.js";
import type { AppLanguage } from "../../shared/language.js";
import type { CameraPreviewStatus } from "../features/overlay-shell/camera.js";

export interface RendererLocale {
	app: {
		title: string;
		overlayReady: string;
		close: string;
	};
	overlay: {
		dragHint: string;
		settingsHint: string;
		summary: string;
		preview: string;
		hintOpenSettings: string;
		resizeHint: string;
		resizeDone: string;
		resizeAction: string;
		resizeHandle: string;
	};
	advanced: {
		title: string;
		description: string;
	};
	sections: {
		settings: string;
		general: string;
		appearance: string;
		camera: string;
		system: string;
		about: string;
	};
	settings: {
		actions: {
			open: string;
			close: string;
			resetAppearance: string;
		};
		hints: {
			panel: string;
			escape: string;
		};
	};
	about: {
		description: string;
		labels: {
			appName: string;
			version: string;
			channel: string;
			mode: string;
			packaged: string;
			platform: string;
			displayProtocol: string;
			electron: string;
			chrome: string;
		};
		channels: Record<AppReleaseChannel, string>;
		modes: Record<AppRuntimeMode, string>;
		displayProtocols: Record<AppDisplayProtocol, string>;
		packagedValues: {
			yes: string;
			no: string;
		};
		diagnostics: {
			title: string;
			hint: string;
			copy: string;
			copied: string;
			copyFailed: string;
		};
	};
	language: {
		label: string;
		description: string;
		options: Record<AppLanguage, string>;
	};
	summary: {
		activeDevice: string;
		overlaySize: string;
		effectiveLanguage: string;
		windowPosition: string;
		windowSize: string;
		platform: string;
	};
	camera: {
		description: string;
		actions: {
			retry: string;
		};
		labels: {
			device: string;
			activeDevice: string;
			deviceCount: string;
			permission: string;
			noDevices: string;
			none: string;
		};
		status: Record<CameraPreviewStatus, string>;
		message: Record<
			CameraPreviewStatus | "savedUnavailableUsingFallback",
			string
		>;
	};
	appearance: {
		description: string;
		labels: {
			theme: string;
			shape: string;
			fitMode: string;
			ringColor: string;
			ringThickness: string;
			overlaySize: string;
		};
		themes: {
			mint: string;
			coral: string;
			sky: string;
			graphite: string;
		};
		shapes: {
			circle: string;
			roundedSquare: string;
		};
		fitModes: {
			cover: string;
			contain: string;
		};
	};
	startup: {
		debugSummary: string;
		actions: {
			reload: string;
		};
		errors: Record<
			"preload-unavailable" | "bootstrap-invalid" | "bootstrap-load-failed",
			{
				title: string;
				message: string;
			}
		>;
		issues: {
			title: string;
			messages: Record<AppBootstrapIssue, string>;
		};
	};
}
