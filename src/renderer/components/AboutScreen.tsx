import { useEffect, useState } from "react";
import type { AboutInfo } from "../../shared/about.js";
import type { AppBootstrap } from "../../shared/bootstrap.js";
import { t } from "../i18n.js";

interface AboutScreenProps {
	bootstrap: AppBootstrap;
}

function getPackagedLabel(packaged: boolean) {
	return packaged
		? t("about.packagedValues.yes")
		: t("about.packagedValues.no");
}

export function AboutScreen({ bootstrap }: AboutScreenProps) {
	const [aboutInfo, setAboutInfo] = useState<AboutInfo | null>(null);

	useEffect(() => {
		document.title = "About Camlet";
	}, []);

	useEffect(() => {
		void window.camlet.getAboutInfo().then((nextInfo) => {
			setAboutInfo(nextInfo);
		});
	}, []);

	if (aboutInfo === null) {
		return (
			<section className="about-screen">
				<div className="about-screen__content">
					<p className="about-screen__description">
						{t("camera.status.loading")}
					</p>
				</div>
			</section>
		);
	}

	return (
		<section className="about-screen">
			<div className="about-screen__content">
				<p className="about-screen__description">{aboutInfo.description}</p>
				<img
					alt={aboutInfo.githubHandle}
					className="about-screen__avatar"
					src={aboutInfo.avatarDataUrl}
				/>
				<nav aria-label={t("sections.about")} className="about-screen__links">
					<a href={aboutInfo.githubUrl} rel="noreferrer" target="_blank">
						rayan6ms GitHub
					</a>
					<a href={aboutInfo.projectUrl} rel="noreferrer" target="_blank">
						Camlet GitHub
					</a>
					<a href={aboutInfo.projectIssuesUrl} rel="noreferrer" target="_blank">
						Camlet GitHub Issues
					</a>
				</nav>

				<dl className="about-screen__meta">
					<div>
						<dt>{t("about.labels.version")}</dt>
						<dd>{bootstrap.app.version}</dd>
					</div>
					<div>
						<dt>{t("about.labels.channel")}</dt>
						<dd>{t(`about.channels.${bootstrap.app.channel}`)}</dd>
					</div>
					<div>
						<dt>{t("about.labels.packaged")}</dt>
						<dd>{getPackagedLabel(bootstrap.app.packaged)}</dd>
					</div>
					<div>
						<dt>{t("about.labels.platform")}</dt>
						<dd>
							{bootstrap.app.platform} / {bootstrap.app.arch}
						</dd>
					</div>
					<div>
						<dt>{t("about.labels.displayProtocol")}</dt>
						<dd>
							{t(`about.displayProtocols.${bootstrap.app.displayProtocol}`)}
						</dd>
					</div>
					<div>
						<dt>{t("about.labels.electron")}</dt>
						<dd>{bootstrap.app.versions.electron}</dd>
					</div>
					<div>
						<dt>{t("about.labels.chrome")}</dt>
						<dd>{bootstrap.app.versions.chrome}</dd>
					</div>
					<div>
						<dt>{t("about.licenseLabel")}</dt>
						<dd>{aboutInfo.license}</dd>
					</div>
				</dl>
			</div>
		</section>
	);
}
