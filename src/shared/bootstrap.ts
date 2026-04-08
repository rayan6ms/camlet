import { z } from "zod";
import { supportedLanguageSchema } from "./language.js";
import { isPrereleaseVersion } from "./release.js";
import { camletSettingsSchema } from "./settings.js";
import { windowStateSchema } from "./window-state.js";

export const appReleaseChannelSchema = z.enum(["stable", "prerelease"]);
export const appRuntimeModeSchema = z.enum(["development", "production"]);
export const appDisplayProtocolSchema = z.enum([
	"wayland",
	"x11",
	"windows",
	"macos",
	"unknown",
]);
export const appBootstrapIssueSchema = z.enum([
	"settings-recovered",
	"settings-persistence-unavailable",
]);

export const appInfoSchema = z.object({
	name: z.string().trim().min(1),
	version: z.string().trim().min(1),
	platform: z.string().trim().min(1),
	arch: z.string().trim().min(1),
	channel: appReleaseChannelSchema,
	mode: appRuntimeModeSchema,
	packaged: z.boolean(),
	displayProtocol: appDisplayProtocolSchema,
	versions: z.object({
		electron: z.string().trim().min(1),
		chrome: z.string().trim().min(1),
	}),
});

export const appBootstrapSchema = z.object({
	app: appInfoSchema,
	locale: z.object({
		system: z.string().trim().min(1),
		effective: supportedLanguageSchema,
	}),
	settings: camletSettingsSchema,
	windowState: windowStateSchema,
	issues: z.array(appBootstrapIssueSchema),
});

export type AppReleaseChannel = z.infer<typeof appReleaseChannelSchema>;
export type AppRuntimeMode = z.infer<typeof appRuntimeModeSchema>;
export type AppDisplayProtocol = z.infer<typeof appDisplayProtocolSchema>;
export type AppBootstrapIssue = z.infer<typeof appBootstrapIssueSchema>;
export type AppInfo = z.infer<typeof appInfoSchema>;
export type AppBootstrap = z.infer<typeof appBootstrapSchema>;

export function resolveAppReleaseChannel(version: string): AppReleaseChannel {
	return isPrereleaseVersion(version) ? "prerelease" : "stable";
}

export function dedupeAppBootstrapIssues(
	issues: Iterable<AppBootstrapIssue>,
): AppBootstrapIssue[] {
	return [...new Set(issues)];
}
