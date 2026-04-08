import type { RendererLocale } from "./schema.js";

interface LocaleShapeComparison {
	missingKeys: string[];
	extraKeys: string[];
	typeMismatches: string[];
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}

function compareLocaleShape(
	baseValue: unknown,
	localeValue: unknown,
	path: string,
	result: LocaleShapeComparison,
) {
	if (typeof baseValue === "string") {
		if (typeof localeValue !== "string") {
			result.typeMismatches.push(path);
		}
		return;
	}

	if (!isRecord(baseValue)) {
		return;
	}

	if (!isRecord(localeValue)) {
		result.typeMismatches.push(path);
		return;
	}

	for (const [key, nestedBaseValue] of Object.entries(baseValue)) {
		const nextPath = path.length > 0 ? `${path}.${key}` : key;

		if (!(key in localeValue)) {
			result.missingKeys.push(nextPath);
			continue;
		}

		compareLocaleShape(nestedBaseValue, localeValue[key], nextPath, result);
	}

	for (const key of Object.keys(localeValue)) {
		if (!(key in baseValue)) {
			const nextPath = path.length > 0 ? `${path}.${key}` : key;
			result.extraKeys.push(nextPath);
		}
	}
}

export function getLocaleShapeComparison(
	baseLocale: RendererLocale,
	locale: RendererLocale,
): LocaleShapeComparison {
	const result: LocaleShapeComparison = {
		missingKeys: [],
		extraKeys: [],
		typeMismatches: [],
	};

	compareLocaleShape(baseLocale, locale, "", result);
	return result;
}
