import type {
	OverlayAppearanceSettings,
	OverlayShape,
} from "../../../shared/appearance.js";
import {
	getEffectiveRingThickness,
	getRoundedSquareRadius,
} from "../../../shared/appearance.js";

export interface OverlayAppearanceModel {
	appearance: OverlayAppearanceSettings;
	lensClassName: string;
	cssVariables: Record<string, string>;
}

const rectangleInsetPercent = 16;

function getRectangleInsetPx(size: number): number {
	return (size * rectangleInsetPercent) / 100;
}

function getRectangleClipPath(
	shape: Extract<OverlayShape, "rectangle-y" | "rectangle-x">,
	inset: number,
	radius: string,
): string {
	return shape === "rectangle-y"
		? `inset(0 ${inset}px round ${radius})`
		: `inset(${inset}px 0 round ${radius})`;
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

function rotatePoint(x: number, y: number, angleRadians: number) {
	const cosine = Math.cos(angleRadians);
	const sine = Math.sin(angleRadians);

	return {
		x: x * cosine - y * sine,
		y: x * sine + y * cosine,
	};
}

function createRoundedDiamondPath(
	size: number,
	cornerRoundness: number,
	inset: number,
): string {
	const diagonal = Math.max(1, size - inset * 2);
	const sideLength = diagonal / Math.SQRT2;
	const halfSide = sideLength / 2;
	const radius = Math.min(Math.max(0, cornerRoundness), halfSide);
	const straightHalf = Math.max(0, halfSide - radius);
	const cornerCenters = [
		{ x: straightHalf, y: -straightHalf, start: -Math.PI / 2, end: 0 },
		{ x: straightHalf, y: straightHalf, start: 0, end: Math.PI / 2 },
		{ x: -straightHalf, y: straightHalf, start: Math.PI / 2, end: Math.PI },
		{
			x: -straightHalf,
			y: -straightHalf,
			start: Math.PI,
			end: (3 * Math.PI) / 2,
		},
	];
	const center = size / 2;
	const points = cornerCenters.flatMap((corner) =>
		Array.from({ length: 8 }, (_, index) => {
			const angle = corner.start + ((corner.end - corner.start) * index) / 7;
			const localX = corner.x + Math.cos(angle) * radius;
			const localY = corner.y + Math.sin(angle) * radius;
			const rotated = rotatePoint(localX, localY, Math.PI / 4);
			return `${(center + rotated.x).toFixed(2)} ${(center + rotated.y).toFixed(2)}`;
		}),
	);

	return `path("M ${points.join(" L ")} Z")`;
}

export function getOverlayShapeClassName(shape: OverlayShape): string {
	switch (shape) {
		case "circle":
			return "overlay-shell__lens--circle";
		case "rounded-square":
			return "overlay-shell__lens--rounded-square";
		case "diamond":
			return "overlay-shell__lens--diamond";
		case "rectangle-y":
			return "overlay-shell__lens--rectangle-y";
		case "rectangle-x":
			return "overlay-shell__lens--rectangle-x";
	}
}

function getOverlayRadiusValue(settings: OverlayAppearanceSettings): string {
	if (settings.overlayShape === "circle") {
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
			return createRoundedDiamondPath(
				settings.overlaySize,
				settings.cornerRoundness,
				0,
			);
		case "rectangle-y":
		case "rectangle-x":
			return getRectangleClipPath(
				settings.overlayShape,
				getRectangleInsetPx(settings.overlaySize),
				radius,
			);
		case "rounded-square":
			return `inset(0 round ${radius})`;
	}
}

function getInnerClipPath(settings: OverlayAppearanceSettings, radius: string) {
	const effectiveRingThickness = getEffectiveRingThickness(
		settings.overlaySize,
		settings.ringThickness,
	);
	const innerSize = Math.max(
		1,
		settings.overlaySize - effectiveRingThickness * 2,
	);

	switch (settings.overlayShape) {
		case "circle":
			return "circle(50% at 50% 50%)";
		case "diamond":
			return createRoundedDiamondPath(
				innerSize,
				Math.max(0, settings.cornerRoundness - effectiveRingThickness / 2),
				0,
			);
		case "rectangle-y":
		case "rectangle-x": {
			const outerInset = Math.min(
				innerSize / 2,
				getRectangleInsetPx(settings.overlaySize),
			);
			return getRectangleClipPath(settings.overlayShape, outerInset, radius);
		}
		case "rounded-square":
			return `inset(0 round ${radius})`;
	}
}

export function getOverlayAppearanceModel(
	settings: OverlayAppearanceSettings,
): OverlayAppearanceModel {
	const ringRgb = hexToRgb(settings.ringColor);
	const accentRgb = hexToRgb(settings.ringAccentColor);
	const effectiveRingThickness = getEffectiveRingThickness(
		settings.overlaySize,
		settings.ringThickness,
	);
	const overlayRadius = getOverlayRadiusValue(settings);
	const innerRoundedSquareRadius = Math.max(
		0,
		getRoundedSquareRadius(settings.overlaySize, settings.cornerRoundness) -
			effectiveRingThickness,
	);
	const innerRadius = `${innerRoundedSquareRadius}px`;
	const overlayClipPath = getOuterClipPath(settings, overlayRadius);
	const innerClipPath = getInnerClipPath(settings, innerRadius);

	return {
		appearance: settings,
		lensClassName: getOverlayShapeClassName(settings.overlayShape),
		cssVariables: {
			"--camlet-overlay-size": `${settings.overlaySize}px`,
			"--camlet-ring-color": settings.ringColor,
			"--camlet-ring-accent-color": settings.ringAccentColor,
			"--camlet-ring-gradient": `linear-gradient(145deg, ${settings.ringColor}, ${settings.ringAccentColor})`,
			"--camlet-ring-glow": `rgba(${ringRgb.r}, ${ringRgb.g}, ${ringRgb.b}, 0.32)`,
			"--camlet-surface-tint": `rgba(${accentRgb.r}, ${accentRgb.g}, ${accentRgb.b}, 0.16)`,
			"--camlet-surface-border": `rgba(${ringRgb.r}, ${ringRgb.g}, ${ringRgb.b}, 0.44)`,
			"--camlet-ring-thickness": `${effectiveRingThickness}px`,
			"--camlet-preview-fit": settings.previewFitMode,
			"--camlet-overlay-radius": overlayRadius,
			"--camlet-overlay-clip-path": overlayClipPath,
			"--camlet-inner-clip-path": innerClipPath,
			"--camlet-overlay-fill": `radial-gradient(circle at 18% 18%, rgba(${accentRgb.r}, ${accentRgb.g}, ${accentRgb.b}, 0.3), transparent 34%), linear-gradient(180deg, rgba(6, 10, 16, 0.94), rgba(6, 10, 16, 0.82))`,
		},
	};
}
