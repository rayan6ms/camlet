export interface CameraDeviceOption {
	deviceId: string;
	label: string;
}

export interface CameraDeviceLike {
	deviceId: string;
	kind: string;
	label: string;
}

export type CameraPreviewStatus =
	| "loading"
	| "preview"
	| "permission-denied"
	| "camera-in-use"
	| "no-camera"
	| "selected-device-unavailable"
	| "error";

export interface CameraSelectionResolution {
	deviceId: string | null;
	savedDeviceWasInvalid: boolean;
}

export interface CameraFailureResolution {
	status: CameraPreviewStatus;
	errorMessage: string | null;
}

function getCameraErrorName(error: unknown): string | null {
	if (
		typeof error === "object" &&
		error !== null &&
		"name" in error &&
		typeof error.name === "string"
	) {
		return error.name;
	}

	return null;
}

export function toCameraDeviceOptions(
	devices: Iterable<CameraDeviceLike>,
): CameraDeviceOption[] {
	return Array.from(devices)
		.filter((device) => device.kind === "videoinput")
		.map((device, index) => ({
			deviceId: device.deviceId,
			label: device.label.trim() || `Camera ${index + 1}`,
		}));
}

export function resolveSelectedCameraDevice(
	devices: CameraDeviceOption[],
	savedDeviceId: string | null,
): CameraSelectionResolution {
	if (devices.length === 0) {
		return {
			deviceId: null,
			savedDeviceWasInvalid: savedDeviceId !== null,
		};
	}

	if (savedDeviceId === null) {
		return {
			deviceId: devices[0]?.deviceId ?? null,
			savedDeviceWasInvalid: false,
		};
	}

	const savedDevice = devices.find(
		(device) => device.deviceId === savedDeviceId,
	);

	if (savedDevice !== undefined) {
		return {
			deviceId: savedDevice.deviceId,
			savedDeviceWasInvalid: false,
		};
	}

	return {
		deviceId: devices[0]?.deviceId ?? null,
		savedDeviceWasInvalid: true,
	};
}

export function getCameraDeviceLabel(
	devices: CameraDeviceOption[],
	deviceId: string | null,
): string | null {
	if (deviceId === null) {
		return null;
	}

	return devices.find((device) => device.deviceId === deviceId)?.label ?? null;
}

export function getCameraSelectValue(
	devices: CameraDeviceOption[],
	deviceId: string | null,
): string {
	if (
		deviceId !== null &&
		devices.some((device) => device.deviceId === deviceId)
	) {
		return deviceId;
	}

	return devices[0]?.deviceId ?? "";
}

export function isRecoverableCameraSelectionError(error: unknown): boolean {
	const errorName = getCameraErrorName(error);

	return errorName === "NotFoundError" || errorName === "OverconstrainedError";
}

export function resolveCameraFailure(
	error: unknown,
	options: {
		devices: CameraDeviceOption[];
		savedDeviceId: string | null;
	},
): CameraFailureResolution {
	const errorName = getCameraErrorName(error);

	if (
		errorName === "NotAllowedError" ||
		errorName === "PermissionDeniedError" ||
		errorName === "SecurityError"
	) {
		return {
			status: "permission-denied",
			errorMessage: null,
		};
	}

	if (
		errorName === "NotReadableError" ||
		errorName === "TrackStartError" ||
		errorName === "AbortError"
	) {
		return {
			status: "camera-in-use",
			errorMessage: null,
		};
	}

	if (
		errorName === "NotFoundError" ||
		errorName === "DevicesNotFoundError" ||
		errorName === "OverconstrainedError"
	) {
		if (options.savedDeviceId !== null && options.devices.length === 0) {
			return {
				status: "selected-device-unavailable",
				errorMessage: null,
			};
		}

		return {
			status: "no-camera",
			errorMessage: null,
		};
	}

	return {
		status: "error",
		errorMessage:
			error instanceof Error
				? error.message
				: "Unknown camera initialization error",
	};
}
