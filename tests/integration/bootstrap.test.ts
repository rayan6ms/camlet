import { describe, expect, it } from "vitest";
import {
	createAppBootstrap,
	normalizeSystemLocale,
	resolveDisplayProtocol,
} from "../../src/main/bootstrap.js";
import { defaultCamletSettings } from "../../src/shared/settings.js";

describe("app bootstrap helpers", () => {
	it("builds the renderer bootstrap payload with resolved locale data", () => {
		const bootstrap = createAppBootstrap({
			app: {
				name: "Camlet",
				version: "0.1.0",
				platform: "linux",
				arch: "x64",
				packaged: true,
				displayProtocol: "wayland",
				electronVersion: "41.1.1",
				chromeVersion: "141.0.0.0",
			},
			settings: {
				...defaultCamletSettings,
				language: "pt-BR",
			},
			systemLocale: "pt-BR",
		});

		expect(bootstrap).toEqual({
			app: {
				name: "Camlet",
				version: "0.1.0",
				platform: "linux",
				arch: "x64",
				channel: "stable",
				mode: "production",
				packaged: true,
				displayProtocol: "wayland",
				versions: {
					electron: "41.1.1",
					chrome: "141.0.0.0",
				},
			},
			locale: {
				system: "pt-BR",
				effective: "pt-BR",
			},
			settings: {
				...defaultCamletSettings,
				language: "pt-BR",
			},
			windowState: defaultCamletSettings.window,
			issues: [],
		});
	});

	it("falls back when runtime metadata or locale are missing", () => {
		const bootstrap = createAppBootstrap({
			app: {},
			settings: defaultCamletSettings,
			systemLocale: "",
		});

		expect(bootstrap.app.name).toBe("Camlet");
		expect(bootstrap.app.version).toBe("0.0.0");
		expect(bootstrap.app.channel).toBe("stable");
		expect(bootstrap.app.mode).toBe("production");
		expect(bootstrap.app.arch).toBe(process.arch);
		expect(bootstrap.app.packaged).toBe(true);
		expect(bootstrap.app.displayProtocol).toBe(
			"linux" === process.platform
				? "unknown"
				: process.platform === "win32"
					? "windows"
					: process.platform === "darwin"
						? "macos"
						: "unknown",
		);
		expect(bootstrap.app.versions.electron).toBe(process.versions.electron);
		expect(bootstrap.app.versions.chrome).toBe(process.versions.chrome);
		expect(bootstrap.locale.system).toBe("en");
		expect(bootstrap.locale.effective).toBe("en");
		expect(bootstrap.issues).toEqual([]);
	});

	it("normalizes missing system locales to the shared fallback language", () => {
		expect(normalizeSystemLocale(undefined)).toBe("en");
		expect(normalizeSystemLocale(null)).toBe("en");
		expect(normalizeSystemLocale("pt-BR")).toBe("pt-BR");
	});

	it("resolves a safe display protocol for platform validation", () => {
		expect(
			resolveDisplayProtocol({
				platform: "linux",
				sessionType: "wayland",
			}),
		).toBe("wayland");
		expect(
			resolveDisplayProtocol({
				platform: "linux",
				sessionType: "x11",
			}),
		).toBe("x11");
		expect(
			resolveDisplayProtocol({
				platform: "linux",
				waylandDisplay: "wayland-0",
			}),
		).toBe("wayland");
		expect(
			resolveDisplayProtocol({
				platform: "win32",
			}),
		).toBe("windows");
		expect(
			resolveDisplayProtocol({
				platform: "darwin",
			}),
		).toBe("macos");
	});

	it("adds release channel, runtime mode, and deduped startup issues", () => {
		const bootstrap = createAppBootstrap({
			app: {
				name: "Camlet",
				version: "0.2.0-beta.1",
				platform: "linux",
				arch: "x64",
				mode: "development",
				packaged: false,
			},
			settings: defaultCamletSettings,
			issues: [
				"settings-recovered",
				"settings-recovered",
				"settings-persistence-unavailable",
			],
		});

		expect(bootstrap.app.channel).toBe("prerelease");
		expect(bootstrap.app.mode).toBe("development");
		expect(bootstrap.app.packaged).toBe(false);
		expect(bootstrap.issues).toEqual([
			"settings-recovered",
			"settings-persistence-unavailable",
		]);
	});
});
