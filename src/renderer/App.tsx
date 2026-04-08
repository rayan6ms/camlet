import { useEffect } from "react";
import type { AppBootstrap } from "../shared/bootstrap.js";
import { AboutScreen } from "./components/AboutScreen.js";
import { AppShell } from "./components/AppShell.js";
import { StartupErrorScreen } from "./components/StartupErrorScreen.js";
import { OverlayShellScreen } from "./features/overlay-shell/OverlayShellScreen.js";
import type { RendererStartupError } from "./startup.js";

interface AppProps {
	bootstrap: AppBootstrap | null;
	startupError: RendererStartupError | null;
}

export function App({ bootstrap, startupError }: AppProps) {
	const isAboutRoute = window.location.hash === "#about";

	useEffect(() => {
		document.title = isAboutRoute ? "About Camlet" : "Camlet";
	}, [isAboutRoute]);

	return (
		<AppShell>
			{startupError !== null ? (
				<StartupErrorScreen error={startupError} />
			) : bootstrap !== null ? (
				isAboutRoute ? (
					<AboutScreen bootstrap={bootstrap} />
				) : (
					<OverlayShellScreen bootstrap={bootstrap} />
				)
			) : null}
		</AppShell>
	);
}
