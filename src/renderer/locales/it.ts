import type { RendererLocale } from "./schema.js";

export const itLocale: RendererLocale = {
	app: {
		title: "Camlet",
		close: "Chiudi Camlet",
		overlayReady: "Overlay webcam configurabile",
	},
	overlay: {
		dragHint: "Trascina qui per spostare l'overlay",
		settingsHint:
			"Fai clic destro sull'overlay oppure usa il pulsante per aprire le impostazioni",
		summary: "Riepilogo overlay",
		preview: "Anteprima overlay webcam",
		hintOpenSettings: "Fai clic destro per aprire le impostazioni",
		resizeHint:
			"Trascina una maniglia per ridimensionare. Fai clic su Fine o sull'overlay per uscire.",
		resizeDone: "Fine",
		resizeAction: "Ridimensiona",
		resizeHandle: "Ridimensiona overlay",
	},
	advanced: {
		title: "Impostazioni avanzate",
		description:
			"Mantieni pulito l'overlay principale e usa questo pannello solo per controlli extra e diagnostica.",
	},
	sections: {
		settings: "Impostazioni",
		general: "Generale",
		appearance: "Aspetto",
		camera: "Fotocamera",
		system: "Sistema",
		about: "Informazioni",
	},
	settings: {
		actions: {
			open: "Apri impostazioni",
			close: "Chiudi impostazioni",
			resetAppearance: "Ripristina aspetto predefinito",
		},
		hints: {
			panel:
				"I controlli compatti restano all'interno dello shell e si aggiornano in tempo reale.",
			escape: "Premi Esc per chiudere le impostazioni.",
		},
	},
	about: {
		description:
			"Usa queste informazioni di build quando convalidi candidate beta o release pacchettizzate.",
		labels: {
			appName: "App",
			version: "Versione",
			channel: "Canale",
			mode: "Runtime",
			packaged: "Pacchettizzata",
			platform: "Piattaforma",
			displayProtocol: "Protocollo display",
			electron: "Electron",
			chrome: "Chrome",
		},
		channels: {
			stable: "Stabile",
			prerelease: "Beta / prerelease",
		},
		modes: {
			development: "Sviluppo",
			production: "Build pacchettizzata",
		},
		displayProtocols: {
			wayland: "Wayland",
			x11: "X11",
			windows: "Desktop Windows",
			macos: "Desktop macOS",
			unknown: "Sconosciuto",
		},
		packagedValues: {
			yes: "Sì",
			no: "No",
		},
		diagnostics: {
			title: "Riepilogo diagnostico",
			hint: "Copia questo riepilogo quando segnali comportamenti della build pacchettizzata o problemi specifici della piattaforma.",
			copy: "Copia diagnostica",
			copied: "Copiato",
			copyFailed: "Copia non disponibile",
		},
	},
	language: {
		label: "Lingua",
		description: "Scegli come Camlet deve mostrare l'interfaccia.",
		options: {
			system: "Predefinita del sistema",
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
		activeDevice: "Dispositivo attivo",
		overlaySize: "Dimensione overlay",
		effectiveLanguage: "Lingua corrente",
		windowPosition: "Posizione",
		windowSize: "Dimensione finestra",
		platform: "Piattaforma",
	},
	camera: {
		description:
			"Seleziona la fotocamera preferita e controlla lo stato dell'anteprima live senza uscire dall'overlay.",
		actions: {
			retry: "Riprova fotocamera",
		},
		labels: {
			device: "Dispositivo fotocamera",
			activeDevice: "Dispositivo attivo",
			deviceCount: "Fotocamere rilevate",
			permission: "Stato anteprima",
			noDevices: "Nessuna fotocamera disponibile",
			none: "Nessuno",
		},
		status: {
			loading: "Caricamento fotocamera",
			preview: "Anteprima fotocamera attiva",
			"permission-denied": "Permesso negato",
			"camera-in-use": "Fotocamera occupata",
			"no-camera": "Nessuna fotocamera trovata",
			"selected-device-unavailable": "Dispositivo salvato non disponibile",
			error: "Errore fotocamera",
		},
		message: {
			loading:
				"Camlet sta richiedendo l'accesso alla fotocamera e preparando l'anteprima.",
			preview:
				"L'anteprima live della webcam è attiva nello shell dell'overlay.",
			"permission-denied":
				"L'accesso alla fotocamera è stato negato. Consenti l'accesso e riprova.",
			"camera-in-use":
				"La fotocamera è già in uso o non è leggibile. Chiudi le altre app che la stanno usando e riprova.",
			"no-camera":
				"Non sono stati rilevati dispositivi di input video. Collega una fotocamera e riprova.",
			"selected-device-unavailable":
				"La fotocamera salvata non è disponibile. Ricollegala oppure scegli un altro dispositivo.",
			error:
				"Camlet non è riuscito ad avviare l'anteprima della fotocamera. Riprova oppure cambia dispositivo.",
			savedUnavailableUsingFallback:
				"La fotocamera salvata in precedenza non era disponibile, quindi Camlet è passato a un altro dispositivo disponibile.",
		},
	},
	appearance: {
		description:
			"Mantieni l'overlay compatto e leggibile mentre regoli in tempo reale la presentazione dell'anello.",
		labels: {
			theme: "Tema",
			shape: "Forma",
			fitMode: "Modalità adattamento",
			ringColor: "Colore anello",
			ringThickness: "Spessore anello",
			overlaySize: "Dimensione overlay",
		},
		themes: {
			mint: "Menta",
			coral: "Corallo",
			sky: "Cielo",
			graphite: "Grafite",
		},
		shapes: {
			circle: "Cerchio",
			roundedSquare: "Quadrato arrotondato",
		},
		fitModes: {
			cover: "Copri",
			contain: "Contieni",
		},
	},
	startup: {
		debugSummary: "Dettagli di avvio",
		actions: {
			reload: "Ricarica Camlet",
		},
		errors: {
			"preload-unavailable": {
				title: "Camlet non è riuscito a connettersi al bridge desktop",
				message:
					"L'API preload non è disponibile, quindi l'overlay non può completare l'avvio. Riavvia l'app e verifica che i file pacchettizzati siano integri.",
			},
			"bootstrap-invalid": {
				title: "Camlet ha ricevuto dati di avvio non validi",
				message:
					"Il renderer ha bloccato l'avvio perché il payload di bootstrap era incompleto o malformato. Riavvia l'app e verifica l'output della build corrente.",
			},
			"bootstrap-load-failed": {
				title: "Camlet non è riuscito a caricare le impostazioni iniziali",
				message:
					"Il renderer non è riuscito a caricare la configurazione iniziale dal processo desktop. Riavvia l'app. Se il problema continua, prova con un profilo impostazioni pulito.",
			},
		},
		issues: {
			title: "Avviso di avvio",
			messages: {
				"settings-recovered":
					"Le impostazioni salvate sono state riparate o reimpostate su valori sicuri per questa sessione.",
				"settings-persistence-unavailable":
					"Camlet non è riuscito a salvare le impostazioni su disco, quindi le modifiche recenti potrebbero durare solo fino alla chiusura dell'app.",
			},
		},
	},
};
