import { execFile as execFileCallback } from "node:child_process";
import { readdir, stat } from "node:fs/promises";
import { createRequire } from "node:module";
import path, { basename, join } from "node:path";
import { promisify } from "node:util";
import { app, BrowserWindow } from "electron/main";

interface SizeEntry {
	kind: "artifact" | "package";
	label: string;
	sizeBytes: number;
}

interface ProcessEntry {
	isOverlayWindow: boolean;
	label: string;
	pid: number;
	type: string;
	workingSetBytes: number;
}

interface DiagnosticsSnapshot {
	artifacts: SizeEntry[];
	mainProcess: {
		arrayBuffersBytes: number;
		externalBytes: number;
		heapTotalBytes: number;
		heapUsedBytes: number;
		rssBytes: number;
	};
	packages: SizeEntry[];
	processes: ProcessEntry[];
	totalWorkingSetBytes: number;
	updatedAt: string;
}

const execFile = promisify(execFileCallback);
const require = createRequire(import.meta.url);
const packageJson = require("../../../package.json") as {
	dependencies?: Record<string, string>;
	devDependencies?: Record<string, string>;
	version?: string;
};

function escapeForJavaScript(value: unknown) {
	return JSON.stringify(value).replace(/</g, "\\u003c");
}

async function getFileSystemSizeWithShell(
	targetPath: string,
): Promise<number | null> {
	if (process.platform === "win32") {
		return null;
	}

	try {
		const { stdout } = await execFile("du", ["-sb", targetPath]);
		const [value] = stdout.trim().split(/\s+/);
		const parsedValue = Number.parseInt(value ?? "", 10);

		return Number.isFinite(parsedValue) ? parsedValue : null;
	} catch {
		return null;
	}
}

async function getFileSystemSizeWithFs(
	targetPath: string,
): Promise<number | null> {
	try {
		const entry = await stat(targetPath);

		if (entry.isFile()) {
			return entry.size;
		}

		if (!entry.isDirectory()) {
			return 0;
		}

		const children = await readdir(targetPath, { withFileTypes: true });
		const sizes = await Promise.all(
			children.map((child) =>
				getFileSystemSizeWithFs(join(targetPath, child.name)),
			),
		);

		return sizes.reduce<number>((total, size) => total + (size ?? 0), 0);
	} catch {
		return null;
	}
}

async function getFileSystemSize(targetPath: string): Promise<number | null> {
	return (
		(await getFileSystemSizeWithShell(targetPath)) ??
		(await getFileSystemSizeWithFs(targetPath))
	);
}

async function getSizeEntry(
	kind: SizeEntry["kind"],
	label: string,
	targetPath: string,
): Promise<SizeEntry | null> {
	const sizeBytes = await getFileSystemSize(targetPath);

	return sizeBytes === null
		? null
		: {
				kind,
				label,
				sizeBytes,
			};
}

async function getArtifactSizes(projectRoot: string) {
	const releasePath = join(projectRoot, "release");
	const releaseEntries = await readdir(releasePath, {
		withFileTypes: true,
	}).catch(() => []);
	const artifacts = await Promise.all([
		getSizeEntry(
			"artifact",
			`executable/${basename(process.execPath)}`,
			process.execPath,
		),
		getSizeEntry(
			"artifact",
			"build/dist-electron",
			join(projectRoot, "dist-electron"),
		),
		getSizeEntry(
			"artifact",
			"build/dist/renderer",
			join(projectRoot, "dist", "renderer"),
		),
		...releaseEntries
			.filter((entry) => entry.isFile())
			.map((entry) =>
				getSizeEntry(
					"artifact",
					`release/${entry.name}`,
					join(releasePath, entry.name),
				),
			),
	]);

	return artifacts
		.filter((entry) => entry !== null)
		.sort((left, right) => left.label.localeCompare(right.label));
}

async function getPackageSizes(projectRoot: string) {
	const dependencyEntries = Object.keys(packageJson.dependencies ?? {}).map(
		(name) => ({
			label: `dependency/${name}`,
			path: join(projectRoot, "node_modules", name),
		}),
	);
	const devDependencyEntries = Object.keys(
		packageJson.devDependencies ?? {},
	).map((name) => ({
		label: `devDependency/${name}`,
		path: join(projectRoot, "node_modules", name),
	}));
	const sizes = await Promise.all(
		[...dependencyEntries, ...devDependencyEntries].map((entry) =>
			getSizeEntry("package", entry.label, entry.path),
		),
	);

	return sizes
		.filter((entry) => entry !== null)
		.sort((left, right) => left.label.localeCompare(right.label));
}

function getProcessLabel(metric: Electron.ProcessMetric) {
	const labels: string[] = [metric.type];

	if (typeof metric.name === "string" && metric.name.trim().length > 0) {
		labels.push(metric.name.trim());
	}

	if (
		typeof metric.serviceName === "string" &&
		metric.serviceName.trim().length > 0 &&
		metric.serviceName.trim() !== metric.name?.trim()
	) {
		labels.push(metric.serviceName.trim());
	}

	return labels.join(":");
}

async function getDiagnosticsSnapshot(
	overlayWindow: BrowserWindow,
	projectRoot: string,
	cachedPackageSizes: SizeEntry[],
): Promise<DiagnosticsSnapshot> {
	const overlayRendererPid = overlayWindow.webContents.getOSProcessId();
	const memoryUsage = process.memoryUsage();
	const processes = app
		.getAppMetrics()
		.map((metric) => ({
			isOverlayWindow: metric.pid === overlayRendererPid,
			label: getProcessLabel(metric),
			pid: metric.pid,
			type: metric.type,
			workingSetBytes: Math.max(
				0,
				Math.round((metric.memory?.workingSetSize ?? 0) * 1024),
			),
		}))
		.sort((left, right) => left.label.localeCompare(right.label));

	return {
		artifacts: await getArtifactSizes(projectRoot),
		mainProcess: {
			arrayBuffersBytes: memoryUsage.arrayBuffers,
			externalBytes: memoryUsage.external,
			heapTotalBytes: memoryUsage.heapTotal,
			heapUsedBytes: memoryUsage.heapUsed,
			rssBytes: memoryUsage.rss,
		},
		packages: cachedPackageSizes,
		processes,
		totalWorkingSetBytes: processes.reduce(
			(total, processEntry) => total + processEntry.workingSetBytes,
			0,
		),
		updatedAt: new Date().toISOString(),
	};
}

function getDiagnosticsWindowHtml(version: string) {
	return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>Camlet Dev Diagnostics</title>
<style>
	:root {
		color-scheme: dark;
		font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
		background: #0a0f13;
		color: #edf4fb;
	}
	* { box-sizing: border-box; }
	body {
		margin: 0;
		background: #0a0f13;
	}
	#app {
		display: grid;
		gap: 18px;
		padding: 16px 18px 20px;
	}
	h1, h2, p { margin: 0; }
	h1 { font-size: 1.05rem; }
	h2 { font-size: 0.92rem; margin-bottom: 10px; }
	.meta {
		display: flex;
		flex-wrap: wrap;
		gap: 8px 14px;
		margin-top: 6px;
		font-size: 0.8rem;
		color: rgba(244, 247, 251, 0.76);
	}
	.muted {
		font-family: "Space Grotesk", "IBM Plex Sans", "Segoe UI", sans-serif;
		font-size: 0.68rem;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: rgba(244, 247, 251, 0.54);
	}
	.grid {
		display: grid;
		gap: 18px 24px;
		grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr);
	}
	.main-metrics {
		display: grid;
		gap: 18px 24px;
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}
	.section-copy {
		margin-bottom: 10px;
		font-size: 0.8rem;
		color: rgba(244, 247, 251, 0.76);
	}
	table {
		width: 100%;
		border-collapse: collapse;
		table-layout: fixed;
		font-size: 0.8rem;
	}
	th, td {
		padding: 7px 8px 7px 0;
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		text-align: left;
		vertical-align: top;
		overflow-wrap: anywhere;
	}
	th {
		font-family: "Space Grotesk", "IBM Plex Sans", "Segoe UI", sans-serif;
		font-size: 0.68rem;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: rgba(244, 247, 251, 0.54);
	}
	td:last-child, th:last-child { text-align: right; padding-right: 0; }
	code {
		font-family: "IBM Plex Mono", monospace;
		font-size: 0.78rem;
		color: #c8fff1;
		white-space: pre-wrap;
		word-break: break-word;
	}
	.inline-note {
		color: rgba(244, 247, 251, 0.62);
		font-family: "Space Grotesk", "IBM Plex Sans", "Segoe UI", sans-serif;
		font-size: 0.68rem;
		letter-spacing: 0.04em;
		text-transform: uppercase;
	}
	@media (max-width: 920px) {
		.grid,
		.main-metrics {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
</head>
<body>
<div id="app">
	<section>
		<h1>Camlet Dev Diagnostics</h1>
		<p class="meta">
			<span>version <code>${version}</code></span>
			<span>runtime <code>development</code></span>
			<span>platform <code>${process.platform}/${process.arch}</code></span>
		</p>
	</section>
	<section>
		<p>Loading diagnostics…</p>
	</section>
</div>
<script>
	const formatBytes = (bytes) => {
		if (bytes < 1024) return \`\${bytes} B\`;
		const units = ["KiB", "MiB", "GiB", "TiB"];
		let value = bytes / 1024;
		let unitIndex = 0;
		while (value >= 1024 && unitIndex < units.length - 1) {
			value /= 1024;
			unitIndex += 1;
		}
		return \`\${value >= 10 ? value.toFixed(1) : value.toFixed(2)} \${units[unitIndex]}\`;
	};
	const escapeHtml = (value) =>
		String(value)
			.replaceAll("&", "&amp;")
			.replaceAll("<", "&lt;")
			.replaceAll(">", "&gt;")
			.replaceAll('"', "&quot;")
			.replaceAll("'", "&#39;");
	const renderRows = (entries, valueKey, noteKey) =>
		entries.map((entry) => \`
			<tr>
				<td><code>\${escapeHtml(entry.label)}</code></td>
				<td>\${noteKey && entry[noteKey] ? '<span class="inline-note">overlay</span>' : ""}</td>
				<td>\${escapeHtml(formatBytes(entry[valueKey]))}</td>
			</tr>
		\`).join("");
	const renderTwoUpRows = (entries, valueKey) => {
		const rows = [];
		for (let index = 0; index < entries.length; index += 2) {
			rows.push([entries[index], entries[index + 1]]);
		}
		return rows.map(([left, right]) => \`
			<tr>
				<td>\${left ? \`<code>\${escapeHtml(left.label)}</code>\` : ""}</td>
				<td>\${left ? escapeHtml(formatBytes(left[valueKey])) : ""}</td>
				<td>\${right ? \`<code>\${escapeHtml(right.label)}</code>\` : ""}</td>
				<td>\${right ? escapeHtml(formatBytes(right[valueKey])) : ""}</td>
			</tr>
		\`).join("");
	};
	window.__CAMLET_UPDATE = (snapshot) => {
		document.getElementById("app").innerHTML = \`
			<section>
				<h1>Camlet Dev Diagnostics</h1>
				<p class="meta">
					<span>version <code>${version}</code></span>
					<span>updated <code>\${escapeHtml(snapshot.updatedAt)}</code></span>
					<span>electron working set <code>\${escapeHtml(formatBytes(snapshot.totalWorkingSetBytes))}</code></span>
				</p>
			</section>
			<section class="main-metrics">
				<section>
					<h2>Process Memory</h2>
					<table>
						<thead>
							<tr><th>Process</th><th></th><th>Working set</th></tr>
						</thead>
						<tbody>\${renderRows(snapshot.processes, "workingSetBytes", "isOverlayWindow")}</tbody>
					</table>
				</section>
				<section>
					<h2>Main Process</h2>
					<table>
						<tbody>
							<tr><th>rss</th><td><code>\${escapeHtml(formatBytes(snapshot.mainProcess.rssBytes))}</code></td></tr>
							<tr><th>heap used</th><td><code>\${escapeHtml(formatBytes(snapshot.mainProcess.heapUsedBytes))}</code></td></tr>
							<tr><th>heap total</th><td><code>\${escapeHtml(formatBytes(snapshot.mainProcess.heapTotalBytes))}</code></td></tr>
							<tr><th>external</th><td><code>\${escapeHtml(formatBytes(snapshot.mainProcess.externalBytes))}</code></td></tr>
							<tr><th>array buffers</th><td><code>\${escapeHtml(formatBytes(snapshot.mainProcess.arrayBuffersBytes))}</code></td></tr>
						</tbody>
					</table>
				</section>
			</section>
			<section class="grid">
				<section>
					<h2>Programs And Outputs</h2>
					<p class="section-copy">Built outputs appear when they exist locally. Packaged artifacts only show up after running a package target.</p>
					<table>
						<thead>
							<tr><th>Item</th><th>Size</th><th>Item</th><th>Size</th></tr>
						</thead>
						<tbody>\${renderTwoUpRows(snapshot.artifacts, "sizeBytes")}</tbody>
					</table>
				</section>
				<section>
					<h2>Installed Packages</h2>
					<p class="section-copy">Direct dependencies and devDependencies from <code>package.json</code>.</p>
					<table>
						<thead>
							<tr><th>Package</th><th>Size</th><th>Package</th><th>Size</th></tr>
						</thead>
						<tbody>\${renderTwoUpRows(snapshot.packages, "sizeBytes")}</tbody>
					</table>
				</section>
			</section>
		\`;
	};
</script>
</body>
</html>`;
}

export async function openDevelopmentDiagnosticsWindow(options: {
	overlayWindow: BrowserWindow;
}) {
	const { overlayWindow } = options;
	const projectRoot = path.resolve(app.getAppPath());
	const cachedPackageSizes = await getPackageSizes(projectRoot);
	const window = new BrowserWindow({
		width: 1080,
		height: 760,
		minWidth: 920,
		minHeight: 560,
		autoHideMenuBar: true,
		backgroundColor: "#0A0F13",
		title: "Camlet Dev Diagnostics",
		webPreferences: {
			contextIsolation: true,
			nodeIntegration: false,
			sandbox: true,
			spellcheck: false,
		},
	});
	const pushSnapshot = async () => {
		if (window.isDestroyed() || overlayWindow.isDestroyed()) {
			return;
		}

		const snapshot = await getDiagnosticsSnapshot(
			overlayWindow,
			projectRoot,
			cachedPackageSizes,
		);

		if (window.isDestroyed()) {
			return;
		}

		await window.webContents.executeJavaScript(
			`window.__CAMLET_UPDATE(${escapeForJavaScript(snapshot)})`,
			true,
		);
	};

	overlayWindow.on("closed", () => {
		if (!window.isDestroyed()) {
			window.close();
		}
	});

	await window.loadURL(
		`data:text/html;charset=utf-8,${encodeURIComponent(
			getDiagnosticsWindowHtml(
				typeof packageJson.version === "string" ? packageJson.version : "0.0.0",
			),
		)}`,
	);
	await pushSnapshot();

	const interval = setInterval(() => {
		void pushSnapshot();
	}, 3000);

	window.on("closed", () => {
		clearInterval(interval);
	});
}
