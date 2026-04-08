import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Session } from "electron/main";

export interface RendererAssetPolicy {
	rendererHtmlPath: string;
	rendererUrl?: string;
}

interface PermissionRequestOptions {
	mediaTypes?: readonly ("video" | "audio")[];
}

export function createRendererAssetPolicy(
	rendererHtmlPath: string,
	rendererUrl?: string,
): RendererAssetPolicy {
	return rendererUrl === undefined
		? {
				rendererHtmlPath,
			}
		: {
				rendererHtmlPath,
				rendererUrl,
			};
}

function isLoopbackHost(hostname: string): boolean {
	return (
		hostname === "localhost" ||
		hostname === "127.0.0.1" ||
		hostname === "::1" ||
		hostname === "[::1]"
	);
}

function tryParseUrl(value: string): URL | null {
	try {
		return new URL(value);
	} catch {
		return null;
	}
}

function normalizeFilePath(value: string): string {
	return path.normalize(path.resolve(value));
}

function getAllowedDevOrigin(rendererUrl?: string): string | null {
	if (rendererUrl === undefined) {
		return null;
	}

	return new URL(rendererUrl).origin;
}

export function validateRendererUrl(rendererUrl?: string): string | undefined {
	if (rendererUrl === undefined) {
		return undefined;
	}

	const parsedUrl = tryParseUrl(rendererUrl);

	if (
		parsedUrl === null ||
		!["http:", "https:"].includes(parsedUrl.protocol) ||
		!isLoopbackHost(parsedUrl.hostname)
	) {
		throw new Error(
			"Camlet development renderer URL must use a local localhost or loopback origin",
		);
	}

	return parsedUrl.toString();
}

export function isTrustedRendererUrl(
	targetUrl: string,
	policy: RendererAssetPolicy,
): boolean {
	const parsedUrl = tryParseUrl(targetUrl);

	if (parsedUrl === null) {
		return false;
	}

	if (parsedUrl.protocol === "file:") {
		try {
			return (
				normalizeFilePath(fileURLToPath(parsedUrl)) ===
				normalizeFilePath(policy.rendererHtmlPath)
			);
		} catch {
			return false;
		}
	}

	const allowedOrigin = getAllowedDevOrigin(policy.rendererUrl);
	return allowedOrigin !== null && parsedUrl.origin === allowedOrigin;
}

export function isAllowedNavigationTarget(
	targetUrl: string,
	policy: RendererAssetPolicy,
): boolean {
	return isTrustedRendererUrl(targetUrl, policy);
}

export function shouldAllowPermission(
	permission: string,
	requestingUrl: string,
	policy: RendererAssetPolicy,
	options: PermissionRequestOptions = {},
): boolean {
	const parsedUrl = tryParseUrl(requestingUrl);
	const isPackagedFileOrigin =
		parsedUrl?.protocol === "file:" && policy.rendererUrl === undefined;

	if (!isPackagedFileOrigin && !isTrustedRendererUrl(requestingUrl, policy)) {
		return false;
	}

	if (permission !== "media") {
		return false;
	}

	if (options.mediaTypes === undefined) {
		return true;
	}

	return (
		options.mediaTypes.includes("video") &&
		!options.mediaTypes.includes("audio")
	);
}

export function configureSessionSecurity(
	session: Session,
	policy: RendererAssetPolicy,
) {
	session.setPermissionCheckHandler(
		(_webContents, permission, requestingOrigin) =>
			shouldAllowPermission(permission, requestingOrigin, policy),
	);
	session.setPermissionRequestHandler(
		(webContents, permission, callback, details) => {
			callback(
				shouldAllowPermission(
					permission,
					details.requestingUrl || webContents.getURL(),
					policy,
					"mediaTypes" in details
						? {
								mediaTypes: details.mediaTypes,
							}
						: undefined,
				),
			);
		},
	);
}
