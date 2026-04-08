import type { RendererLocale } from "./schema.js";

export const ptBrLocale: RendererLocale = {
	app: {
		title: "Camlet",
		close: "Fechar Camlet",
		overlayReady: "Shell configurável de webcam",
	},
	overlay: {
		dragHint: "Arraste aqui para mover o overlay",
		settingsHint:
			"Clique com o botão direito no overlay ou use o botão para alternar as configurações",
		summary: "Resumo do overlay",
		preview: "Pré-visualização do overlay da webcam",
		hintOpenSettings: "Clique com o botão direito para abrir as configurações",
		resizeHint:
			"Arraste uma alça para redimensionar. Clique em Concluir ou no overlay para sair.",
		resizeDone: "Concluir",
		resizeAction: "Redimensionar",
		resizeHandle: "Redimensionar overlay",
	},
	advanced: {
		title: "Configurações avançadas",
		description:
			"Mantenha o overlay principal limpo e use este painel apenas para controles extras e diagnósticos.",
	},
	sections: {
		settings: "Configurações",
		general: "Geral",
		appearance: "Aparência",
		camera: "Câmera",
		system: "Sistema",
		about: "Sobre",
	},
	settings: {
		actions: {
			open: "Abrir configurações",
			close: "Fechar configurações",
			resetAppearance: "Restaurar aparência padrão",
		},
		hints: {
			panel:
				"Os controles compactos ficam dentro do shell e aplicam as mudanças imediatamente.",
			escape: "Pressione Escape para fechar as configurações.",
		},
	},
	about: {
		description:
			"Use estas informações da build ao validar versões beta ou releases empacotadas.",
		labels: {
			appName: "Aplicativo",
			version: "Versão",
			channel: "Canal",
			mode: "Runtime",
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
		modes: {
			development: "Desenvolvimento",
			production: "Build empacotada",
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
		diagnostics: {
			title: "Resumo de diagnóstico",
			hint: "Copie este resumo ao relatar comportamento de build empacotada ou problemas específicos de plataforma.",
			copy: "Copiar diagnóstico",
			copied: "Copiado",
			copyFailed: "Cópia indisponível",
		},
	},
	language: {
		label: "Idioma",
		description: "Escolha como o Camlet deve exibir a interface.",
		options: {
			system: "Padrão do sistema",
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
		activeDevice: "Dispositivo ativo",
		overlaySize: "Tamanho do overlay",
		effectiveLanguage: "Idioma atual",
		windowPosition: "Posição",
		windowSize: "Tamanho da janela",
		platform: "Plataforma",
	},
	camera: {
		description:
			"Escolha a câmera preferida e revise o estado da pré-visualização sem sair do overlay.",
		actions: {
			retry: "Tentar câmera novamente",
		},
		labels: {
			device: "Dispositivo de câmera",
			activeDevice: "Dispositivo ativo",
			deviceCount: "Câmeras detectadas",
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
		message: {
			loading:
				"O Camlet está solicitando acesso à câmera e preparando a pré-visualização.",
			preview:
				"A pré-visualização da webcam está ativa dentro do shell de overlay.",
			"permission-denied":
				"O acesso à câmera foi negado. Permita o acesso e tente novamente.",
			"camera-in-use":
				"A câmera já está em uso ou não pode ser lida. Feche outros aplicativos que a estejam usando e tente novamente.",
			"no-camera":
				"Nenhum dispositivo de vídeo foi detectado. Conecte uma câmera e tente novamente.",
			"selected-device-unavailable":
				"A câmera salva não está disponível. Reconecte o dispositivo ou escolha outra câmera.",
			error:
				"O Camlet não conseguiu iniciar a pré-visualização da câmera. Tente novamente ou troque o dispositivo.",
			savedUnavailableUsingFallback:
				"A câmera salva anteriormente não estava disponível, então o Camlet mudou para outro dispositivo disponível.",
		},
	},
	appearance: {
		description:
			"Mantenha o overlay compacto e legível enquanto ajusta a apresentação do anel em tempo real.",
		labels: {
			theme: "Tema",
			shape: "Forma",
			fitMode: "Modo de encaixe",
			ringColor: "Cor do anel",
			ringThickness: "Espessura do anel",
			overlaySize: "Tamanho do overlay",
		},
		themes: {
			mint: "Menta",
			coral: "Coral",
			sky: "Céu",
			graphite: "Grafite",
		},
		shapes: {
			circle: "Círculo",
			roundedSquare: "Quadrado arredondado",
		},
		fitModes: {
			cover: "Cobrir",
			contain: "Conter",
		},
	},
	startup: {
		debugSummary: "Detalhes da inicialização",
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
		issues: {
			title: "Aviso de inicialização",
			messages: {
				"settings-recovered":
					"As configurações salvas foram reparadas ou redefinidas para valores seguros nesta sessão.",
				"settings-persistence-unavailable":
					"O Camlet não conseguiu salvar as configurações em disco, então as mudanças recentes podem durar apenas até o app fechar.",
			},
		},
	},
};
