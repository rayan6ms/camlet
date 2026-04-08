import { z } from "zod";

export const aboutInfoSchema = z.object({
	description: z.string().trim().min(1),
	scope: z.array(z.string().trim().min(1)),
	license: z.string().trim().min(1),
	githubHandle: z.string().trim().min(1),
	githubUrl: z.string().url(),
	projectUrl: z.string().url(),
	projectIssuesUrl: z.string().url(),
	avatarDataUrl: z.string().trim().min(1),
});

export type AboutInfo = z.infer<typeof aboutInfoSchema>;
