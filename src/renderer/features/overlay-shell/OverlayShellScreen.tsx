import {
	useEffect,
	useEffectEvent,
	useId,
	useRef,
	useState,
	useSyncExternalStore,
} from "react";
import {
	defaultOverlayAppearanceSettings,
	getRoundedSquareRadius,
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
	id: "mint" | "ocean" | "ember" | "orchid" | "grove" | "graphite";
	ringColor: string;
	ringAccentColor: string;
}

const startupHintTimeoutMs = 2600;
const interactiveSelector = "button, input, select, textarea, label";

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

interface RingShapeDefinitionBase {
	strokeWidth: number;
}

type RingShapeDefinition =
	| (RingShapeDefinitionBase & {
			type: "circle";
			cx: number;
			cy: number;
			r: number;
	  })
	| (RingShapeDefinitionBase & {
			type: "rect";
			x: number;
			y: number;
			width: number;
			height: number;
			rx: number;
	  })
	| (RingShapeDefinitionBase & {
			type: "polygon";
			points: string;
	  });

function getRingShapeDefinition(
	appearance: ReturnType<typeof getOverlayAppearanceSettings> & {
		overlaySize: number;
	},
): RingShapeDefinition {
	const { overlayShape, overlaySize, ringThickness, cornerRoundness } =
		appearance;
	const strokeInset = ringThickness / 2;

	switch (overlayShape) {
		case "circle":
			return {
				type: "circle",
				cx: overlaySize / 2,
				cy: overlaySize / 2,
				r: overlaySize / 2 - strokeInset,
				strokeWidth: ringThickness,
			};
		case "rounded-square":
			return {
				type: "rect",
				x: strokeInset,
				y: strokeInset,
				width: overlaySize - ringThickness,
				height: overlaySize - ringThickness,
				rx: Math.max(
					0,
					getRoundedSquareRadius(overlaySize, cornerRoundness) - strokeInset,
				),
				strokeWidth: ringThickness,
			};
		case "rectangle": {
			const inset = overlaySize * 0.16;
			const width = Math.max(1, overlaySize - inset * 2 - ringThickness);
			const height = Math.max(1, overlaySize - ringThickness);
			return {
				type: "rect",
				x: inset + strokeInset,
				y: strokeInset,
				width,
				height,
				rx: Math.max(
					0,
					Math.min(
						getRoundedSquareRadius(Math.min(width, height), cornerRoundness),
						width / 2,
						height / 2,
					),
				),
				strokeWidth: ringThickness,
			};
		}
		case "diamond":
			return {
				type: "polygon",
				points: [
					`${overlaySize / 2},${strokeInset}`,
					`${overlaySize - strokeInset},${overlaySize / 2}`,
					`${overlaySize / 2},${overlaySize - strokeInset}`,
					`${strokeInset},${overlaySize / 2}`,
				].join(" "),
				strokeWidth: ringThickness,
			};
	}
}

function OverlayRingShape({
	definition,
	className,
	stroke,
}: {
	className: string;
	definition: RingShapeDefinition;
	stroke?: string;
}) {
	switch (definition.type) {
		case "circle":
			return (
				<circle
					className={className}
					cx={definition.cx}
					cy={definition.cy}
					r={definition.r}
					stroke={stroke}
					strokeWidth={definition.strokeWidth}
				/>
			);
		case "rect":
			return (
				<rect
					className={className}
					height={definition.height}
					rx={definition.rx}
					stroke={stroke}
					strokeWidth={definition.strokeWidth}
					width={definition.width}
					x={definition.x}
					y={definition.y}
				/>
			);
		case "polygon":
			return (
				<polygon
					className={className}
					points={definition.points}
					stroke={stroke}
					strokeWidth={definition.strokeWidth}
				/>
			);
	}
}

function OverlayRingSvg({
	appearance,
}: {
	appearance: ReturnType<typeof getOverlayAppearanceSettings> & {
		overlaySize: number;
	};
}) {
	const gradientId = useId().replace(/:/g, "");
	const definition = getRingShapeDefinition(appearance);
	const highlightStrokeWidth = Math.max(1, appearance.ringThickness * 0.16);

	return (
		<svg
			aria-hidden="true"
			className="camlet-surface__ring"
			viewBox={`0 0 ${appearance.overlaySize} ${appearance.overlaySize}`}
		>
			<defs>
				<linearGradient id={gradientId} x1="12%" x2="88%" y1="8%" y2="92%">
					<stop offset="0%" stopColor={appearance.ringColor} />
					<stop offset="100%" stopColor={appearance.ringAccentColor} />
				</linearGradient>
			</defs>
			<OverlayRingShape
				className="camlet-surface__ring-shape"
				definition={definition}
				stroke={`url(#${gradientId})`}
			/>
			<OverlayRingShape
				className="camlet-surface__ring-highlight"
				definition={{ ...definition, strokeWidth: highlightStrokeWidth }}
			/>
		</svg>
	);
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
				rectangle: t("appearance.shapes.rectangle"),
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
	const [settings, setSettings] = useState<CamletSettings>(bootstrap.settings);
	const [resizeMode, setResizeMode] = useState(false);
	const [showStartupHint, setShowStartupHint] = useState(true);
	const appearanceRequestIdRef = useRef(0);
	const languageRequestIdRef = useRef(0);
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
	const activeThemeId = getThemeId(
		settings.ringColor,
		settings.ringAccentColor,
	);
	const languageOptions = selectableAppLanguages;

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
				onPointerUp={(event) => {
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
				<OverlayRingSvg
					appearance={{
						...appearance,
						overlaySize: surfaceSize,
					}}
				/>

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
					<>
						<div className="camlet-resize-banner">
							<p className="camlet-resize-banner__copy">
								{t("overlay.resizeAction")}
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
