import type { RendererLocale } from "./schema.js";

export const ptBrLocale: RendererLocale = {
	app: {
		title: "Camlet",
		close: "Fechar Camlet",
	},
	overlay: {
		preview: "Pré-visualização do overlay da webcam",
		hintOpenSettings: "Clique com o botão direito para abrir as configurações",
		resizeDone: "Concluir",
		resizeAction: "Redimensionar",
	},
	advanced: {
		title: "Configurações avançadas",
	},
	sections: {
		system: "Sistema",
		about: "Sobre",
	},
	settings: {
		actions: {
			resetAppearance: "Restaurar aparência padrão",
		},
	},
	about: {
		windowTitle: "Sobre o Camlet",
		licenseLabel: "Licença",
		labels: {
			version: "Versão",
			channel: "Canal",
			packaged: "Empacotado",
			platform: "Plataforma",
			displayProtocol: "Protocolo de exibição",
			electron: "Electron",
			chrome: "Chrome",
		},
		channels: {
			stable: "Estável",
			prerelease: "Beta / pré-release",
		},
		displayProtocols: {
			wayland: "Wayland",
			x11: "X11",
			windows: "Desktop Windows",
			macos: "Desktop macOS",
			unknown: "Desconhecido",
		},
		packagedValues: {
			yes: "Sim",
			no: "Não",
		},
	},
	language: {
		label: "Idioma",
		options: {
			system: "Padrão do sistema",
			en: "English",
			"pt-BR": "Português (Brasil)",
		},
	},
	camera: {
		actions: {
			retry: "Tentar câmera novamente",
		},
		labels: {
			device: "Dispositivo de câmera",
			activeDevice: "Dispositivo ativo",
			permission: "Estado da visualização",
			noDevices: "Nenhuma câmera disponível",
			none: "Nenhum",
		},
		status: {
			loading: "Carregando câmera",
			preview: "Pré-visualização da câmera ativa",
			"permission-denied": "Permissão negada",
			"camera-in-use": "Câmera ocupada",
			"no-camera": "Nenhuma câmera encontrada",
			"selected-device-unavailable": "Dispositivo salvo indisponível",
			error: "Erro de câmera",
		},
	},
	appearance: {
		labels: {
			theme: "Tema",
			shape: "Forma",
			cornerRoundness: "Arredondamento",
			fitMode: "Modo de encaixe",
			ringThickness: "Espessura do anel",
		},
		themes: {
			mint: "Menta",
			ocean: "Oceano",
			ember: "Brasa",
			orchid: "Orquídea",
			grove: "Bosque",
			graphite: "Grafite",
		},
		shapes: {
			circle: "Círculo",
			roundedSquare: "Quadrado",
			diamond: "Diamante",
			rectangleY: "Retângulo Y",
			rectangleX: "Retângulo X",
		},
		fitModes: {
			cover: "Cobrir",
			contain: "Conter",
		},
	},
	startup: {
		actions: {
			reload: "Recarregar Camlet",
		},
		errors: {
			"preload-unavailable": {
				title:
					"O Camlet não conseguiu se conectar à sua ponte da área de trabalho",
				message:
					"A API de preload está ausente, então o overlay não pode concluir a inicialização. Reinicie o app e confirme se os arquivos empacotados estão íntegros.",
			},
			"bootstrap-invalid": {
				title: "O Camlet recebeu dados de inicialização inválidos",
				message:
					"O renderer bloqueou a inicialização porque o payload de bootstrap estava incompleto ou malformado. Reinicie o app e verifique a build atual.",
			},
			"bootstrap-load-failed": {
				title: "O Camlet não conseguiu carregar as configurações iniciais",
				message:
					"O renderer não conseguiu carregar a configuração inicial a partir do processo desktop. Reinicie o app. Se continuar falhando, teste com um perfil limpo de configurações.",
			},
		},
	},
};
