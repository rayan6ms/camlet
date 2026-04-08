import type { RendererLocale } from "./schema.js";

export const enLocale: RendererLocale = {
	app: {
		title: "Camlet",
		close: "Close Camlet",
	},
	overlay: {
		preview: "Webcam overlay preview",
		hintOpenSettings: "Right click to open settings",
		resizeDone: "Done",
		resizeAction: "Resize",
	},
	advanced: {
		title: "Advanced settings",
	},
	sections: {
		system: "System",
		about: "About",
	},
	settings: {
		actions: {
			resetAppearance: "Reset appearance defaults",
		},
	},
	about: {
		windowTitle: "About Camlet",
		licenseLabel: "License",
		labels: {
			version: "Version",
			channel: "Channel",
			packaged: "Packaged",
			platform: "Platform",
			displayProtocol: "Protocol",
			electron: "Electron",
			chrome: "Chrome",
		},
		channels: {
			stable: "Stable",
			prerelease: "Beta / prerelease",
		},
		displayProtocols: {
			wayland: "Wayland",
			x11: "X11",
			windows: "Windows desktop",
			macos: "macOS desktop",
			unknown: "Unknown",
		},
		packagedValues: {
			yes: "Yes",
			no: "No",
		},
	},
	language: {
		label: "Language",
		options: {
			system: "System default",
			en: "English",
			"pt-BR": "Português (Brasil)",
		},
	},
	camera: {
		actions: {
			retry: "Retry camera",
		},
		labels: {
			device: "Camera device",
			activeDevice: "Active device",
			permission: "Preview state",
			noDevices: "No camera devices",
			none: "None",
		},
		status: {
			loading: "Loading camera",
			preview: "Camera preview active",
			"permission-denied": "Permission denied",
			"camera-in-use": "Camera busy",
			"no-camera": "No camera found",
			"selected-device-unavailable": "Selected device unavailable",
			error: "Camera error",
		},
	},
	appearance: {
		labels: {
			theme: "Theme",
			shape: "Shape",
			cornerRoundness: "Corner roundness",
			fitMode: "Fit mode",
			ringThickness: "Ring thickness",
		},
		themes: {
			mint: "Mint",
			ocean: "Ocean",
			ember: "Ember",
			orchid: "Orchid",
			grove: "Grove",
			graphite: "Graphite",
		},
		shapes: {
			circle: "Circle",
			roundedSquare: "Square",
			diamond: "Diamond",
			rectangleY: "Rectangle Y",
			rectangleX: "Rectangle X",
		},
		fitModes: {
			cover: "Cover",
			contain: "Contain",
		},
	},
	startup: {
		actions: {
			reload: "Reload Camlet",
		},
		errors: {
			"preload-unavailable": {
				title: "Camlet could not connect to its desktop bridge",
				message:
					"The preload API is missing, so the overlay cannot finish starting. Restart the app and confirm the packaged files are intact.",
			},
			"bootstrap-invalid": {
				title: "Camlet received invalid startup data",
				message:
					"The renderer blocked startup because the bootstrap payload was incomplete or malformed. Restart the app and verify the current build output.",
			},
			"bootstrap-load-failed": {
				title: "Camlet could not load startup settings",
				message:
					"The renderer could not load startup configuration from the desktop process. Restart the app. If it keeps failing, test with a clean settings profile.",
			},
		},
	},
};
