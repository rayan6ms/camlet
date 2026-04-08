import {
	enumSchema,
	numberSchema,
	objectSchema,
	partialObjectSchema,
	stringSchema,
} from "./validation.js";

const hexColorPattern = /^#(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;
const minimumOverlaySize = 96;
const maximumOverlaySize = 640;
const minimumRingThickness = 0;
const maximumRingThickness = 10;
const minimumCornerRoundness = 0;
const maximumCornerRoundness = 72;
const overlayShapeValues = [
	"circle",
	"rounded-square",
	"diamond",
	"rectangle-y",
	"rectangle-x",
] as const;
const previewFitModeValues = ["cover", "contain"] as const;

export type OverlayShape = (typeof overlayShapeValues)[number];
export type PreviewFitMode = (typeof previewFitModeValues)[number];
export interface OverlayAppearanceSettings {
	overlayShape: OverlayShape;
	overlaySize: number;
	ringColor: string;
	ringAccentColor: string;
	ringThickness: number;
	cornerRoundness: number;
	previewFitMode: PreviewFitMode;
}
export type OverlayAppearanceSettingsPatch = Partial<OverlayAppearanceSettings>;

export const overlayShapeSchema = enumSchema(overlayShapeValues);
export const previewFitModeSchema = enumSchema(previewFitModeValues);
export const overlaySizeSchema = numberSchema({
	integer: true,
	min: minimumOverlaySize,
	max: maximumOverlaySize,
});
export const ringColorSchema = stringSchema({ pattern: hexColorPattern });
export const ringAccentColorSchema = stringSchema({ pattern: hexColorPattern });
export const ringThicknessSchema = numberSchema({
	integer: true,
	min: minimumRingThickness,
	max: maximumRingThickness,
});
export const cornerRoundnessSchema = numberSchema({
	integer: true,
	min: minimumCornerRoundness,
	max: maximumCornerRoundness,
});

export const overlayAppearanceSettingsSchema = objectSchema({
	overlayShape: overlayShapeSchema,
	overlaySize: overlaySizeSchema,
	ringColor: ringColorSchema,
	ringAccentColor: ringAccentColorSchema,
	ringThickness: ringThicknessSchema,
	cornerRoundness: cornerRoundnessSchema,
	previewFitMode: previewFitModeSchema,
});

export const overlayAppearanceSettingsPatchSchema = partialObjectSchema({
	overlayShape: overlayShapeSchema,
	overlaySize: overlaySizeSchema,
	ringColor: ringColorSchema,
	ringAccentColor: ringAccentColorSchema,
	ringThickness: ringThicknessSchema,
	cornerRoundness: cornerRoundnessSchema,
	previewFitMode: previewFitModeSchema,
});

export const defaultOverlayAppearanceSettings: OverlayAppearanceSettings = {
	overlayShape: "circle",
	overlaySize: 224,
	ringColor: "#7CE2C6",
	ringAccentColor: "#C8FFF1",
	ringThickness: 4,
	cornerRoundness: 26,
	previewFitMode: "cover",
};

const ringThicknessOptions = [0, 2, 4, 6, 8, 10] as const;

export function normalizeRingThickness(ringThickness: number): number {
	const roundedValue = Math.min(
		maximumRingThickness,
		Math.max(minimumRingThickness, Math.round(ringThickness)),
	);

	return ringThicknessOptions.reduce((closest, option) =>
		Math.abs(option - roundedValue) < Math.abs(closest - roundedValue)
			? option
			: closest,
	);
}

export function getRoundedSquareRadius(
	size: number,
	cornerRoundness = defaultOverlayAppearanceSettings.cornerRoundness,
): number {
	return Math.min(Math.max(0, cornerRoundness), Math.round(size / 2));
}

export function getEffectiveRingThickness(
	_overlaySize: number,
	ringThickness: number,
): number {
	return normalizeRingThickness(ringThickness);
}

export function isOverlayAppearanceSettingsEqual(
	a: OverlayAppearanceSettings,
	b: OverlayAppearanceSettings,
): boolean {
	return (
		a.overlayShape === b.overlayShape &&
		a.overlaySize === b.overlaySize &&
		a.ringColor === b.ringColor &&
		a.ringAccentColor === b.ringAccentColor &&
		a.ringThickness === b.ringThickness &&
		a.cornerRoundness === b.cornerRoundness &&
		a.previewFitMode === b.previewFitMode
	);
}

export function isDefaultOverlayAppearanceSettings(
	settings: OverlayAppearanceSettings,
): boolean {
	return isOverlayAppearanceSettingsEqual(
		settings,
		defaultOverlayAppearanceSettings,
	);
}
