import { supportedLanguageSchema } from "./language.js";
import { isPrereleaseVersion } from "./release.js";
import type { CamletSettings } from "./settings.js";
import { camletSettingsSchema } from "./settings.js";
import {
	arraySchema,
	booleanSchema,
	enumSchema,
	objectSchema,
	stringSchema,
} from "./validation.js";
import type { WindowState } from "./window-state.js";
import { windowStateSchema } from "./window-state.js";

const appReleaseChannelValues = ["stable", "prerelease"] as const;
const appRuntimeModeValues = ["development", "production"] as const;
const appDisplayProtocolValues = [
	"wayland",
	"x11",
	"windows",
	"macos",
	"unknown",
] as const;
const appBootstrapIssueValues = [
	"settings-recovered",
	"settings-persistence-unavailable",
] as const;

export type AppReleaseChannel = (typeof appReleaseChannelValues)[number];
export type AppRuntimeMode = (typeof appRuntimeModeValues)[number];
export type AppDisplayProtocol = (typeof appDisplayProtocolValues)[number];
export type AppBootstrapIssue = (typeof appBootstrapIssueValues)[number];

export interface AppInfo {
	name: string;
	version: string;
	platform: string;
	arch: string;
	channel: AppReleaseChannel;
	mode: AppRuntimeMode;
	packaged: boolean;
	displayProtocol: AppDisplayProtocol;
	versions: {
		electron: string;
		chrome: string;
	};
}

export interface AppBootstrap {
	app: AppInfo;
	locale: {
		system: string;
		effective: import("./language.js").SupportedLanguage;
	};
	settings: CamletSettings;
	windowState: WindowState;
	issues: AppBootstrapIssue[];
}

export const appReleaseChannelSchema = enumSchema(appReleaseChannelValues);
export const appRuntimeModeSchema = enumSchema(appRuntimeModeValues);
export const appDisplayProtocolSchema = enumSchema(appDisplayProtocolValues);
export const appBootstrapIssueSchema = enumSchema(appBootstrapIssueValues);

export const appInfoSchema = objectSchema({
	name: stringSchema({ trim: true, minLength: 1 }),
	version: stringSchema({ trim: true, minLength: 1 }),
	platform: stringSchema({ trim: true, minLength: 1 }),
	arch: stringSchema({ trim: true, minLength: 1 }),
	channel: appReleaseChannelSchema,
	mode: appRuntimeModeSchema,
	packaged: booleanSchema(),
	displayProtocol: appDisplayProtocolSchema,
	versions: objectSchema({
		electron: stringSchema({ trim: true, minLength: 1 }),
		chrome: stringSchema({ trim: true, minLength: 1 }),
	}),
});

export const appBootstrapSchema = objectSchema({
	app: appInfoSchema,
	locale: objectSchema({
		system: stringSchema({ trim: true, minLength: 1 }),
		effective: supportedLanguageSchema,
	}),
	settings: camletSettingsSchema,
	windowState: windowStateSchema,
	issues: arraySchema(appBootstrapIssueSchema),
});

export function resolveAppReleaseChannel(version: string): AppReleaseChannel {
	return isPrereleaseVersion(version) ? "prerelease" : "stable";
}

export function dedupeAppBootstrapIssues(
	issues: Iterable<AppBootstrapIssue>,
): AppBootstrapIssue[] {
	return [...new Set(issues)];
}
