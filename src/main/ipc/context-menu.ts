import {
	app,
	type BrowserWindow,
	Menu,
	type MenuItemConstructorOptions,
} from "electron/main";
import type {
	CamletContextMenuAction,
	CamletContextMenuRequest,
} from "../../shared/ipc.js";

const ringThicknessOptions = [0, 2, 4, 6, 8, 10];
const cornerRoundnessOptions = [0, 12, 24, 36, 48, 60];
const themeIds = [
	"mint",
	"ocean",
	"ember",
	"orchid",
	"grove",
	"graphite",
] as const;
const contextMenuRequests = new WeakMap<
	BrowserWindow,
	CamletContextMenuRequest
>();

function sendContextMenuAction(
	window: BrowserWindow,
	action: CamletContextMenuAction,
) {
	if (window.isDestroyed() || window.webContents.isDestroyed()) {
		return;
	}

	window.webContents.send("app:context-menu-action", action);
}

function createContextMenuTemplate(
	window: BrowserWindow,
	request: CamletContextMenuRequest,
): MenuItemConstructorOptions[] {
	const cameraSubmenu =
		request.cameraOptions.length > 0
			? request.cameraOptions.map<MenuItemConstructorOptions>((device) => ({
					label: device.label,
					type: "radio",
					checked: device.deviceId === request.selectedCameraDeviceId,
					click: () => {
						sendContextMenuAction(window, {
							type: "set-camera",
							deviceId: device.deviceId,
						});
					},
				}))
			: [
					{
						label: request.labels.noDevices,
						enabled: false,
					},
				];
	const themeSubmenu = themeIds.map<MenuItemConstructorOptions>((themeId) => ({
		label: request.labels.themeOptions[themeId],
		type: "radio",
		checked: request.selectedThemeId === themeId,
		click: () => {
			sendContextMenuAction(window, {
				type: "set-theme",
				themeId,
			});
		},
	}));

	return [
		{
			label: request.labels.theme,
			submenu: themeSubmenu,
		},
		{
			label: request.labels.shape,
			submenu: [
				{
					label: request.labels.shapeOptions.circle,
					type: "radio",
					checked: request.selectedShape === "circle",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-shape",
							shape: "circle",
						});
					},
				},
				{
					label: request.labels.shapeOptions.roundedSquare,
					type: "radio",
					checked: request.selectedShape === "rounded-square",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-shape",
							shape: "rounded-square",
						});
					},
				},
				{
					label: request.labels.shapeOptions.rectangleY,
					type: "radio",
					checked: request.selectedShape === "rectangle-y",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-shape",
							shape: "rectangle-y",
						});
					},
				},
				{
					label: request.labels.shapeOptions.rectangleX,
					type: "radio",
					checked: request.selectedShape === "rectangle-x",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-shape",
							shape: "rectangle-x",
						});
					},
				},
				{
					label: request.labels.shapeOptions.diamond,
					type: "radio",
					checked: request.selectedShape === "diamond",
					click: () => {
						sendContextMenuAction(window, {
							type: "set-shape",
							shape: "diamond",
						});
					},
				},
			],
		},
		{
			label: request.labels.language,
			submenu: Object.entries(request.labels.languageOptions).map(
				([language, label]) => ({
					label,
					type: "radio",
					checked: language === request.selectedLanguage,
					click: () => {
						sendContextMenuAction(window, {
							type: "set-language",
							language:
								language as CamletContextMenuRequest["selectedLanguage"],
						});
					},
				}),
			),
		},
		{
			label: request.labels.cameraInput,
			submenu: cameraSubmenu,
		},
		{
			type: "separator",
		},
		{
			label: request.labels.resize,
			click: () => {
				sendContextMenuAction(window, {
					type: "enter-resize-mode",
				});
			},
		},
		{
			label: request.labels.advancedSettings,
			submenu: [
				{
					label: `${request.labels.status}: ${request.cameraStatusLabel}`,
					enabled: false,
				},
				{
					label: `${request.labels.activeDevice}: ${request.activeCameraLabel}`,
					enabled: false,
				},
				{
					label: `${request.labels.displayProtocol}: ${request.displayProtocolLabel}`,
					enabled: false,
				},
				{
					type: "separator",
				},
				{
					label: request.labels.fitMode,
					submenu: [
						{
							label: request.labels.fitModeOptions.cover,
							type: "radio",
							checked: request.selectedFitMode === "cover",
							click: () => {
								sendContextMenuAction(window, {
									type: "set-fit-mode",
									fitMode: "cover",
								});
							},
						},
						{
							label: request.labels.fitModeOptions.contain,
							type: "radio",
							checked: request.selectedFitMode === "contain",
							click: () => {
								sendContextMenuAction(window, {
									type: "set-fit-mode",
									fitMode: "contain",
								});
							},
						},
					],
				},
				{
					label: request.labels.ringThickness,
					submenu: ringThicknessOptions.map<MenuItemConstructorOptions>(
						(ringThickness) => ({
							label: `${ringThickness}px`,
							type: "radio",
							checked: request.selectedRingThickness === ringThickness,
							click: () => {
								sendContextMenuAction(window, {
									type: "set-ring-thickness",
									ringThickness,
								});
							},
						}),
					),
				},
				{
					label: request.labels.cornerRoundness,
					submenu: cornerRoundnessOptions.map<MenuItemConstructorOptions>(
						(cornerRoundness) => ({
							label: `${cornerRoundness}px`,
							type: "radio",
							checked: request.selectedCornerRoundness === cornerRoundness,
							click: () => {
								sendContextMenuAction(window, {
									type: "set-corner-roundness",
									cornerRoundness,
								});
							},
						}),
					),
				},
				{
					type: "separator",
				},
				{
					label: request.labels.retryCamera,
					click: () => {
						sendContextMenuAction(window, {
							type: "retry-camera",
						});
					},
				},
				{
					label: request.labels.resetAppearance,
					click: () => {
						sendContextMenuAction(window, {
							type: "reset-appearance",
						});
					},
				},
			],
		},
		{
			label: request.labels.aboutCamlet,
			click: () => {
				sendContextMenuAction(window, {
					type: "open-about-window",
				});
			},
		},
		{
			type: "separator",
		},
		{
			label: request.labels.closeApp,
			click: () => {
				app.quit();
			},
		},
	];
}

export function updateContextMenuRequest(
	window: BrowserWindow,
	request: CamletContextMenuRequest,
) {
	contextMenuRequests.set(window, request);
}

export function showMainWindowContextMenu(window: BrowserWindow) {
	const request = contextMenuRequests.get(window);

	if (request === undefined) {
		return;
	}

	const menu = Menu.buildFromTemplate(
		createContextMenuTemplate(window, request),
	);
	menu.popup({
		window,
	});
}
