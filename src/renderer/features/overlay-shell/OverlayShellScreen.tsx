import {
	useEffect,
	useEffectEvent,
	useRef,
	useState,
	useSyncExternalStore,
} from "react";
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
import {
	getCurrentLanguage,
	initializeI18n,
	subscribeToLanguageChange,
	t,
} from "../../i18n.js";
import { getOverlayAppearanceModel } from "./appearance.js";
import { useCameraPreview } from "./useCameraPreview.js";

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

function useAppLanguage() {
	return useSyncExternalStore(
		subscribeToLanguageChange,
		getCurrentLanguage,
		getCurrentLanguage,
	);
}

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
	cameraStatus: ReturnType<typeof useCameraPreview>["state"]["status"];
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

export function OverlayShellScreen({ bootstrap }: OverlayShellScreenProps) {
	useAppLanguage();
	const [windowState, setWindowState] = useState<WindowState>(() => ({
		...bootstrap.windowState,
		width: window.innerWidth,
		height: window.innerHeight,
	}));
	const [displayWorkArea, setDisplayWorkArea] = useState<DisplayWorkArea>(
		() => ({
			x: 0,
			y: 0,
			width: window.innerWidth,
			height: window.innerHeight,
		}),
	);
	const [settings, setSettings] = useState<CamletSettings>(bootstrap.settings);
	const [resizeMode, setResizeMode] = useState(false);
	const [showStartupHint, setShowStartupHint] = useState(true);
	const appearanceRequestIdRef = useRef(0);
	const languageRequestIdRef = useRef(0);
	const { state, videoRef, retry, selectDevice } = useCameraPreview({
		initialSelectedDeviceId: bootstrap.settings.selectedCameraDeviceId,
	});
	const appearance = getOverlayAppearanceSettings(settings);
	const surfaceSize = Math.max(
		96,
		Math.min(windowState.width, windowState.height),
	);
	const effectiveRingThickness = getEffectiveRingThickness(
		surfaceSize,
		appearance.ringThickness,
	);
	const maximumSquareWindowSize = getMaximumSquareWindowSize(displayWorkArea);
	const canDecreaseSize = windowState.width > minimumWindowWidth;
	const canIncreaseSize = windowState.width < maximumSquareWindowSize;
	const appearanceModel = getOverlayAppearanceModel({
		...appearance,
		overlaySize: surfaceSize,
	});
	const activeThemeId = getThemeId(
		settings.ringColor,
		settings.ringAccentColor,
	);
	const languageOptions = selectableAppLanguages;

	const syncDisplayWorkArea = useEffectEvent(() => {
		void window.camlet
			.getCurrentDisplayWorkArea()
			.then((nextDisplayWorkArea) => {
				setDisplayWorkArea(nextDisplayWorkArea);
			});
	});

	const handleContextMenuAction = useEffectEvent(
		(action: CamletContextMenuAction) => {
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
					void selectDevice(action.deviceId);
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
					void retry();
					return;
				case "reset-appearance":
					void updateAppearance(defaultOverlayAppearanceSettings);
					return;
				case "enter-resize-mode":
					setResizeMode(true);
					return;
				case "open-about-window":
					void window.camlet.openAboutWindow();
					return;
				case "close-app":
					return;
			}
		},
	);

	useEffect(() => {
		return window.camlet.onWindowStateChange((nextWindowState) => {
			setWindowState(nextWindowState);
			syncDisplayWorkArea();
		});
	}, []);

	useEffect(() => {
		return window.camlet.onContextMenuAction(handleContextMenuAction);
	}, []);

	useEffect(() => {
		void window.camlet.setWindowResizable(false);
	}, []);

	useEffect(() => {
		syncDisplayWorkArea();
	}, []);

	useEffect(() => {
		if (!showStartupHint) {
			return;
		}

		const timeout = window.setTimeout(() => {
			setShowStartupHint(false);
		}, startupHintTimeoutMs);

		return () => {
			window.clearTimeout(timeout);
		};
	}, [showStartupHint]);

	useEffect(() => {
		if (state.selectedDeviceId === null) {
			return;
		}

		setSettings((currentSettings) =>
			currentSettings.selectedCameraDeviceId === state.selectedDeviceId
				? currentSettings
				: {
						...currentSettings,
						selectedCameraDeviceId: state.selectedDeviceId,
					},
		);
	}, [state.selectedDeviceId]);

	async function updateAppearance(patch: OverlayAppearanceSettingsPatch) {
		setSettings((currentSettings) =>
			applyOverlayAppearanceSettingsPatch(currentSettings, patch),
		);

		const requestId = appearanceRequestIdRef.current + 1;
		appearanceRequestIdRef.current = requestId;
		const nextSettings =
			await window.camlet.updateOverlayAppearanceSettings(patch);

		if (requestId === appearanceRequestIdRef.current) {
			setSettings(nextSettings);
		}
	}

	async function updateLanguage(language: AppLanguage) {
		if (settings.language === language) {
			return;
		}

		const previousLanguage = settings.language;
		const requestId = languageRequestIdRef.current + 1;
		languageRequestIdRef.current = requestId;

		setSettings((currentSettings) => ({
			...currentSettings,
			language,
		}));
		await initializeI18n(resolveAppLanguage(language, bootstrap.locale.system));

		try {
			const nextSettings = await window.camlet.setLanguage(language);
			await initializeI18n(
				resolveAppLanguage(nextSettings.language, bootstrap.locale.system),
			);

			if (requestId === languageRequestIdRef.current) {
				setSettings(nextSettings);
			}
		} catch (error) {
			if (requestId === languageRequestIdRef.current) {
				setSettings((currentSettings) => ({
					...currentSettings,
					language: previousLanguage,
				}));
				await initializeI18n(
					resolveAppLanguage(previousLanguage, bootstrap.locale.system),
				);
			}
			console.error("Failed to update language", error);
		}
	}

	async function resizeByStep(delta: number) {
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

		const nextWindowState =
			await window.camlet.setWindowState(requestedWindowState);
		setWindowState(nextWindowState);
	}

	async function moveByStep(deltaX: number, deltaY: number) {
		const nextWindowState = await window.camlet.setWindowState({
			...windowState,
			x: windowState.x + deltaX,
			y: windowState.y + deltaY,
		});
		setWindowState(nextWindowState);
	}

	const handleKeyboardShortcut = useEffectEvent((event: KeyboardEvent) => {
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
	});

	useEffect(() => {
		window.addEventListener("keydown", handleKeyboardShortcut);

		return () => {
			window.removeEventListener("keydown", handleKeyboardShortcut);
		};
	}, []);

	const openContextMenu = useEffectEvent(() => {
		setShowStartupHint(false);
		void window.camlet.showContextMenu(
			createContextMenuRequest({
				activeCameraLabel: state.activeDeviceLabel,
				activeThemeId,
				cameraOptions: state.devices.map((device) => ({
					deviceId: device.deviceId,
					label: device.label,
				})),
				cameraStatus: state.status,
				cornerRoundness: settings.cornerRoundness,
				displayProtocol: bootstrap.app.displayProtocol,
				languageOptions,
				language: settings.language,
				overlayShape: settings.overlayShape,
				previewFitMode: settings.previewFitMode,
				selectedCameraDeviceId: state.selectedDeviceId,
				ringThickness: settings.ringThickness,
			}),
		);
	});

	useEffect(() => {
		void window.camlet.updateContextMenuState(
			createContextMenuRequest({
				activeCameraLabel: state.activeDeviceLabel,
				activeThemeId,
				cameraOptions: state.devices.map((device) => ({
					deviceId: device.deviceId,
					label: device.label,
				})),
				cameraStatus: state.status,
				cornerRoundness: settings.cornerRoundness,
				displayProtocol: bootstrap.app.displayProtocol,
				languageOptions,
				language: settings.language,
				overlayShape: settings.overlayShape,
				previewFitMode: settings.previewFitMode,
				selectedCameraDeviceId: state.selectedDeviceId,
				ringThickness: settings.ringThickness,
			}),
		);
	}, [
		activeThemeId,
		bootstrap.app.displayProtocol,
		settings.cornerRoundness,
		settings.language,
		settings.overlayShape,
		settings.previewFitMode,
		settings.ringThickness,
		state.activeDeviceLabel,
		state.devices,
		state.selectedDeviceId,
		state.status,
	]);

	return (
		<div
			aria-label={t("app.title")}
			className="camlet-stage"
			onContextMenu={(event) => {
				event.preventDefault();
				openContextMenu();
			}}
			role="application"
		>
			<section
				aria-label={t("overlay.preview")}
				className={`${appearanceModel.lensClassName} camlet-surface ${resizeMode ? "camlet-surface--resizing" : ""}`}
				onDoubleClick={(event) => {
					event.preventDefault();
				}}
				style={{
					...appearanceModel.cssVariables,
					width: `${surfaceSize}px`,
					height: `${surfaceSize}px`,
				}}
			>
				{effectiveRingThickness > 0 ? (
					<div aria-hidden="true" className="camlet-surface__ring" />
				) : null}
				<div
					className={`${appearanceModel.lensClassName} camlet-surface__viewport`}
				>
					<video
						autoPlay
						className="camlet-surface__video"
						muted
						playsInline
						ref={videoRef}
					/>
				</div>
				{showStartupHint && state.status === "preview" ? (
					<div className="camlet-hint" role="status">
						{t("overlay.hintOpenSettings")}
					</div>
				) : null}

				{state.status !== "preview" ? (
					<div className="camlet-status-card">
						<p className="camlet-status-card__title">
							{t(`camera.status.${state.status}`)}
						</p>
						{state.status !== "loading" ? (
							<button
								className="camlet-button"
								onClick={() => {
									void retry();
								}}
								type="button"
							>
								{t("camera.actions.retry")}
							</button>
						) : null}
					</div>
				) : null}

				{resizeMode ? (
					<div className="camlet-resize-banner">
						<p className="camlet-resize-banner__copy">
							{t("overlay.resizeAction")}
						</p>
						<div className="camlet-resize-banner__controls">
							<button
								aria-label="Decrease overlay size"
								className="camlet-button camlet-button--quiet"
								disabled={!canDecreaseSize}
								onClick={() => {
									void resizeByStep(-resizeStep);
								}}
								type="button"
							>
								-
							</button>
							<button
								aria-label="Increase overlay size"
								className="camlet-button camlet-button--quiet"
								disabled={!canIncreaseSize}
								onClick={() => {
									void resizeByStep(resizeStep);
								}}
								type="button"
							>
								+
							</button>
						</div>
						<button
							className="camlet-button camlet-button--quiet"
							onClick={() => {
								setResizeMode(false);
							}}
							type="button"
						>
							{t("overlay.resizeDone")}
						</button>
					</div>
				) : null}
			</section>
		</div>
	);
}
