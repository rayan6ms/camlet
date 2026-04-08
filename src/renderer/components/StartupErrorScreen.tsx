import { t } from "../i18n.js";
import type { RendererStartupError } from "../startup.js";

interface StartupErrorScreenProps {
	error: RendererStartupError;
}

export function StartupErrorScreen({ error }: StartupErrorScreenProps) {
	return (
		<section
			aria-labelledby="camlet-startup-error-title"
			className="startup-screen"
		>
			<p className="startup-screen__eyebrow">{t("app.title")}</p>
			<h1 className="startup-screen__title" id="camlet-startup-error-title">
				{t(`startup.errors.${error.code}.title`)}
			</h1>
			<p className="startup-screen__copy">
				{t(`startup.errors.${error.code}.message`)}
			</p>
			<button
				className="camlet-button"
				onClick={() => {
					window.location.reload();
				}}
				type="button"
			>
				{t("startup.actions.reload")}
			</button>
			{import.meta.env.DEV && error.detail !== null ? (
				<details className="startup-screen__details">
					<summary>{t("startup.debugSummary")}</summary>
					<code>{error.detail}</code>
				</details>
			) : null}
		</section>
	);
}
