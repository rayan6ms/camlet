import type { SupportedLanguage } from "../../shared/language.js";
import { deLocale } from "./de.js";
import { enLocale } from "./en.js";
import { esLocale } from "./es.js";
import { frLocale } from "./fr.js";
import { itLocale } from "./it.js";
import { jaLocale } from "./ja.js";
import { ptBrLocale } from "./pt-BR.js";
import type { RendererLocale } from "./schema.js";

export const rendererLocales = {
	en: enLocale,
	"pt-BR": ptBrLocale,
	es: esLocale,
	fr: frLocale,
	de: deLocale,
	it: itLocale,
	ja: jaLocale,
} as const satisfies Record<SupportedLanguage, RendererLocale>;

export const baseRendererLocale = rendererLocales.en;
