import { describe, expect, it } from "vitest";
import {
	getOverlayAppearanceModel,
	getOverlayShapeClassName,
} from "../../src/renderer/features/overlay-shell/appearance.js";
import {
	defaultOverlayAppearanceSettings,
	getRoundedSquareRadius,
	isDefaultOverlayAppearanceSettings,
	isOverlayAppearanceSettingsEqual,
} from "../../src/shared/appearance.js";

describe("overlay appearance helpers", () => {
	it("maps shape names to lens classes", () => {
		expect(getOverlayShapeClassName("circle")).toBe(
			"overlay-shell__lens--circle",
		);
		expect(getOverlayShapeClassName("rounded-square")).toBe(
			"overlay-shell__lens--rounded-square",
		);
		expect(getOverlayShapeClassName("diamond")).toBe(
			"overlay-shell__lens--diamond",
		);
		expect(getOverlayShapeClassName("rectangle")).toBe(
			"overlay-shell__lens--rectangle",
		);
	});

	it("computes a bounded rounded-square radius from overlay size", () => {
		expect(getRoundedSquareRadius(96)).toBe(26);
		expect(getRoundedSquareRadius(320, 64)).toBe(64);
	});

	it("returns css variables for the current appearance", () => {
		expect(
			getOverlayAppearanceModel({
				...defaultOverlayAppearanceSettings,
				overlayShape: "rectangle",
				overlaySize: 300,
				ringColor: "#FFAA00",
				ringAccentColor: "#FFF0AA",
				ringThickness: 10,
				cornerRoundness: 32,
				previewFitMode: "contain",
			}),
		).toEqual({
			lensClassName: "overlay-shell__lens--rectangle",
			cssVariables: {
				"--camlet-overlay-size": "300px",
				"--camlet-ring-color": "#FFAA00",
				"--camlet-ring-accent-color": "#FFF0AA",
				"--camlet-ring-gradient": "linear-gradient(145deg, #FFAA00, #FFF0AA)",
				"--camlet-ring-glow": "rgba(255, 170, 0, 0.32)",
				"--camlet-surface-tint": "rgba(255, 240, 170, 0.16)",
				"--camlet-surface-border": "rgba(255, 170, 0, 0.44)",
				"--camlet-ring-thickness": "10px",
				"--camlet-preview-fit": "contain",
				"--camlet-overlay-radius": "32px",
				"--camlet-overlay-clip-path": "inset(0 16% round 32px)",
				"--camlet-inner-clip-path": "inset(0 16% round 22px)",
				"--camlet-overlay-fill":
					"radial-gradient(circle at 18% 18%, rgba(255, 240, 170, 0.3), transparent 34%), linear-gradient(180deg, rgba(6, 10, 16, 0.94), rgba(6, 10, 16, 0.82))",
			},
		});
	});

	it("detects default and non-default appearance values", () => {
		expect(
			isOverlayAppearanceSettingsEqual(
				defaultOverlayAppearanceSettings,
				defaultOverlayAppearanceSettings,
			),
		).toBe(true);
		expect(
			isDefaultOverlayAppearanceSettings(defaultOverlayAppearanceSettings),
		).toBe(true);
		expect(
			isDefaultOverlayAppearanceSettings({
				...defaultOverlayAppearanceSettings,
				ringThickness: defaultOverlayAppearanceSettings.ringThickness + 1,
			}),
		).toBe(false);
	});
});
