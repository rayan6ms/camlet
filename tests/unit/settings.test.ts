import { describe, expect, it } from "vitest";
import {
	resolveAppLanguage,
	resolveSupportedLanguage,
	selectableAppLanguages,
} from "../../src/shared/language.js";
import {
	applyCamletSettingsPatch,
	camletSettingsSchema,
	defaultCamletSettings,
	mergeCamletSettings,
} from "../../src/shared/settings.js";

describe("Camlet settings schema", () => {
	it("accepts the default settings object", () => {
		expect(camletSettingsSchema.parse(defaultCamletSettings)).toEqual(
			defaultCamletSettings,
		);
	});

	it("migrates legacy appearance and device data", () => {
		const mergedSettings = mergeCamletSettings({
			selectedCameraDevice: "legacy-camera-id",
			shape: "ring",
			color: "#FF5500",
			accentColor: "#FFD080",
			size: 320,
			borderWidth: 14,
			radius: 18,
			position: {
				x: 160,
				y: 72,
			},
			window: {
				width: 420,
			},
		});

		expect(mergedSettings.selectedCameraDeviceId).toBe("legacy-camera-id");
		expect(mergedSettings.overlayShape).toBe("circle");
		expect(mergedSettings.ringColor).toBe("#FF5500");
		expect(mergedSettings.ringAccentColor).toBe("#FFD080");
		expect(mergedSettings.overlaySize).toBe(320);
		expect(mergedSettings.ringThickness).toBe(14);
		expect(mergedSettings.cornerRoundness).toBe(18);
		expect(mergedSettings.window).toEqual({
			x: 160,
			y: 72,
			width: 420,
			height: defaultCamletSettings.window.height,
		});
	});

	it("falls back to defaults when persisted appearance values are invalid", () => {
		const mergedSettings = mergeCamletSettings({
			ringColor: "green",
			ringAccentColor: "pink",
			ringThickness: 100,
			cornerRoundness: 400,
			overlaySize: 10,
			previewFitMode: "stretch",
		});

		expect(mergedSettings.ringColor).toBe(defaultCamletSettings.ringColor);
		expect(mergedSettings.ringAccentColor).toBe(
			defaultCamletSettings.ringAccentColor,
		);
		expect(mergedSettings.ringThickness).toBe(
			defaultCamletSettings.ringThickness,
		);
		expect(mergedSettings.cornerRoundness).toBe(
			defaultCamletSettings.cornerRoundness,
		);
		expect(mergedSettings.overlaySize).toBe(defaultCamletSettings.overlaySize);
		expect(mergedSettings.previewFitMode).toBe(
			defaultCamletSettings.previewFitMode,
		);
	});

	it("applies typed patches without dropping nested window state", () => {
		const nextSettings = applyCamletSettingsPatch(defaultCamletSettings, {
			overlaySize: 320,
			cornerRoundness: 42,
			previewFitMode: "contain",
			window: {
				height: 288,
			},
		});

		expect(nextSettings.overlaySize).toBe(320);
		expect(nextSettings.cornerRoundness).toBe(42);
		expect(nextSettings.previewFitMode).toBe("contain");
		expect(nextSettings.window).toEqual({
			...defaultCamletSettings.window,
			height: 288,
		});
	});

	it("preserves supported persisted language values", () => {
		const mergedSettings = mergeCamletSettings({
			language: "ja",
		});

		expect(mergedSettings.language).toBe("ja");
	});

	it("falls back to the default language mode when persisted language is invalid", () => {
		const mergedSettings = mergeCamletSettings({
			language: "de-DE",
		});

		expect(mergedSettings.language).toBe(defaultCamletSettings.language);
	});
});

describe("language helpers", () => {
	it("maps supported system locales to the configured app locales", () => {
		expect(resolveSupportedLanguage("pt-PT")).toBe("pt-BR");
		expect(resolveSupportedLanguage("pt-BR")).toBe("pt-BR");
		expect(resolveSupportedLanguage("en-US")).toBe("en");
		expect(resolveSupportedLanguage("es-MX")).toBe("es");
		expect(resolveSupportedLanguage("fr-CA")).toBe("fr");
		expect(resolveSupportedLanguage("de-DE")).toBe("de");
		expect(resolveSupportedLanguage("it-IT")).toBe("it");
		expect(resolveSupportedLanguage("ja-JP")).toBe("ja");
	});

	it("falls back to English for unsupported locales", () => {
		expect(resolveAppLanguage("system", "nl-NL")).toBe("en");
	});

	it("keeps the language selector order stable", () => {
		expect(selectableAppLanguages).toEqual([
			"system",
			"en",
			"pt-BR",
			"es",
			"fr",
			"de",
			"it",
			"ja",
		]);
	});
});
