import type { AppBootstrap } from "../shared/bootstrap.js";
import { AppShell } from "./components/AppShell.js";
import { StartupErrorScreen } from "./components/StartupErrorScreen.js";
import { OverlayShellScreen } from "./features/overlay-shell/OverlayShellScreen.js";
import type { RendererStartupError } from "./startup.js";

interface AppProps {
	bootstrap: AppBootstrap | null;
	startupError: RendererStartupError | null;
}

export function App({ bootstrap, startupError }: AppProps) {
	return (
		<AppShell>
			{startupError !== null ? (
				<StartupErrorScreen error={startupError} />
			) : bootstrap !== null ? (
				<OverlayShellScreen bootstrap={bootstrap} />
			) : null}
		</AppShell>
	);
}
