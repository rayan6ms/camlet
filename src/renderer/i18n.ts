import i18next from "i18next";
import {
	fallbackLanguage,
	resolveSupportedLanguage,
	type SupportedLanguage,
} from "../shared/language.js";
import { loadRendererLocale } from "./locales/load.js";

let initialized = false;

function getResolvedLanguage(language?: string): SupportedLanguage {
	return resolveSupportedLanguage(language);
}

async function ensureLocaleLoaded(language: SupportedLanguage) {
	if (i18next.hasResourceBundle(language, "translation")) {
		return;
	}

	const translation = await loadRendererLocale(language);
	i18next.addResourceBundle(language, "translation", translation, true, true);
}

async function loadInitialResources(language: SupportedLanguage) {
	const primaryTranslation = await loadRendererLocale(language);

	if (language === fallbackLanguage) {
		return {
			[language]: {
				translation: primaryTranslation,
			},
		};
	}

	return {
		[fallbackLanguage]: {
			translation: await loadRendererLocale(fallbackLanguage),
		},
		[language]: {
			translation: primaryTranslation,
		},
	};
}

export async function initializeI18n(language: SupportedLanguage) {
	if (!initialized) {
		await i18next.init({
			resources: await loadInitialResources(language),
			lng: language,
			fallbackLng: fallbackLanguage,
			interpolation: {
				escapeValue: false,
			},
		});

		initialized = true;
		return;
	}

	await ensureLocaleLoaded(fallbackLanguage);
	await ensureLocaleLoaded(language);
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
