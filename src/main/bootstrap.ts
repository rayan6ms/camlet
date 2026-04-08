import {
	type AppBootstrap,
	type AppBootstrapIssue,
	type AppDisplayProtocol,
	type AppRuntimeMode,
	dedupeAppBootstrapIssues,
	resolveAppReleaseChannel,
} from "../shared/bootstrap.js";
import { fallbackLanguage, resolveAppLanguage } from "../shared/language.js";
import type { CamletSettings } from "../shared/settings.js";

export interface AppRuntimeInfo {
	name?: string | null;
	version?: string | null;
	platform?: string | null;
	arch?: string | null;
	mode?: AppRuntimeMode | null;
	packaged?: boolean | null;
	displayProtocol?: AppDisplayProtocol | null;
	electronVersion?: string | null;
	chromeVersion?: string | null;
}

function normalizeRuntimeValue(
	value: string | null | undefined,
	fallback: string,
) {
	return typeof value === "string" && value.trim().length > 0
		? value.trim()
		: fallback;
}

export function resolveDisplayProtocol(options?: {
	platform?: string | null | undefined;
	sessionType?: string | null | undefined;
	waylandDisplay?: string | null | undefined;
	display?: string | null | undefined;
	ozonePlatform?: string | null | undefined;
}): AppDisplayProtocol {
	const platform = options?.platform?.trim() || process.platform;

	if (platform === "win32") {
		return "windows";
	}

	if (platform === "darwin") {
		return "macos";
	}

	if (platform !== "linux") {
		return "unknown";
	}

	const ozonePlatform = options?.ozonePlatform?.trim().toLowerCase();

	if (ozonePlatform === "x11") {
		return "x11";
	}

	if (ozonePlatform === "wayland") {
		return "wayland";
	}

	const sessionType = options?.sessionType?.trim().toLowerCase();

	if (sessionType === "wayland") {
		return "wayland";
	}

	if (sessionType === "x11" || sessionType === "xorg") {
		return "x11";
	}

	if (options?.waylandDisplay?.trim()) {
		return "wayland";
	}

	if (options?.display?.trim()) {
		return "x11";
	}

	return "unknown";
}

export function normalizeSystemLocale(locale?: string | null): string {
	return typeof locale === "string" && locale.trim().length > 0
		? locale
		: fallbackLanguage;
}

export function createAppBootstrap(options: {
	app: AppRuntimeInfo;
	settings: CamletSettings;
	systemLocale?: string | null;
	issues?: Iterable<AppBootstrapIssue>;
}): AppBootstrap {
	const systemLocale = normalizeSystemLocale(options.systemLocale);
	const version = options.app.version?.trim() || "0.0.0";
	const platform = normalizeRuntimeValue(
		options.app.platform,
		process.platform,
	);
	const mode = options.app.mode ?? "production";

	return {
		app: {
			name: normalizeRuntimeValue(options.app.name, "Camlet"),
			version,
			platform,
			arch: normalizeRuntimeValue(options.app.arch, process.arch),
			channel: resolveAppReleaseChannel(version),
			mode,
			packaged: options.app.packaged ?? mode === "production",
			displayProtocol:
				options.app.displayProtocol ??
				resolveDisplayProtocol({
					platform,
				}),
			versions: {
				electron: normalizeRuntimeValue(
					options.app.electronVersion,
					process.versions.electron,
				),
				chrome: normalizeRuntimeValue(
					options.app.chromeVersion,
					process.versions.chrome,
				),
			},
		},
		locale: {
			system: systemLocale,
			effective: resolveAppLanguage(options.settings.language, systemLocale),
		},
		settings: options.settings,
		windowState: options.settings.window,
		issues: dedupeAppBootstrapIssues(options.issues ?? []),
	};
}
