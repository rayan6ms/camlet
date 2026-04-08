import { describe, expect, it, vi } from "vitest";
import { resolveRendererStartupState } from "../../src/renderer/startup.js";
import { defaultCamletSettings } from "../../src/shared/settings.js";

describe("renderer startup helpers", () => {
	it("returns a preload-unavailable error when the preload bridge is missing", async () => {
		await expect(
			resolveRendererStartupState({
				camletApi: null,
				preferredLanguage: "pt-BR",
			}),
		).resolves.toEqual({
			bootstrap: null,
			language: "pt-BR",
			startupError: {
				code: "preload-unavailable",
				detail: null,
			},
		});
	});

	it("returns a bootstrap-invalid error when the payload shape is malformed", async () => {
		await expect(
			resolveRendererStartupState({
				camletApi: {
					getBootstrap: vi.fn(async () => ({
						app: {
							name: "Camlet",
						},
					})),
				},
				preferredLanguage: "en-US",
			}),
		).resolves.toMatchObject({
			bootstrap: null,
			language: "en",
			startupError: {
				code: "bootstrap-invalid",
			},
		});
	});

	it("returns the parsed bootstrap and effective language when the payload is valid", async () => {
		await expect(
			resolveRendererStartupState({
				camletApi: {
					getBootstrap: vi.fn(async () => ({
						app: {
							name: "Camlet",
							version: "0.2.0-beta.1",
							platform: "linux",
							arch: "x64",
							channel: "prerelease",
							mode: "development",
							packaged: false,
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
						settings: defaultCamletSettings,
						windowState: defaultCamletSettings.window,
						issues: ["settings-recovered"],
					})),
				},
				preferredLanguage: "en",
			}),
		).resolves.toEqual({
			bootstrap: {
				app: {
					name: "Camlet",
					version: "0.2.0-beta.1",
					platform: "linux",
					arch: "x64",
					channel: "prerelease",
					mode: "development",
					packaged: false,
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
				settings: defaultCamletSettings,
				windowState: defaultCamletSettings.window,
				issues: ["settings-recovered"],
			},
			language: "pt-BR",
			startupError: null,
		});
	});

	it("maps bootstrap load failures to a translated-friendly error code", async () => {
		await expect(
			resolveRendererStartupState({
				camletApi: {
					getBootstrap: vi.fn(async () => {
						throw new Error("settings unavailable");
					}),
				},
				preferredLanguage: "en",
			}),
		).resolves.toEqual({
			bootstrap: null,
			language: "en",
			startupError: {
				code: "bootstrap-load-failed",
				detail: "settings unavailable",
			},
		});
	});
});
