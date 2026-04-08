import { afterEach, describe, expect, it, vi } from "vitest";
import {
	type SettingsStoreAdapter,
	SettingsStoreService,
} from "../../src/main/services/settings-store-core.js";
import type { CamletSettings } from "../../src/shared/settings.js";
import { defaultCamletSettings } from "../../src/shared/settings.js";

class FakeSettingsStoreAdapter implements SettingsStoreAdapter {
	readonly writes: CamletSettings[] = [];
	throwOnRead = false;
	throwOnWrite = false;

	constructor(private currentValue: unknown) {}

	read(): unknown {
		if (this.throwOnRead) {
			throw new Error("read failed");
		}

		return this.currentValue;
	}

	write(settings: CamletSettings): void {
		this.writes.push(settings);

		if (this.throwOnWrite) {
			throw new Error("write failed");
		}

		this.currentValue = structuredClone(settings);
	}
}

afterEach(() => {
	vi.restoreAllMocks();
});

describe("settings store integration", () => {
	it("repairs malformed persisted settings on startup", () => {
		const adapter = new FakeSettingsStoreAdapter({
			language: "de-DE",
			selectedCameraDeviceId: "",
			ringColor: "green",
			window: {
				x: 24,
				width: 80,
			},
		});
		const service = new SettingsStoreService(adapter);
		const repairedSettings = service.getSettings();

		expect(repairedSettings.language).toBe(defaultCamletSettings.language);
		expect(repairedSettings.selectedCameraDeviceId).toBeNull();
		expect(repairedSettings.ringColor).toBe(defaultCamletSettings.ringColor);
		expect(repairedSettings.window).toEqual({
			...defaultCamletSettings.window,
			x: 24,
		});
		expect(adapter.writes.at(-1)).toEqual(repairedSettings);
	});

	it("continues from cached settings when persisted reads fail later", () => {
		const adapter = new FakeSettingsStoreAdapter({
			...defaultCamletSettings,
			overlaySize: 280,
		});
		const service = new SettingsStoreService(adapter);
		vi.spyOn(console, "error").mockImplementation(() => {});
		adapter.throwOnRead = true;

		const nextSettings = service.updateSettings({
			ringThickness: 14,
		});

		expect(nextSettings.overlaySize).toBe(280);
		expect(nextSettings.ringThickness).toBe(10);
		expect(adapter.writes.at(-1)?.ringThickness).toBe(10);
	});

	it("keeps serving in-memory settings when disk writes fail", () => {
		const adapter = new FakeSettingsStoreAdapter(defaultCamletSettings);
		const service = new SettingsStoreService(adapter);
		vi.spyOn(console, "error").mockImplementation(() => {});
		adapter.throwOnWrite = true;

		const nextSettings = service.setLanguage("pt-BR");

		expect(nextSettings.language).toBe("pt-BR");
		expect(service.getSettings().language).toBe("pt-BR");
	});
});
