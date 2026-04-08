import type { AppBootstrapIssue } from "../../shared/bootstrap.js";
import type { AppLanguage } from "../../shared/language.js";
import {
	applyCamletSettingsPatch,
	applyOverlayAppearanceSettingsPatch,
	type CamletSettings,
	type CamletSettingsPatch,
	camletSettingsSchema,
	defaultCamletSettings,
	mergeCamletSettings,
	type OverlayAppearanceSettingsPatch,
} from "../../shared/settings.js";
import type { WindowState } from "../../shared/window-state.js";

export interface SettingsStoreAdapter {
	read(): unknown;
	write(settings: CamletSettings): void;
}

function cloneDefaultSettings(): CamletSettings {
	return mergeCamletSettings(defaultCamletSettings);
}

function cloneSettings(settings: CamletSettings): CamletSettings {
	return mergeCamletSettings(settings);
}

export class SettingsStoreService {
	private cachedSettings = cloneDefaultSettings();
	private persistenceHealthy = true;
	private readonly bootstrapIssues = new Set<AppBootstrapIssue>();

	constructor(private readonly adapter: SettingsStoreAdapter) {
		const repairedSettings = this.readSettingsSnapshot();
		this.persistSettingsSnapshot(repairedSettings);
	}

	private readSettingsSnapshot(): CamletSettings {
		if (!this.persistenceHealthy) {
			return cloneSettings(this.cachedSettings);
		}

		try {
			const persistedSettings = this.adapter.read();

			if (!camletSettingsSchema.safeParse(persistedSettings).success) {
				this.bootstrapIssues.add("settings-recovered");
			}

			const nextSettings = mergeCamletSettings(persistedSettings);
			this.cachedSettings = cloneSettings(nextSettings);
			return nextSettings;
		} catch (error) {
			this.bootstrapIssues.add("settings-recovered");
			console.error(
				"Failed to read persisted Camlet settings, falling back to cached defaults.",
				error,
			);
			return cloneSettings(this.cachedSettings);
		}
	}

	private persistSettingsSnapshot(settings: CamletSettings): CamletSettings {
		const nextSettings = cloneSettings(settings);
		this.cachedSettings = nextSettings;

		try {
			this.adapter.write(nextSettings);
			this.persistenceHealthy = true;
		} catch (error) {
			this.persistenceHealthy = false;
			this.bootstrapIssues.add("settings-persistence-unavailable");
			console.error(
				"Failed to persist Camlet settings, continuing with in-memory settings.",
				error,
			);
		}

		return cloneSettings(nextSettings);
	}

	getSettings(): CamletSettings {
		return this.persistSettingsSnapshot(this.readSettingsSnapshot());
	}

	getBootstrapIssues(): AppBootstrapIssue[] {
		return [...this.bootstrapIssues];
	}

	updateSettings(patch: CamletSettingsPatch): CamletSettings {
		const nextSettings = applyCamletSettingsPatch(this.getSettings(), patch);
		return this.persistSettingsSnapshot(nextSettings);
	}

	setLanguage(language: AppLanguage): CamletSettings {
		return this.updateSettings({ language });
	}

	setSelectedCameraDeviceId(deviceId: string | null): CamletSettings {
		return this.updateSettings({ selectedCameraDeviceId: deviceId });
	}

	updateOverlayAppearanceSettings(
		patch: OverlayAppearanceSettingsPatch,
	): CamletSettings {
		const nextSettings = applyOverlayAppearanceSettingsPatch(
			this.getSettings(),
			patch,
		);
		return this.persistSettingsSnapshot(nextSettings);
	}

	getWindowState(): WindowState {
		return this.getSettings().window;
	}

	setWindowState(window: WindowState): CamletSettings {
		return this.updateSettings({ window });
	}
}
