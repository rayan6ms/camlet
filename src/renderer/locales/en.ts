import type { RendererLocale } from "./schema.js";

export const enLocale: RendererLocale = {
	app: {
		title: "Camlet",
		close: "Close Camlet",
		overlayReady: "Configurable webcam overlay shell",
	},
	overlay: {
		dragHint: "Drag here to move the overlay",
		settingsHint:
			"Right-click the overlay or use the button to toggle settings",
		summary: "Overlay summary",
		preview: "Webcam overlay preview",
		hintOpenSettings: "Right click to open settings",
		resizeHint: "Drag a handle to resize. Click Done or the overlay to finish.",
		resizeDone: "Done",
		resizeAction: "Resize",
		resizeHandle: "Resize overlay",
	},
	advanced: {
		title: "Advanced settings",
		description:
			"Keep the main overlay clean and use this panel only for extra controls and diagnostics.",
	},
	sections: {
		settings: "Settings",
		general: "General",
		appearance: "Appearance",
		camera: "Camera",
		system: "System",
		about: "About",
	},
	settings: {
		actions: {
			open: "Open settings",
			close: "Close settings",
			resetAppearance: "Reset appearance defaults",
		},
		hints: {
			panel: "Compact overlay controls stay inside the shell and update live.",
			escape: "Press Escape to close settings.",
		},
	},
	about: {
		description:
			"Use this build info when validating beta candidates or packaged releases.",
		labels: {
			appName: "App",
			version: "Version",
			channel: "Channel",
			mode: "Runtime",
			packaged: "Packaged",
			platform: "Platform",
			displayProtocol: "Display protocol",
			electron: "Electron",
			chrome: "Chrome",
		},
		channels: {
			stable: "Stable",
			prerelease: "Beta / prerelease",
		},
		modes: {
			development: "Development",
			production: "Packaged build",
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
		diagnostics: {
			title: "Diagnostics summary",
			hint: "Copy this summary when reporting packaged-build behavior or platform-specific issues.",
			copy: "Copy diagnostics",
			copied: "Copied",
			copyFailed: "Copy unavailable",
		},
	},
	language: {
		label: "Language",
		description: "Choose how Camlet should display the interface.",
		options: {
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
	summary: {
		activeDevice: "Active device",
		overlaySize: "Overlay size",
		effectiveLanguage: "Current language",
		windowPosition: "Position",
		windowSize: "Window size",
		platform: "Platform",
	},
	camera: {
		description:
			"Select the preferred camera and review the live preview state without leaving the overlay.",
		actions: {
			retry: "Retry camera",
		},
		labels: {
			device: "Camera device",
			activeDevice: "Active device",
			deviceCount: "Detected cameras",
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
		message: {
			loading: "Camlet is requesting camera access and preparing the preview.",
			preview: "Live webcam preview is running inside the overlay shell.",
			"permission-denied":
				"Camera access was denied. Allow access and retry to continue.",
			"camera-in-use":
				"The camera is already in use or unreadable. Close other apps using it and retry.",
			"no-camera":
				"No video input devices were detected. Connect a camera and retry.",
			"selected-device-unavailable":
				"The saved camera is unavailable. Reconnect it or choose another device.",
			error:
				"Camlet could not start the camera preview. Retry or switch devices.",
			savedUnavailableUsingFallback:
				"The previously saved camera was unavailable, so Camlet switched to another available device.",
		},
	},
	appearance: {
		description:
			"Keep the overlay compact and readable while adjusting the live ring presentation.",
		labels: {
			theme: "Theme",
			shape: "Shape",
			fitMode: "Fit mode",
			ringColor: "Ring color",
			ringThickness: "Ring thickness",
			overlaySize: "Overlay size",
		},
		themes: {
			mint: "Mint",
			coral: "Coral",
			sky: "Sky",
			graphite: "Graphite",
		},
		shapes: {
			circle: "Circle",
			roundedSquare: "Rounded square",
		},
		fitModes: {
			cover: "Cover",
			contain: "Contain",
		},
	},
	startup: {
		debugSummary: "Startup details",
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
		issues: {
			title: "Startup notice",
			messages: {
				"settings-recovered":
					"Saved settings were repaired or reset to safe values for this session.",
				"settings-persistence-unavailable":
					"Camlet could not save settings to disk, so recent changes may only persist until the app closes.",
			},
		},
	},
};
