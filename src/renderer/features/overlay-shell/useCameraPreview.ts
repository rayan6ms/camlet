import { useEffect, useEffectEvent, useRef, useState } from "react";
import {
	type CameraDeviceOption,
	type CameraPreviewStatus,
	getCameraDeviceLabel,
	isRecoverableCameraSelectionError,
	resolveCameraFailure,
	resolveSelectedCameraDevice,
	toCameraDeviceOptions,
} from "./camera.js";

interface CameraPreviewState {
	status: CameraPreviewStatus;
	devices: CameraDeviceOption[];
	selectedDeviceId: string | null;
	activeDeviceId: string | null;
	activeDeviceLabel: string | null;
	fallbackNotice: boolean;
	errorMessage: string | null;
}

interface UseCameraPreviewOptions {
	initialSelectedDeviceId: string | null;
}

function hasMediaDevicesApi() {
	return (
		navigator.mediaDevices !== undefined &&
		typeof navigator.mediaDevices.getUserMedia === "function" &&
		typeof navigator.mediaDevices.enumerateDevices === "function"
	);
}

function hasDeviceChangeEvents() {
	return (
		hasMediaDevicesApi() &&
		typeof navigator.mediaDevices.addEventListener === "function" &&
		typeof navigator.mediaDevices.removeEventListener === "function"
	);
}

async function enumerateCameraDevices(): Promise<CameraDeviceOption[]> {
	const devices = await navigator.mediaDevices.enumerateDevices();
	return toCameraDeviceOptions(devices);
}

async function requestCameraStream(
	selectedDeviceId: string | null,
): Promise<{ stream: MediaStream; fallbackUsed: boolean }> {
	if (selectedDeviceId !== null) {
		try {
			return {
				stream: await navigator.mediaDevices.getUserMedia({
					audio: false,
					video: {
						deviceId: {
							exact: selectedDeviceId,
						},
					},
				}),
				fallbackUsed: false,
			};
		} catch (error) {
			if (!isRecoverableCameraSelectionError(error)) {
				throw error;
			}
		}
	}

	return {
		stream: await navigator.mediaDevices.getUserMedia({
			audio: false,
			video: true,
		}),
		fallbackUsed: selectedDeviceId !== null,
	};
}

export function useCameraPreview({
	initialSelectedDeviceId,
}: UseCameraPreviewOptions) {
	const videoRef = useRef<HTMLVideoElement | null>(null);
	const streamRef = useRef<MediaStream | null>(null);
	const requestIdRef = useRef(0);
	const selectedDeviceIdRef = useRef<string | null>(initialSelectedDeviceId);
	const activeDeviceIdRef = useRef<string | null>(null);
	const [state, setState] = useState<CameraPreviewState>({
		status: "loading",
		devices: [],
		selectedDeviceId: initialSelectedDeviceId,
		activeDeviceId: null,
		activeDeviceLabel: null,
		fallbackNotice: false,
		errorMessage: null,
	});

	const stopStream = useEffectEvent(() => {
		for (const track of streamRef.current?.getTracks() ?? []) {
			track.stop();
		}

		streamRef.current = null;
		activeDeviceIdRef.current = null;

		if (videoRef.current !== null) {
			videoRef.current.srcObject = null;
		}
	});

	const attachStream = useEffectEvent(async (stream: MediaStream) => {
		streamRef.current = stream;

		if (videoRef.current === null) {
			return;
		}

		videoRef.current.srcObject = stream;

		try {
			await videoRef.current.play();
		} catch {}
	});

	const persistSelectedDeviceId = useEffectEvent(
		async (deviceId: string | null) => {
			selectedDeviceIdRef.current = deviceId;
			const nextSettings =
				await window.camlet.setSelectedCameraDeviceId(deviceId);
			selectedDeviceIdRef.current = nextSettings.selectedCameraDeviceId;
			return nextSettings.selectedCameraDeviceId;
		},
	);

	const refreshCamera = useEffectEvent(
		async (requestedDeviceId: string | null) => {
			if (!hasMediaDevicesApi()) {
				setState((currentState) => ({
					...currentState,
					status: "error",
					errorMessage:
						"Media devices API is not available in this environment.",
				}));
				return;
			}

			const requestId = requestIdRef.current + 1;
			requestIdRef.current = requestId;

			setState((currentState) => ({
				...currentState,
				status: "loading",
				selectedDeviceId: requestedDeviceId,
				fallbackNotice: false,
				errorMessage: null,
			}));

			stopStream();

			try {
				const { stream, fallbackUsed } =
					await requestCameraStream(requestedDeviceId);

				if (requestId !== requestIdRef.current) {
					for (const track of stream.getTracks()) {
						track.stop();
					}

					return;
				}

				await attachStream(stream);

				const devices = await enumerateCameraDevices();
				const activeTrack = stream.getVideoTracks()[0];
				const activeDeviceId =
					activeTrack?.getSettings().deviceId ??
					resolveSelectedCameraDevice(devices, requestedDeviceId).deviceId;
				const nextSelectedDeviceId = activeDeviceId ?? requestedDeviceId;

				activeDeviceIdRef.current = activeDeviceId ?? null;

				if (
					nextSelectedDeviceId !== null &&
					nextSelectedDeviceId !== selectedDeviceIdRef.current
				) {
					await persistSelectedDeviceId(nextSelectedDeviceId);
				}

				setState({
					status: "preview",
					devices,
					selectedDeviceId: nextSelectedDeviceId,
					activeDeviceId: activeDeviceId ?? null,
					activeDeviceLabel: getCameraDeviceLabel(
						devices,
						activeDeviceId ?? null,
					),
					fallbackNotice: fallbackUsed,
					errorMessage: null,
				});
			} catch (error) {
				if (requestId !== requestIdRef.current) {
					return;
				}

				const devices = await enumerateCameraDevices().catch(() => []);
				const failure = resolveCameraFailure(error, {
					devices,
					savedDeviceId: requestedDeviceId,
				});

				stopStream();
				setState({
					status: failure.status,
					devices,
					selectedDeviceId: requestedDeviceId,
					activeDeviceId: null,
					activeDeviceLabel: null,
					fallbackNotice: false,
					errorMessage: failure.errorMessage,
				});
			}
		},
	);

	useEffect(() => {
		void refreshCamera(selectedDeviceIdRef.current);

		if (!hasDeviceChangeEvents()) {
			return () => {
				requestIdRef.current += 1;
				stopStream();
			};
		}

		const handleDeviceChange = async () => {
			const devices = await enumerateCameraDevices().catch(() => []);
			const activeDeviceId = activeDeviceIdRef.current;
			const savedDeviceId = selectedDeviceIdRef.current;
			const hasActiveDevice = devices.some(
				(device) => device.deviceId === activeDeviceId,
			);
			const hasSavedDevice = devices.some(
				(device) => device.deviceId === savedDeviceId,
			);

			if (devices.length === 0) {
				stopStream();
				setState({
					status:
						savedDeviceId !== null
							? "selected-device-unavailable"
							: "no-camera",
					devices,
					selectedDeviceId: savedDeviceId,
					activeDeviceId: null,
					activeDeviceLabel: null,
					fallbackNotice: false,
					errorMessage: null,
				});
				return;
			}

			if (hasActiveDevice && (savedDeviceId === null || hasSavedDevice)) {
				setState((currentState) => ({
					...currentState,
					devices,
					activeDeviceLabel: getCameraDeviceLabel(devices, activeDeviceId),
				}));
				return;
			}

			const resolution = resolveSelectedCameraDevice(devices, savedDeviceId);
			void refreshCamera(resolution.deviceId);
		};

		navigator.mediaDevices.addEventListener("devicechange", handleDeviceChange);

		return () => {
			navigator.mediaDevices.removeEventListener(
				"devicechange",
				handleDeviceChange,
			);
			requestIdRef.current += 1;
			stopStream();
		};
	}, []);

	return {
		videoRef,
		state,
		retry: () => refreshCamera(selectedDeviceIdRef.current),
		selectDevice: async (deviceId: string) => {
			const nextSelectedDeviceId = await persistSelectedDeviceId(deviceId);
			await refreshCamera(nextSelectedDeviceId);
		},
	};
}
