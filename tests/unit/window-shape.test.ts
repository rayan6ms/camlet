import { describe, expect, it } from "vitest";
import { createWindowShapeRectangles } from "../../src/shared/window-shape.js";

describe("window shape rectangles", () => {
	it("creates a clipped circular shape", () => {
		const rectangles = createWindowShapeRectangles(120, 120, "circle");

		expect(rectangles.length).toBeGreaterThan(1);
		expect(rectangles[0]).toMatchObject({
			y: 0,
			height: expect.any(Number),
		});
		expect(rectangles[0]?.width).toBeLessThan(120);
		expect(rectangles.at(-1)?.width).toBe(rectangles[0]?.width);
		expect(
			rectangles.some(
				(rectangle) => rectangle.x === 0 && rectangle.width === 120,
			),
		).toBe(true);
	});

	it("creates a rounded-square shape with full-width middle rows", () => {
		const rectangles = createWindowShapeRectangles(
			120,
			120,
			"rounded-square",
			24,
		);

		expect(rectangles[0]?.width).toBeLessThan(120);
		expect(
			rectangles.some(
				(rectangle) => rectangle.x === 0 && rectangle.width === 120,
			),
		).toBe(true);
		expect(rectangles.at(-1)?.width).toBe(rectangles[0]?.width);
	});

	it("creates a centered portrait rectangle shape", () => {
		const rectangles = createWindowShapeRectangles(120, 120, "rectangle-y", 18);

		expect(rectangles.every((rectangle) => rectangle.width < 120)).toBe(true);
		expect(rectangles[0]?.x).toBeGreaterThan(0);
	});

	it("creates a centered landscape rectangle shape", () => {
		const rectangles = createWindowShapeRectangles(120, 120, "rectangle-x", 18);

		expect(rectangles[0]?.y).toBeGreaterThan(0);
		expect(
			rectangles.some(
				(rectangle) => rectangle.x === 0 && rectangle.width === 120,
			),
		).toBe(true);
	});

	it("creates a diamond shape with a narrow top and bottom", () => {
		const rectangles = createWindowShapeRectangles(120, 120, "diamond", 0);

		expect(rectangles[0]?.width).toBeLessThan(30);
		expect(
			rectangles.some(
				(rectangle) => rectangle.x <= 1 && rectangle.width >= 118,
			),
		).toBe(true);
	});

	it("applies roundness to the diamond shape rows", () => {
		const sharp = createWindowShapeRectangles(120, 120, "diamond", 0);
		const rounded = createWindowShapeRectangles(120, 120, "diamond", 28);

		expect(rounded[0]?.width).toBeGreaterThan(sharp[0]?.width ?? 0);
	});
});
