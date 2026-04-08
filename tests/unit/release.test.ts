import { describe, expect, it } from "vitest";
import {
	isPrereleaseVersion,
	isReleaseTag,
	parseReleaseTag,
} from "../../src/shared/release.js";

describe("release tag helpers", () => {
	it("parses stable release tags", () => {
		expect(parseReleaseTag("v0.1.0")).toEqual({
			tag: "v0.1.0",
			version: "0.1.0",
			channel: "stable",
		});
	});

	it("parses prerelease tags", () => {
		expect(parseReleaseTag("v0.2.0-beta.1")).toEqual({
			tag: "v0.2.0-beta.1",
			version: "0.2.0-beta.1",
			channel: "prerelease",
		});
	});

	it("accepts build metadata in valid tags", () => {
		expect(parseReleaseTag("v1.0.0+build.5")).toEqual({
			tag: "v1.0.0+build.5",
			version: "1.0.0+build.5",
			channel: "stable",
		});
	});

	it("rejects malformed release tags", () => {
		expect(parseReleaseTag("0.1.0")).toBeNull();
		expect(parseReleaseTag("v1.0")).toBeNull();
		expect(parseReleaseTag("vx.y.z")).toBeNull();
		expect(isReleaseTag("release-1.0.0")).toBe(false);
	});

	it("detects prerelease versions by semantic version suffix", () => {
		expect(isPrereleaseVersion("0.2.0-beta.1")).toBe(true);
		expect(isPrereleaseVersion("0.2.0")).toBe(false);
	});
});
