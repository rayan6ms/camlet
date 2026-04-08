import { enumSchema } from "./validation.js";

export const supportedLanguages = ["en", "pt-BR"] as const;
export const appLanguages = ["system", ...supportedLanguages] as const;

export type SupportedLanguage = (typeof supportedLanguages)[number];
export type AppLanguage = (typeof appLanguages)[number];

export const supportedLanguageSchema = enumSchema(supportedLanguages);
export const appLanguageSchema = enumSchema(appLanguages);

export const fallbackLanguage: SupportedLanguage = "en";
export const defaultAppLanguage: AppLanguage = "system";

const systemLocaleMappings = [
	{
		language: "pt-BR",
		prefixes: ["pt"],
	},
	{
		language: "en",
		prefixes: ["en"],
	},
] as const satisfies ReadonlyArray<{
	language: SupportedLanguage;
	prefixes: readonly string[];
}>;

export const selectableAppLanguages = appLanguages;

export function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null;
}

function normalizeLanguageTag(value?: string | null): string | null {
	if (typeof value !== "string") {
		return null;
	}

	const normalized = value.trim().replaceAll("_", "-").toLowerCase();
	return normalized.length > 0 ? normalized : null;
}

export function resolveSupportedLanguage(
	value?: string | null,
): SupportedLanguage {
	const normalized = normalizeLanguageTag(value);

	if (normalized === null) {
		return fallbackLanguage;
	}

	for (const mapping of systemLocaleMappings) {
		if (
			mapping.prefixes.some(
				(prefix) =>
					normalized === prefix || normalized.startsWith(`${prefix}-`),
			)
		) {
			return mapping.language;
		}
	}

	return fallbackLanguage;
}

export function resolveAppLanguage(
	language: AppLanguage,
	systemLocale?: string | null,
): SupportedLanguage {
	if (language === "system") {
		return resolveSupportedLanguage(systemLocale);
	}

	return language;
}
