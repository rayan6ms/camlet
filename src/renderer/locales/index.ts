import type { SupportedLanguage } from "../../shared/language.js";
import { enLocale } from "./en.js";
import { ptBrLocale } from "./pt-BR.js";
import type { RendererLocale } from "./schema.js";

export const rendererLocales = {
	en: enLocale,
	"pt-BR": ptBrLocale,
} as const satisfies Record<SupportedLanguage, RendererLocale>;

export const baseRendererLocale = rendererLocales.en;
