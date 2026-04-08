import { z } from "zod";

export const supportedLanguages = [
	"en",
	"pt-BR",
	"es",
	"fr",
	"de",
	"it",
	"ja",
] as const;
export const appLanguages = ["system", ...supportedLanguages] as const;

export const supportedLanguageSchema = z.enum(supportedLanguages);
export const appLanguageSchema = z.enum(appLanguages);

export type SupportedLanguage = z.infer<typeof supportedLanguageSchema>;
export type AppLanguage = z.infer<typeof appLanguageSchema>;

export const fallbackLanguage: SupportedLanguage = "en";
export const defaultAppLanguage: AppLanguage = "system";

const systemLocaleMappings = [
	{
		language: "pt-BR",
		prefixes: ["pt"],
	},
	{
		language: "es",
		prefixes: ["es"],
	},
	{
		language: "fr",
		prefixes: ["fr"],
	},
	{
		language: "de",
		prefixes: ["de"],
	},
	{
		language: "it",
		prefixes: ["it"],
	},
	{
		language: "ja",
		prefixes: ["ja"],
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
