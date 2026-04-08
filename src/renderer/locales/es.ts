import type { RendererLocale } from "./schema.js";

export const esLocale: RendererLocale = {
	app: {
		title: "Camlet",
		close: "Cerrar Camlet",
		overlayReady: "Superposición de webcam configurable",
	},
	overlay: {
		dragHint: "Arrastra aquí para mover la superposición",
		settingsHint:
			"Haz clic derecho en la superposición o usa el botón para mostrar la configuración",
		summary: "Resumen de la superposición",
		preview: "Vista previa de la superposición de la webcam",
		hintOpenSettings: "Haz clic derecho para abrir la configuración",
		resizeHint:
			"Arrastra un tirador para cambiar el tamaño. Haz clic en Listo o en la superposición para salir.",
		resizeDone: "Listo",
		resizeAction: "Redimensionar",
		resizeHandle: "Redimensionar superposición",
	},
	advanced: {
		title: "Configuración avanzada",
		description:
			"Mantén la superposición principal limpia y usa este panel solo para controles extra y diagnósticos.",
	},
	sections: {
		settings: "Configuración",
		general: "General",
		appearance: "Apariencia",
		camera: "Cámara",
		system: "Sistema",
		about: "Acerca de",
	},
	settings: {
		actions: {
			open: "Abrir configuración",
			close: "Cerrar configuración",
			resetAppearance: "Restablecer apariencia predeterminada",
		},
		hints: {
			panel:
				"Los controles compactos permanecen dentro del shell y se actualizan en vivo.",
			escape: "Pulsa Escape para cerrar la configuración.",
		},
	},
	about: {
		windowTitle: "Acerca de Camlet",
		description:
			"Usa esta información de compilación al validar versiones beta o lanzamientos empaquetados.",
		licenseLabel: "Licencia",
		readmeTitle: "Resumen del README",
		labels: {
			appName: "Aplicación",
			version: "Versión",
			channel: "Canal",
			mode: "Entorno",
			packaged: "Empaquetada",
			platform: "Plataforma",
			displayProtocol: "Protocolo de pantalla",
			electron: "Electron",
			chrome: "Chrome",
		},
		channels: {
			stable: "Estable",
			prerelease: "Beta / preliminar",
		},
		modes: {
			development: "Desarrollo",
			production: "Build empaquetada",
		},
		displayProtocols: {
			wayland: "Wayland",
			x11: "X11",
			windows: "Escritorio Windows",
			macos: "Escritorio macOS",
			unknown: "Desconocido",
		},
		packagedValues: {
			yes: "Sí",
			no: "No",
		},
		diagnostics: {
			title: "Resumen de diagnóstico",
			hint: "Copia este resumen al informar comportamientos de builds empaquetadas o problemas específicos de la plataforma.",
			copy: "Copiar diagnóstico",
			copied: "Copiado",
			copyFailed: "Copia no disponible",
		},
	},
	language: {
		label: "Idioma",
		description: "Elige cómo debe mostrar Camlet la interfaz.",
		options: {
			system: "Predeterminado del sistema",
			en: "English",
			"pt-BR": "Português (Brasil)",
			es: "Español",
			fr: "Français",
			de: "Deutsch",
			it: "Italiano",
			ja: "日本語",
		},
	},
	summary: {
		activeDevice: "Dispositivo activo",
		overlaySize: "Tamaño de la superposición",
		effectiveLanguage: "Idioma actual",
		windowPosition: "Posición",
		windowSize: "Tamaño de la ventana",
		platform: "Plataforma",
	},
	camera: {
		description:
			"Selecciona la cámara preferida y revisa el estado de la vista previa sin salir de la superposición.",
		actions: {
			retry: "Reintentar cámara",
		},
		labels: {
			device: "Dispositivo de cámara",
			activeDevice: "Dispositivo activo",
			deviceCount: "Cámaras detectadas",
			permission: "Estado de la vista previa",
			noDevices: "No hay cámaras disponibles",
			none: "Ninguno",
		},
		status: {
			loading: "Cargando cámara",
			preview: "Vista previa de la cámara activa",
			"permission-denied": "Permiso denegado",
			"camera-in-use": "Cámara ocupada",
			"no-camera": "No se encontró ninguna cámara",
			"selected-device-unavailable":
				"El dispositivo guardado no está disponible",
			error: "Error de cámara",
		},
		message: {
			loading:
				"Camlet está solicitando acceso a la cámara y preparando la vista previa.",
			preview:
				"La vista previa en vivo de la webcam se está ejecutando dentro del shell de la superposición.",
			"permission-denied":
				"Se denegó el acceso a la cámara. Permite el acceso y vuelve a intentarlo.",
			"camera-in-use":
				"La cámara ya está en uso o no se puede leer. Cierra otras aplicaciones que la estén usando y vuelve a intentarlo.",
			"no-camera":
				"No se detectaron dispositivos de entrada de video. Conecta una cámara y vuelve a intentarlo.",
			"selected-device-unavailable":
				"La cámara guardada no está disponible. Vuelve a conectarla o elige otro dispositivo.",
			error:
				"Camlet no pudo iniciar la vista previa de la cámara. Reintenta o cambia de dispositivo.",
			savedUnavailableUsingFallback:
				"La cámara guardada anteriormente no estaba disponible, así que Camlet cambió a otro dispositivo disponible.",
		},
	},
	appearance: {
		description:
			"Mantén la superposición compacta y legible mientras ajustas la presentación del anillo en tiempo real.",
		labels: {
			theme: "Tema",
			shape: "Forma",
			cornerRoundness: "Redondeo",
			fitMode: "Modo de ajuste",
			ringColor: "Color del anillo",
			ringThickness: "Grosor del anillo",
			overlaySize: "Tamaño de la superposición",
		},
		themes: {
			mint: "Menta",
			ocean: "Océano",
			ember: "Ascua",
			orchid: "Orquídea",
			grove: "Bosque",
			graphite: "Grafito",
		},
		shapes: {
			circle: "Círculo",
			roundedSquare: "Cuadrado",
			diamond: "Diamante",
			rectangle: "Rectángulo",
		},
		fitModes: {
			cover: "Cubrir",
			contain: "Contener",
		},
	},
	startup: {
		debugSummary: "Detalles de inicio",
		actions: {
			reload: "Recargar Camlet",
		},
		errors: {
			"preload-unavailable": {
				title: "Camlet no pudo conectarse a su puente de escritorio",
				message:
					"Falta la API de preload, por lo que la superposición no puede terminar de iniciarse. Reinicia la aplicación y confirma que los archivos empaquetados estén intactos.",
			},
			"bootstrap-invalid": {
				title: "Camlet recibió datos de inicio no válidos",
				message:
					"El renderer bloqueó el inicio porque la carga de bootstrap estaba incompleta o malformada. Reinicia la aplicación y verifica la build actual.",
			},
			"bootstrap-load-failed": {
				title: "Camlet no pudo cargar la configuración inicial",
				message:
					"El renderer no pudo cargar la configuración inicial desde el proceso de escritorio. Reinicia la aplicación. Si sigue fallando, prueba con un perfil de configuración limpio.",
			},
		},
		issues: {
			title: "Aviso de inicio",
			messages: {
				"settings-recovered":
					"La configuración guardada se reparó o restableció a valores seguros para esta sesión.",
				"settings-persistence-unavailable":
					"Camlet no pudo guardar la configuración en disco, así que los cambios recientes podrían conservarse solo hasta que cierres la aplicación.",
			},
		},
	},
};
