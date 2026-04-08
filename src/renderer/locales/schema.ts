import type {
	AppDisplayProtocol,
	AppReleaseChannel,
} from "../../shared/bootstrap.js";
import type { AppLanguage } from "../../shared/language.js";
import type { CameraPreviewStatus } from "../features/overlay-shell/camera.js";

export interface RendererLocale {
	app: {
		title: string;
		close: string;
	};
	overlay: {
		preview: string;
		hintOpenSettings: string;
		resizeDone: string;
		resizeAction: string;
	};
	advanced: {
		title: string;
	};
	sections: {
		system: string;
		about: string;
	};
	settings: {
		actions: {
			resetAppearance: string;
		};
	};
	about: {
		windowTitle: string;
		licenseLabel: string;
		labels: {
			version: string;
			channel: string;
			packaged: string;
			platform: string;
			displayProtocol: string;
			electron: string;
			chrome: string;
		};
		channels: Record<AppReleaseChannel, string>;
		displayProtocols: Record<AppDisplayProtocol, string>;
		packagedValues: {
			yes: string;
			no: string;
		};
	};
	language: {
		label: string;
		options: Record<AppLanguage, string>;
	};
	camera: {
		actions: {
			retry: string;
		};
		labels: {
			device: string;
			activeDevice: string;
			permission: string;
			noDevices: string;
			none: string;
		};
		status: Record<CameraPreviewStatus, string>;
	};
	appearance: {
		labels: {
			theme: string;
			shape: string;
			cornerRoundness: string;
			fitMode: string;
			ringThickness: string;
		};
		themes: {
			mint: string;
			ocean: string;
			ember: string;
			orchid: string;
			grove: string;
			graphite: string;
		};
		shapes: {
			circle: string;
			roundedSquare: string;
			diamond: string;
			rectangleY: string;
			rectangleX: string;
		};
		fitModes: {
			cover: string;
			contain: string;
		};
	};
	startup: {
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
	};
}
