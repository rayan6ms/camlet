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
		const rectangles = createWindowShapeRectangles(120, 120, "rounded-square");

		expect(rectangles[0]?.width).toBeLessThan(120);
		expect(
			rectangles.some(
				(rectangle) => rectangle.x === 0 && rectangle.width === 120,
			),
		).toBe(true);
		expect(rectangles.at(-1)?.width).toBe(rectangles[0]?.width);
	});
});
