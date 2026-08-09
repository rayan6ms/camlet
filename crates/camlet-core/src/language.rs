//! Supported languages and compile-time-complete message catalogs.

use serde::{Deserialize, Serialize};

/// Persisted language selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppLanguage {
    /// Resolve from the operating-system locale.
    #[default]
    #[serde(rename = "system")]
    System,
    /// English.
    #[serde(rename = "en")]
    English,
    /// Brazilian Portuguese.
    #[serde(rename = "pt-BR")]
    PortugueseBrazil,
}

impl AppLanguage {
    /// All user-selectable values in stable menu order.
    pub const ALL: [Self; 3] = [Self::System, Self::English, Self::PortugueseBrazil];
}

/// Concrete message catalog selected after system-locale resolution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SupportedLanguage {
    /// English fallback catalog.
    #[default]
    English,
    /// Brazilian Portuguese catalog.
    PortugueseBrazil,
}

/// Strings for a camera status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraStatusMessages {
    /// Capture startup is in progress.
    pub loading: &'static str,
    /// Preview is active.
    pub preview: &'static str,
    /// Permission was denied.
    pub permission_denied: &'static str,
    /// Device is owned by another process.
    pub camera_in_use: &'static str,
    /// No device exists.
    pub no_camera: &'static str,
    /// Saved device disappeared.
    pub selected_device_unavailable: &'static str,
    /// Unclassified backend error.
    pub error: &'static str,
}

/// Compile-time-complete product strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Catalog {
    /// Application title.
    pub app_title: &'static str,
    /// Quit action.
    pub close_app: &'static str,
    /// Accessible preview label.
    pub preview: &'static str,
    /// First-run settings hint.
    pub settings_hint: &'static str,
    /// Resize action.
    pub resize: &'static str,
    /// Finish resize action.
    pub resize_done: &'static str,
    /// Advanced menu heading.
    pub advanced: &'static str,
    /// System section heading.
    pub system: &'static str,
    /// About section heading.
    pub about: &'static str,
    /// Reset appearance action.
    pub reset_appearance: &'static str,
    /// About window title.
    pub about_window: &'static str,
    /// Native application description.
    pub about_description: &'static str,
    /// License label.
    pub license: &'static str,
    /// Version label.
    pub version: &'static str,
    /// Platform label.
    pub platform: &'static str,
    /// Display protocol label.
    pub display_protocol: &'static str,
    /// Runtime label.
    pub runtime: &'static str,
    /// Release channel label.
    pub release_channel: &'static str,
    /// Stable release channel.
    pub stable_channel: &'static str,
    /// Prerelease channel.
    pub prerelease_channel: &'static str,
    /// Copy privacy-safe diagnostics action.
    pub copy_diagnostics: &'static str,
    /// Diagnostics copy acknowledgement.
    pub diagnostics_copied: &'static str,
    /// Language selector label.
    pub language: &'static str,
    /// System-language option.
    pub language_system: &'static str,
    /// English option.
    pub language_english: &'static str,
    /// Brazilian Portuguese option.
    pub language_portuguese_brazil: &'static str,
    /// Retry-camera action.
    pub retry_camera: &'static str,
    /// Camera device label.
    pub camera_device: &'static str,
    /// Camera capture-rate label.
    pub camera_fps: &'static str,
    /// Active camera label.
    pub active_camera: &'static str,
    /// Camera status label.
    pub preview_state: &'static str,
    /// Empty camera list label.
    pub no_devices: &'static str,
    /// Empty value label.
    pub none: &'static str,
    /// Camera status strings.
    pub camera_status: CameraStatusMessages,
    /// Theme label.
    pub theme: &'static str,
    /// Shape label.
    pub shape: &'static str,
    /// Corner radius label.
    pub corner_roundness: &'static str,
    /// Fit mode label.
    pub fit_mode: &'static str,
    /// Ring width label.
    pub ring_thickness: &'static str,
    /// Theme names in `ThemeId::ALL` order.
    pub themes: [&'static str; 6],
    /// Shape names in `OverlayShape::ALL` order.
    pub shapes: [&'static str; 6],
    /// Fit names in `PreviewFitMode::ALL` order.
    pub fit_modes: [&'static str; 2],
    /// Generic startup error title.
    pub startup_error_title: &'static str,
    /// Generic startup recovery guidance.
    pub startup_error_message: &'static str,
    /// Reload action.
    pub reload: &'static str,
    /// Continue with safe in-memory defaults.
    pub continue_with_defaults: &'static str,
}

/// English messages.
pub const ENGLISH: Catalog = Catalog {
    app_title: "Camlet",
    close_app: "Close Camlet",
    preview: "Webcam overlay preview",
    settings_hint: "Right click to open settings",
    resize: "Resize",
    resize_done: "Done",
    advanced: "Advanced settings",
    system: "System",
    about: "About",
    reset_appearance: "Reset appearance defaults",
    about_window: "About Camlet",
    about_description: "Camlet is a lightweight native floating camera overlay built with Rust and Iced.",
    license: "License",
    version: "Version",
    platform: "Platform",
    display_protocol: "Protocol",
    runtime: "Runtime",
    release_channel: "Channel",
    stable_channel: "Stable",
    prerelease_channel: "Beta / prerelease",
    copy_diagnostics: "Copy diagnostics",
    diagnostics_copied: "Diagnostics copied",
    language: "Language",
    language_system: "System default",
    language_english: "English",
    language_portuguese_brazil: "Português (Brasil)",
    retry_camera: "Retry camera",
    camera_device: "Camera device",
    camera_fps: "Camera frame rate",
    active_camera: "Active device",
    preview_state: "Preview state",
    no_devices: "No camera devices",
    none: "None",
    camera_status: CameraStatusMessages {
        loading: "Loading camera",
        preview: "Camera preview active",
        permission_denied: "Permission denied",
        camera_in_use: "Camera busy",
        no_camera: "No camera found",
        selected_device_unavailable: "Selected device unavailable",
        error: "Camera error",
    },
    theme: "Theme",
    shape: "Shape",
    corner_roundness: "Corner roundness",
    fit_mode: "Fit mode",
    ring_thickness: "Ring thickness",
    themes: ["Mint", "Ocean", "Ember", "Orchid", "Grove", "Graphite"],
    shapes: [
        "Original",
        "Circle",
        "Square",
        "Rectangle Y",
        "Rectangle X",
        "Diamond",
    ],
    fit_modes: ["Cover", "Contain"],
    startup_error_title: "Camlet could not start",
    startup_error_message: "Restart Camlet. If the problem continues, try a clean settings profile.",
    reload: "Reload Camlet",
    continue_with_defaults: "Continue with safe defaults",
};

/// Brazilian Portuguese messages.
pub const PORTUGUESE_BRAZIL: Catalog = Catalog {
    app_title: "Camlet",
    close_app: "Fechar Camlet",
    preview: "Pré-visualização do overlay da webcam",
    settings_hint: "Clique com o botão direito para abrir as configurações",
    resize: "Redimensionar",
    resize_done: "Concluir",
    advanced: "Configurações avançadas",
    system: "Sistema",
    about: "Sobre",
    reset_appearance: "Restaurar aparência padrão",
    about_window: "Sobre o Camlet",
    about_description: "O Camlet é um overlay de câmera flutuante, leve e nativo, feito com Rust e Iced.",
    license: "Licença",
    version: "Versão",
    platform: "Plataforma",
    display_protocol: "Protocolo de exibição",
    runtime: "Runtime",
    release_channel: "Canal",
    stable_channel: "Estável",
    prerelease_channel: "Beta / pré-lançamento",
    copy_diagnostics: "Copiar diagnóstico",
    diagnostics_copied: "Diagnóstico copiado",
    language: "Idioma",
    language_system: "Padrão do sistema",
    language_english: "English",
    language_portuguese_brazil: "Português (Brasil)",
    retry_camera: "Tentar câmera novamente",
    camera_device: "Dispositivo de câmera",
    camera_fps: "Taxa de quadros da câmera",
    active_camera: "Dispositivo ativo",
    preview_state: "Estado da visualização",
    no_devices: "Nenhuma câmera disponível",
    none: "Nenhum",
    camera_status: CameraStatusMessages {
        loading: "Carregando câmera",
        preview: "Pré-visualização da câmera ativa",
        permission_denied: "Permissão negada",
        camera_in_use: "Câmera ocupada",
        no_camera: "Nenhuma câmera encontrada",
        selected_device_unavailable: "Dispositivo salvo indisponível",
        error: "Erro de câmera",
    },
    theme: "Tema",
    shape: "Forma",
    corner_roundness: "Arredondamento",
    fit_mode: "Modo de encaixe",
    ring_thickness: "Espessura do anel",
    themes: ["Menta", "Oceano", "Brasa", "Orquídea", "Bosque", "Grafite"],
    shapes: [
        "Original",
        "Círculo",
        "Quadrado",
        "Retângulo Y",
        "Retângulo X",
        "Diamante",
    ],
    fit_modes: ["Cobrir", "Conter"],
    startup_error_title: "O Camlet não conseguiu iniciar",
    startup_error_message: "Reinicie o Camlet. Se o problema continuar, tente um perfil limpo de configurações.",
    reload: "Recarregar Camlet",
    continue_with_defaults: "Continuar com padrões seguros",
};

/// Resolves a system locale tag into a supported language.
#[must_use]
pub fn resolve_supported_language(system_locale: Option<&str>) -> SupportedLanguage {
    let normalized = system_locale
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.replace('_', "-").to_lowercase());

    match normalized.as_deref() {
        Some(value) if value == "pt" || value.starts_with("pt-") => {
            SupportedLanguage::PortugueseBrazil
        }
        _ => SupportedLanguage::English,
    }
}

/// Resolves the persisted selector and returns its complete catalog.
#[must_use]
pub fn catalog(language: AppLanguage, system_locale: Option<&str>) -> &'static Catalog {
    match language {
        AppLanguage::English => &ENGLISH,
        AppLanguage::PortugueseBrazil => &PORTUGUESE_BRAZIL,
        AppLanguage::System => match resolve_supported_language(system_locale) {
            SupportedLanguage::English => &ENGLISH,
            SupportedLanguage::PortugueseBrazil => &PORTUGUESE_BRAZIL,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AppLanguage, ENGLISH, PORTUGUESE_BRAZIL, SupportedLanguage, catalog,
        resolve_supported_language,
    };

    #[test]
    fn resolves_language_prefixes_and_fallback() {
        assert_eq!(
            resolve_supported_language(Some("pt_PT")),
            SupportedLanguage::PortugueseBrazil
        );
        assert_eq!(
            resolve_supported_language(Some("en-US")),
            SupportedLanguage::English
        );
        assert_eq!(
            resolve_supported_language(Some("nl-NL")),
            SupportedLanguage::English
        );
    }

    #[test]
    fn explicit_language_ignores_system_locale() {
        assert_eq!(catalog(AppLanguage::English, Some("pt-BR")), &ENGLISH);
        assert_eq!(
            catalog(AppLanguage::PortugueseBrazil, Some("en-US")),
            &PORTUGUESE_BRAZIL
        );
    }

    #[test]
    fn catalogs_have_stable_option_counts() {
        for catalog in [ENGLISH, PORTUGUESE_BRAZIL] {
            assert_eq!(catalog.themes.len(), 6);
            assert_eq!(catalog.shapes.len(), 6);
            assert_eq!(catalog.fit_modes.len(), 2);
        }
    }
}
