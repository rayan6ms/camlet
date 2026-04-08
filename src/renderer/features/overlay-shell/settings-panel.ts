import type { CameraPreviewStatus } from "./camera.js";

export interface SettingsPanelState {
	isOpen: boolean;
}

export type SettingsPanelAction =
	| {
			type: "toggle";
	  }
	| {
			type: "open";
	  }
	| {
			type: "close";
	  };

export function shouldAutoOpenSettingsPanel(
	status: CameraPreviewStatus,
): boolean {
	return (
		status === "permission-denied" ||
		status === "camera-in-use" ||
		status === "no-camera" ||
		status === "selected-device-unavailable" ||
		status === "error"
	);
}

export function createInitialSettingsPanelState(
	status: CameraPreviewStatus,
): SettingsPanelState {
	return {
		isOpen: shouldAutoOpenSettingsPanel(status),
	};
}

export function reduceSettingsPanelState(
	state: SettingsPanelState,
	action: SettingsPanelAction,
): SettingsPanelState {
	switch (action.type) {
		case "toggle":
			return {
				isOpen: !state.isOpen,
			};
		case "open":
			return {
				isOpen: true,
			};
		case "close":
			return {
				isOpen: false,
			};
	}
}
