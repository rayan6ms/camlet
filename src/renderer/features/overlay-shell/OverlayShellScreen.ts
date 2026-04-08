import {
	defaultOverlayAppearanceSettings,
	getEffectiveRingThickness,
} from "../../../shared/appearance.js";
import type {
	AppBootstrap,
	AppDisplayProtocol,
} from "../../../shared/bootstrap.js";
import type {
	CamletContextMenuAction,
	CamletContextMenuRequest,
} from "../../../shared/ipc.js";
import {
	type AppLanguage,
	resolveAppLanguage,
	type SupportedLanguage,
	selectableAppLanguages,
} from "../../../shared/language.js";
import {
	applyOverlayAppearanceSettingsPatch,
	type CamletSettings,
	getOverlayAppearanceSettings,
	type OverlayAppearanceSettingsPatch,
} from "../../../shared/settings.js";
import {
	type DisplayWorkArea,
	getMaximumSquareWindowSize,
	minimumWindowWidth,
	resizeSquareWindowStateByDelta,
	resizeStep,
	type WindowState,
} from "../../../shared/window-state.js";
import type { MountedScreen } from "../../components/AboutScreen.js";
import { initializeI18n, subscribeToLanguageChange, t } from "../../i18n.js";
import { getOverlayAppearanceModel } from "./appearance.js";
import {
	type CameraPreviewState,
	createCameraPreviewController,
} from "./useCameraPreview.js";

interface OverlayShellScreenProps {
	bootstrap: AppBootstrap;
}

interface ThemePreset {
	id: "mint" | "ocean" | "ember" | "orchid" | "grove" | "graphite";
	ringColor: string;
	ringAccentColor: string;
}

const startupHintTimeoutMs = 2600;
const moveStep = 1;
const moveStepLarge = 24;
const themePresets: ThemePreset[] = [
	{
		id: "mint",
		ringColor: "#7CE2C6",
		ringAccentColor: "#C8FFF1",
	},
	{
		id: "ocean",
		ringColor: "#4DA7FF",
		ringAccentColor: "#77F1E1",
	},
	{
		id: "ember",
		ringColor: "#FF6C4D",
		ringAccentColor: "#FFB067",
	},
	{
		id: "orchid",
		ringColor: "#B78CFF",
		ringAccentColor: "#F1A8FF",
	},
	{
		id: "grove",
		ringColor: "#57D97B",
		ringAccentColor: "#B8FF9D",
	},
	{
		id: "graphite",
		ringColor: "#F2F5F9",
		ringAccentColor: "#93A1B8",
	},
];

function getDisplayProtocolLabel(protocol: AppDisplayProtocol): string {
	return t(`about.displayProtocols.${protocol}`);
}

function getThemeId(
	ringColor: string,
	ringAccentColor: string,
): ThemePreset["id"] | null {
	return (
		themePresets.find(
			(theme) =>
				theme.ringColor === ringColor.toUpperCase() &&
				theme.ringAccentColor === ringAccentColor.toUpperCase(),
		)?.id ?? null
	);
}

function getLanguageLabel(language: SupportedLanguage | AppLanguage) {
	return t(`language.options.${language}`);
}

function createContextMenuRequest(input: {
	activeCameraLabel: string | null;
	activeThemeId: ThemePreset["id"] | null;
	cameraOptions: Array<{ deviceId: string; label: string }>;
	cameraStatus: CameraPreviewState["status"];
	cornerRoundness: number;
	displayProtocol: AppDisplayProtocol;
	languageOptions: readonly AppLanguage[];
	language: AppLanguage;
	overlayShape: CamletSettings["overlayShape"];
	previewFitMode: CamletSettings["previewFitMode"];
	selectedCameraDeviceId: string | null;
	ringThickness: number;
}): CamletContextMenuRequest {
	return {
		labels: {
			theme: t("appearance.labels.theme"),
			shape: t("appearance.labels.shape"),
			cornerRoundness: t("appearance.labels.cornerRoundness"),
			language: t("language.label"),
			cameraInput: t("camera.labels.device"),
			resize: t("overlay.resizeAction"),
			advancedSettings: t("advanced.title"),
			aboutCamlet: t("about.windowTitle"),
			closeApp: t("app.close"),
			retryCamera: t("camera.actions.retry"),
			resetAppearance: t("settings.actions.resetAppearance"),
			fitMode: t("appearance.labels.fitMode"),
			ringThickness: t("appearance.labels.ringThickness"),
			systemInfo: t("sections.system"),
			status: t("camera.labels.permission"),
			activeDevice: t("camera.labels.activeDevice"),
			displayProtocol: t("about.labels.displayProtocol"),
			noDevices: t("camera.labels.noDevices"),
			themeOptions: {
				mint: t("appearance.themes.mint"),
				ocean: t("appearance.themes.ocean"),
				ember: t("appearance.themes.ember"),
				orchid: t("appearance.themes.orchid"),
				grove: t("appearance.themes.grove"),
				graphite: t("appearance.themes.graphite"),
			},
			shapeOptions: {
				circle: t("appearance.shapes.circle"),
				roundedSquare: t("appearance.shapes.roundedSquare"),
				diamond: t("appearance.shapes.diamond"),
				rectangleY: t("appearance.shapes.rectangleY"),
				rectangleX: t("appearance.shapes.rectangleX"),
			},
			fitModeOptions: {
				cover: t("appearance.fitModes.cover"),
				contain: t("appearance.fitModes.contain"),
			},
			languageOptions: Object.fromEntries(
				input.languageOptions.map((language) => [
					language,
					getLanguageLabel(language),
				]),
			) as Record<AppLanguage, string>,
		},
		selectedThemeId: input.activeThemeId,
		selectedShape: input.overlayShape,
		selectedLanguage: input.language,
		selectedFitMode: input.previewFitMode,
		selectedRingThickness: input.ringThickness,
		selectedCornerRoundness: input.cornerRoundness,
		cameraOptions: input.cameraOptions,
		selectedCameraDeviceId: input.selectedCameraDeviceId,
		cameraStatusLabel: t(`camera.status.${input.cameraStatus}`),
		activeCameraLabel: input.activeCameraLabel ?? t("camera.labels.none"),
		displayProtocolLabel: getDisplayProtocolLabel(input.displayProtocol),
	};
}

export function createOverlayShellScreen({
	bootstrap,
}: OverlayShellScreenProps): MountedScreen {
	document.title = "Camlet";

	let destroyed = false;
	let windowState: WindowState = {
		...bootstrap.windowState,
		width: window.innerWidth,
		height: window.innerHeight,
	};
	let displayWorkArea: DisplayWorkArea = {
		x: 0,
		y: 0,
		width: window.innerWidth,
		height: window.innerHeight,
	};
	let settings = bootstrap.settings;
	let resizeMode = false;
	let showStartupHint = true;
	let appearanceRequestId = 0;
	let languageRequestId = 0;
	const languageOptions = selectableAppLanguages;
	const cameraPreview = createCameraPreviewController({
		initialSelectedDeviceId: bootstrap.settings.selectedCameraDeviceId,
	});
	let cameraState = cameraPreview.getState();

	const stage = document.createElement("div");
	stage.className = "camlet-stage";
	stage.role = "application";

	const surface = document.createElement("section");
	surface.addEventListener("dblclick", (event) => {
		event.preventDefault();
	});

	const ring = document.createElement("div");
	ring.className = "camlet-surface__ring";
	ring.setAttribute("aria-hidden", "true");

	const viewport = document.createElement("div");
	viewport.className = "camlet-surface__viewport";

	const video = document.createElement("video");
	video.autoplay = true;
	video.className = "camlet-surface__video";
	video.muted = true;
	video.playsInline = true;
	viewport.append(video);

	const hint = document.createElement("div");
	hint.className = "camlet-hint";
	hint.role = "status";

	const statusCard = document.createElement("div");
	statusCard.className = "camlet-status-card";

	const statusTitle = document.createElement("p");
	statusTitle.className = "camlet-status-card__title";

	const retryButton = document.createElement("button");
	retryButton.className = "camlet-button";
	retryButton.type = "button";
	retryButton.addEventListener("click", () => {
		void cameraPreview.retry();
	});
	statusCard.append(statusTitle, retryButton);

	const resizeBanner = document.createElement("div");
	resizeBanner.className = "camlet-resize-banner";

	const resizeCopy = document.createElement("p");
	resizeCopy.className = "camlet-resize-banner__copy";

	const resizeControls = document.createElement("div");
	resizeControls.className = "camlet-resize-banner__controls";

	const decreaseSizeButton = document.createElement("button");
	decreaseSizeButton.setAttribute("aria-label", "Decrease overlay size");
	decreaseSizeButton.className = "camlet-button camlet-button--quiet";
	decreaseSizeButton.type = "button";
	decreaseSizeButton.textContent = "-";
	decreaseSizeButton.addEventListener("click", () => {
		void resizeByStep(-resizeStep);
	});

	const increaseSizeButton = document.createElement("button");
	increaseSizeButton.setAttribute("aria-label", "Increase overlay size");
	increaseSizeButton.className = "camlet-button camlet-button--quiet";
	increaseSizeButton.type = "button";
	increaseSizeButton.textContent = "+";
	increaseSizeButton.addEventListener("click", () => {
		void resizeByStep(resizeStep);
	});
	resizeControls.append(decreaseSizeButton, increaseSizeButton);

	const finishResizeButton = document.createElement("button");
	finishResizeButton.className = "camlet-button camlet-button--quiet";
	finishResizeButton.type = "button";
	finishResizeButton.addEventListener("click", () => {
		resizeMode = false;
		render();
	});

	resizeBanner.append(resizeCopy, resizeControls, finishResizeButton);
	surface.append(ring, viewport, hint, statusCard, resizeBanner);
	stage.append(surface);

	cameraPreview.attachVideoElement(video);

	function render() {
		if (destroyed) {
			return;
		}

		const appearance = getOverlayAppearanceSettings(settings);
		const surfaceSize = Math.max(
			96,
			Math.min(windowState.width, windowState.height),
		);
		const effectiveRingThickness = getEffectiveRingThickness(
			surfaceSize,
			appearance.ringThickness,
		);
		const usesNativeWindowShape =
			bootstrap.app.platform === "linux" || bootstrap.app.platform === "win32";
		const appearanceModel = getOverlayAppearanceModel({
			...appearance,
			overlaySize: surfaceSize,
		});
		const usesNativeRinglessShape =
			effectiveRingThickness === 0 && usesNativeWindowShape;
		const canDecreaseSize = windowState.width > minimumWindowWidth;
		const maximumSquareWindowSize = getMaximumSquareWindowSize(displayWorkArea);
		const canIncreaseSize = windowState.width < maximumSquareWindowSize;

		stage.setAttribute("aria-label", t("app.title"));
		surface.setAttribute("aria-label", t("overlay.preview"));
		surface.className = [
			appearanceModel.lensClassName,
			"camlet-surface",
			resizeMode ? "camlet-surface--resizing" : "",
			usesNativeRinglessShape ? "camlet-surface--ringless-native-shape" : "",
		]
			.filter((className) => className.length > 0)
			.join(" ");
		viewport.className = `${appearanceModel.lensClassName} camlet-surface__viewport`;
		surface.style.width = `${surfaceSize}px`;
		surface.style.height = `${surfaceSize}px`;

		for (const [name, value] of Object.entries(appearanceModel.cssVariables)) {
			surface.style.setProperty(name, value);
		}

		ring.hidden = effectiveRingThickness === 0;
		hint.hidden = !(showStartupHint && cameraState.status === "preview");
		hint.textContent = t("overlay.hintOpenSettings");

		statusCard.hidden = cameraState.status === "preview";
		statusTitle.textContent = t(`camera.status.${cameraState.status}`);
		retryButton.hidden = cameraState.status === "loading";
		retryButton.textContent = t("camera.actions.retry");

		resizeBanner.hidden = !resizeMode;
		resizeCopy.textContent = t("overlay.resizeAction");
		decreaseSizeButton.disabled = !canDecreaseSize;
		increaseSizeButton.disabled = !canIncreaseSize;
		finishResizeButton.textContent = t("overlay.resizeDone");
	}

	function buildContextMenuRequest(): CamletContextMenuRequest {
		return createContextMenuRequest({
			activeCameraLabel: cameraState.activeDeviceLabel,
			activeThemeId: getThemeId(settings.ringColor, settings.ringAccentColor),
			cameraOptions: cameraState.devices.map((device) => ({
				deviceId: device.deviceId,
				label: device.label,
			})),
			cameraStatus: cameraState.status,
			cornerRoundness: settings.cornerRoundness,
			displayProtocol: bootstrap.app.displayProtocol,
			languageOptions,
			language: settings.language,
			overlayShape: settings.overlayShape,
			previewFitMode: settings.previewFitMode,
			selectedCameraDeviceId: cameraState.selectedDeviceId,
			ringThickness: settings.ringThickness,
		});
	}

	function updateContextMenuState() {
		if (destroyed) {
			return;
		}

		void window.camlet.updateContextMenuState(buildContextMenuRequest());
	}

	async function syncDisplayWorkArea() {
		const nextDisplayWorkArea = await window.camlet.getCurrentDisplayWorkArea();

		if (destroyed) {
			return;
		}

		displayWorkArea = nextDisplayWorkArea;
		render();
	}

	function syncSettingsWithSelectedCamera() {
		if (cameraState.selectedDeviceId === null) {
			return;
		}

		if (settings.selectedCameraDeviceId === cameraState.selectedDeviceId) {
			return;
		}

		settings = {
			...settings,
			selectedCameraDeviceId: cameraState.selectedDeviceId,
		};
	}

	async function updateAppearance(patch: OverlayAppearanceSettingsPatch) {
		settings = applyOverlayAppearanceSettingsPatch(settings, patch);
		render();
		updateContextMenuState();

		const nextRequestId = appearanceRequestId + 1;
		appearanceRequestId = nextRequestId;
		const nextSettings =
			await window.camlet.updateOverlayAppearanceSettings(patch);

		if (destroyed || nextRequestId !== appearanceRequestId) {
			return;
		}

		settings = nextSettings;
		render();
		updateContextMenuState();
	}

	async function updateLanguage(language: AppLanguage) {
		if (settings.language === language) {
			return;
		}

		const previousLanguage = settings.language;
		const nextRequestId = languageRequestId + 1;
		languageRequestId = nextRequestId;

		settings = {
			...settings,
			language,
		};
		await initializeI18n(resolveAppLanguage(language, bootstrap.locale.system));
		render();
		updateContextMenuState();

		try {
			const nextSettings = await window.camlet.setLanguage(language);
			await initializeI18n(
				resolveAppLanguage(nextSettings.language, bootstrap.locale.system),
			);

			if (destroyed || nextRequestId !== languageRequestId) {
				return;
			}

			settings = nextSettings;
			render();
			updateContextMenuState();
		} catch (error) {
			if (!destroyed && nextRequestId === languageRequestId) {
				settings = {
					...settings,
					language: previousLanguage,
				};
				await initializeI18n(
					resolveAppLanguage(previousLanguage, bootstrap.locale.system),
				);
				render();
				updateContextMenuState();
			}
			console.error("Failed to update language", error);
		}
	}

	async function resizeByStep(delta: number) {
		const maximumSquareWindowSize = getMaximumSquareWindowSize(displayWorkArea);
		const canDecreaseSize = windowState.width > minimumWindowWidth;
		const canIncreaseSize = windowState.width < maximumSquareWindowSize;

		if ((delta < 0 && !canDecreaseSize) || (delta > 0 && !canIncreaseSize)) {
			return;
		}

		const requestedWindowState = resizeSquareWindowStateByDelta(
			windowState,
			delta,
			maximumSquareWindowSize,
		);

		if (requestedWindowState.width === windowState.width) {
			return;
		}

		windowState = await window.camlet.setWindowState(requestedWindowState);
		render();
	}

	async function moveByStep(deltaX: number, deltaY: number) {
		windowState = await window.camlet.setWindowState({
			...windowState,
			x: windowState.x + deltaX,
			y: windowState.y + deltaY,
		});
		render();
	}

	function openContextMenu() {
		showStartupHint = false;
		render();
		void window.camlet.showContextMenu(buildContextMenuRequest());
	}

	function handleContextMenuAction(action: CamletContextMenuAction) {
		switch (action.type) {
			case "set-theme": {
				const nextTheme = themePresets.find(
					(theme) => theme.id === action.themeId,
				);

				if (nextTheme !== undefined) {
					void updateAppearance({
						ringColor: nextTheme.ringColor,
						ringAccentColor: nextTheme.ringAccentColor,
					});
				}
				return;
			}
			case "set-shape":
				void updateAppearance({
					overlayShape: action.shape,
				});
				return;
			case "set-corner-roundness":
				void updateAppearance({
					cornerRoundness: action.cornerRoundness,
				});
				return;
			case "set-language":
				void updateLanguage(action.language);
				return;
			case "set-camera":
				void cameraPreview.selectDevice(action.deviceId);
				return;
			case "set-fit-mode":
				void updateAppearance({
					previewFitMode: action.fitMode,
				});
				return;
			case "set-ring-thickness":
				void updateAppearance({
					ringThickness: action.ringThickness,
				});
				return;
			case "retry-camera":
				void cameraPreview.retry();
				return;
			case "reset-appearance":
				void updateAppearance(defaultOverlayAppearanceSettings);
				return;
			case "enter-resize-mode":
				resizeMode = true;
				render();
				return;
			case "open-about-window":
				void window.camlet.openAboutWindow();
				return;
			case "close-app":
				return;
		}
	}

	function handleKeyboardShortcut(event: KeyboardEvent) {
		if (
			!document.hasFocus() ||
			event.defaultPrevented ||
			event.altKey ||
			event.ctrlKey ||
			event.metaKey
		) {
			return;
		}

		const target = event.target;
		if (
			target instanceof HTMLElement &&
			target.closest("input, textarea, select")
		) {
			return;
		}

		const keyboardMoveStep = event.shiftKey ? moveStepLarge : moveStep;

		switch (event.code) {
			case "ArrowUp":
				event.preventDefault();
				void moveByStep(0, -keyboardMoveStep);
				return;
			case "ArrowDown":
				event.preventDefault();
				void moveByStep(0, keyboardMoveStep);
				return;
			case "ArrowLeft":
				event.preventDefault();
				void moveByStep(-keyboardMoveStep, 0);
				return;
			case "ArrowRight":
				event.preventDefault();
				void moveByStep(keyboardMoveStep, 0);
				return;
			case "Minus":
			case "NumpadSubtract":
				event.preventDefault();
				void resizeByStep(-resizeStep);
				return;
			case "Equal":
			case "NumpadAdd":
				event.preventDefault();
				void resizeByStep(resizeStep);
				return;
		}
	}

	stage.addEventListener("contextmenu", (event) => {
		event.preventDefault();
		openContextMenu();
	});

	const removeWindowStateListener = window.camlet.onWindowStateChange(
		(nextWindowState) => {
			windowState = nextWindowState;
			render();
			void syncDisplayWorkArea();
		},
	);
	const removeContextMenuListener = window.camlet.onContextMenuAction(
		handleContextMenuAction,
	);
	const removeLanguageListener = subscribeToLanguageChange(() => {
		render();
		updateContextMenuState();
	});
	const removeCameraListener = cameraPreview.subscribe((nextState) => {
		cameraState = nextState;
		syncSettingsWithSelectedCamera();
		render();
		updateContextMenuState();
	});
	const startupHintTimeout = window.setTimeout(() => {
		showStartupHint = false;
		render();
	}, startupHintTimeoutMs);

	window.addEventListener("keydown", handleKeyboardShortcut);
	void window.camlet.setWindowResizable(false);
	void syncDisplayWorkArea();
	render();
	updateContextMenuState();

	return {
		destroy() {
			destroyed = true;
			window.clearTimeout(startupHintTimeout);
			window.removeEventListener("keydown", handleKeyboardShortcut);
			removeCameraListener();
			removeLanguageListener();
			removeContextMenuListener();
			removeWindowStateListener();
			cameraPreview.destroy();
		},
		element: stage,
	};
}
