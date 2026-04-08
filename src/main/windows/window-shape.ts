import type { BrowserWindow } from "electron/main";
import type { OverlayAppearanceSettings } from "../../shared/appearance.js";
import { createWindowShapeRectangles } from "../../shared/window-shape.js";

const supportsWindowShape =
	process.platform === "linux" || process.platform === "win32";

export function applyMainWindowShape(
	window: BrowserWindow,
	appearance: Pick<
		OverlayAppearanceSettings,
		"overlayShape" | "cornerRoundness"
	>,
) {
	if (!supportsWindowShape || window.isDestroyed()) {
		return;
	}

	const { width, height } = window.getBounds();
	window.setShape(
		createWindowShapeRectangles(
			width,
			height,
			appearance.overlayShape,
			appearance.cornerRoundness,
		),
	);
}
