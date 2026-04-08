import { type AppBootstrap, appBootstrapSchema } from "../shared/bootstrap.js";
import {
	resolveSupportedLanguage,
	type SupportedLanguage,
} from "../shared/language.js";

export type RendererStartupErrorCode =
	| "preload-unavailable"
	| "bootstrap-invalid"
	| "bootstrap-load-failed";

export interface RendererStartupError {
	code: RendererStartupErrorCode;
	detail: string | null;
}

export interface RendererStartupState {
	bootstrap: AppBootstrap | null;
	language: SupportedLanguage;
	startupError: RendererStartupError | null;
}

function getErrorDetail(error: unknown): string | null {
	if (error instanceof Error && error.message.trim().length > 0) {
		return error.message;
	}

	return null;
}

function hasBootstrapApi(
	value: unknown,
): value is { getBootstrap: () => Promise<unknown> } {
	return (
		typeof value === "object" &&
		value !== null &&
		"getBootstrap" in value &&
		typeof value.getBootstrap === "function"
	);
}

function getBootstrapValidationDetail(bootstrap: unknown): string {
	const result = appBootstrapSchema.safeParse(bootstrap);

	if (result.success) {
		return "";
	}

	return result.error.issues
		.map((issue) => {
			const path = issue.path.join(".");
			return path.length > 0 ? `${path}: ${issue.message}` : issue.message;
		})
		.join("; ");
}

export async function resolveRendererStartupState(options: {
	camletApi?: {
		getBootstrap?: (() => Promise<unknown>) | undefined;
	} | null;
	preferredLanguage?: string | null;
}): Promise<RendererStartupState> {
	const fallbackLanguage = resolveSupportedLanguage(options.preferredLanguage);

	if (!hasBootstrapApi(options.camletApi)) {
		return {
			bootstrap: null,
			language: fallbackLanguage,
			startupError: {
				code: "preload-unavailable",
				detail: null,
			},
		};
	}

	try {
		const bootstrap = await options.camletApi.getBootstrap();
		const result = appBootstrapSchema.safeParse(bootstrap);

		if (!result.success) {
			return {
				bootstrap: null,
				language: fallbackLanguage,
				startupError: {
					code: "bootstrap-invalid",
					detail: getBootstrapValidationDetail(bootstrap),
				},
			};
		}

		return {
			bootstrap: result.data,
			language: result.data.locale.effective,
			startupError: null,
		};
	} catch (error) {
		return {
			bootstrap: null,
			language: fallbackLanguage,
			startupError: {
				code: "bootstrap-load-failed",
				detail: getErrorDetail(error),
			},
		};
	}
}
