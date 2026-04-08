import type {
	OverlayAppearanceSettings,
	OverlayShape,
} from "../../../shared/appearance.js";
import { getRoundedSquareRadius } from "../../../shared/appearance.js";

export interface OverlayAppearanceModel {
	lensClassName: string;
	cssVariables: Record<string, string>;
}

const rectangleInsetPercent = 16;

function hexToRgb(hexColor: string) {
	const value = hexColor.replace("#", "");
	const normalized =
		value.length === 8 ? value.slice(0, 6) : value.padEnd(6, "0");

	return {
		r: Number.parseInt(normalized.slice(0, 2), 16),
		g: Number.parseInt(normalized.slice(2, 4), 16),
		b: Number.parseInt(normalized.slice(4, 6), 16),
	};
}

export function getOverlayShapeClassName(shape: OverlayShape): string {
	switch (shape) {
		case "circle":
			return "overlay-shell__lens--circle";
		case "rounded-square":
			return "overlay-shell__lens--rounded-square";
		case "diamond":
			return "overlay-shell__lens--diamond";
		case "rectangle":
			return "overlay-shell__lens--rectangle";
	}
}

function getOverlayRadiusValue(settings: OverlayAppearanceSettings): string {
	if (
		settings.overlayShape === "circle" ||
		settings.overlayShape === "diamond"
	) {
		return "999px";
	}

	return `${getRoundedSquareRadius(
		settings.overlaySize,
		settings.cornerRoundness,
	)}px`;
}

function getOuterClipPath(settings: OverlayAppearanceSettings, radius: string) {
	switch (settings.overlayShape) {
		case "circle":
			return "circle(50% at 50% 50%)";
		case "diamond":
			return "polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%)";
		case "rectangle":
			return `inset(0 ${rectangleInsetPercent}% round ${radius})`;
		case "rounded-square":
			return `inset(0 round ${radius})`;
	}
}

function getInnerClipPath(settings: OverlayAppearanceSettings, radius: string) {
	switch (settings.overlayShape) {
		case "circle":
			return "circle(50% at 50% 50%)";
		case "diamond":
			return "polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%)";
		case "rectangle":
			return `inset(0 ${rectangleInsetPercent}% round ${radius})`;
		case "rounded-square":
			return `inset(0 round ${radius})`;
	}
}

export function getOverlayAppearanceModel(
	settings: OverlayAppearanceSettings,
): OverlayAppearanceModel {
	const ringRgb = hexToRgb(settings.ringColor);
	const accentRgb = hexToRgb(settings.ringAccentColor);
	const overlayRadius = getOverlayRadiusValue(settings);
	const innerRoundedSquareRadius = Math.max(
		0,
		getRoundedSquareRadius(settings.overlaySize, settings.cornerRoundness) -
			settings.ringThickness,
	);
	const innerRadius = `${innerRoundedSquareRadius}px`;
	const overlayClipPath = getOuterClipPath(settings, overlayRadius);
	const innerClipPath = getInnerClipPath(settings, innerRadius);

	return {
		lensClassName: getOverlayShapeClassName(settings.overlayShape),
		cssVariables: {
			"--camlet-overlay-size": `${settings.overlaySize}px`,
			"--camlet-ring-color": settings.ringColor,
			"--camlet-ring-accent-color": settings.ringAccentColor,
			"--camlet-ring-gradient": `linear-gradient(145deg, ${settings.ringColor}, ${settings.ringAccentColor})`,
			"--camlet-ring-glow": `rgba(${ringRgb.r}, ${ringRgb.g}, ${ringRgb.b}, 0.32)`,
			"--camlet-surface-tint": `rgba(${accentRgb.r}, ${accentRgb.g}, ${accentRgb.b}, 0.16)`,
			"--camlet-surface-border": `rgba(${ringRgb.r}, ${ringRgb.g}, ${ringRgb.b}, 0.44)`,
			"--camlet-ring-thickness": `${settings.ringThickness}px`,
			"--camlet-preview-fit": settings.previewFitMode,
			"--camlet-overlay-radius": overlayRadius,
			"--camlet-overlay-clip-path": overlayClipPath,
			"--camlet-inner-clip-path": innerClipPath,
			"--camlet-overlay-fill": `radial-gradient(circle at 18% 18%, rgba(${accentRgb.r}, ${accentRgb.g}, ${accentRgb.b}, 0.3), transparent 34%), linear-gradient(180deg, rgba(6, 10, 16, 0.94), rgba(6, 10, 16, 0.82))`,
		},
	};
}
