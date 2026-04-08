import { describe, expect, it } from "vitest";
import {
	getOverlayAppearanceModel,
	getOverlayShapeClassName,
} from "../../src/renderer/features/overlay-shell/appearance.js";
import {
	defaultOverlayAppearanceSettings,
	getEffectiveRingThickness,
	getRoundedSquareRadius,
	isDefaultOverlayAppearanceSettings,
	isOverlayAppearanceSettingsEqual,
	normalizeRingThickness,
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
		expect(getOverlayShapeClassName("rectangle-y")).toBe(
			"overlay-shell__lens--rectangle-y",
		);
		expect(getOverlayShapeClassName("rectangle-x")).toBe(
			"overlay-shell__lens--rectangle-x",
		);
	});

	it("computes a bounded rounded-square radius from overlay size", () => {
		expect(getRoundedSquareRadius(96)).toBe(26);
		expect(getRoundedSquareRadius(320, 64)).toBe(64);
	});

	it("normalizes ring thickness labels to the new visual scale", () => {
		expect(normalizeRingThickness(-1)).toBe(0);
		expect(normalizeRingThickness(1)).toBe(0);
		expect(normalizeRingThickness(5)).toBe(4);
		expect(normalizeRingThickness(7)).toBe(6);
		expect(normalizeRingThickness(14)).toBe(10);
		expect(getEffectiveRingThickness(224, 0)).toBe(0);
		expect(getEffectiveRingThickness(224, 6)).toBe(6);
		expect(getEffectiveRingThickness(224, 10)).toBe(10);
	});

	it("returns css variables for the current appearance", () => {
		const input = {
			...defaultOverlayAppearanceSettings,
			overlayShape: "rectangle-y" as const,
			overlaySize: 300,
			ringColor: "#FFAA00",
			ringAccentColor: "#FFF0AA",
			ringThickness: 10,
			cornerRoundness: 32,
			previewFitMode: "contain" as const,
		};
		const effectiveRingThickness = getEffectiveRingThickness(
			input.overlaySize,
			input.ringThickness,
		);

		expect(getOverlayAppearanceModel(input)).toEqual({
			appearance: input,
			lensClassName: "overlay-shell__lens--rectangle-y",
			cssVariables: {
				"--camlet-overlay-size": "300px",
				"--camlet-ring-color": "#FFAA00",
				"--camlet-ring-accent-color": "#FFF0AA",
				"--camlet-ring-gradient": "linear-gradient(145deg, #FFAA00, #FFF0AA)",
				"--camlet-ring-glow": "rgba(255, 170, 0, 0.32)",
				"--camlet-surface-tint": "rgba(255, 240, 170, 0.16)",
				"--camlet-surface-border": "rgba(255, 170, 0, 0.44)",
				"--camlet-ring-thickness": `${effectiveRingThickness}px`,
				"--camlet-preview-fit": "contain",
				"--camlet-overlay-radius": "32px",
				"--camlet-overlay-clip-path": "inset(0 48px round 32px)",
				"--camlet-inner-clip-path": `inset(0 48px round ${Math.max(0, 32 - effectiveRingThickness)}px)`,
				"--camlet-overlay-fill":
					"radial-gradient(circle at 18% 18%, rgba(255, 240, 170, 0.3), transparent 34%), linear-gradient(180deg, rgba(6, 10, 16, 0.94), rgba(6, 10, 16, 0.82))",
			},
		});
	});

	it("builds a landscape rectangle clip path", () => {
		const model = getOverlayAppearanceModel({
			...defaultOverlayAppearanceSettings,
			overlayShape: "rectangle-x",
			overlaySize: 300,
			cornerRoundness: 32,
		});

		expect(model.cssVariables["--camlet-overlay-clip-path"]).toBe(
			"inset(48px 0 round 32px)",
		);
	});

	it("builds a rounded clip path for the diamond shape", () => {
		const model = getOverlayAppearanceModel({
			...defaultOverlayAppearanceSettings,
			overlayShape: "diamond",
			overlaySize: 224,
			cornerRoundness: 36,
		});

		expect(model.cssVariables["--camlet-overlay-clip-path"]).toContain(
			'path("M',
		);
		expect(model.cssVariables["--camlet-inner-clip-path"]).toContain('path("M');
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
