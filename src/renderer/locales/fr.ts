import type { RendererLocale } from "./schema.js";

export const frLocale: RendererLocale = {
	app: {
		title: "Camlet",
		close: "Fermer Camlet",
		overlayReady: "Overlay webcam configurable",
	},
	overlay: {
		dragHint: "Faites glisser ici pour déplacer l'overlay",
		settingsHint:
			"Faites un clic droit sur l'overlay ou utilisez le bouton pour afficher les paramètres",
		summary: "Résumé de l'overlay",
		preview: "Aperçu de l'overlay webcam",
		hintOpenSettings: "Cliquez avec le bouton droit pour ouvrir les réglages",
		resizeHint:
			"Faites glisser une poignée pour redimensionner. Cliquez sur Terminer ou sur l'overlay pour quitter.",
		resizeDone: "Terminer",
		resizeAction: "Redimensionner",
		resizeHandle: "Redimensionner l'overlay",
	},
	advanced: {
		title: "Réglages avancés",
		description:
			"Gardez l'overlay principal propre et utilisez ce panneau uniquement pour les contrôles supplémentaires et le diagnostic.",
	},
	sections: {
		settings: "Paramètres",
		general: "Général",
		appearance: "Apparence",
		camera: "Caméra",
		system: "Système",
		about: "À propos",
	},
	settings: {
		actions: {
			open: "Ouvrir les paramètres",
			close: "Fermer les paramètres",
			resetAppearance: "Réinitialiser l'apparence par défaut",
		},
		hints: {
			panel:
				"Les contrôles compacts restent dans le shell et se mettent à jour en direct.",
			escape: "Appuyez sur Échap pour fermer les paramètres.",
		},
	},
	about: {
		description:
			"Utilisez ces informations de build pour valider les versions bêta ou les livraisons empaquetées.",
		labels: {
			appName: "Application",
			version: "Version",
			channel: "Canal",
			mode: "Exécution",
			packaged: "Empaquetée",
			platform: "Plateforme",
			displayProtocol: "Protocole d'affichage",
			electron: "Electron",
			chrome: "Chrome",
		},
		channels: {
			stable: "Stable",
			prerelease: "Bêta / préversion",
		},
		modes: {
			development: "Développement",
			production: "Build empaqueté",
		},
		displayProtocols: {
			wayland: "Wayland",
			x11: "X11",
			windows: "Bureau Windows",
			macos: "Bureau macOS",
			unknown: "Inconnu",
		},
		packagedValues: {
			yes: "Oui",
			no: "Non",
		},
		diagnostics: {
			title: "Résumé de diagnostic",
			hint: "Copiez ce résumé lorsque vous signalez un comportement de build empaqueté ou un problème spécifique à la plateforme.",
			copy: "Copier le diagnostic",
			copied: "Copié",
			copyFailed: "Copie indisponible",
		},
	},
	language: {
		label: "Langue",
		description: "Choisissez la langue utilisée par Camlet pour l'interface.",
		options: {
			system: "Par défaut du système",
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
		activeDevice: "Périphérique actif",
		overlaySize: "Taille de l'overlay",
		effectiveLanguage: "Langue actuelle",
		windowPosition: "Position",
		windowSize: "Taille de la fenêtre",
		platform: "Plateforme",
	},
	camera: {
		description:
			"Sélectionnez la caméra préférée et vérifiez l'état de l'aperçu en direct sans quitter l'overlay.",
		actions: {
			retry: "Réessayer la caméra",
		},
		labels: {
			device: "Périphérique caméra",
			activeDevice: "Périphérique actif",
			deviceCount: "Caméras détectées",
			permission: "État de l'aperçu",
			noDevices: "Aucune caméra disponible",
			none: "Aucun",
		},
		status: {
			loading: "Chargement de la caméra",
			preview: "Aperçu de la caméra actif",
			"permission-denied": "Autorisation refusée",
			"camera-in-use": "Caméra occupée",
			"no-camera": "Aucune caméra trouvée",
			"selected-device-unavailable":
				"Le périphérique enregistré n'est pas disponible",
			error: "Erreur de caméra",
		},
		message: {
			loading: "Camlet demande l'accès à la caméra et prépare l'aperçu.",
			preview:
				"L'aperçu webcam en direct fonctionne dans le shell de l'overlay.",
			"permission-denied":
				"L'accès à la caméra a été refusé. Autorisez l'accès puis réessayez.",
			"camera-in-use":
				"La caméra est déjà utilisée ou illisible. Fermez les autres applications qui l'utilisent puis réessayez.",
			"no-camera":
				"Aucun périphérique d'entrée vidéo n'a été détecté. Connectez une caméra puis réessayez.",
			"selected-device-unavailable":
				"La caméra enregistrée n'est pas disponible. Reconnectez-la ou choisissez un autre périphérique.",
			error:
				"Camlet n'a pas pu démarrer l'aperçu de la caméra. Réessayez ou changez de périphérique.",
			savedUnavailableUsingFallback:
				"La caméra enregistrée précédemment n'était pas disponible, Camlet a donc basculé vers un autre périphérique disponible.",
		},
	},
	appearance: {
		description:
			"Gardez l'overlay compact et lisible tout en ajustant la présentation de l'anneau en direct.",
		labels: {
			theme: "Thème",
			shape: "Forme",
			fitMode: "Mode d'ajustement",
			ringColor: "Couleur de l'anneau",
			ringThickness: "Épaisseur de l'anneau",
			overlaySize: "Taille de l'overlay",
		},
		themes: {
			mint: "Menthe",
			coral: "Corail",
			sky: "Ciel",
			graphite: "Graphite",
		},
		shapes: {
			circle: "Cercle",
			roundedSquare: "Carré arrondi",
		},
		fitModes: {
			cover: "Couvrir",
			contain: "Contenir",
		},
	},
	startup: {
		debugSummary: "Détails du démarrage",
		actions: {
			reload: "Recharger Camlet",
		},
		errors: {
			"preload-unavailable": {
				title: "Camlet n'a pas pu se connecter à son pont desktop",
				message:
					"L'API preload est absente, l'overlay ne peut donc pas terminer son démarrage. Redémarrez l'application et confirmez que les fichiers empaquetés sont intacts.",
			},
			"bootstrap-invalid": {
				title: "Camlet a reçu des données de démarrage invalides",
				message:
					"Le renderer a bloqué le démarrage parce que la charge bootstrap était incomplète ou mal formée. Redémarrez l'application et vérifiez la build actuelle.",
			},
			"bootstrap-load-failed": {
				title: "Camlet n'a pas pu charger les paramètres de démarrage",
				message:
					"Le renderer n'a pas pu charger la configuration de démarrage depuis le processus desktop. Redémarrez l'application. Si le problème persiste, testez avec un profil de configuration propre.",
			},
		},
		issues: {
			title: "Avis de démarrage",
			messages: {
				"settings-recovered":
					"Les paramètres enregistrés ont été réparés ou réinitialisés à des valeurs sûres pour cette session.",
				"settings-persistence-unavailable":
					"Camlet n'a pas pu enregistrer les paramètres sur disque, les modifications récentes pourraient donc ne durer que jusqu'à la fermeture de l'application.",
			},
		},
	},
};
