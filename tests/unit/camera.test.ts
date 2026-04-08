import { describe, expect, it } from "vitest";
import {
	getCameraDeviceLabel,
	getCameraSelectValue,
	isRecoverableCameraSelectionError,
	resolveCameraFailure,
	resolveSelectedCameraDevice,
	toCameraDeviceOptions,
} from "../../src/renderer/features/overlay-shell/camera.js";

const devices = [
	{
		deviceId: "front-camera",
		kind: "videoinput",
		label: "Front Camera",
	},
	{
		deviceId: "rear-camera",
		kind: "videoinput",
		label: "   ",
	},
	{
		deviceId: "microphone",
		kind: "audioinput",
		label: "Mic",
	},
] as const;

describe("camera device helpers", () => {
	it("filters non-video devices and applies fallback labels", () => {
		expect(toCameraDeviceOptions(devices)).toEqual([
			{
				deviceId: "front-camera",
				label: "Front Camera",
			},
			{
				deviceId: "rear-camera",
				label: "Camera 2",
			},
		]);
	});

	it("resolves the saved device when available", () => {
		expect(
			resolveSelectedCameraDevice(
				toCameraDeviceOptions(devices),
				"rear-camera",
			),
		).toEqual({
			deviceId: "rear-camera",
			savedDeviceWasInvalid: false,
		});
	});

	it("falls back to the first available device when the saved selection is invalid", () => {
		expect(
			resolveSelectedCameraDevice(
				toCameraDeviceOptions(devices),
				"missing-camera",
			),
		).toEqual({
			deviceId: "front-camera",
			savedDeviceWasInvalid: true,
		});
	});

	it("returns null when no video devices exist", () => {
		expect(resolveSelectedCameraDevice([], "missing-camera")).toEqual({
			deviceId: null,
			savedDeviceWasInvalid: true,
		});
	});

	it("looks up device labels by id", () => {
		expect(
			getCameraDeviceLabel(toCameraDeviceOptions(devices), "front-camera"),
		).toBe("Front Camera");
		expect(
			getCameraDeviceLabel(toCameraDeviceOptions(devices), "missing"),
		).toBe(null);
	});

	it("produces a safe select value when the current device id is missing", () => {
		expect(
			getCameraSelectValue(toCameraDeviceOptions(devices), "rear-camera"),
		).toBe("rear-camera");
		expect(
			getCameraSelectValue(toCameraDeviceOptions(devices), "missing"),
		).toBe("front-camera");
		expect(getCameraSelectValue([], "missing")).toBe("");
	});
});

describe("camera failure resolution", () => {
	it("maps permission errors to permission-denied", () => {
		expect(
			resolveCameraFailure(
				{
					name: "NotAllowedError",
				},
				{
					devices: toCameraDeviceOptions(devices),
					savedDeviceId: null,
				},
			),
		).toEqual({
			status: "permission-denied",
			errorMessage: null,
		});
	});

	it("maps missing devices to selected-device-unavailable when a saved camera exists", () => {
		expect(
			resolveCameraFailure(
				{
					name: "NotFoundError",
				},
				{
					devices: [],
					savedDeviceId: "saved-camera",
				},
			),
		).toEqual({
			status: "selected-device-unavailable",
			errorMessage: null,
		});
	});

	it("maps busy camera failures to camera-in-use", () => {
		expect(
			resolveCameraFailure(
				{
					name: "NotReadableError",
				},
				{
					devices: toCameraDeviceOptions(devices),
					savedDeviceId: null,
				},
			),
		).toEqual({
			status: "camera-in-use",
			errorMessage: null,
		});
	});

	it("maps unexpected failures to error with a message", () => {
		expect(
			resolveCameraFailure(new Error("boom"), {
				devices: toCameraDeviceOptions(devices),
				savedDeviceId: null,
			}),
		).toEqual({
			status: "error",
			errorMessage: "boom",
		});
	});

	it("detects recoverable saved-device errors", () => {
		expect(isRecoverableCameraSelectionError({ name: "NotFoundError" })).toBe(
			true,
		);
		expect(
			isRecoverableCameraSelectionError({ name: "OverconstrainedError" }),
		).toBe(true);
		expect(isRecoverableCameraSelectionError({ name: "NotAllowedError" })).toBe(
			false,
		);
	});
});
