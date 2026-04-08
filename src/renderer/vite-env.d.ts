/// <reference types="vite/client" />

import type { CamletApi } from "../shared/ipc.js";

declare global {
	interface Window {
		camlet: CamletApi;
	}
}
