import { describe, expect, it } from "vitest";
import {
	baseRendererLocale,
	rendererLocales,
} from "../../src/renderer/locales/index.js";
import { getLocaleShapeComparison } from "../../src/renderer/locales/validation.js";
import {
	type SupportedLanguage,
	supportedLanguages,
} from "../../src/shared/language.js";

describe("renderer locale registry", () => {
	it("stays aligned with the supported language list", () => {
		expect(Object.keys(rendererLocales)).toEqual(supportedLanguages);
	});

	it("keeps every locale structurally aligned with English", () => {
		for (const language of supportedLanguages) {
			const comparison = getLocaleShapeComparison(
				baseRendererLocale,
				rendererLocales[language],
			);

			expect(comparison).toEqual({
				missingKeys: [],
				extraKeys: [],
				typeMismatches: [],
			});
		}
	});

	it("keeps language option labels complete in every locale", () => {
		for (const language of supportedLanguages) {
			const optionKeys = Object.keys(
				rendererLocales[language].language.options,
			) as Array<"system" | SupportedLanguage>;

			expect(optionKeys).toEqual(["system", ...supportedLanguages]);
		}
	});
});
