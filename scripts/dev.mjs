import { spawn } from "node:child_process";
import { existsSync, watch } from "node:fs";
import path from "node:path";
import process from "node:process";
import { setTimeout as delay } from "node:timers/promises";

const projectRoot = process.cwd();
const pnpmBinary = process.platform === "win32" ? "pnpm.cmd" : "pnpm";
const devServerUrl = "http://127.0.0.1:5173";
const distElectronPath = path.join(projectRoot, "dist-electron");
const mainEntryPath = path.join(projectRoot, "dist-electron/main/index.js");
const electronOnly = process.argv.includes("--electron-only");

const managedChildren = [];
let electronProcess = null;
let isRestarting = false;
let isShuttingDown = false;

function spawnManagedProcess(label, args, extraEnv = {}) {
	const env = {
		...process.env,
		...extraEnv,
	};

	for (const [name, value] of Object.entries(extraEnv)) {
		if (value === undefined) {
			delete env[name];
		}
	}

	const child = spawn(pnpmBinary, args, {
		cwd: projectRoot,
		env,
		stdio: "inherit",
	});

	child.on("exit", (code) => {
		if (isShuttingDown || label === "electron") {
			return;
		}

		console.error(`[camlet:${label}] exited with code ${code ?? "null"}`);
		void shutdown(code ?? 1);
	});

	managedChildren.push(child);
	return child;
}

async function waitForMainBuild() {
	for (;;) {
		if (existsSync(mainEntryPath)) {
			return;
		}

		await delay(250);
	}
}

async function waitForRenderer() {
	for (;;) {
		try {
			const response = await fetch(devServerUrl);

			if (response.ok) {
				return;
			}
		} catch {}

		await delay(250);
	}
}

function startElectron() {
	const electronEnv = {
		NODE_ENV: "development",
		VITE_DEV_SERVER_URL: devServerUrl,
		ELECTRON_RUN_AS_NODE: undefined,
	};

	electronProcess = spawnManagedProcess(
		"electron",
		["exec", "electron", "."],
		electronEnv,
	);

	electronProcess.on("exit", (code) => {
		if (isShuttingDown || isRestarting) {
			return;
		}

		void shutdown(code ?? 0);
	});
}

async function restartElectron() {
	if (isRestarting || isShuttingDown || electronProcess === null) {
		return;
	}

	isRestarting = true;
	electronProcess.kill("SIGTERM");
	await delay(200);
	startElectron();
	isRestarting = false;
}

async function shutdown(exitCode = 0) {
	if (isShuttingDown) {
		return;
	}

	isShuttingDown = true;

	for (const child of managedChildren) {
		if (!child.killed) {
			child.kill("SIGTERM");
		}
	}

	await delay(150);
	process.exit(exitCode);
}

for (const signal of ["SIGINT", "SIGTERM"]) {
	process.on(signal, () => {
		void shutdown(0);
	});
}

if (!electronOnly) {
	spawnManagedProcess("renderer", [
		"exec",
		"vite",
		"--config",
		"vite.config.ts",
	]);
	spawnManagedProcess("electron:ts", [
		"exec",
		"tsc",
		"-p",
		"tsconfig.electron.json",
		"--watch",
		"--preserveWatchOutput",
	]);
}

await Promise.all([waitForMainBuild(), waitForRenderer()]);
startElectron();

if (existsSync(distElectronPath)) {
	const watcher = watch(
		distElectronPath,
		{ recursive: true },
		(_, filename) => {
			if (typeof filename !== "string") {
				return;
			}

			if (filename.endsWith(".js") || filename.endsWith(".cjs")) {
				void restartElectron();
			}
		},
	);

	process.on("exit", () => watcher.close());
}
