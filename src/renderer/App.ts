import type { AppBootstrap } from "../shared/bootstrap.js";
import {
	createAboutScreen,
	type MountedScreen,
} from "./components/AboutScreen.js";
import { createStartupErrorScreen } from "./components/StartupErrorScreen.js";
import { createOverlayShellScreen } from "./features/overlay-shell/OverlayShellScreen.js";
import type { RendererStartupError } from "./startup.js";

interface AppProps {
	bootstrap: AppBootstrap | null;
	rootElement: HTMLElement;
	startupError: RendererStartupError | null;
}

export function mountApp({
	bootstrap,
	rootElement,
	startupError,
}: AppProps): () => void {
	const isAboutRoute = window.location.hash === "#about";
	document.title = isAboutRoute ? "About Camlet" : "Camlet";

	const shell = document.createElement("main");
	shell.className = "app-shell";

	let mountedScreen: MountedScreen | null = null;

	if (startupError !== null) {
		shell.append(createStartupErrorScreen({ error: startupError }).element);
	} else if (bootstrap !== null) {
		mountedScreen = isAboutRoute
			? createAboutScreen({ bootstrap })
			: createOverlayShellScreen({ bootstrap });
		shell.append(mountedScreen.element);
	}

	rootElement.replaceChildren(shell);

	return () => {
		mountedScreen?.destroy();
	};
}
