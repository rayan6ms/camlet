import { arraySchema, objectSchema, stringSchema } from "./validation.js";

export interface AboutInfo {
	description: string;
	scope: string[];
	license: string;
	githubHandle: string;
	githubUrl: string;
	projectUrl: string;
	projectIssuesUrl: string;
	avatarDataUrl: string;
}

export const aboutInfoSchema = objectSchema({
	description: stringSchema({ trim: true, minLength: 1 }),
	scope: arraySchema(stringSchema({ trim: true, minLength: 1 })),
	license: stringSchema({ trim: true, minLength: 1 }),
	githubHandle: stringSchema({ trim: true, minLength: 1 }),
	githubUrl: stringSchema({ url: true }),
	projectUrl: stringSchema({ url: true }),
	projectIssuesUrl: stringSchema({ url: true }),
	avatarDataUrl: stringSchema({ trim: true, minLength: 1 }),
});
