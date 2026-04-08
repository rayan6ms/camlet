import type { SupportedLanguage } from "../../shared/language.js";
import type { RendererLocale } from "./schema.js";

export const rendererLocaleLoaders = {
	en: async () => (await import("./en.js")).enLocale,
	"pt-BR": async () => (await import("./pt-BR.js")).ptBrLocale,
} as const satisfies Record<SupportedLanguage, () => Promise<RendererLocale>>;

export async function loadRendererLocale(
	language: SupportedLanguage,
): Promise<RendererLocale> {
	return rendererLocaleLoaders[language]();
}
