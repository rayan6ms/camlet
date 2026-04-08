import {
	type CameraDeviceOption,
	type CameraPreviewStatus,
	getCameraDeviceLabel,
	isRecoverableCameraSelectionError,
	resolveCameraFailure,
	resolveSelectedCameraDevice,
	toCameraDeviceOptions,
} from "./camera.js";

export interface CameraPreviewState {
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

export interface CameraPreviewController {
	attachVideoElement(element: HTMLVideoElement | null): void;
	destroy(): void;
	getState(): CameraPreviewState;
	retry(): Promise<void>;
	selectDevice(deviceId: string): Promise<void>;
	subscribe(listener: (state: CameraPreviewState) => void): () => void;
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

export function createCameraPreviewController({
	initialSelectedDeviceId,
}: UseCameraPreviewOptions): CameraPreviewController {
	let videoElement: HTMLVideoElement | null = null;
	let stream: MediaStream | null = null;
	let requestId = 0;
	let selectedDeviceId = initialSelectedDeviceId;
	let activeDeviceId: string | null = null;
	let deviceChangeListener: (() => Promise<void>) | null = null;
	let state: CameraPreviewState = {
		status: "loading",
		devices: [],
		selectedDeviceId: initialSelectedDeviceId,
		activeDeviceId: null,
		activeDeviceLabel: null,
		fallbackNotice: false,
		errorMessage: null,
	};
	const listeners = new Set<(state: CameraPreviewState) => void>();

	function notifyListeners() {
		for (const listener of listeners) {
			listener(state);
		}
	}

	function setState(
		nextState:
			| CameraPreviewState
			| ((currentState: CameraPreviewState) => CameraPreviewState),
	) {
		state = typeof nextState === "function" ? nextState(state) : nextState;
		notifyListeners();
	}

	function stopStream() {
		for (const track of stream?.getTracks() ?? []) {
			track.stop();
		}

		stream = null;
		activeDeviceId = null;

		if (videoElement !== null) {
			videoElement.srcObject = null;
		}
	}

	async function attachStream(nextStream: MediaStream) {
		stream = nextStream;

		if (videoElement === null) {
			return;
		}

		videoElement.srcObject = nextStream;

		try {
			await videoElement.play();
		} catch {}
	}

	async function persistSelectedDeviceId(deviceId: string | null) {
		selectedDeviceId = deviceId;
		const nextSettings =
			await window.camlet.setSelectedCameraDeviceId(deviceId);
		selectedDeviceId = nextSettings.selectedCameraDeviceId;
		return nextSettings.selectedCameraDeviceId;
	}

	async function refreshCamera(requestedDeviceId: string | null) {
		if (!hasMediaDevicesApi()) {
			setState((currentState) => ({
				...currentState,
				status: "error",
				errorMessage: "Media devices API is not available in this environment.",
			}));
			return;
		}

		const nextRequestId = requestId + 1;
		requestId = nextRequestId;

		setState((currentState) => ({
			...currentState,
			status: "loading",
			selectedDeviceId: requestedDeviceId,
			fallbackNotice: false,
			errorMessage: null,
		}));

		stopStream();

		try {
			const { stream: nextStream, fallbackUsed } =
				await requestCameraStream(requestedDeviceId);

			if (nextRequestId !== requestId) {
				for (const track of nextStream.getTracks()) {
					track.stop();
				}

				return;
			}

			await attachStream(nextStream);

			const devices = await enumerateCameraDevices();
			const activeTrack = nextStream.getVideoTracks()[0];
			const resolvedActiveDeviceId =
				activeTrack?.getSettings().deviceId ??
				resolveSelectedCameraDevice(devices, requestedDeviceId).deviceId;
			const nextSelectedDeviceId = resolvedActiveDeviceId ?? requestedDeviceId;

			activeDeviceId = resolvedActiveDeviceId ?? null;

			if (
				nextSelectedDeviceId !== null &&
				nextSelectedDeviceId !== selectedDeviceId
			) {
				await persistSelectedDeviceId(nextSelectedDeviceId);
			}

			setState({
				status: "preview",
				devices,
				selectedDeviceId: nextSelectedDeviceId,
				activeDeviceId: resolvedActiveDeviceId ?? null,
				activeDeviceLabel: getCameraDeviceLabel(
					devices,
					resolvedActiveDeviceId ?? null,
				),
				fallbackNotice: fallbackUsed,
				errorMessage: null,
			});
		} catch (error) {
			if (nextRequestId !== requestId) {
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
	}

	void refreshCamera(selectedDeviceId);

	if (hasDeviceChangeEvents()) {
		deviceChangeListener = async () => {
			const devices = await enumerateCameraDevices().catch(() => []);
			const hasActiveDevice = devices.some(
				(device) => device.deviceId === activeDeviceId,
			);
			const hasSavedDevice = devices.some(
				(device) => device.deviceId === selectedDeviceId,
			);

			if (devices.length === 0) {
				stopStream();
				setState({
					status:
						selectedDeviceId !== null
							? "selected-device-unavailable"
							: "no-camera",
					devices,
					selectedDeviceId,
					activeDeviceId: null,
					activeDeviceLabel: null,
					fallbackNotice: false,
					errorMessage: null,
				});
				return;
			}

			if (hasActiveDevice && (selectedDeviceId === null || hasSavedDevice)) {
				setState((currentState) => ({
					...currentState,
					devices,
					activeDeviceLabel: getCameraDeviceLabel(devices, activeDeviceId),
				}));
				return;
			}

			const resolution = resolveSelectedCameraDevice(devices, selectedDeviceId);
			await refreshCamera(resolution.deviceId);
		};

		navigator.mediaDevices.addEventListener(
			"devicechange",
			deviceChangeListener,
		);
	}

	return {
		attachVideoElement(element) {
			videoElement = element;

			if (videoElement === null || stream === null) {
				return;
			}

			videoElement.srcObject = stream;
			void videoElement.play().catch(() => {});
		},
		destroy() {
			if (deviceChangeListener !== null) {
				navigator.mediaDevices.removeEventListener(
					"devicechange",
					deviceChangeListener,
				);
			}

			requestId += 1;
			stopStream();
			listeners.clear();
		},
		getState() {
			return state;
		},
		retry() {
			return refreshCamera(selectedDeviceId);
		},
		selectDevice: async (deviceId: string) => {
			const nextSelectedDeviceId = await persistSelectedDeviceId(deviceId);
			await refreshCamera(nextSelectedDeviceId);
		},
		subscribe(listener) {
			listeners.add(listener);
			return () => {
				listeners.delete(listener);
			};
		},
	};
}
