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
	});

	it("computes a bounded rounded-square radius from overlay size", () => {
		expect(getRoundedSquareRadius(96)).toBe(21);
		expect(getRoundedSquareRadius(320)).toBe(70);
	});

	it("returns css variables for the current appearance", () => {
		expect(
			getOverlayAppearanceModel({
				...defaultOverlayAppearanceSettings,
				overlayShape: "rounded-square",
				overlaySize: 300,
				ringColor: "#FFAA00",
				ringThickness: 10,
				previewFitMode: "contain",
			}),
		).toEqual({
			lensClassName: "overlay-shell__lens--rounded-square",
			cssVariables: {
				"--camlet-overlay-size": "300px",
				"--camlet-ring-color": "#FFAA00",
				"--camlet-ring-glow": "rgba(255, 170, 0, 0.32)",
				"--camlet-surface-tint": "rgba(255, 170, 0, 0.18)",
				"--camlet-surface-border": "rgba(255, 170, 0, 0.44)",
				"--camlet-ring-thickness": "10px",
				"--camlet-preview-fit": "contain",
				"--camlet-overlay-radius": "66px",
				"--camlet-overlay-clip-path": "inset(0 round 66px)",
				"--camlet-inner-clip-path": "inset(0 round 56px)",
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
