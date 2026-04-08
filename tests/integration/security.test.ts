import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import {
	createRendererAssetPolicy,
	isAllowedNavigationTarget,
	isTrustedRendererUrl,
	shouldAllowPermission,
	validateRendererUrl,
} from "../../src/main/security.js";

const rendererHtmlPath = "/tmp/camlet/dist/renderer/index.html";

describe("main-process security helpers", () => {
	it("accepts loopback Vite dev server URLs and rejects remote ones", () => {
		expect(validateRendererUrl("http://127.0.0.1:5173")).toBe(
			"http://127.0.0.1:5173/",
		);
		expect(validateRendererUrl("https://localhost:4173")).toBe(
			"https://localhost:4173/",
		);
		expect(() => validateRendererUrl("https://example.com")).toThrow(
			/local localhost or loopback origin/,
		);
	});

	it("builds a consistent renderer asset policy for packaged and dev modes", () => {
		expect(createRendererAssetPolicy(rendererHtmlPath)).toEqual({
			rendererHtmlPath,
		});
		expect(
			createRendererAssetPolicy(rendererHtmlPath, "http://127.0.0.1:5173/"),
		).toEqual({
			rendererHtmlPath,
			rendererUrl: "http://127.0.0.1:5173/",
		});
	});

	it("allows only the packaged renderer file in production mode", () => {
		const productionPolicy = {
			rendererHtmlPath,
		};

		expect(
			isTrustedRendererUrl(
				pathToFileURL(rendererHtmlPath).toString(),
				productionPolicy,
			),
		).toBe(true);
		expect(
			isAllowedNavigationTarget(
				pathToFileURL("/tmp/camlet/dist/renderer/other.html").toString(),
				productionPolicy,
			),
		).toBe(false);
	});

	it("allows same-origin dev navigation and denies other origins", () => {
		const developmentPolicy = {
			rendererHtmlPath,
			rendererUrl: "http://127.0.0.1:5173/",
		};

		expect(
			isAllowedNavigationTarget(
				"http://127.0.0.1:5173/settings?view=compact",
				developmentPolicy,
			),
		).toBe(true);
		expect(
			isAllowedNavigationTarget("https://example.com", developmentPolicy),
		).toBe(false);
	});

	it("allows media permission requests only for trusted renderer content", () => {
		const developmentPolicy = {
			rendererHtmlPath,
			rendererUrl: "http://localhost:5173/",
		};

		expect(
			shouldAllowPermission(
				"media",
				"http://localhost:5173/",
				developmentPolicy,
			),
		).toBe(true);
		expect(
			shouldAllowPermission(
				"notifications",
				"http://localhost:5173/",
				developmentPolicy,
			),
		).toBe(false);
		expect(
			shouldAllowPermission("media", "https://example.com", developmentPolicy),
		).toBe(false);
		expect(
			shouldAllowPermission("media", "file://", {
				rendererHtmlPath,
			}),
		).toBe(true);
	});

	it("denies microphone-capable media requests", () => {
		const developmentPolicy = {
			rendererHtmlPath,
			rendererUrl: "http://localhost:5173/",
		};

		expect(
			shouldAllowPermission(
				"media",
				"http://localhost:5173/",
				developmentPolicy,
				{
					mediaTypes: ["video"],
				},
			),
		).toBe(true);
		expect(
			shouldAllowPermission(
				"media",
				"http://localhost:5173/",
				developmentPolicy,
				{
					mediaTypes: ["audio"],
				},
			),
		).toBe(false);
		expect(
			shouldAllowPermission(
				"media",
				"http://localhost:5173/",
				developmentPolicy,
				{
					mediaTypes: ["video", "audio"],
				},
			),
		).toBe(false);
	});
});
