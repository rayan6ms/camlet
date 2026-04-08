import type {
	OverlayAppearanceSettings,
	OverlayShape,
} from "../../../shared/appearance.js";
import { getRoundedSquareRadius } from "../../../shared/appearance.js";

export interface OverlayAppearanceModel {
	lensClassName: string;
	cssVariables: Record<string, string>;
}

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
	return shape === "circle"
		? "overlay-shell__lens--circle"
		: "overlay-shell__lens--rounded-square";
}

export function getOverlayAppearanceModel(
	settings: OverlayAppearanceSettings,
): OverlayAppearanceModel {
	const ringRgb = hexToRgb(settings.ringColor);
	const overlayRadius =
		settings.overlayShape === "circle"
			? "999px"
			: `${getRoundedSquareRadius(settings.overlaySize)}px`;
	const overlayClipPath =
		settings.overlayShape === "circle"
			? "circle(50% at 50% 50%)"
			: `inset(0 round ${overlayRadius})`;
	const innerRoundedSquareRadius = Math.max(
		0,
		getRoundedSquareRadius(settings.overlaySize) - settings.ringThickness,
	);
	const innerClipPath =
		settings.overlayShape === "circle"
			? "circle(50% at 50% 50%)"
			: `inset(0 round ${innerRoundedSquareRadius}px)`;

	return {
		lensClassName: getOverlayShapeClassName(settings.overlayShape),
		cssVariables: {
			"--camlet-overlay-size": `${settings.overlaySize}px`,
			"--camlet-ring-color": settings.ringColor,
			"--camlet-ring-glow": `rgba(${ringRgb.r}, ${ringRgb.g}, ${ringRgb.b}, 0.32)`,
			"--camlet-surface-tint": `rgba(${ringRgb.r}, ${ringRgb.g}, ${ringRgb.b}, 0.18)`,
			"--camlet-surface-border": `rgba(${ringRgb.r}, ${ringRgb.g}, ${ringRgb.b}, 0.44)`,
			"--camlet-ring-thickness": `${settings.ringThickness}px`,
			"--camlet-preview-fit": settings.previewFitMode,
			"--camlet-overlay-radius": overlayRadius,
			"--camlet-overlay-clip-path": overlayClipPath,
			"--camlet-inner-clip-path": innerClipPath,
		},
	};
}
