import type { RendererLocale } from "./schema.js";

export const deLocale: RendererLocale = {
	app: {
		title: "Camlet",
		close: "Camlet schließen",
		overlayReady: "Konfigurierbares Webcam-Overlay",
	},
	overlay: {
		dragHint: "Hier ziehen, um das Overlay zu verschieben",
		settingsHint:
			"Mit Rechtsklick auf das Overlay oder über die Schaltfläche die Einstellungen umschalten",
		summary: "Overlay-Zusammenfassung",
		preview: "Webcam-Overlay-Vorschau",
		hintOpenSettings: "Rechtsklick, um die Einstellungen zu öffnen",
		resizeHint:
			"Ziehe einen Griff zum Skalieren. Klicke auf Fertig oder auf das Overlay, um den Modus zu beenden.",
		resizeDone: "Fertig",
		resizeAction: "Größe ändern",
		resizeHandle: "Overlay-Größe ändern",
	},
	advanced: {
		title: "Erweiterte Einstellungen",
		description:
			"Halte das Haupt-Overlay sauber und verwende dieses Panel nur für zusätzliche Steuerung und Diagnose.",
	},
	sections: {
		settings: "Einstellungen",
		general: "Allgemein",
		appearance: "Darstellung",
		camera: "Kamera",
		system: "System",
		about: "Info",
	},
	settings: {
		actions: {
			open: "Einstellungen öffnen",
			close: "Einstellungen schließen",
			resetAppearance: "Standarddarstellung wiederherstellen",
		},
		hints: {
			panel:
				"Die kompakten Steuerelemente bleiben im Overlay und werden sofort aktualisiert.",
			escape: "Drücke Escape, um die Einstellungen zu schließen.",
		},
	},
	about: {
		windowTitle: "Über Camlet",
		description:
			"Verwende diese Build-Informationen beim Prüfen von Beta-Kandidaten oder paketierten Releases.",
		licenseLabel: "Lizenz",
		readmeTitle: "README-Umfang",
		labels: {
			appName: "App",
			version: "Version",
			channel: "Kanal",
			mode: "Laufzeit",
			packaged: "Paketiert",
			platform: "Plattform",
			displayProtocol: "Anzeigeprotokoll",
			electron: "Electron",
			chrome: "Chrome",
		},
		channels: {
			stable: "Stabil",
			prerelease: "Beta / Vorabversion",
		},
		modes: {
			development: "Entwicklung",
			production: "Paketierter Build",
		},
		displayProtocols: {
			wayland: "Wayland",
			x11: "X11",
			windows: "Windows-Desktop",
			macos: "macOS-Desktop",
			unknown: "Unbekannt",
		},
		packagedValues: {
			yes: "Ja",
			no: "Nein",
		},
		diagnostics: {
			title: "Diagnosezusammenfassung",
			hint: "Kopiere diese Zusammenfassung, wenn du paketiertes Verhalten oder plattformspezifische Probleme meldest.",
			copy: "Diagnose kopieren",
			copied: "Kopiert",
			copyFailed: "Kopieren nicht verfügbar",
		},
	},
	language: {
		label: "Sprache",
		description:
			"Lege fest, in welcher Sprache Camlet die Oberfläche anzeigen soll.",
		options: {
			system: "Systemstandard",
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
		activeDevice: "Aktives Gerät",
		overlaySize: "Overlay-Größe",
		effectiveLanguage: "Aktuelle Sprache",
		windowPosition: "Position",
		windowSize: "Fenstergröße",
		platform: "Plattform",
	},
	camera: {
		description:
			"Wähle die bevorzugte Kamera aus und prüfe den Status der Live-Vorschau, ohne das Overlay zu verlassen.",
		actions: {
			retry: "Kamera erneut versuchen",
		},
		labels: {
			device: "Kameragerät",
			activeDevice: "Aktives Gerät",
			deviceCount: "Erkannte Kameras",
			permission: "Vorschaustatus",
			noDevices: "Keine Kameras verfügbar",
			none: "Keine",
		},
		status: {
			loading: "Kamera wird geladen",
			preview: "Kameravorschau aktiv",
			"permission-denied": "Zugriff verweigert",
			"camera-in-use": "Kamera belegt",
			"no-camera": "Keine Kamera gefunden",
			"selected-device-unavailable": "Gespeichertes Gerät nicht verfügbar",
			error: "Kamerafehler",
		},
		message: {
			loading: "Camlet fordert Kamerazugriff an und bereitet die Vorschau vor.",
			preview: "Die Live-Webcam-Vorschau läuft im Overlay-Shell.",
			"permission-denied":
				"Der Kamerazugriff wurde verweigert. Erlaube den Zugriff und versuche es erneut.",
			"camera-in-use":
				"Die Kamera wird bereits verwendet oder kann nicht gelesen werden. Schließe andere Apps, die sie verwenden, und versuche es erneut.",
			"no-camera":
				"Es wurden keine Videoeingabegeräte erkannt. Schließe eine Kamera an und versuche es erneut.",
			"selected-device-unavailable":
				"Die gespeicherte Kamera ist nicht verfügbar. Schließe sie erneut an oder wähle ein anderes Gerät.",
			error:
				"Camlet konnte die Kameravorschau nicht starten. Versuche es erneut oder wechsle das Gerät.",
			savedUnavailableUsingFallback:
				"Die zuvor gespeicherte Kamera war nicht verfügbar, daher hat Camlet auf ein anderes verfügbares Gerät umgeschaltet.",
		},
	},
	appearance: {
		description:
			"Halte das Overlay kompakt und lesbar, während du die Ringdarstellung live anpasst.",
		labels: {
			theme: "Thema",
			shape: "Form",
			cornerRoundness: "Eckenradius",
			fitMode: "Anpassungsmodus",
			ringColor: "Ringfarbe",
			ringThickness: "Ringstärke",
			overlaySize: "Overlay-Größe",
		},
		themes: {
			mint: "Mint",
			ocean: "Ozean",
			ember: "Glut",
			orchid: "Orchidee",
			grove: "Hain",
			graphite: "Graphit",
		},
		shapes: {
			circle: "Kreis",
			roundedSquare: "Quadrat",
			diamond: "Diamant",
			rectangle: "Rechteck",
		},
		fitModes: {
			cover: "Füllen",
			contain: "Einpassen",
		},
	},
	startup: {
		debugSummary: "Startdetails",
		actions: {
			reload: "Camlet neu laden",
		},
		errors: {
			"preload-unavailable": {
				title:
					"Camlet konnte keine Verbindung zu seiner Desktop-Brücke herstellen",
				message:
					"Die Preload-API fehlt, daher kann das Overlay den Start nicht abschließen. Starte die App neu und prüfe, ob die paketierten Dateien intakt sind.",
			},
			"bootstrap-invalid": {
				title: "Camlet hat ungültige Startdaten erhalten",
				message:
					"Der Renderer hat den Start blockiert, weil die Bootstrap-Daten unvollständig oder fehlerhaft waren. Starte die App neu und überprüfe den aktuellen Build-Ausgabestand.",
			},
			"bootstrap-load-failed": {
				title: "Camlet konnte die Starteinstellungen nicht laden",
				message:
					"Der Renderer konnte die Startkonfiguration nicht aus dem Desktop-Prozess laden. Starte die App neu. Wenn das Problem bestehen bleibt, teste mit einem sauberen Einstellungsprofil.",
			},
		},
		issues: {
			title: "Start-Hinweis",
			messages: {
				"settings-recovered":
					"Gespeicherte Einstellungen wurden für diese Sitzung repariert oder auf sichere Werte zurückgesetzt.",
				"settings-persistence-unavailable":
					"Camlet konnte die Einstellungen nicht auf die Festplatte schreiben, daher bleiben aktuelle Änderungen möglicherweise nur bis zum Schließen der App erhalten.",
			},
		},
	},
};
