import { describe, expect, it } from "vitest";
import {
	clampWindowStateToDisplay,
	defaultWindowState,
	getDragOffset,
	isWindowStateEqual,
	mergeWindowState,
	moveWindowStateWithPointer,
	resizeSquareWindowStateWithPointer,
} from "../../src/shared/window-state.js";

describe("window state defaults and validation", () => {
	it("uses defaults when stored window state is missing", () => {
		expect(mergeWindowState(undefined)).toEqual(defaultWindowState);
	});

	it("recovers from invalid persisted dimensions", () => {
		expect(
			mergeWindowState({
				x: -40,
				y: 30,
				width: 120,
				height: 0,
			}),
		).toEqual({
			x: -40,
			y: 30,
			width: 120,
			height: defaultWindowState.height,
		});
	});
});

describe("window state clamping", () => {
	it("clamps oversized and off-screen bounds into the work area", () => {
		expect(
			clampWindowStateToDisplay(
				{
					x: 4000,
					y: -500,
					width: 900,
					height: 900,
				},
				{
					x: 0,
					y: 0,
					width: 1280,
					height: 720,
				},
			),
		).toEqual({
			x: 380,
			y: 0,
			width: 900,
			height: 720,
		});
	});

	it("fully contains the window inside a smaller work area", () => {
		expect(
			clampWindowStateToDisplay(
				{
					x: -300,
					y: -200,
					width: 360,
					height: 360,
				},
				{
					x: 0,
					y: 0,
					width: 300,
					height: 260,
				},
			),
		).toEqual({
			x: 0,
			y: 0,
			width: 300,
			height: 260,
		});
	});
});

describe("window drag math", () => {
	it("keeps the original pointer offset during dragging", () => {
		const offset = getDragOffset(defaultWindowState, {
			screenX: 180,
			screenY: 210,
		});

		expect(offset).toEqual({
			x: 132,
			y: 162,
		});

		expect(
			moveWindowStateWithPointer(
				defaultWindowState,
				{
					screenX: 320,
					screenY: 350,
				},
				offset,
			),
		).toEqual({
			...defaultWindowState,
			x: 188,
			y: 188,
		});
	});

	it("compares window states structurally", () => {
		expect(
			isWindowStateEqual(defaultWindowState, { ...defaultWindowState }),
		).toBe(true);
		expect(
			isWindowStateEqual(defaultWindowState, {
				...defaultWindowState,
				x: defaultWindowState.x + 1,
			}),
		).toBe(false);
	});
});

describe("window resize math", () => {
	it("expands the square window from the bottom-right handle", () => {
		expect(
			resizeSquareWindowStateWithPointer(
				defaultWindowState,
				{
					screenX: defaultWindowState.x + defaultWindowState.width,
					screenY: defaultWindowState.y + defaultWindowState.height,
				},
				{
					screenX: defaultWindowState.x + defaultWindowState.width + 60,
					screenY: defaultWindowState.y + defaultWindowState.height + 24,
				},
				"se",
			),
		).toEqual({
			...defaultWindowState,
			width: 284,
			height: 284,
		});
	});

	it("keeps the opposite corner fixed when resizing from the top-left", () => {
		expect(
			resizeSquareWindowStateWithPointer(
				defaultWindowState,
				{
					screenX: 48,
					screenY: 48,
				},
				{
					screenX: -8,
					screenY: -8,
				},
				"nw",
			),
		).toEqual({
			x: -8,
			y: -8,
			width: 280,
			height: 280,
		});
	});
});
