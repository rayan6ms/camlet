import { z } from "zod";

const hexColorPattern = /^#(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;

export const overlayShapeSchema = z.enum([
	"circle",
	"rounded-square",
	"diamond",
	"rectangle-y",
	"rectangle-x",
]);
export const previewFitModeSchema = z.enum(["cover", "contain"]);
export const overlaySizeSchema = z.number().int().min(96).max(640);
export const ringColorSchema = z.string().regex(hexColorPattern);
export const ringAccentColorSchema = z.string().regex(hexColorPattern);
export const ringThicknessSchema = z.number().int().min(0).max(10);
export const cornerRoundnessSchema = z.number().int().min(0).max(72);

export const overlayAppearanceSettingsSchema = z.object({
	overlayShape: overlayShapeSchema,
	overlaySize: overlaySizeSchema,
	ringColor: ringColorSchema,
	ringAccentColor: ringAccentColorSchema,
	ringThickness: ringThicknessSchema,
	cornerRoundness: cornerRoundnessSchema,
	previewFitMode: previewFitModeSchema,
});

export const overlayAppearanceSettingsPatchSchema =
	overlayAppearanceSettingsSchema.partial();

export type OverlayShape = z.infer<typeof overlayShapeSchema>;
export type PreviewFitMode = z.infer<typeof previewFitModeSchema>;
export type OverlayAppearanceSettings = z.infer<
	typeof overlayAppearanceSettingsSchema
>;
export type OverlayAppearanceSettingsPatch = z.infer<
	typeof overlayAppearanceSettingsPatchSchema
>;

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
		ringThicknessSchema.maxValue ?? 10,
		Math.max(ringThicknessSchema.minValue ?? 0, Math.round(ringThickness)),
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
