import { mountApp } from "./App.js";
import { initializeI18n } from "./i18n.js";
import { resolveRendererStartupState } from "./startup.js";
import "./styles/app.css";

const rootElement = document.getElementById("root");

if (rootElement === null) {
	throw new Error("Renderer root element was not found");
}

const appRoot = rootElement;

async function bootstrapRenderer() {
	const startupState = await resolveRendererStartupState({
		camletApi: window.camlet,
		preferredLanguage: navigator.language,
	});
	await initializeI18n(startupState.language);
	mountApp({
		bootstrap: startupState.bootstrap,
		rootElement: appRoot,
		startupError: startupState.startupError,
	});
}

void bootstrapRenderer().catch((error) => {
	console.error("Renderer bootstrap failed", error);
	rootElement.textContent = "Camlet failed to start.";
});
