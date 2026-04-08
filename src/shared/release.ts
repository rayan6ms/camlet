export interface ParsedReleaseTag {
	tag: string;
	version: string;
	channel: "stable" | "prerelease";
}

const semverCorePattern = String.raw`(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)`;
const semverPrereleasePattern = String.raw`(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?`;
const semverBuildPattern = String.raw`(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?`;
const releaseTagPattern = new RegExp(
	`^v(?<version>${semverCorePattern}${semverPrereleasePattern}${semverBuildPattern})$`,
);

export function isPrereleaseVersion(version: string): boolean {
	return version.includes("-");
}

export function parseReleaseTag(tag: string): ParsedReleaseTag | null {
	const match = releaseTagPattern.exec(tag);
	const version = match?.groups?.version;

	if (version === undefined) {
		return null;
	}

	return {
		tag,
		version,
		channel: isPrereleaseVersion(version) ? "prerelease" : "stable",
	};
}

export function isReleaseTag(tag: string): boolean {
	return parseReleaseTag(tag) !== null;
}
