import { t } from "../i18n.js";
import type { RendererStartupError } from "../startup.js";

interface StartupErrorScreenProps {
	error: RendererStartupError;
}

export interface StartupErrorView {
	element: HTMLElement;
}

export function createStartupErrorScreen({
	error,
}: StartupErrorScreenProps): StartupErrorView {
	const section = document.createElement("section");
	section.className = "startup-screen";
	section.setAttribute("aria-labelledby", "camlet-startup-error-title");

	const eyebrow = document.createElement("p");
	eyebrow.className = "startup-screen__eyebrow";
	eyebrow.textContent = t("app.title");

	const title = document.createElement("h1");
	title.className = "startup-screen__title";
	title.id = "camlet-startup-error-title";
	title.textContent = t(`startup.errors.${error.code}.title`);

	const copy = document.createElement("p");
	copy.className = "startup-screen__copy";
	copy.textContent = t(`startup.errors.${error.code}.message`);

	const reloadButton = document.createElement("button");
	reloadButton.className = "camlet-button";
	reloadButton.type = "button";
	reloadButton.textContent = t("startup.actions.reload");
	reloadButton.addEventListener("click", () => {
		window.location.reload();
	});

	section.append(eyebrow, title, copy, reloadButton);

	if (import.meta.env.DEV && error.detail !== null) {
		const details = document.createElement("details");
		details.className = "startup-screen__details";

		const summary = document.createElement("summary");
		summary.textContent = "Startup details";

		const code = document.createElement("code");
		code.textContent = error.detail;

		details.append(summary, code);
		section.append(details);
	}

	return {
		element: section,
	};
}
