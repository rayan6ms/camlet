import { describe, expect, it } from "vitest";
import { getOverlayDiagnosticsSummary } from "../../src/renderer/features/overlay-shell/diagnostics.js";
import { defaultCamletSettings } from "../../src/shared/settings.js";

describe("overlay diagnostics summary", () => {
	it("formats a stable runtime summary for packaged-build validation", () => {
		expect(
			getOverlayDiagnosticsSummary({
				bootstrap: {
					app: {
						name: "Camlet",
						version: "0.2.0-beta.1",
						platform: "linux",
						arch: "x64",
						channel: "prerelease",
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
					settings: defaultCamletSettings,
					windowState: defaultCamletSettings.window,
					issues: ["settings-recovered"],
				},
				effectiveLanguage: "pt-BR",
				camera: {
					status: "preview",
					activeDeviceLabel: "Front Camera",
					deviceCount: 2,
					fallbackNotice: true,
				},
			}),
		).toBe(`app=Camlet
version=0.2.0-beta.1
channel=prerelease
runtime=production
packaged=yes
platform=linux
arch=x64
display=wayland
electron=41.1.1
chrome=141.0.0.0
systemLocale=pt-BR
uiLanguage=pt-BR
cameraStatus=preview
activeCamera=Front Camera
detectedCameras=2
fallbackCameraUsed=yes
startupIssues=settings-recovered`);
	});

	it("uses safe placeholders when optional diagnostics are absent", () => {
		expect(
			getOverlayDiagnosticsSummary({
				bootstrap: {
					app: {
						name: "Camlet",
						version: "0.1.0",
						platform: "win32",
						arch: "x64",
						channel: "stable",
						mode: "development",
						packaged: false,
						displayProtocol: "windows",
						versions: {
							electron: "41.1.1",
							chrome: "141.0.0.0",
						},
					},
					locale: {
						system: "en-US",
						effective: "en",
					},
					settings: defaultCamletSettings,
					windowState: defaultCamletSettings.window,
					issues: [],
				},
				effectiveLanguage: "en",
				camera: {
					status: "no-camera",
					activeDeviceLabel: null,
					deviceCount: 0,
					fallbackNotice: false,
				},
			}),
		).toContain("startupIssues=none");
	});
});
