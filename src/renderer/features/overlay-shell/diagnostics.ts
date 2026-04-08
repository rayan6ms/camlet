import type { AppBootstrap } from "../../../shared/bootstrap.js";
import type { SupportedLanguage } from "../../../shared/language.js";
import type { CameraPreviewStatus } from "./camera.js";

export interface OverlayDiagnosticsSummaryInput {
	bootstrap: AppBootstrap;
	effectiveLanguage: SupportedLanguage;
	camera: {
		status: CameraPreviewStatus;
		activeDeviceLabel: string | null;
		deviceCount: number;
		fallbackNotice: boolean;
	};
}

function formatList(values: string[]) {
	return values.length > 0 ? values.join(", ") : "none";
}

export function getOverlayDiagnosticsSummary(
	input: OverlayDiagnosticsSummaryInput,
) {
	const { app, issues, locale } = input.bootstrap;

	return [
		`app=${app.name}`,
		`version=${app.version}`,
		`channel=${app.channel}`,
		`runtime=${app.mode}`,
		`packaged=${app.packaged ? "yes" : "no"}`,
		`platform=${app.platform}`,
		`arch=${app.arch}`,
		`display=${app.displayProtocol}`,
		`electron=${app.versions.electron}`,
		`chrome=${app.versions.chrome}`,
		`systemLocale=${locale.system}`,
		`uiLanguage=${input.effectiveLanguage}`,
		`cameraStatus=${input.camera.status}`,
		`activeCamera=${input.camera.activeDeviceLabel ?? "none"}`,
		`detectedCameras=${String(input.camera.deviceCount)}`,
		`fallbackCameraUsed=${input.camera.fallbackNotice ? "yes" : "no"}`,
		`startupIssues=${formatList(issues)}`,
	].join("\n");
}
