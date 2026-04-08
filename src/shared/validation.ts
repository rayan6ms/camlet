export type ValidationPath = Array<string | number>;

export interface ValidationIssue {
	path: ValidationPath;
	message: string;
}

export class ValidationError extends Error {
	constructor(public readonly issues: ValidationIssue[]) {
		super(
			issues.length > 0
				? issues
						.map((issue) =>
							issue.path.length > 0
								? `${issue.path.join(".")}: ${issue.message}`
								: issue.message,
						)
						.join("; ")
				: "Validation failed",
		);
		this.name = "ValidationError";
	}
}

export type SafeParseResult<T> =
	| {
			success: true;
			data: T;
	  }
	| {
			success: false;
			error: ValidationError;
	  };

export interface Schema<T> {
	parse(value: unknown): T;
	safeParse(value: unknown): SafeParseResult<T>;
	validate(value: unknown, path: ValidationPath): T;
}

type SchemaShape = Record<string, Schema<unknown>>;

type InferSchemaType<TSchema> =
	TSchema extends Schema<infer TValue> ? TValue : never;

type InferShape<TShape extends SchemaShape> = {
	[TKey in keyof TShape]: InferSchemaType<TShape[TKey]>;
};

type StringKeyOf<T> = Extract<keyof T, string>;

function getObjectKeys<T extends Record<string, unknown>>(
	value: T,
): Array<StringKeyOf<T>> {
	return Object.keys(value) as Array<StringKeyOf<T>>;
}

function createSchema<T>(
	validate: (value: unknown, path: ValidationPath) => T,
): Schema<T> {
	return {
		parse(value) {
			return validate(value, []);
		},
		safeParse(value) {
			try {
				return {
					success: true,
					data: validate(value, []),
				};
			} catch (error) {
				if (error instanceof ValidationError) {
					return {
						success: false,
						error,
					};
				}

				throw error;
			}
		},
		validate,
	};
}

function throwIssue(path: ValidationPath, message: string): never {
	throw new ValidationError([{ path, message }]);
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function enumSchema<const TValues extends readonly string[]>(
	values: TValues,
): Schema<TValues[number]> {
	return createSchema((value, path) => {
		if (typeof value !== "string") {
			throwIssue(path, "Expected a string");
		}

		if (!values.includes(value)) {
			throwIssue(path, `Expected one of: ${values.join(", ")}`);
		}

		return value;
	});
}

export function stringSchema(options?: {
	trim?: boolean;
	minLength?: number;
	pattern?: RegExp;
	url?: boolean;
}): Schema<string> {
	return createSchema((value, path) => {
		if (typeof value !== "string") {
			throwIssue(path, "Expected a string");
		}

		const nextValue = options?.trim ? value.trim() : value;

		if (
			options?.minLength !== undefined &&
			nextValue.length < options.minLength
		) {
			throwIssue(
				path,
				`String must have at least ${options.minLength} character${options.minLength === 1 ? "" : "s"}`,
			);
		}

		if (options?.pattern !== undefined && !options.pattern.test(nextValue)) {
			throwIssue(path, "Invalid format");
		}

		if (options?.url) {
			try {
				new URL(nextValue);
			} catch {
				throwIssue(path, "Invalid URL");
			}
		}

		return nextValue;
	});
}

export function numberSchema(options?: {
	integer?: boolean;
	min?: number;
	max?: number;
}): Schema<number> {
	return createSchema((value, path) => {
		if (typeof value !== "number" || !Number.isFinite(value)) {
			throwIssue(path, "Expected a number");
		}

		if (options?.integer && !Number.isInteger(value)) {
			throwIssue(path, "Expected an integer");
		}

		if (options?.min !== undefined && value < options.min) {
			throwIssue(path, `Number must be at least ${options.min}`);
		}

		if (options?.max !== undefined && value > options.max) {
			throwIssue(path, `Number must be at most ${options.max}`);
		}

		return value;
	});
}

export function booleanSchema(): Schema<boolean> {
	return createSchema((value, path) => {
		if (typeof value !== "boolean") {
			throwIssue(path, "Expected a boolean");
		}

		return value;
	});
}

export function nullableSchema<T>(schema: Schema<T>): Schema<T | null> {
	return createSchema((value, path) => {
		if (value === null) {
			return null;
		}

		return schema.validate(value, path);
	});
}

export function arraySchema<T>(itemSchema: Schema<T>): Schema<T[]> {
	return createSchema((value, path) => {
		if (!Array.isArray(value)) {
			throwIssue(path, "Expected an array");
		}

		const nextValue: T[] = [];
		const issues: ValidationIssue[] = [];

		for (const [index, item] of value.entries()) {
			try {
				nextValue.push(itemSchema.validate(item, [...path, index]));
			} catch (error) {
				if (error instanceof ValidationError) {
					issues.push(...error.issues);
					continue;
				}

				throw error;
			}
		}

		if (issues.length > 0) {
			throw new ValidationError(issues);
		}

		return nextValue;
	});
}

export function objectSchema<TShape extends SchemaShape>(
	shape: TShape,
): Schema<InferShape<TShape>> {
	return createSchema((value, path) => {
		if (!isRecord(value)) {
			throwIssue(path, "Expected an object");
		}

		const nextValue = {} as InferShape<TShape>;
		const issues: ValidationIssue[] = [];

		for (const key of getObjectKeys(shape)) {
			const schema = shape[key];

			if (schema === undefined) {
				continue;
			}

			try {
				nextValue[key as keyof TShape] = schema.validate(value[key], [
					...path,
					key,
				]) as InferShape<TShape>[keyof TShape];
			} catch (error) {
				if (error instanceof ValidationError) {
					issues.push(...error.issues);
					continue;
				}

				throw error;
			}
		}

		if (issues.length > 0) {
			throw new ValidationError(issues);
		}

		return nextValue;
	});
}

export function partialObjectSchema<TShape extends SchemaShape>(
	shape: TShape,
): Schema<Partial<InferShape<TShape>>> {
	return createSchema((value, path) => {
		if (!isRecord(value)) {
			throwIssue(path, "Expected an object");
		}

		const nextValue: Partial<InferShape<TShape>> = {};
		const issues: ValidationIssue[] = [];

		for (const key of getObjectKeys(shape)) {
			if (!(key in value) || value[key] === undefined) {
				continue;
			}

			const schema = shape[key];

			if (schema === undefined) {
				continue;
			}

			try {
				nextValue[key as keyof TShape] = schema.validate(value[key], [
					...path,
					key,
				]) as InferShape<TShape>[keyof TShape];
			} catch (error) {
				if (error instanceof ValidationError) {
					issues.push(...error.issues);
					continue;
				}

				throw error;
			}
		}

		if (issues.length > 0) {
			throw new ValidationError(issues);
		}

		return nextValue;
	});
}
