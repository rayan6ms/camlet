import {
	useEffect,
	useEffectEvent,
	useRef,
	useState,
	useSyncExternalStore,
} from "react";
import { defaultOverlayAppearanceSettings } from "../../../shared/appearance.js";
import type {
	AppBootstrap,
	AppDisplayProtocol,
} from "../../../shared/bootstrap.js";
import type { CamletContextMenuAction } from "../../../shared/ipc.js";
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
	type ResizeHandle,
	resizeSquareWindowStateWithPointer,
	type ScreenPoint,
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
	id: "mint" | "coral" | "sky" | "graphite";
	ringColor: string;
}

const startupHintTimeoutMs = 2600;
const interactiveSelector = "button, input, select, textarea, label";

const themePresets: ThemePreset[] = [
	{
		id: "mint",
		ringColor: "#7CE2C6",
	},
	{
		id: "coral",
		ringColor: "#FF8B73",
	},
	{
		id: "sky",
		ringColor: "#69B7FF",
	},
	{
		id: "graphite",
		ringColor: "#F4F7FB",
	},
];

function useAppLanguage() {
	return useSyncExternalStore(
		subscribeToLanguageChange,
		getCurrentLanguage,
		getCurrentLanguage,
	);
}

function toScreenPoint(event: React.PointerEvent<HTMLElement>): ScreenPoint {
	return {
		screenX: Math.round(event.screenX),
		screenY: Math.round(event.screenY),
	};
}

function isInteractiveTarget(
	target: EventTarget | null,
): target is HTMLElement {
	return (
		target instanceof HTMLElement &&
		target.closest(interactiveSelector) !== null
	);
}

function getDisplayProtocolLabel(protocol: AppDisplayProtocol): string {
	return t(`about.displayProtocols.${protocol}`);
}

function getThemeId(ringColor: string): ThemePreset["id"] | null {
	return (
		themePresets.find((theme) => theme.ringColor === ringColor.toUpperCase())
			?.id ?? null
	);
}

function getLanguageOptions(): AppLanguage[] {
	return [...selectableAppLanguages];
}

function getLanguageLabel(language: SupportedLanguage | AppLanguage) {
	return t(`language.options.${language}`);
}

export function OverlayShellScreen({ bootstrap }: OverlayShellScreenProps) {
	useAppLanguage();
	const [windowState, setWindowState] = useState<WindowState>(() => ({
		...bootstrap.windowState,
		width: window.innerWidth,
		height: window.innerHeight,
	}));
	const [settings, setSettings] = useState<CamletSettings>(bootstrap.settings);
	const [resizeMode, setResizeMode] = useState(false);
	const [showStartupHint, setShowStartupHint] = useState(true);
	const appearanceRequestIdRef = useRef(0);
	const languageRequestIdRef = useRef(0);
	const dragSessionRef = useRef<{
		pointerId: number;
	} | null>(null);
	const resizeSessionRef = useRef<{
		handle: ResizeHandle;
		pointerId: number;
		startPointer: ScreenPoint;
		startWindowState: WindowState;
	} | null>(null);
	const surfaceRef = useRef<HTMLElement | null>(null);
	const { state, videoRef, retry, selectDevice } = useCameraPreview({
		initialSelectedDeviceId: bootstrap.settings.selectedCameraDeviceId,
	});
	const appearance = getOverlayAppearanceSettings(settings);
	const surfaceSize = Math.max(
		96,
		Math.min(windowState.width, windowState.height),
	);
	const appearanceModel = getOverlayAppearanceModel({
		...appearance,
		overlaySize: surfaceSize,
	});
	const activeThemeId = getThemeId(settings.ringColor);
	const languageOptions = getLanguageOptions();

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
						});
					}
					return;
				}
				case "set-shape":
					void updateAppearance({
						overlayShape: action.shape,
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
				case "close-app":
					return;
			}
		},
	);

	useEffect(() => {
		return window.camlet.onWindowStateChange((nextWindowState) => {
			setWindowState(nextWindowState);
		});
	}, []);

	useEffect(() => {
		return window.camlet.onContextMenuAction(handleContextMenuAction);
	}, []);

	useEffect(() => {
		void window.camlet.setWindowResizable(resizeMode);
	}, [resizeMode]);

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

	useEffect(() => {
		const handlePointerDown = (event: PointerEvent) => {
			if (
				resizeMode &&
				surfaceRef.current !== null &&
				event.target instanceof Node &&
				!surfaceRef.current.contains(event.target)
			) {
				setResizeMode(false);
			}
		};

		const handleKeyDown = (event: KeyboardEvent) => {
			if (event.key === "Escape") {
				setResizeMode(false);
			}
		};

		window.addEventListener("pointerdown", handlePointerDown);
		window.addEventListener("keydown", handleKeyDown);

		return () => {
			window.removeEventListener("pointerdown", handlePointerDown);
			window.removeEventListener("keydown", handleKeyDown);
		};
	}, [resizeMode]);

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

	const openContextMenu = () => {
		setShowStartupHint(false);

		void window.camlet.showContextMenu({
			labels: {
				theme: t("appearance.labels.theme"),
				shape: t("appearance.labels.shape"),
				language: t("language.label"),
				cameraInput: t("camera.labels.device"),
				resize: t("overlay.resizeAction"),
				advancedSettings: t("advanced.title"),
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
					coral: t("appearance.themes.coral"),
					sky: t("appearance.themes.sky"),
					graphite: t("appearance.themes.graphite"),
				},
				shapeOptions: {
					circle: t("appearance.shapes.circle"),
					roundedSquare: t("appearance.shapes.roundedSquare"),
				},
				fitModeOptions: {
					cover: t("appearance.fitModes.cover"),
					contain: t("appearance.fitModes.contain"),
				},
				languageOptions: Object.fromEntries(
					languageOptions.map((language) => [
						language,
						getLanguageLabel(language),
					]),
				) as Record<AppLanguage, string>,
			},
			selectedThemeId: activeThemeId,
			selectedShape: settings.overlayShape,
			selectedLanguage: settings.language,
			selectedFitMode: settings.previewFitMode,
			selectedRingThickness: settings.ringThickness,
			cameraOptions: state.devices.map((device) => ({
				deviceId: device.deviceId,
				label: device.label,
			})),
			selectedCameraDeviceId: state.selectedDeviceId,
			cameraStatusLabel: t(`camera.status.${state.status}`),
			activeCameraLabel: state.activeDeviceLabel ?? t("camera.labels.none"),
			displayProtocolLabel: getDisplayProtocolLabel(
				bootstrap.app.displayProtocol,
			),
		});
	};

	const endWindowDrag = useEffectEvent((pointerId: number) => {
		if (dragSessionRef.current?.pointerId !== pointerId) {
			return;
		}

		dragSessionRef.current = null;
		void window.camlet.endWindowDrag();
	});

	useEffect(() => {
		const handlePointerMove = (event: PointerEvent) => {
			if (dragSessionRef.current?.pointerId !== event.pointerId) {
				return;
			}

			if ((event.buttons & 1) !== 1) {
				endWindowDrag(event.pointerId);
				return;
			}

			void window.camlet.updateWindowDrag({
				screenX: Math.round(event.screenX),
				screenY: Math.round(event.screenY),
			});
		};

		const handlePointerEnd = (event: PointerEvent) => {
			endWindowDrag(event.pointerId);
		};

		window.addEventListener("pointermove", handlePointerMove);
		window.addEventListener("pointerup", handlePointerEnd);
		window.addEventListener("pointercancel", handlePointerEnd);

		return () => {
			window.removeEventListener("pointermove", handlePointerMove);
			window.removeEventListener("pointerup", handlePointerEnd);
			window.removeEventListener("pointercancel", handlePointerEnd);
		};
	}, []);

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
				onPointerDown={(event) => {
					if (
						resizeMode ||
						event.button !== 0 ||
						isInteractiveTarget(event.target)
					) {
						return;
					}

					event.preventDefault();
					setShowStartupHint(false);
					dragSessionRef.current = {
						pointerId: event.pointerId,
					};
					void window.camlet.startWindowDrag(toScreenPoint(event));
				}}
				onPointerUp={(event) => {
					endWindowDrag(event.pointerId);

					if (resizeMode && !isInteractiveTarget(event.target)) {
						setResizeMode(false);
					}
				}}
				ref={surfaceRef}
				style={{
					...appearanceModel.cssVariables,
					width: `${surfaceSize}px`,
					height: `${surfaceSize}px`,
				}}
			>
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
				<div className="camlet-surface__ring" />

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
						<p className="camlet-status-card__copy">
							{state.errorMessage ?? t(`camera.message.${state.status}`)}
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
					<>
						<div className="camlet-resize-banner">
							<p className="camlet-resize-banner__copy">
								{t("overlay.resizeHint")}
							</p>
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

						{(["nw", "ne", "se", "sw"] as ResizeHandle[]).map((handle) => (
							<button
								aria-label={t("overlay.resizeHandle")}
								className={`camlet-resize-handle camlet-resize-handle--${handle}`}
								key={handle}
								onPointerCancel={(event) => {
									if (resizeSessionRef.current?.pointerId !== event.pointerId) {
										return;
									}

									if (event.currentTarget.hasPointerCapture(event.pointerId)) {
										event.currentTarget.releasePointerCapture(event.pointerId);
									}

									resizeSessionRef.current = null;
								}}
								onPointerDown={(event) => {
									event.preventDefault();
									event.stopPropagation();
									resizeSessionRef.current = {
										handle,
										pointerId: event.pointerId,
										startPointer: toScreenPoint(event),
										startWindowState: windowState,
									};
									event.currentTarget.setPointerCapture(event.pointerId);
								}}
								onPointerMove={(event) => {
									const session = resizeSessionRef.current;

									if (session?.pointerId !== event.pointerId) {
										return;
									}

									if ((event.buttons & 1) !== 1) {
										if (
											event.currentTarget.hasPointerCapture(event.pointerId)
										) {
											event.currentTarget.releasePointerCapture(
												event.pointerId,
											);
										}

										resizeSessionRef.current = null;
										return;
									}

									void window.camlet.setWindowState(
										resizeSquareWindowStateWithPointer(
											session.startWindowState,
											session.startPointer,
											toScreenPoint(event),
											session.handle,
										),
									);
								}}
								onPointerUp={(event) => {
									if (resizeSessionRef.current?.pointerId !== event.pointerId) {
										return;
									}

									if (event.currentTarget.hasPointerCapture(event.pointerId)) {
										event.currentTarget.releasePointerCapture(event.pointerId);
									}

									resizeSessionRef.current = null;
								}}
								type="button"
							/>
						))}
					</>
				) : null}
			</section>
		</div>
	);
}
