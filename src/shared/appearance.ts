import { z } from "zod";

const hexColorPattern = /^#(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;

export const overlayShapeSchema = z.enum([
	"circle",
	"rounded-square",
	"diamond",
	"rectangle",
]);
export const previewFitModeSchema = z.enum(["cover", "contain"]);
export const overlaySizeSchema = z.number().int().min(96).max(640);
export const ringColorSchema = z.string().regex(hexColorPattern);
export const ringAccentColorSchema = z.string().regex(hexColorPattern);
export const ringThicknessSchema = z.number().int().min(2).max(48);
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
	ringThickness: 8,
	cornerRoundness: 26,
	previewFitMode: "cover",
};

export function getRoundedSquareRadius(
	size: number,
	cornerRoundness = defaultOverlayAppearanceSettings.cornerRoundness,
): number {
	return Math.min(Math.max(0, cornerRoundness), Math.round(size / 2));
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
