import { type BrowserWindow, screen } from "electron/main";
import { ipcChannels } from "../../shared/ipc.js";
import {
	clampWindowStateToDisplay,
	type DisplayWorkArea,
	type DragOffset,
	getDragOffset,
	isWindowStateEqual,
	mergeWindowState,
	moveWindowStateWithPointer,
	type ScreenPoint,
	type WindowState,
} from "../../shared/window-state.js";
import type { SettingsStoreService } from "../services/settings-store.js";
import { applyMainWindowShape } from "./window-shape.js";

const legacyDefaultWindowSize = 360;

function toDisplayWorkArea(bounds: Electron.Rectangle): DisplayWorkArea {
	return {
		x: bounds.x,
		y: bounds.y,
		width: bounds.width,
		height: bounds.height,
	};
}

function getDisplayWorkAreaForWindowState(
	windowState: WindowState,
): DisplayWorkArea {
	return toDisplayWorkArea(screen.getDisplayMatching(windowState).bounds);
}

export function getCurrentDisplayWorkArea(
	window: BrowserWindow,
): DisplayWorkArea {
	return getDisplayWorkAreaForWindowState(getWindowStateFromWindow(window));
}

function getDisplayWorkAreaForPointer(pointer: ScreenPoint): DisplayWorkArea {
	return toDisplayWorkArea(
		screen.getDisplayNearestPoint({
			x: pointer.screenX,
			y: pointer.screenY,
		}).bounds,
	);
}

export function getSafeMainWindowState(
	settingsStore: SettingsStoreService,
): WindowState {
	const storedWindowState = settingsStore.getWindowState();
	const settings = settingsStore.getSettings();
	const preferredSize = settings.overlaySize;
	const normalizedWindowState =
		storedWindowState.width !== storedWindowState.height ||
		(storedWindowState.width === legacyDefaultWindowSize &&
			storedWindowState.height === legacyDefaultWindowSize)
			? {
					...storedWindowState,
					width: preferredSize,
					height: preferredSize,
				}
			: storedWindowState;

	return clampWindowStateToDisplay(
		normalizedWindowState,
		getDisplayWorkAreaForWindowState(normalizedWindowState),
	);
}

export function getWindowStateFromWindow(window: BrowserWindow): WindowState {
	return mergeWindowState(window.getBounds());
}

function emitWindowState(window: BrowserWindow, windowState: WindowState) {
	if (window.isDestroyed() || window.webContents.isDestroyed()) {
		return;
	}

	window.webContents.send(ipcChannels.windowStateChanged, windowState);
}

function getClampedWindowState(window: BrowserWindow): WindowState {
	const windowState = getWindowStateFromWindow(window);
	return clampWindowStateToDisplay(
		windowState,
		getDisplayWorkAreaForWindowState(windowState),
	);
}

export function persistWindowStateNow(
	window: BrowserWindow,
	settingsStore: SettingsStoreService,
): WindowState {
	const windowState = getClampedWindowState(window);
	settingsStore.setWindowState(windowState);
	emitWindowState(window, windowState);
	return windowState;
}

export function moveMainWindowWithPointer(
	window: BrowserWindow,
	pointer: ScreenPoint,
	offset: DragOffset,
): WindowState {
	const currentWindowState = getWindowStateFromWindow(window);
	const nextWindowState = clampWindowStateToDisplay(
		moveWindowStateWithPointer(currentWindowState, pointer, offset),
		getDisplayWorkAreaForPointer(pointer),
	);

	if (!isWindowStateEqual(currentWindowState, nextWindowState)) {
		window.setBounds(nextWindowState);
	}

	return nextWindowState;
}

export function setMainWindowState(
	window: BrowserWindow,
	windowState: WindowState,
): WindowState {
	const nextWindowState = clampWindowStateToDisplay(
		windowState,
		getDisplayWorkAreaForWindowState(windowState),
	);
	const currentWindowState = getWindowStateFromWindow(window);

	if (!isWindowStateEqual(currentWindowState, nextWindowState)) {
		window.setBounds(nextWindowState);
	}

	return nextWindowState;
}

export function createWindowDragOffset(
	window: BrowserWindow,
	pointer: ScreenPoint,
): DragOffset {
	return getDragOffset(getWindowStateFromWindow(window), pointer);
}

export function bindMainWindowState(
	window: BrowserWindow,
	settingsStore: SettingsStoreService,
) {
	let isApplyingBounds = false;
	let persistTimer: ReturnType<typeof setTimeout> | undefined;

	const schedulePersist = (windowState: WindowState) => {
		if (persistTimer !== undefined) {
			clearTimeout(persistTimer);
		}

		persistTimer = setTimeout(() => {
			settingsStore.setWindowState(windowState);
		}, 120);
	};

	const syncWindowState = () => {
		if (isApplyingBounds) {
			return;
		}

		const currentWindowState = getWindowStateFromWindow(window);
		const clampedWindowState = clampWindowStateToDisplay(
			currentWindowState,
			getDisplayWorkAreaForWindowState(currentWindowState),
		);

		if (!isWindowStateEqual(currentWindowState, clampedWindowState)) {
			isApplyingBounds = true;
			window.setBounds(clampedWindowState);
			isApplyingBounds = false;
		}

		emitWindowState(window, clampedWindowState);
		schedulePersist(clampedWindowState);
	};

	window.on("move", syncWindowState);
	window.on("resize", () => {
		const settings = settingsStore.getSettings();
		applyMainWindowShape(window, {
			overlayShape: settings.overlayShape,
			cornerRoundness: settings.cornerRoundness,
		});
		syncWindowState();
	});
	window.on("close", () => {
		if (persistTimer !== undefined) {
			clearTimeout(persistTimer);
		}

		persistWindowStateNow(window, settingsStore);
	});

	syncWindowState();
}
