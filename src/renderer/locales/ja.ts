import type { RendererLocale } from "./schema.js";

export const jaLocale: RendererLocale = {
	app: {
		title: "Camlet",
		close: "Camlet を終了",
		overlayReady: "設定可能なWebカメラオーバーレイ",
	},
	overlay: {
		dragHint: "ここをドラッグしてオーバーレイを移動",
		settingsHint: "オーバーレイを右クリックするか、ボタンで設定を切り替えます",
		summary: "オーバーレイ概要",
		preview: "Webカメラオーバーレイのプレビュー",
		hintOpenSettings: "右クリックで設定を開きます",
		resizeHint:
			"ハンドルをドラッグしてサイズを変更します。完了を押すかオーバーレイをクリックすると終了します。",
		resizeDone: "完了",
		resizeAction: "サイズ変更",
		resizeHandle: "オーバーレイのサイズを変更",
	},
	advanced: {
		title: "詳細設定",
		description:
			"メインのオーバーレイはシンプルに保ち、追加の操作や診断が必要なときだけこのパネルを使います。",
	},
	sections: {
		settings: "設定",
		general: "一般",
		appearance: "外観",
		camera: "カメラ",
		system: "システム",
		about: "情報",
	},
	settings: {
		actions: {
			open: "設定を開く",
			close: "設定を閉じる",
			resetAppearance: "外観を既定値に戻す",
		},
		hints: {
			panel: "コンパクトな操作はシェル内に収まり、変更はその場で反映されます。",
			escape: "設定を閉じるには Escape を押します。",
		},
	},
	about: {
		windowTitle: "Camlet について",
		description:
			"ベータ候補やパッケージ版を検証するときは、このビルド情報を使用してください。",
		licenseLabel: "ライセンス",
		readmeTitle: "README の概要",
		labels: {
			appName: "アプリ",
			version: "バージョン",
			channel: "チャネル",
			mode: "ランタイム",
			packaged: "パッケージ済み",
			platform: "プラットフォーム",
			displayProtocol: "表示プロトコル",
			electron: "Electron",
			chrome: "Chrome",
		},
		channels: {
			stable: "安定版",
			prerelease: "ベータ / プレリリース",
		},
		modes: {
			development: "開発",
			production: "パッケージ版ビルド",
		},
		displayProtocols: {
			wayland: "Wayland",
			x11: "X11",
			windows: "Windowsデスクトップ",
			macos: "macOSデスクトップ",
			unknown: "不明",
		},
		packagedValues: {
			yes: "はい",
			no: "いいえ",
		},
		diagnostics: {
			title: "診断サマリー",
			hint: "パッケージ版の挙動やプラットフォーム固有の問題を報告するときは、このサマリーをコピーしてください。",
			copy: "診断をコピー",
			copied: "コピー済み",
			copyFailed: "コピー不可",
		},
	},
	language: {
		label: "言語",
		description: "Camlet のインターフェース表示言語を選択します。",
		options: {
			system: "システム既定",
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
		activeDevice: "使用中のデバイス",
		overlaySize: "オーバーレイサイズ",
		effectiveLanguage: "現在の言語",
		windowPosition: "位置",
		windowSize: "ウィンドウサイズ",
		platform: "プラットフォーム",
	},
	camera: {
		description:
			"オーバーレイを離れずに、使用するカメラを選択してライブプレビューの状態を確認できます。",
		actions: {
			retry: "カメラを再試行",
		},
		labels: {
			device: "カメラデバイス",
			activeDevice: "使用中のデバイス",
			deviceCount: "検出されたカメラ",
			permission: "プレビュー状態",
			noDevices: "利用可能なカメラがありません",
			none: "なし",
		},
		status: {
			loading: "カメラを読み込み中",
			preview: "カメラプレビューが有効です",
			"permission-denied": "権限が拒否されました",
			"camera-in-use": "カメラは使用中です",
			"no-camera": "カメラが見つかりません",
			"selected-device-unavailable": "保存されたデバイスは利用できません",
			error: "カメラエラー",
		},
		message: {
			loading:
				"Camlet はカメラへのアクセスを要求し、プレビューを準備しています。",
			preview: "ライブWebカメラプレビューがオーバーレイ内で動作しています。",
			"permission-denied":
				"カメラへのアクセスが拒否されました。アクセスを許可してから再試行してください。",
			"camera-in-use":
				"カメラはすでに使用中か、読み取れません。利用中の他のアプリを閉じて再試行してください。",
			"no-camera":
				"映像入力デバイスが検出されませんでした。カメラを接続して再試行してください。",
			"selected-device-unavailable":
				"保存されていたカメラは利用できません。再接続するか、別のデバイスを選択してください。",
			error:
				"Camlet はカメラプレビューを開始できませんでした。再試行するか、別のデバイスに切り替えてください。",
			savedUnavailableUsingFallback:
				"以前保存されていたカメラが利用できなかったため、Camlet は別の利用可能なデバイスに切り替えました。",
		},
	},
	appearance: {
		description:
			"リング表示を調整しながら、オーバーレイをコンパクトで見やすい状態に保ちます。",
		labels: {
			theme: "テーマ",
			shape: "形状",
			cornerRoundness: "角の丸み",
			fitMode: "表示モード",
			ringColor: "リングカラー",
			ringThickness: "リングの太さ",
			overlaySize: "オーバーレイサイズ",
		},
		themes: {
			mint: "ミント",
			ocean: "オーシャン",
			ember: "エンバー",
			orchid: "オーキッド",
			grove: "グローブ",
			graphite: "グラファイト",
		},
		shapes: {
			circle: "円",
			roundedSquare: "四角形",
			diamond: "ダイヤ",
			rectangleY: "長方形 Y",
			rectangleX: "長方形 X",
		},
		fitModes: {
			cover: "カバー",
			contain: "全体表示",
		},
	},
	startup: {
		debugSummary: "起動の詳細",
		actions: {
			reload: "Camlet を再読み込み",
		},
		errors: {
			"preload-unavailable": {
				title: "Camlet はデスクトップブリッジに接続できませんでした",
				message:
					"preload API が見つからないため、オーバーレイは起動を完了できません。アプリを再起動し、パッケージされたファイルが壊れていないことを確認してください。",
			},
			"bootstrap-invalid": {
				title: "Camlet は無効な起動データを受け取りました",
				message:
					"bootstrap ペイロードが不完全または不正なため、renderer が起動を停止しました。アプリを再起動し、現在のビルド出力を確認してください。",
			},
			"bootstrap-load-failed": {
				title: "Camlet は起動設定を読み込めませんでした",
				message:
					"renderer はデスクトッププロセスから初期設定を読み込めませんでした。アプリを再起動してください。改善しない場合は、設定プロファイルを新しくして確認してください。",
			},
		},
		issues: {
			title: "起動時の通知",
			messages: {
				"settings-recovered":
					"保存されていた設定は、このセッションのために安全な値へ修復またはリセットされました。",
				"settings-persistence-unavailable":
					"Camlet は設定をディスクへ保存できなかったため、最近の変更はアプリを閉じるまでしか保持されない可能性があります。",
			},
		},
	},
};
