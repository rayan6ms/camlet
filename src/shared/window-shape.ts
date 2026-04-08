import type { OverlayShape } from "./appearance.js";
import { getRoundedSquareRadius } from "./appearance.js";

interface ShapeRow {
	x: number;
	width: number;
}

export interface WindowShapeRectangle {
	x: number;
	y: number;
	width: number;
	height: number;
}

function clamp(value: number, min: number, max: number) {
	return Math.min(Math.max(value, min), max);
}

function toRow(x: number, width: number, maximumWidth: number): ShapeRow {
	const clampedX = clamp(Math.floor(x), 0, maximumWidth);
	const clampedRight = clamp(Math.ceil(x + width), clampedX, maximumWidth);

	return {
		x: clampedX,
		width: Math.max(1, clampedRight - clampedX),
	};
}

function getEllipseRow(
	width: number,
	height: number,
	rowCenterY: number,
): ShapeRow | null {
	const radiusX = width / 2;
	const radiusY = height / 2;
	const centerX = radiusX;
	const centerY = radiusY;
	const distanceY = Math.abs(rowCenterY - centerY);

	if (distanceY >= radiusY) {
		return null;
	}

	const horizontalRadius =
		radiusX * Math.sqrt(1 - (distanceY * distanceY) / (radiusY * radiusY));

	return toRow(centerX - horizontalRadius, horizontalRadius * 2, width);
}

function getRoundedSquareRow(
	width: number,
	height: number,
	rowCenterY: number,
): ShapeRow {
	const radius = Math.min(
		getRoundedSquareRadius(Math.min(width, height)),
		width / 2,
		height / 2,
	);
	const upperCurveLimit = radius;
	const lowerCurveLimit = height - radius;

	if (rowCenterY >= upperCurveLimit && rowCenterY <= lowerCurveLimit) {
		return {
			x: 0,
			width,
		};
	}

	const distanceFromCurve =
		rowCenterY < upperCurveLimit
			? upperCurveLimit - rowCenterY
			: rowCenterY - lowerCurveLimit;
	const horizontalInset =
		radius - Math.sqrt(radius * radius - distanceFromCurve * distanceFromCurve);

	return toRow(horizontalInset, width - horizontalInset * 2, width);
}

function compressRows(rows: Array<ShapeRow | null>): WindowShapeRectangle[] {
	const rectangles: WindowShapeRectangle[] = [];
	let activeRectangle: WindowShapeRectangle | null = null;

	for (const [index, row] of rows.entries()) {
		if (row === null) {
			activeRectangle = null;
			continue;
		}

		if (
			activeRectangle !== null &&
			activeRectangle.x === row.x &&
			activeRectangle.width === row.width &&
			activeRectangle.y + activeRectangle.height === index
		) {
			activeRectangle.height += 1;
			continue;
		}

		activeRectangle = {
			x: row.x,
			y: index,
			width: row.width,
			height: 1,
		};
		rectangles.push(activeRectangle);
	}

	return rectangles;
}

export function createWindowShapeRectangles(
	width: number,
	height: number,
	shape: OverlayShape,
): WindowShapeRectangle[] {
	if (width <= 0 || height <= 0) {
		return [];
	}

	const rows = Array.from({ length: height }, (_, y) => {
		const rowCenterY = y + 0.5;

		return shape === "circle"
			? getEllipseRow(width, height, rowCenterY)
			: getRoundedSquareRow(width, height, rowCenterY);
	});

	return compressRows(rows);
}
