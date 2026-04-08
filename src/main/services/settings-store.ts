import Store from "electron-store";
import type { CamletSettings } from "../../shared/settings.js";
import { defaultCamletSettings } from "../../shared/settings.js";
import {
	SettingsStoreService as BaseSettingsStoreService,
	type SettingsStoreAdapter,
} from "./settings-store-core.js";

class ElectronSettingsStoreAdapter implements SettingsStoreAdapter {
	private readonly store = new Store<CamletSettings>({
		name: "settings",
		defaults: defaultCamletSettings,
	});

	read(): unknown {
		return this.store.store;
	}

	write(settings: CamletSettings): void {
		this.store.store = settings;
	}
}

export class SettingsStoreService extends BaseSettingsStoreService {
	constructor(
		adapter: SettingsStoreAdapter = new ElectronSettingsStoreAdapter(),
	) {
		super(adapter);
	}
}

export type { SettingsStoreAdapter } from "./settings-store-core.js";
