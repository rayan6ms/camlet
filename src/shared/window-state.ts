import { isRecord } from "./language.js";
import { numberSchema, objectSchema, type Schema } from "./validation.js";

export const minimumWindowWidth = 176;
export const minimumWindowHeight = 176;
export const defaultWindowState = {
	x: 48,
	y: 48,
	width: 224,
	height: 224,
} as const;
const windowCoordinateSchema = numberSchema({ integer: true });
const windowWidthSchema = numberSchema({
	integer: true,
	min: minimumWindowWidth,
});
const windowHeightSchema = numberSchema({
	integer: true,
	min: minimumWindowHeight,
});

export interface ScreenPoint {
	screenX: number;
	screenY: number;
}

export interface WindowState {
	x: number;
	y: number;
	width: number;
	height: number;
}

export const screenPointSchema = objectSchema({
	screenX: windowCoordinateSchema,
	screenY: windowCoordinateSchema,
});

export const windowStateSchema = objectSchema({
	x: windowCoordinateSchema,
	y: windowCoordinateSchema,
	width: windowWidthSchema,
	height: windowHeightSchema,
});

export interface DisplayWorkArea {
	x: number;
	y: number;
	width: number;
	height: number;
}

export const displayWorkAreaSchema = objectSchema({
	x: windowCoordinateSchema,
	y: windowCoordinateSchema,
	width: numberSchema({ integer: true, min: 1 }),
	height: numberSchema({ integer: true, min: 1 }),
});

export interface DragOffset {
	x: number;
	y: number;
}

export type ResizeHandle = "nw" | "ne" | "se" | "sw";
export const resizeStep = 24;

function clamp(value: number, min: number, max: number) {
	return Math.min(Math.max(value, min), max);
}

function roundWindowState(state: WindowState): WindowState {
	return {
		x: Math.round(state.x),
		y: Math.round(state.y),
		width: Math.round(state.width),
		height: Math.round(state.height),
	};
}

function parseWindowStateValue<T>(
	schema: Schema<T>,
	value: unknown,
	fallback: T,
): T {
	const result = schema.safeParse(value);
	return result.success ? result.data : fallback;
}

export function mergeWindowState(value: unknown): WindowState {
	if (!isRecord(value)) {
		return { ...defaultWindowState };
	}

	return roundWindowState({
		x: parseWindowStateValue(
			windowCoordinateSchema,
			value.x,
			defaultWindowState.x,
		),
		y: parseWindowStateValue(
			windowCoordinateSchema,
			value.y,
			defaultWindowState.y,
		),
		width: parseWindowStateValue(
			windowWidthSchema,
			value.width,
			defaultWindowState.width,
		),
		height: parseWindowStateValue(
			windowHeightSchema,
			value.height,
			defaultWindowState.height,
		),
	});
}

export function clampWindowStateToDisplay(
	state: WindowState,
	workArea: DisplayWorkArea,
): WindowState {
	const minimumWidth = Math.min(minimumWindowWidth, workArea.width);
	const minimumHeight = Math.min(minimumWindowHeight, workArea.height);
	const width = clamp(
		state.width,
		minimumWidth,
		Math.max(minimumWidth, workArea.width),
	);
	const height = clamp(
		state.height,
		minimumHeight,
		Math.max(minimumHeight, workArea.height),
	);

	return {
		x: clamp(state.x, workArea.x, workArea.x + workArea.width - width),
		y: clamp(state.y, workArea.y, workArea.y + workArea.height - height),
		width,
		height,
	};
}

export function getMaximumSquareWindowSize(workArea: DisplayWorkArea): number {
	return Math.max(
		Math.min(workArea.width, workArea.height),
		Math.min(minimumWindowWidth, minimumWindowHeight),
	);
}

export function getDragOffset(
	windowState: WindowState,
	pointer: ScreenPoint,
): DragOffset {
	return {
		x: pointer.screenX - windowState.x,
		y: pointer.screenY - windowState.y,
	};
}

export function moveWindowStateWithPointer(
	windowState: WindowState,
	pointer: ScreenPoint,
	offset: DragOffset,
): WindowState {
	return {
		...windowState,
		x: pointer.screenX - offset.x,
		y: pointer.screenY - offset.y,
	};
}

export function isWindowStateEqual(a: WindowState, b: WindowState): boolean {
	return (
		a.x === b.x && a.y === b.y && a.width === b.width && a.height === b.height
	);
}

function getResizeDelta(
	handle: ResizeHandle,
	pointerDeltaX: number,
	pointerDeltaY: number,
) {
	switch (handle) {
		case "se":
			return Math.abs(pointerDeltaX) >= Math.abs(pointerDeltaY)
				? pointerDeltaX
				: pointerDeltaY;
		case "nw":
			return Math.abs(pointerDeltaX) >= Math.abs(pointerDeltaY)
				? -pointerDeltaX
				: -pointerDeltaY;
		case "ne":
			return Math.abs(pointerDeltaX) >= Math.abs(pointerDeltaY)
				? pointerDeltaX
				: -pointerDeltaY;
		case "sw":
			return Math.abs(pointerDeltaX) >= Math.abs(pointerDeltaY)
				? -pointerDeltaX
				: pointerDeltaY;
	}
}

export function resizeSquareWindowStateWithPointer(
	windowState: WindowState,
	startPointer: ScreenPoint,
	nextPointer: ScreenPoint,
	handle: ResizeHandle,
): WindowState {
	const pointerDeltaX = nextPointer.screenX - startPointer.screenX;
	const pointerDeltaY = nextPointer.screenY - startPointer.screenY;
	const size = Math.max(
		minimumWindowWidth,
		windowState.width + getResizeDelta(handle, pointerDeltaX, pointerDeltaY),
	);
	const right = windowState.x + windowState.width;
	const bottom = windowState.y + windowState.height;

	switch (handle) {
		case "se":
			return {
				...windowState,
				width: size,
				height: size,
			};
		case "nw":
			return {
				x: right - size,
				y: bottom - size,
				width: size,
				height: size,
			};
		case "ne":
			return {
				...windowState,
				y: bottom - size,
				width: size,
				height: size,
			};
		case "sw":
			return {
				...windowState,
				x: right - size,
				width: size,
				height: size,
			};
	}
}

export function resizeSquareWindowStateByDelta(
	windowState: WindowState,
	delta: number,
	maximumSize?: number,
): WindowState {
	const size = clamp(
		windowState.width + delta,
		minimumWindowWidth,
		maximumSize ?? Number.POSITIVE_INFINITY,
	);
	const centerX = windowState.x + windowState.width / 2;
	const centerY = windowState.y + windowState.height / 2;

	return {
		x: Math.round(centerX - size / 2),
		y: Math.round(centerY - size / 2),
		width: size,
		height: size,
	};
}
