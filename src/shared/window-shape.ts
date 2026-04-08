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
	cornerRoundness: number,
): ShapeRow {
	const radius = Math.min(
		getRoundedSquareRadius(Math.min(width, height), cornerRoundness),
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

function getRectangleRow(
	width: number,
	height: number,
	rowCenterY: number,
	cornerRoundness: number,
	orientation: "x" | "y",
): ShapeRow | null {
	const horizontalInset = orientation === "y" ? width * 0.16 : 0;
	const verticalInset = orientation === "x" ? height * 0.16 : 0;
	const innerWidth = width - horizontalInset * 2;
	const innerHeight = height - verticalInset * 2;
	const shiftedRowCenterY = rowCenterY - verticalInset;

	if (shiftedRowCenterY < 0 || shiftedRowCenterY > innerHeight) {
		return null;
	}

	const radius = Math.min(
		getRoundedSquareRadius(Math.min(innerWidth, innerHeight), cornerRoundness),
		innerWidth / 2,
		innerHeight / 2,
	);
	const upperCurveLimit = radius;
	const lowerCurveLimit = innerHeight - radius;

	if (
		shiftedRowCenterY >= upperCurveLimit &&
		shiftedRowCenterY <= lowerCurveLimit
	) {
		return toRow(horizontalInset, innerWidth, width);
	}

	const distanceFromCurve =
		shiftedRowCenterY < upperCurveLimit
			? upperCurveLimit - shiftedRowCenterY
			: shiftedRowCenterY - lowerCurveLimit;
	const curveInset =
		radius - Math.sqrt(radius * radius - distanceFromCurve * distanceFromCurve);

	return toRow(
		horizontalInset + curveInset,
		innerWidth - curveInset * 2,
		width,
	);
}

function getDiamondRow(
	width: number,
	height: number,
	rowCenterY: number,
	cornerRoundness: number,
): ShapeRow | null {
	const centerX = width / 2;
	const centerY = height / 2;
	const sideLength = Math.min(width, height) / Math.SQRT2;
	const halfSide = sideLength / 2;
	const radius = Math.min(Math.max(0, cornerRoundness), halfSide);
	let left = Number.POSITIVE_INFINITY;
	let right = Number.NEGATIVE_INFINITY;

	for (let x = 0; x < width; x += 1) {
		const dx = x + 0.5 - centerX;
		const dy = rowCenterY - centerY;
		const localX = (dx + dy) / Math.SQRT2;
		const localY = (-dx + dy) / Math.SQRT2;
		const qx = Math.abs(localX) - (halfSide - radius);
		const qy = Math.abs(localY) - (halfSide - radius);
		const outsideX = Math.max(qx, 0);
		const outsideY = Math.max(qy, 0);
		const inside =
			(qx <= 0 && qy <= 0) || Math.hypot(outsideX, outsideY) <= radius;

		if (!inside) {
			continue;
		}

		left = Math.min(left, x);
		right = Math.max(right, x + 1);
	}

	if (!Number.isFinite(left) || !Number.isFinite(right)) {
		return null;
	}

	return toRow(left, right - left, width);
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
	cornerRoundness = 26,
): WindowShapeRectangle[] {
	if (width <= 0 || height <= 0) {
		return [];
	}

	const rows = Array.from({ length: height }, (_, y) => {
		const rowCenterY = y + 0.5;

		switch (shape) {
			case "circle":
				return getEllipseRow(width, height, rowCenterY);
			case "rounded-square":
				return getRoundedSquareRow(width, height, rowCenterY, cornerRoundness);
			case "rectangle-y":
				return getRectangleRow(width, height, rowCenterY, cornerRoundness, "y");
			case "rectangle-x":
				return getRectangleRow(width, height, rowCenterY, cornerRoundness, "x");
			case "diamond":
				return getDiamondRow(width, height, rowCenterY, cornerRoundness);
		}

		return null;
	});

	return compressRows(rows);
}
