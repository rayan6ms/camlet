import {
	cornerRoundnessSchema,
	defaultOverlayAppearanceSettings,
	normalizeRingThickness,
	type OverlayAppearanceSettings,
	type OverlayAppearanceSettingsPatch,
	overlayAppearanceSettingsPatchSchema,
	overlayAppearanceSettingsSchema,
	overlayShapeSchema,
	overlaySizeSchema,
	previewFitModeSchema,
	ringAccentColorSchema,
	ringColorSchema,
	ringThicknessSchema,
} from "./appearance.js";
import {
	type AppLanguage,
	appLanguageSchema,
	defaultAppLanguage,
	isRecord,
} from "./language.js";
import {
	nullableSchema,
	numberSchema,
	objectSchema,
	type Schema,
	stringSchema,
} from "./validation.js";
import {
	defaultWindowState,
	mergeWindowState,
	minimumWindowHeight,
	minimumWindowWidth,
	type WindowState,
} from "./window-state.js";

const selectedCameraDeviceIdSchema = nullableSchema(
	stringSchema({ minLength: 1 }),
);

export interface CamletSettings {
	language: AppLanguage;
	selectedCameraDeviceId: string | null;
	overlayShape: OverlayAppearanceSettings["overlayShape"];
	overlaySize: number;
	ringColor: string;
	ringAccentColor: string;
	ringThickness: number;
	cornerRoundness: number;
	previewFitMode: OverlayAppearanceSettings["previewFitMode"];
	window: WindowState;
}

export const camletSettingsSchema = objectSchema({
	language: appLanguageSchema,
	selectedCameraDeviceId: selectedCameraDeviceIdSchema,
	overlayShape: overlayShapeSchema,
	overlaySize: overlaySizeSchema,
	ringColor: ringColorSchema,
	ringAccentColor: ringAccentColorSchema,
	ringThickness: ringThicknessSchema,
	cornerRoundness: cornerRoundnessSchema,
	previewFitMode: previewFitModeSchema,
	window: objectSchema({
		x: numberSchema({ integer: true }),
		y: numberSchema({ integer: true }),
		width: numberSchema({ integer: true, min: minimumWindowWidth }),
		height: numberSchema({ integer: true, min: minimumWindowHeight }),
	}),
});

export const overlayAppearanceSettingsKeys = [
	"overlayShape",
	"overlaySize",
	"ringColor",
	"ringAccentColor",
	"ringThickness",
	"cornerRoundness",
	"previewFitMode",
] as const;

export type CamletSettingsPatch = Partial<Omit<CamletSettings, "window">> & {
	window?: Partial<WindowState>;
};

export const defaultCamletSettings: CamletSettings = {
	language: defaultAppLanguage,
	selectedCameraDeviceId: null,
	...defaultOverlayAppearanceSettings,
	window: defaultWindowState,
};

function parseWithFallback<T>(
	schema: Schema<T>,
	value: unknown,
	fallback: T,
): T {
	const result = schema.safeParse(value);
	return result.success ? result.data : fallback;
}

function resolveLegacyOverlayShape(value: unknown): unknown {
	if (value === "ring") {
		return "circle";
	}

	if (value === "rectangle") {
		return "rectangle-y";
	}

	return value;
}

export function getOverlayAppearanceSettings(
	settings: Pick<
		CamletSettings,
		(typeof overlayAppearanceSettingsKeys)[number]
	>,
): OverlayAppearanceSettings {
	return {
		overlayShape: settings.overlayShape,
		overlaySize: settings.overlaySize,
		ringColor: settings.ringColor,
		ringAccentColor: settings.ringAccentColor,
		ringThickness: settings.ringThickness,
		cornerRoundness: settings.cornerRoundness,
		previewFitMode: settings.previewFitMode,
	};
}

export function mergeOverlayAppearanceSettings(
	value: unknown,
): OverlayAppearanceSettings {
	if (!isRecord(value)) {
		return { ...defaultOverlayAppearanceSettings };
	}

	const rawRingThickness = value.ringThickness ?? value.borderWidth;

	return {
		overlayShape: parseWithFallback(
			overlayShapeSchema,
			resolveLegacyOverlayShape(value.overlayShape ?? value.shape),
			defaultOverlayAppearanceSettings.overlayShape,
		),
		overlaySize: parseWithFallback(
			overlaySizeSchema,
			value.overlaySize ?? value.size,
			defaultOverlayAppearanceSettings.overlaySize,
		),
		ringColor: parseWithFallback(
			ringColorSchema,
			value.ringColor ?? value.color,
			defaultOverlayAppearanceSettings.ringColor,
		),
		ringAccentColor: parseWithFallback(
			ringAccentColorSchema,
			value.ringAccentColor ?? value.accentColor,
			defaultOverlayAppearanceSettings.ringAccentColor,
		),
		ringThickness: parseWithFallback(
			ringThicknessSchema,
			typeof rawRingThickness === "number"
				? normalizeRingThickness(rawRingThickness)
				: rawRingThickness,
			defaultOverlayAppearanceSettings.ringThickness,
		),
		cornerRoundness: parseWithFallback(
			cornerRoundnessSchema,
			value.cornerRoundness ?? value.radius,
			defaultOverlayAppearanceSettings.cornerRoundness,
		),
		previewFitMode: parseWithFallback(
			previewFitModeSchema,
			value.previewFitMode,
			defaultOverlayAppearanceSettings.previewFitMode,
		),
	};
}

export function mergeCamletSettings(value: unknown): CamletSettings {
	if (!isRecord(value)) {
		return {
			...defaultCamletSettings,
			window: { ...defaultCamletSettings.window },
		};
	}

	const legacyPosition = isRecord(value.position) ? value.position : undefined;
	const windowValue = isRecord(value.window)
		? {
				...legacyPosition,
				...value.window,
			}
		: legacyPosition;
	const overlayAppearance = mergeOverlayAppearanceSettings(value);

	return {
		language: parseWithFallback(
			appLanguageSchema,
			value.language,
			defaultCamletSettings.language,
		),
		selectedCameraDeviceId: parseWithFallback(
			selectedCameraDeviceIdSchema,
			value.selectedCameraDeviceId ?? value.selectedCameraDevice,
			defaultCamletSettings.selectedCameraDeviceId,
		),
		...overlayAppearance,
		window: mergeWindowState(windowValue),
	};
}

export function applyCamletSettingsPatch(
	current: CamletSettings,
	patch: CamletSettingsPatch,
): CamletSettings {
	return mergeCamletSettings({
		...current,
		...patch,
		window: {
			...current.window,
			...patch.window,
		},
	});
}

export function applyOverlayAppearanceSettingsPatch(
	current: CamletSettings,
	patch: OverlayAppearanceSettingsPatch,
): CamletSettings {
	const nextPatch = overlayAppearanceSettingsPatchSchema.parse(patch);

	return applyCamletSettingsPatch(current, {
		...(nextPatch.overlayShape !== undefined
			? { overlayShape: nextPatch.overlayShape }
			: {}),
		...(nextPatch.overlaySize !== undefined
			? { overlaySize: nextPatch.overlaySize }
			: {}),
		...(nextPatch.ringColor !== undefined
			? { ringColor: nextPatch.ringColor }
			: {}),
		...(nextPatch.ringAccentColor !== undefined
			? { ringAccentColor: nextPatch.ringAccentColor }
			: {}),
		...(nextPatch.ringThickness !== undefined
			? { ringThickness: nextPatch.ringThickness }
			: {}),
		...(nextPatch.cornerRoundness !== undefined
			? { cornerRoundness: nextPatch.cornerRoundness }
			: {}),
		...(nextPatch.previewFitMode !== undefined
			? { previewFitMode: nextPatch.previewFitMode }
			: {}),
	});
}

export {
	type OverlayAppearanceSettings,
	type OverlayAppearanceSettingsPatch,
	overlayAppearanceSettingsPatchSchema,
	overlayAppearanceSettingsSchema,
};
