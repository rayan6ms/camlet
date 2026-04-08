import i18next from "i18next";
import {
	fallbackLanguage,
	resolveSupportedLanguage,
	type SupportedLanguage,
} from "../shared/language.js";
import { rendererLocales } from "./locales/index.js";

const resources = Object.fromEntries(
	Object.entries(rendererLocales).map(([language, translation]) => [
		language,
		{ translation },
	]),
) as Record<
	SupportedLanguage,
	{ translation: (typeof rendererLocales)[SupportedLanguage] }
>;

let initialized = false;

function getResolvedLanguage(language?: string): SupportedLanguage {
	return resolveSupportedLanguage(language);
}

export async function initializeI18n(language: SupportedLanguage) {
	if (!initialized) {
		await i18next.init({
			resources,
			lng: language,
			fallbackLng: fallbackLanguage,
			interpolation: {
				escapeValue: false,
			},
		});

		initialized = true;
		return;
	}

	await i18next.changeLanguage(language);
}

export function subscribeToLanguageChange(onChange: () => void) {
	i18next.on("languageChanged", onChange);
	return () => i18next.off("languageChanged", onChange);
}

export function getCurrentLanguage(): SupportedLanguage {
	return getResolvedLanguage(i18next.resolvedLanguage ?? i18next.language);
}

export function t(key: string, options?: Record<string, unknown>): string {
	if (options === undefined) {
		return String(i18next.t(key));
	}

	return String(i18next.t(key, options as never));
}
