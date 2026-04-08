import { describe, expect, it } from "vitest";
import {
	createInitialSettingsPanelState,
	reduceSettingsPanelState,
	shouldAutoOpenSettingsPanel,
} from "../../src/renderer/features/overlay-shell/settings-panel.js";

describe("settings panel helpers", () => {
	it("auto-opens the panel for actionable camera failures", () => {
		expect(shouldAutoOpenSettingsPanel("permission-denied")).toBe(true);
		expect(shouldAutoOpenSettingsPanel("camera-in-use")).toBe(true);
		expect(shouldAutoOpenSettingsPanel("no-camera")).toBe(true);
		expect(shouldAutoOpenSettingsPanel("selected-device-unavailable")).toBe(
			true,
		);
		expect(shouldAutoOpenSettingsPanel("error")).toBe(true);
	});

	it("keeps the panel closed during loading and healthy preview states", () => {
		expect(shouldAutoOpenSettingsPanel("loading")).toBe(false);
		expect(shouldAutoOpenSettingsPanel("preview")).toBe(false);
	});

	it("creates the expected initial state from a camera status", () => {
		expect(createInitialSettingsPanelState("preview")).toEqual({
			isOpen: false,
		});
		expect(createInitialSettingsPanelState("permission-denied")).toEqual({
			isOpen: true,
		});
	});

	it("reduces explicit open, close, and toggle actions", () => {
		const closedState = { isOpen: false };
		const openedState = reduceSettingsPanelState(closedState, {
			type: "open",
		});
		const toggledState = reduceSettingsPanelState(openedState, {
			type: "toggle",
		});

		expect(openedState).toEqual({ isOpen: true });
		expect(toggledState).toEqual({ isOpen: false });
		expect(
			reduceSettingsPanelState(toggledState, {
				type: "close",
			}),
		).toEqual({
			isOpen: false,
		});
	});
});
