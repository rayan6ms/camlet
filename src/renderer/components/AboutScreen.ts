import type { AboutInfo } from "../../shared/about.js";
import type { AppBootstrap } from "../../shared/bootstrap.js";
import { t } from "../i18n.js";

interface AboutScreenProps {
	bootstrap: AppBootstrap;
}

export interface MountedScreen {
	destroy(): void;
	element: HTMLElement;
}

function getPackagedLabel(packaged: boolean) {
	return packaged
		? t("about.packagedValues.yes")
		: t("about.packagedValues.no");
}

function createLink(href: string, label: string): HTMLAnchorElement {
	const link = document.createElement("a");
	link.href = href;
	link.rel = "noreferrer";
	link.target = "_blank";
	link.textContent = label;
	return link;
}

function createMetaItem(label: string, value: string): HTMLDivElement {
	const row = document.createElement("div");
	const term = document.createElement("dt");
	term.textContent = label;
	const detail = document.createElement("dd");
	detail.textContent = value;
	row.append(term, detail);
	return row;
}

export function createAboutScreen({
	bootstrap,
}: AboutScreenProps): MountedScreen {
	document.title = "About Camlet";

	const section = document.createElement("section");
	section.className = "about-screen";

	const content = document.createElement("div");
	content.className = "about-screen__content";
	section.append(content);

	let destroyed = false;

	function renderLoading() {
		const description = document.createElement("p");
		description.className = "about-screen__description";
		description.textContent = t("camera.status.loading");
		content.replaceChildren(description);
	}

	function renderAboutInfo(aboutInfo: AboutInfo) {
		const description = document.createElement("p");
		description.className = "about-screen__description";
		description.textContent = aboutInfo.description;

		const avatar = document.createElement("img");
		avatar.alt = aboutInfo.githubHandle;
		avatar.className = "about-screen__avatar";
		avatar.src = aboutInfo.avatarDataUrl;

		const links = document.createElement("nav");
		links.className = "about-screen__links";
		links.setAttribute("aria-label", t("sections.about"));
		links.append(
			createLink(aboutInfo.githubUrl, "rayan6ms GitHub"),
			createLink(aboutInfo.projectUrl, "Camlet GitHub"),
			createLink(aboutInfo.projectIssuesUrl, "Camlet GitHub Issues"),
		);

		const metadata = document.createElement("dl");
		metadata.className = "about-screen__meta";
		metadata.append(
			createMetaItem(t("about.labels.version"), bootstrap.app.version),
			createMetaItem(
				t("about.labels.channel"),
				t(`about.channels.${bootstrap.app.channel}`),
			),
			createMetaItem(
				t("about.labels.packaged"),
				getPackagedLabel(bootstrap.app.packaged),
			),
			createMetaItem(
				t("about.labels.platform"),
				`${bootstrap.app.platform} / ${bootstrap.app.arch}`,
			),
			createMetaItem(
				t("about.labels.displayProtocol"),
				t(`about.displayProtocols.${bootstrap.app.displayProtocol}`),
			),
			createMetaItem(
				t("about.labels.electron"),
				bootstrap.app.versions.electron,
			),
			createMetaItem(t("about.labels.chrome"), bootstrap.app.versions.chrome),
			createMetaItem(t("about.licenseLabel"), aboutInfo.license),
		);

		content.replaceChildren(description, avatar, links, metadata);
	}

	renderLoading();

	void window.camlet.getAboutInfo().then((nextInfo) => {
		if (destroyed) {
			return;
		}

		renderAboutInfo(nextInfo);
	});

	return {
		destroy() {
			destroyed = true;
		},
		element: section,
	};
}
