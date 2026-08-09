use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use camlet_camera::{
    CameraDevice, CameraError, CameraWorker, CameraWorkerCommand, CameraWorkerEvent,
    CameraWorkerEvents, CaptureRequest, NokhwaFrameSource, SyntheticFrameSource, VideoFrame,
};
use camlet_core::appearance::{
    CORNER_ROUNDNESS_OPTIONS, MAXIMUM_OVERLAY_SIZE, RING_THICKNESS_OPTIONS,
};
use camlet_core::geometry::MINIMUM_WINDOW_SIZE;
use camlet_core::language::{AppLanguage, Catalog, catalog};
use camlet_core::menu::MenuModel;
use camlet_core::settings::{AppSettings, CAMERA_FPS_OPTIONS, write_settings};
use camlet_core::state::{Action, AppState, CameraOption, CameraStatus, Effect};
use camlet_core::{APP_NAME, FrameSourceKind};
use iced::futures::SinkExt;
use iced::keyboard::key::{Code, Named, Physical};
use iced::keyboard::{Key, Modifiers};
use iced::widget::{
    button, column, container, mouse_area, opaque, row, rule, scrollable, shader, space, stack,
    text,
};
use iced::{
    Background, Border, Color, Element, Length, Point, Size, Subscription, Task, Theme, clipboard,
    event, keyboard, mouse, stream, theme, window,
};
use num_traits::ToPrimitive;

use crate::RunError;
use crate::automation::{AutomationAction, AutomationError, AutomationSession};
use crate::gpu_overlay::{OverlayProgram, validate_source_frame};
use crate::profile::{NativeProfile, load as load_profile};
use crate::renderer::render_overlay;
use crate::screenshot::{
    ScreenshotError, validate_reference_alpha, validate_transparent_overlay,
    write_white_background_ppm,
};
use crate::{AutomationMode, Cli};

const PERSISTENCE_DEBOUNCE: Duration = Duration::from_millis(250);
const MENU_WINDOW_WIDTH: f32 = 286.0;
const MENU_WINDOW_HEIGHT: f32 = 310.0;
const SUBMENU_WINDOW_WIDTH: f32 = 330.0;
const MENU_WINDOW_GAP: f32 = 4.0;
const ABOUT_WINDOW_WIDTH: f32 = 380.0;
const ABOUT_WINDOW_HEIGHT: f32 = 430.0;
const AUTHOR_URL: &str = "https://github.com/rayan6ms";
const PROJECT_URL: &str = "https://github.com/rayan6ms/camlet";
const ISSUES_URL: &str = "https://github.com/rayan6ms/camlet/issues";

struct Camlet {
    product: AppState,
    automation: AutomationMode,
    frame_source_kind: FrameSourceKind,
    camera_worker: Option<CameraWorker>,
    camera_events: Option<CameraWorkerEvents>,
    source_frame: Option<Arc<VideoFrame>>,
    placeholder_frame: Arc<VideoFrame>,
    frame_revision: u64,
    capture_first_frame_at: Option<Instant>,
    capture_frame_count: u64,
    preview_error: Option<String>,
    panel: Panel,
    screenshot_path: Option<PathBuf>,
    screenshot_state: ScreenshotState,
    window_id: Option<window::Id>,
    settings_window_id: Option<window::Id>,
    submenu_window_id: Option<window::Id>,
    active_submenu: Option<MenuPage>,
    preview_cursor: Point,
    menu_origin: Point,
    scale_factor: f32,
    monitor_maximum: u16,
    monitor_size: Option<Size>,
    profile_directory: Option<PathBuf>,
    settings_path: Option<PathBuf>,
    settings_writable: bool,
    profile_origin: &'static str,
    persist_generation: u64,
    persist_scheduled: bool,
    pending_window_position: Option<(i32, i32)>,
    pending_window_size: Option<u16>,
    lifecycle: Lifecycle,
    diagnostics_state: DiagnosticsState,
    automation_session: Option<AutomationSession>,
    system_locale: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    Preview,
    Menu,
    About,
    StartupError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuPage {
    Theme,
    Shape,
    Language,
    Camera,
    Advanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticsState {
    Ready,
    Copied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScreenshotState {
    Waiting,
    Requested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Running,
    ShuttingDown,
}

#[derive(Debug, Clone)]
enum Message {
    WindowEvent(window::Id, window::Event),
    MonitorSize(Option<Size>),
    KeyPressed {
        window_id: window::Id,
        key: Key,
        physical: Physical,
        modifiers: Modifiers,
    },
    PreviewPressed,
    PreviewCursorMoved(Point),
    OpenMenu,
    OpenSubmenu(MenuPage),
    MenuProduct(Action),
    ClosePanel,
    WindowOpened,
    Product(Action),
    CameraPoll(CameraPollResult),
    PersistReady(u64),
    CaptureScreenshot,
    ScreenshotReady(window::Screenshot),
    CameraShutdownFinished(bool),
    RetryStartup,
    ProfileRetried(Option<NativeProfile>),
    ContinueWithDefaults,
    CopyDiagnostics,
    OpenLink(&'static str),
    ExternalLinkOpened(bool),
    AutomationStep,
    AutomationResume,
    AutomationCapture(PathBuf),
    AutomationScreenshotReady(PathBuf, window::Screenshot),
}

#[derive(Debug, Clone)]
enum CameraPollResult {
    Event(CameraWorkerEvent),
    Timeout,
    Disconnected,
}

pub fn run(cli: &Cli) -> Result<(), RunError> {
    let automation_session = match (cli.automation_script(), cli.automation_output()) {
        (Some(script), Some(output)) => Some(AutomationSession::load(script, output)?),
        (None, None) => None,
        _ => return Err(AutomationError::Schema.into()),
    };
    let profile_directory = cli.profile_directory().map(PathBuf::from);
    let (profile, startup_error) = load_profile(cli.profile_directory()).map_or_else(
        |_| {
            (
                NativeProfile {
                    settings: AppSettings::default(),
                    settings_path: None,
                    writable: false,
                    origin: "defaults-after-error",
                },
                true,
            )
        },
        |profile| (profile, false),
    );
    let initial_window = profile.settings.window;
    let automation = cli.automation();
    let frame_source = cli.frame_source();
    let screenshot_path = cli.screenshot().map(PathBuf::from);
    let main_window_settings = window::Settings {
        size: Size::new(
            f32::from(initial_window.width),
            f32::from(initial_window.height),
        ),
        position: window::Position::Specific(Point::new(
            initial_window.x.to_f32().unwrap_or(48.0),
            initial_window.y.to_f32().unwrap_or(48.0),
        )),
        min_size: Some(Size::new(
            f32::from(MINIMUM_WINDOW_SIZE),
            f32::from(MINIMUM_WINDOW_SIZE),
        )),
        max_size: Some(Size::new(
            f32::from(MAXIMUM_OVERLAY_SIZE),
            f32::from(MAXIMUM_OVERLAY_SIZE),
        )),
        resizable: false,
        decorations: false,
        transparent: true,
        level: window::Level::AlwaysOnTop,
        exit_on_close_request: false,
        ..window::Settings::default()
    };

    iced::daemon(
        move || {
            let (mut state, boot_task) = boot(
                frame_source,
                automation,
                screenshot_path.clone(),
                profile.clone(),
                profile_directory.clone(),
                startup_error,
                automation_session.clone(),
            );
            let (id, open_task) = window::open(main_window_settings.clone());
            state.window_id = Some(id);
            (
                state,
                Task::batch([boot_task, open_task.map(|_| Message::WindowOpened)]),
            )
        },
        update,
        window_view,
    )
    .title(|state: &Camlet, id| {
        if state.settings_window_id == Some(id) {
            format!("{APP_NAME} Settings")
        } else {
            APP_NAME.to_owned()
        }
    })
    .theme(Theme::Dark)
    .style(|_, selected_theme| theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: selected_theme.palette().text,
    })
    .subscription(subscription)
    .antialiasing(true)
    .run()
    .map_err(Into::into)
}

fn boot(
    frame_source_kind: FrameSourceKind,
    automation: AutomationMode,
    screenshot_path: Option<PathBuf>,
    profile: NativeProfile,
    profile_directory: Option<PathBuf>,
    startup_error: bool,
    automation_session: Option<AutomationSession>,
) -> (Camlet, Task<Message>) {
    let product = AppState::new(profile.settings);
    let worker = match frame_source_kind {
        FrameSourceKind::Real => CameraWorker::spawn(|| Box::new(NokhwaFrameSource::default())),
        FrameSourceKind::Synthetic => {
            CameraWorker::spawn(|| Box::new(SyntheticFrameSource::default()))
        }
    };
    let (camera_worker, camera_events, preview_error) = worker.map_or_else(
        |error| {
            (
                None,
                None,
                Some(format!("Camera worker could not start: {error}")),
            )
        },
        |worker| {
            let events = worker.events();
            (Some(worker), Some(events), None)
        },
    );
    let preview_cursor = Point::new(
        f32::from(product.settings.window.width) / 2.0,
        f32::from(product.settings.window.height) / 2.0,
    );
    let mut state = Camlet {
        product,
        automation,
        frame_source_kind,
        camera_worker,
        camera_events,
        source_frame: None,
        placeholder_frame: camera_placeholder_frame(),
        frame_revision: 0,
        capture_first_frame_at: None,
        capture_frame_count: 0,
        preview_error,
        panel: if startup_error {
            Panel::StartupError
        } else {
            Panel::Preview
        },
        screenshot_path,
        screenshot_state: ScreenshotState::Waiting,
        window_id: None,
        settings_window_id: None,
        submenu_window_id: None,
        active_submenu: None,
        preview_cursor,
        menu_origin: Point::ORIGIN,
        scale_factor: 1.0,
        monitor_maximum: MAXIMUM_OVERLAY_SIZE,
        monitor_size: None,
        profile_directory,
        settings_path: profile.settings_path,
        settings_writable: profile.writable,
        profile_origin: profile.origin,
        persist_generation: 0,
        persist_scheduled: false,
        pending_window_position: None,
        pending_window_size: None,
        lifecycle: Lifecycle::Running,
        diagnostics_state: DiagnosticsState::Ready,
        automation_session,
        system_locale: sys_locale::get_locale(),
    };
    rerender(&mut state);
    (state, Task::none())
}

fn update(state: &mut Camlet, message: Message) -> Task<Message> {
    match message {
        Message::WindowEvent(id, event) => handle_window_event(state, id, &event),
        Message::MonitorSize(size) => {
            if let Some(size) = size {
                state.monitor_size = Some(size);
                let maximum = size
                    .width
                    .min(size.height)
                    .floor()
                    .to_u16()
                    .unwrap_or(MAXIMUM_OVERLAY_SIZE);
                state.monitor_maximum = maximum.clamp(MINIMUM_WINDOW_SIZE, MAXIMUM_OVERLAY_SIZE);
            }
            Task::none()
        }
        Message::KeyPressed {
            window_id,
            key,
            physical,
            modifiers,
        } => handle_key_pressed(state, window_id, &key, physical, modifiers),
        Message::PreviewPressed => {
            if state.settings_window_id.is_some() || state.submenu_window_id.is_some() {
                close_settings_window(state)
            } else {
                state.window_id.map_or_else(Task::none, |id| {
                    if state.product.resize_mode {
                        window::drag_resize(id, window::Direction::SouthEast)
                    } else {
                        window::drag(id)
                    }
                })
            }
        }
        Message::PreviewCursorMoved(position) => {
            state.preview_cursor = position;
            Task::none()
        }
        Message::OpenMenu => open_settings_window(state),
        Message::OpenSubmenu(page) => open_submenu_window(state, page),
        Message::MenuProduct(action) => {
            let action_task = apply_product_action(state, action);
            let close_task = close_settings_window(state);
            Task::batch([action_task, close_task])
        }
        Message::ClosePanel => close_settings_window(state),
        Message::WindowOpened => Task::none(),
        Message::Product(action) => apply_product_action(state, action),
        Message::CameraPoll(result) => handle_camera_poll(state, result),
        Message::PersistReady(generation) => {
            if generation != state.persist_generation {
                return persistence_delay(state.persist_generation);
            }
            state.persist_scheduled = false;
            persist_now(state);
            Task::none()
        }
        Message::CaptureScreenshot => state.window_id.map_or_else(Task::none, |id| {
            window::screenshot(id).map(Message::ScreenshotReady)
        }),
        Message::ScreenshotReady(screenshot) => finish_screenshot(state, &screenshot),
        Message::CameraShutdownFinished(joined) => {
            if !joined {
                eprintln!("camlet: camera worker did not shut down cleanly");
            }
            iced::exit()
        }
        Message::RetryStartup => retry_profile(state),
        Message::ProfileRetried(profile) => finish_profile_retry(state, profile),
        Message::ContinueWithDefaults => {
            state.panel = Panel::Preview;
            send_camera_command(state, CameraWorkerCommand::Enumerate)
        }
        Message::CopyDiagnostics => {
            state.diagnostics_state = DiagnosticsState::Copied;
            clipboard::write(diagnostics_json(state))
        }
        Message::OpenLink(url) => Task::perform(
            async move { webbrowser::open(url).is_ok() },
            Message::ExternalLinkOpened,
        ),
        Message::ExternalLinkOpened(opened) => {
            if !opened {
                eprintln!("camlet: could not open the requested external link");
            }
            Task::none()
        }
        Message::AutomationStep => run_automation_step(state),
        Message::AutomationResume => resume_automation(state),
        Message::AutomationCapture(path) => capture_automation(state, path),
        Message::AutomationScreenshotReady(path, screenshot) => {
            finish_automation_screenshot(state, &path, &screenshot)
        }
    }
}

fn handle_key_pressed(
    state: &mut Camlet,
    window_id: window::Id,
    key: &Key,
    physical: Physical,
    modifiers: Modifiers,
) -> Task<Message> {
    if *key == Key::Named(Named::Escape) {
        if state.settings_window_id.is_some() || state.submenu_window_id.is_some() {
            return close_settings_window(state);
        }
        if state.window_id == Some(window_id) && state.product.resize_mode {
            return apply_product_action(state, Action::SetResizeMode(false));
        }
    }
    if state.window_id != Some(window_id) {
        return Task::none();
    }
    keyboard_action(key, physical, modifiers, state.monitor_maximum)
        .map_or_else(Task::none, |action| apply_product_action(state, action))
}

fn resume_automation(state: &mut Camlet) -> Task<Message> {
    let resume = apply_product_action(state, Action::RetryCamera);
    Task::batch([resume, delayed_automation_step(Duration::from_millis(100))])
}

fn capture_automation(state: &mut Camlet, path: PathBuf) -> Task<Message> {
    state.window_id.map_or_else(
        || fail_automation(state),
        |id| {
            window::screenshot(id)
                .map(move |screenshot| Message::AutomationScreenshotReady(path.clone(), screenshot))
        },
    )
}

fn open_settings_window(state: &mut Camlet) -> Task<Message> {
    state.panel = Panel::Menu;
    state.diagnostics_state = DiagnosticsState::Ready;
    state.active_submenu = None;
    let position = menu_window_position(state, MENU_WINDOW_WIDTH, MENU_WINDOW_HEIGHT);
    state.menu_origin = position;
    let settings =
        popup_window_settings(Size::new(MENU_WINDOW_WIDTH, MENU_WINDOW_HEIGHT), position);
    let mut tasks = Vec::new();
    if let Some(id) = state.submenu_window_id.take() {
        tasks.push(window::close(id));
    }
    if let Some(id) = state.settings_window_id.take() {
        tasks.push(window::close(id));
    }
    let (id, open_task) = window::open(settings);
    state.settings_window_id = Some(id);
    tasks.push(open_task.map(|_| Message::WindowOpened));
    Task::batch(tasks)
}

fn open_submenu_window(state: &mut Camlet, page: MenuPage) -> Task<Message> {
    if state.active_submenu == Some(page)
        && let Some(id) = state.submenu_window_id
    {
        return window::gain_focus(id);
    }

    let size = Size::new(SUBMENU_WINDOW_WIDTH, submenu_height(state, page));
    let position = submenu_window_position(state, size);
    let settings = popup_window_settings(size, position);
    let old_close = state.submenu_window_id.take().map(window::close);
    let (id, open_task) = window::open(settings);
    state.submenu_window_id = Some(id);
    state.active_submenu = Some(page);
    Task::batch([
        old_close.unwrap_or_else(Task::none),
        open_task.map(|_| Message::WindowOpened),
    ])
}

fn popup_window_settings(size: Size, position: Point) -> window::Settings {
    window::Settings {
        size,
        position: window::Position::Specific(position),
        min_size: None,
        max_size: None,
        resizable: false,
        decorations: false,
        transparent: true,
        level: window::Level::AlwaysOnTop,
        exit_on_close_request: false,
        platform_specific: popup_platform_settings(),
        ..window::Settings::default()
    }
}

#[cfg(target_os = "linux")]
fn popup_platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific {
        application_id: "camlet-menu".to_owned(),
        override_redirect: true,
    }
}

#[cfg(target_os = "windows")]
fn popup_platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific {
        skip_taskbar: true,
        ..window::settings::PlatformSpecific::default()
    }
}

#[cfg(target_os = "macos")]
fn popup_platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific {
        title_hidden: true,
        titlebar_transparent: true,
        fullsize_content_view: true,
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn popup_platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific::default()
}

fn close_settings_window(state: &mut Camlet) -> Task<Message> {
    state.panel = Panel::Preview;
    state.active_submenu = None;
    Task::batch([
        state
            .submenu_window_id
            .take()
            .map_or_else(Task::none, window::close),
        state
            .settings_window_id
            .take()
            .map_or_else(Task::none, window::close),
    ])
}

fn menu_window_position(state: &Camlet, width: f32, height: f32) -> Point {
    let overlay = state.product.settings.window;
    let mut x = overlay.x.to_f32().unwrap_or(0.0) + state.preview_cursor.x;
    let mut y = overlay.y.to_f32().unwrap_or(0.0) + state.preview_cursor.y;

    if let Some(monitor) = state.monitor_size {
        if overlay.x >= 0 {
            x = x.clamp(0.0, (monitor.width - width).max(0.0));
        }
        if overlay.y >= 0 {
            y = y.clamp(0.0, (monitor.height - height).max(0.0));
        }
    }

    Point::new(x, y)
}

fn submenu_window_position(state: &Camlet, size: Size) -> Point {
    let mut x = state.menu_origin.x + MENU_WINDOW_WIDTH + MENU_WINDOW_GAP;
    let mut y = state.menu_origin.y;
    if let Some(monitor) = state.monitor_size {
        if x >= 0.0 && x + size.width > monitor.width {
            x = state.menu_origin.x - size.width - MENU_WINDOW_GAP;
        }
        if state.product.settings.window.y >= 0 {
            y = y.clamp(0.0, (monitor.height - size.height).max(0.0));
        }
    }
    Point::new(x, y)
}

fn submenu_height(state: &Camlet, page: MenuPage) -> f32 {
    let rows = match page {
        MenuPage::Theme | MenuPage::Shape => 6,
        MenuPage::Language => 3,
        MenuPage::Camera => state.product.cameras.len().max(1),
        MenuPage::Advanced => 11,
    };
    f32::from(u16::try_from(rows).unwrap_or(u16::MAX)).mul_add(34.0, 12.0)
}

fn handle_window_event(state: &mut Camlet, id: window::Id, event: &window::Event) -> Task<Message> {
    if state.submenu_window_id == Some(id) {
        return match event {
            window::Event::CloseRequested | window::Event::Closed => {
                state.submenu_window_id = None;
                state.active_submenu = None;
                if matches!(event, window::Event::CloseRequested) {
                    window::close(id)
                } else {
                    Task::none()
                }
            }
            _ => Task::none(),
        };
    }
    if state.settings_window_id == Some(id) {
        return match event {
            window::Event::CloseRequested | window::Event::Closed => {
                state.settings_window_id = None;
                let submenu_close = state
                    .submenu_window_id
                    .take()
                    .map_or_else(Task::none, window::close);
                state.active_submenu = None;
                state.panel = Panel::Preview;
                if matches!(event, window::Event::CloseRequested) {
                    Task::batch([window::close(id), submenu_close])
                } else {
                    submenu_close
                }
            }
            _ => Task::none(),
        };
    }
    if state.window_id != Some(id) {
        return Task::none();
    }

    match event {
        window::Event::Opened { position, size } => {
            if let Some(position) = *position {
                record_position(state, position);
            }
            record_size(state, *size);
            let monitor_task = window::monitor_size(id).map(Message::MonitorSize);
            let enumerate_task = if state.panel == Panel::StartupError {
                if state.automation == AutomationMode::ExitAfterLaunch {
                    Task::done(Message::ContinueWithDefaults)
                } else {
                    Task::none()
                }
            } else {
                send_camera_command(state, CameraWorkerCommand::Enumerate)
            };
            Task::batch([monitor_task, enumerate_task])
        }
        window::Event::Moved(position) => handle_window_moved(state, *position),
        window::Event::Resized(size) => handle_window_resized(state, id, *size),
        window::Event::Rescaled(scale_factor) => {
            state.scale_factor = scale_factor.max(0.5);
            rerender(state);
            Task::none()
        }
        window::Event::Closed => iced::exit(),
        window::Event::CloseRequested => clean_shutdown(state, id),
        _ => Task::none(),
    }
}

fn subscription(state: &Camlet) -> Subscription<Message> {
    let window_events = event::listen_with(|event, _status, id| match event {
        event::Event::Window(window_event) => Some(Message::WindowEvent(id, window_event)),
        event::Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) => Some(Message::KeyPressed {
            window_id: id,
            key,
            physical: physical_key,
            modifiers,
        }),
        _ => None,
    });
    let camera_events = state
        .camera_events
        .as_ref()
        .map_or_else(Subscription::none, |events| {
            Subscription::run_with(CameraEventFeed(events.clone()), camera_event_stream)
                .map(Message::CameraPoll)
        });
    Subscription::batch([window_events, camera_events])
}

#[derive(Debug, Clone)]
struct CameraEventFeed(CameraWorkerEvents);

// There is exactly one camera worker for an application lifetime. Keeping its recipe identity
// stable lets Iced retain one bounded stream instead of spawning a frame-sized task per poll.
impl PartialEq for CameraEventFeed {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for CameraEventFeed {}

impl std::hash::Hash for CameraEventFeed {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        0_u8.hash(state);
    }
}

fn camera_event_stream(
    feed: &CameraEventFeed,
) -> impl iced::futures::Stream<Item = CameraPollResult> + use<> {
    let events = feed.0.clone();
    stream::channel(1, async move |mut output| {
        loop {
            let result = match events.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => CameraPollResult::Event(event),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => CameraPollResult::Timeout,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    CameraPollResult::Disconnected
                }
            };
            let disconnected = matches!(result, CameraPollResult::Disconnected);
            if output.send(result).await.is_err() || disconnected {
                break;
            }
        }
    })
}

fn retry_profile(state: &Camlet) -> Task<Message> {
    let directory = state.profile_directory.clone();
    Task::perform(
        async move { load_profile(directory.as_deref()).ok() },
        Message::ProfileRetried,
    )
}

fn finish_profile_retry(state: &mut Camlet, profile: Option<NativeProfile>) -> Task<Message> {
    let Some(profile) = profile else {
        return Task::none();
    };
    state.product = AppState::new(profile.settings);
    state.settings_path = profile.settings_path;
    state.settings_writable = profile.writable;
    state.profile_origin = profile.origin;
    state.panel = Panel::Preview;
    state.preview_error = None;
    rerender(state);
    send_camera_command(state, CameraWorkerCommand::Enumerate)
}

fn diagnostics_json(state: &Camlet) -> String {
    let appearance = &state.product.settings.appearance;
    let measured_fps = measured_capture_fps(state);
    let snapshot = serde_json::json!({
        "schemaVersion": 1,
        "application": {
            "name": APP_NAME,
            "version": env!("CARGO_PKG_VERSION"),
            "runtime": "Rust + Iced 0.14",
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "displayProtocol": display_protocol(),
            "releaseChannel": release_channel(),
        },
        "profile": {
            "origin": state.profile_origin,
            "writable": state.settings_writable,
        },
        "camera": {
            "source": state.frame_source_kind.as_str(),
            "status": camera_status_code(state.product.camera_status),
            "requestedFps": state.product.settings.camera_fps,
            "measuredFps": measured_fps,
            "deviceCount": state.product.cameras.len(),
            "activeDevicePresent": state.product.active_camera_id.is_some(),
            "droppedFrames": state.camera_worker.as_ref().map_or(0, CameraWorker::dropped_frames),
        },
        "appearance": {
            "shape": appearance.shape,
            "size": appearance.size,
            "fit": appearance.fit,
            "ringThickness": appearance.ring_thickness,
            "cornerRoundness": appearance.corner_roundness,
        },
    });
    serde_json::to_string_pretty(&snapshot)
        .unwrap_or_else(|_| "{\"schemaVersion\":1,\"status\":\"unavailable\"}".to_owned())
}

fn measured_capture_fps(state: &Camlet) -> Option<f64> {
    let first_frame_at = state.capture_first_frame_at?;
    let intervals = state.capture_frame_count.checked_sub(1)?;
    if intervals == 0 {
        return None;
    }
    let intervals = intervals.to_f64()?;
    let elapsed = first_frame_at.elapsed().as_secs_f64();
    (elapsed > 0.0).then(|| ((intervals / elapsed) * 10.0).round() / 10.0)
}

fn release_channel() -> &'static str {
    if env!("CARGO_PKG_VERSION").contains('-') {
        "prerelease"
    } else {
        "stable"
    }
}

fn display_protocol() -> &'static str {
    let requested_backend = std::env::var("WINIT_UNIX_BACKEND").ok();
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if std::env::var_os("CAMLET_X11_RELAUNCHED").is_some()
        || (std::env::var_os("WAYLAND_DISPLAY").is_none()
            && std::env::var_os("WAYLAND_SOCKET").is_none()
            && std::env::var_os("DISPLAY").is_some())
    {
        "x11"
    } else if requested_backend.as_deref() == Some("wayland")
        || std::env::var_os("WAYLAND_DISPLAY").is_some()
    {
        "wayland"
    } else if std::env::var_os("DISPLAY").is_some() {
        "x11"
    } else {
        "unknown"
    }
}

const fn camera_status_code(status: CameraStatus) -> &'static str {
    match status {
        CameraStatus::Loading => "loading",
        CameraStatus::Preview => "preview",
        CameraStatus::PermissionDenied => "permission-denied",
        CameraStatus::CameraInUse => "camera-in-use",
        CameraStatus::NoCamera => "no-camera",
        CameraStatus::SelectedDeviceUnavailable => "selected-device-unavailable",
        CameraStatus::Error => "error",
    }
}

fn handle_camera_poll(state: &mut Camlet, result: CameraPollResult) -> Task<Message> {
    if state.lifecycle == Lifecycle::ShuttingDown {
        return Task::none();
    }

    match result {
        CameraPollResult::Event(CameraWorkerEvent::Devices(Ok(devices))) => {
            let cameras = devices.into_iter().map(camera_option).collect();
            let status_task = apply_product_action(state, Action::DevicesChanged(cameras));
            if matches!(
                state.product.camera_status,
                CameraStatus::NoCamera | CameraStatus::SelectedDeviceUnavailable
            ) {
                state.source_frame = None;
                state.preview_error = Some(
                    match state.product.camera_status {
                        CameraStatus::NoCamera => "No camera was found",
                        CameraStatus::SelectedDeviceUnavailable => {
                            "The selected camera is unavailable"
                        }
                        _ => unreachable!(),
                    }
                    .to_owned(),
                );
                Task::batch([status_task, finish_failed_automation(state)])
            } else {
                status_task
            }
        }
        CameraPollResult::Event(CameraWorkerEvent::Devices(Err(_)))
            if state.product.camera_status == CameraStatus::Preview =>
        {
            Task::none()
        }
        CameraPollResult::Event(
            CameraWorkerEvent::Devices(Err(error))
            | CameraWorkerEvent::Started(Err(error))
            | CameraWorkerEvent::Frame(Err(error)),
        ) => {
            state.source_frame = None;
            state.preview_error = Some(camera_error_label(error).to_owned());
            let status_task =
                apply_product_action(state, Action::CameraFailed(camera_status(error)));
            Task::batch([status_task, finish_failed_automation(state)])
        }
        CameraPollResult::Event(CameraWorkerEvent::Frame(Ok(frame))) => {
            state.source_frame = Some(Arc::new(frame));
            state.frame_revision = state.frame_revision.saturating_add(1);
            if state.capture_first_frame_at.is_none() {
                state.capture_first_frame_at = Some(Instant::now());
            }
            state.capture_frame_count = state.capture_frame_count.saturating_add(1);
            let ready_task = if state.product.camera_status == CameraStatus::Preview {
                rerender(state);
                Task::none()
            } else {
                apply_product_action(state, Action::CameraReady)
            };
            Task::batch([ready_task, finish_automation_if_needed(state)])
        }
        CameraPollResult::Event(
            CameraWorkerEvent::Started(Ok(())) | CameraWorkerEvent::Stopped,
        )
        | CameraPollResult::Timeout => Task::none(),
        CameraPollResult::Disconnected => {
            state.source_frame = None;
            state.preview_error = Some("Camera worker stopped unexpectedly".to_owned());
            let status_task =
                apply_product_action(state, Action::CameraFailed(CameraStatus::Error));
            Task::batch([status_task, finish_failed_automation(state)])
        }
    }
}

fn camera_option(device: CameraDevice) -> CameraOption {
    CameraOption {
        id: device.id,
        label: device.label,
    }
}

const fn camera_status(error: CameraError) -> CameraStatus {
    match error {
        CameraError::PermissionDenied => CameraStatus::PermissionDenied,
        CameraError::DeviceNotFound => CameraStatus::SelectedDeviceUnavailable,
        CameraError::Busy => CameraStatus::CameraInUse,
        CameraError::Backend => CameraStatus::Error,
    }
}

const fn camera_error_label(error: CameraError) -> &'static str {
    match error {
        CameraError::PermissionDenied => "Camera permission was denied",
        CameraError::DeviceNotFound => "Camera device was not found",
        CameraError::Busy => "Camera is already in use",
        CameraError::Backend => "Camera backend failed",
    }
}

fn finish_automation_if_needed(state: &mut Camlet) -> Task<Message> {
    if state.automation == AutomationMode::Scripted {
        let started = state
            .automation_session
            .as_mut()
            .is_some_and(|session| session.start().unwrap_or(false));
        if started {
            return Task::done(Message::AutomationStep);
        }
        if state
            .automation_session
            .as_ref()
            .is_none_or(|session| !session.started())
        {
            return fail_automation(state);
        }
        return Task::none();
    }
    let Some(id) = state.window_id else {
        return Task::none();
    };
    if state.screenshot_path.is_some()
        && state.source_frame.is_some()
        && state.screenshot_state == ScreenshotState::Waiting
    {
        state.screenshot_state = ScreenshotState::Requested;
        Task::perform(
            async {
                std::thread::sleep(Duration::from_millis(100));
            },
            |()| Message::CaptureScreenshot,
        )
    } else if state.automation == AutomationMode::ExitAfterLaunch {
        clean_shutdown(state, id)
    } else {
        Task::none()
    }
}

fn finish_failed_automation(state: &mut Camlet) -> Task<Message> {
    if state.automation == AutomationMode::Scripted {
        return fail_automation(state);
    }
    if state.screenshot_path.is_some() && state.screenshot_state == ScreenshotState::Waiting {
        state.screenshot_state = ScreenshotState::Requested;
        return Task::perform(
            async {
                std::thread::sleep(Duration::from_millis(100));
            },
            |()| Message::CaptureScreenshot,
        );
    }
    if state.automation == AutomationMode::ExitAfterLaunch {
        state
            .window_id
            .map_or_else(Task::none, |id| clean_shutdown(state, id))
    } else {
        Task::none()
    }
}

fn send_camera_command(state: &Camlet, command: CameraWorkerCommand) -> Task<Message> {
    if state
        .camera_worker
        .as_ref()
        .is_some_and(|worker| worker.send(command))
    {
        Task::none()
    } else {
        Task::done(Message::CameraPoll(CameraPollResult::Disconnected))
    }
}

fn run_automation_step(state: &mut Camlet) -> Task<Message> {
    let Some(action) = state
        .automation_session
        .as_mut()
        .and_then(AutomationSession::next)
    else {
        return fail_automation(state);
    };
    match action {
        AutomationAction::WaitForPreview => {
            if state.product.camera_status == CameraStatus::Preview {
                Task::done(Message::AutomationStep)
            } else {
                delayed_automation_step(Duration::from_millis(50))
            }
        }
        AutomationAction::SetTheme(theme) => {
            automation_product_action(state, Action::SetTheme(theme))
        }
        AutomationAction::SetShape(shape) => {
            automation_product_action(state, Action::SetShape(shape))
        }
        AutomationAction::SetFit(fit) => automation_product_action(state, Action::SetFit(fit)),
        AutomationAction::SetRingThickness(value) => {
            automation_product_action(state, Action::SetRingThickness(value))
        }
        AutomationAction::SetCornerRoundness(value) => {
            automation_product_action(state, Action::SetCornerRoundness(value))
        }
        AutomationAction::ResizeStep { grow } => {
            let resize = apply_product_action(
                state,
                Action::ResizeByStep {
                    grow,
                    maximum: state.monitor_maximum,
                },
            );
            Task::batch([resize, delayed_automation_step(Duration::from_millis(50))])
        }
        AutomationAction::RestartCamera => {
            let restart = apply_product_action(state, Action::RetryCamera);
            Task::batch([restart, delayed_automation_step(Duration::from_millis(100))])
        }
        AutomationAction::SuspendResume(duration) => {
            let stop = execute_effect(state, &Effect::StopCamera);
            Task::batch([stop, automation_delay(duration, Message::AutomationResume)])
        }
        AutomationAction::Delay(duration) => delayed_automation_step(duration),
        AutomationAction::OpenMenu => Task::batch([
            open_settings_window(state),
            delayed_automation_step(Duration::from_millis(250)),
        ]),
        AutomationAction::OpenAdvancedMenu => Task::batch([
            open_submenu_window(state, MenuPage::Advanced),
            delayed_automation_step(Duration::from_millis(250)),
        ]),
        AutomationAction::Screenshot(filename) => {
            let Some(session) = state.automation_session.as_ref() else {
                return fail_automation(state);
            };
            let path = session.output_path(&filename);
            automation_delay(Duration::from_millis(100), Message::AutomationCapture(path))
        }
        AutomationAction::Diagnostics(filename) => {
            let diagnostics = diagnostics_json(state);
            let written = state
                .automation_session
                .as_ref()
                .is_some_and(|session| session.write(&filename, diagnostics.as_bytes()).is_ok());
            if written {
                Task::done(Message::AutomationStep)
            } else {
                fail_automation(state)
            }
        }
        AutomationAction::Quit => {
            let completed = state
                .automation_session
                .as_ref()
                .is_some_and(|session| session.complete().is_ok());
            if !completed {
                return fail_automation(state);
            }
            if let Some(id) = state.window_id {
                clean_shutdown(state, id)
            } else {
                fail_automation(state)
            }
        }
    }
}

fn automation_product_action(state: &mut Camlet, action: Action) -> Task<Message> {
    Task::batch([
        apply_product_action(state, action),
        Task::done(Message::AutomationStep),
    ])
}

fn delayed_automation_step(duration: Duration) -> Task<Message> {
    automation_delay(duration, Message::AutomationStep)
}

fn automation_delay(duration: Duration, message: Message) -> Task<Message> {
    Task::perform(
        async move {
            std::thread::sleep(duration);
            message
        },
        std::convert::identity,
    )
}

fn finish_automation_screenshot(
    state: &mut Camlet,
    path: &std::path::Path,
    screenshot: &window::Screenshot,
) -> Task<Message> {
    let result = validate_live_screenshot(state, screenshot)
        .and_then(|()| write_white_background_ppm(path, screenshot));
    if result.is_ok() {
        Task::done(Message::AutomationStep)
    } else {
        fail_automation(state)
    }
}

fn fail_automation(state: &mut Camlet) -> Task<Message> {
    if let Some(session) = state.automation_session.as_ref() {
        session.fail();
    }
    if let Some(error) = state.preview_error.as_deref() {
        eprintln!("camlet: automation scenario failed: {error}");
    } else {
        eprintln!("camlet: automation scenario failed");
    }
    state
        .window_id
        .map_or_else(Task::none, |id| clean_shutdown(state, id))
}

fn apply_product_action(state: &mut Camlet, action: Action) -> Task<Message> {
    let resize_mode = match &action {
        Action::SetResizeMode(enabled) => Some(*enabled),
        _ => None,
    };
    let effects = state.product.update(action);
    rerender(state);

    let mut tasks = effects
        .iter()
        .map(|effect| execute_effect(state, effect))
        .collect::<Vec<_>>();
    if let Some(enabled) = resize_mode
        && let Some(id) = state.window_id
    {
        tasks.push(window::set_resizable(id, enabled));
        if enabled {
            tasks.push(close_settings_window(state));
        }
    }
    Task::batch(tasks)
}

fn execute_effect(state: &mut Camlet, effect: &Effect) -> Task<Message> {
    match effect {
        Effect::PersistSettings => request_persistence(state),
        Effect::FlushSettings => {
            persist_now(state);
            Task::none()
        }
        Effect::MoveWindow(window_state) => state.window_id.map_or_else(Task::none, |id| {
            state.pending_window_position = Some((window_state.x, window_state.y));
            window::move_to(
                id,
                Point::new(
                    window_state.x.to_f32().unwrap_or(0.0),
                    window_state.y.to_f32().unwrap_or(0.0),
                ),
            )
        }),
        Effect::ResizeWindow(window_state) => state.window_id.map_or_else(Task::none, |id| {
            state.pending_window_position = Some((window_state.x, window_state.y));
            state.pending_window_size = Some(window_state.width);
            Task::batch([
                window::move_to(
                    id,
                    Point::new(
                        window_state.x.to_f32().unwrap_or(0.0),
                        window_state.y.to_f32().unwrap_or(0.0),
                    ),
                ),
                window::resize(
                    id,
                    Size::new(
                        f32::from(window_state.width),
                        f32::from(window_state.height),
                    ),
                ),
            ])
        }),
        Effect::EnumerateCameras => send_camera_command(state, CameraWorkerCommand::Enumerate),
        Effect::StartCamera(device_id) => {
            state.source_frame = None;
            state.preview_error = None;
            state.capture_first_frame_at = None;
            state.capture_frame_count = 0;
            send_camera_command(
                state,
                CameraWorkerCommand::Start {
                    device_id: Some(device_id.clone()),
                    request: CaptureRequest {
                        width: 640,
                        height: 480,
                        frame_interval: Duration::from_nanos(
                            1_000_000_000 / u64::from(state.product.settings.camera_fps),
                        ),
                    },
                },
            )
        }
        Effect::StopCamera => {
            state.source_frame = None;
            send_camera_command(state, CameraWorkerCommand::Stop)
        }
        Effect::Quit => state
            .window_id
            .map_or_else(Task::none, |id| clean_shutdown(state, id)),
        Effect::OpenAbout => {
            state.panel = Panel::About;
            state.diagnostics_state = DiagnosticsState::Ready;
            state.active_submenu = None;
            Task::batch([
                state
                    .submenu_window_id
                    .take()
                    .map_or_else(Task::none, window::close),
                state.settings_window_id.map_or_else(Task::none, |id| {
                    window::resize(id, Size::new(ABOUT_WINDOW_WIDTH, ABOUT_WINDOW_HEIGHT))
                }),
            ])
        }
    }
}

fn request_persistence(state: &mut Camlet) -> Task<Message> {
    state.persist_generation = state.persist_generation.saturating_add(1);
    if state.persist_scheduled {
        Task::none()
    } else {
        state.persist_scheduled = true;
        persistence_delay(state.persist_generation)
    }
}

fn persistence_delay(generation: u64) -> Task<Message> {
    Task::perform(
        async move {
            std::thread::sleep(PERSISTENCE_DEBOUNCE);
            generation
        },
        Message::PersistReady,
    )
}

fn persist_now(state: &Camlet) {
    if !state.settings_writable {
        return;
    }
    if let Some(path) = state.settings_path.as_deref()
        && write_settings(path, &state.product.settings).is_err()
    {
        eprintln!("camlet: settings write failed");
    }
}

fn clean_shutdown(state: &mut Camlet, _id: window::Id) -> Task<Message> {
    if state.lifecycle == Lifecycle::ShuttingDown {
        return Task::none();
    }
    state.lifecycle = Lifecycle::ShuttingDown;
    persist_now(state);
    state.camera_events = None;
    state
        .camera_worker
        .take()
        .map_or_else(iced::exit, |worker| {
            Task::perform(
                async move { worker.shutdown() },
                Message::CameraShutdownFinished,
            )
        })
}

fn finish_screenshot(state: &mut Camlet, screenshot: &window::Screenshot) -> Task<Message> {
    if let Some(path) = state.screenshot_path.as_deref() {
        let result = validate_live_screenshot(state, screenshot)
            .and_then(|()| write_white_background_ppm(path, screenshot));
        if result.is_err() {
            eprintln!("camlet: screenshot failed");
        }
    }
    state
        .window_id
        .map_or_else(Task::none, |id| clean_shutdown(state, id))
}

fn validate_live_screenshot(
    state: &Camlet,
    screenshot: &window::Screenshot,
) -> Result<(), ScreenshotError> {
    validate_transparent_overlay(screenshot)?;
    let source = state
        .source_frame
        .as_ref()
        .unwrap_or(&state.placeholder_frame);
    if screenshot.size.width != screenshot.size.height {
        return Err(ScreenshotError::ReferenceMismatch);
    }
    let reference = render_overlay(
        source,
        &state.product.settings.appearance,
        screenshot.size.width,
    )
    .map_err(|_| ScreenshotError::ReferenceMismatch)?;
    validate_reference_alpha(
        screenshot,
        reference.width,
        reference.height,
        &reference.rgba,
        2,
    )
}

fn record_position(state: &mut Camlet, position: Point) {
    state.product.settings.window.x = position.x.round().to_i32().unwrap_or(0);
    state.product.settings.window.y = position.y.round().to_i32().unwrap_or(0);
}

fn record_size(state: &mut Camlet, size: Size) {
    let logical = size
        .width
        .min(size.height)
        .round()
        .to_u16()
        .unwrap_or(MINIMUM_WINDOW_SIZE)
        .clamp(MINIMUM_WINDOW_SIZE, state.monitor_maximum);
    state.product.settings.window.width = logical;
    state.product.settings.window.height = logical;
    state.product.settings.appearance.size = logical.min(MAXIMUM_OVERLAY_SIZE);
}

fn handle_window_moved(state: &mut Camlet, position: Point) -> Task<Message> {
    let observed = (
        position.x.round().to_i32().unwrap_or(0),
        position.y.round().to_i32().unwrap_or(0),
    );
    if let Some(expected) = state.pending_window_position {
        if observed == expected {
            state.pending_window_position = None;
        }
        return Task::none();
    }

    record_position(state, position);
    request_persistence(state)
}

fn handle_window_resized(state: &mut Camlet, id: window::Id, size: Size) -> Task<Message> {
    let observed = size
        .width
        .min(size.height)
        .round()
        .to_u16()
        .unwrap_or(MINIMUM_WINDOW_SIZE);
    if let Some(expected) = state.pending_window_size {
        if observed == expected {
            state.pending_window_size = None;
        }
        return Task::none();
    }

    record_size(state, size);
    rerender(state);
    let square = state.product.settings.window.width;
    let resize_task = if size.width.round().to_u16() != Some(square)
        || size.height.round().to_u16() != Some(square)
    {
        state.pending_window_size = Some(square);
        window::resize(id, Size::new(f32::from(square), f32::from(square)))
    } else {
        Task::none()
    };
    Task::batch([request_persistence(state), resize_task])
}

fn keyboard_action(
    key: &Key,
    physical: Physical,
    modifiers: Modifiers,
    maximum: u16,
) -> Option<Action> {
    let accelerated = modifiers.shift();
    match key.as_ref() {
        Key::Named(Named::ArrowUp) => Some(Action::NudgeWindow {
            x: 0,
            y: -1,
            accelerated,
        }),
        Key::Named(Named::ArrowDown) => Some(Action::NudgeWindow {
            x: 0,
            y: 1,
            accelerated,
        }),
        Key::Named(Named::ArrowLeft) => Some(Action::NudgeWindow {
            x: -1,
            y: 0,
            accelerated,
        }),
        Key::Named(Named::ArrowRight) => Some(Action::NudgeWindow {
            x: 1,
            y: 0,
            accelerated,
        }),
        Key::Character("-") => Some(Action::ResizeByStep {
            grow: false,
            maximum,
        }),
        Key::Character("=" | "+") => Some(Action::ResizeByStep {
            grow: true,
            maximum,
        }),
        _ => match physical {
            Physical::Code(Code::NumpadSubtract) => Some(Action::ResizeByStep {
                grow: false,
                maximum,
            }),
            Physical::Code(Code::NumpadAdd) => Some(Action::ResizeByStep {
                grow: true,
                maximum,
            }),
            _ => None,
        },
    }
}

fn rerender(state: &mut Camlet) {
    let Some(source_frame) = state.source_frame.as_ref() else {
        return;
    };
    match validate_source_frame(source_frame) {
        Ok(()) => state.preview_error = None,
        Err(error) => {
            state.preview_error = Some(error.to_string());
        }
    }
}

fn preview_view(state: &Camlet) -> Element<'_, Message> {
    let source = state
        .source_frame
        .as_ref()
        .unwrap_or(&state.placeholder_frame);
    let overlay: Element<'_, Message> = shader(OverlayProgram::new(
        source,
        &state.product.settings.appearance,
        state.frame_revision,
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    if state.source_frame.is_some() {
        return overlay;
    }

    let detail = state
        .preview_error
        .as_deref()
        .unwrap_or(match state.frame_source_kind {
            FrameSourceKind::Real => "Initializing camera…",
            FrameSourceKind::Synthetic => "Synthetic preview unavailable",
        });
    let catalog = catalog(
        state.product.settings.language,
        state.system_locale.as_deref(),
    );
    let mut message = column![
        text(camera_status_text(catalog, state.product.camera_status))
            .size(18)
            .color(Color::WHITE),
        text(detail).size(12).color(Color::from_rgb8(207, 218, 232)),
    ]
    .align_x(iced::Alignment::Center)
    .spacing(9)
    .padding(24);
    if state.product.camera_status != CameraStatus::Loading {
        message = message.push(
            button(text(catalog.retry_camera).size(12))
                .height(32)
                .padding([6, 15])
                .style(retry_button_style)
                .on_press(Message::Product(Action::RetryCamera)),
        );
    }
    stack![
        overlay,
        container(message)
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
    ]
    .into()
}

fn retry_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgba8(64, 76, 96, 0.96),
        button::Status::Pressed => Color::from_rgba8(35, 44, 59, 0.98),
        button::Status::Active | button::Status::Disabled => Color::from_rgba8(46, 57, 75, 0.92),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: Border {
            color: Color::from_rgba8(157, 177, 207, 0.28),
            width: 1.0,
            radius: 999.0.into(),
        },
        ..button::Style::default()
    }
}

fn camera_placeholder_frame() -> Arc<VideoFrame> {
    const WIDTH: u32 = 4;
    const HEIGHT: u32 = 3;
    let mut rgba = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for _ in 0..WIDTH * HEIGHT {
        rgba.extend_from_slice(&[8, 13, 22, u8::MAX]);
    }
    Arc::new(VideoFrame {
        width: WIDTH,
        height: HEIGHT,
        sequence: 0,
        rgba,
    })
}

fn window_view(state: &Camlet, id: window::Id) -> Element<'_, Message> {
    if state.submenu_window_id == Some(id) {
        return submenu_view(state);
    }
    if state.settings_window_id == Some(id) {
        return match state.panel {
            Panel::About => about_view(state),
            _ => root_menu_view(state),
        };
    }
    main_view(state)
}

fn main_view(state: &Camlet) -> Element<'_, Message> {
    let preview = mouse_area(
        container(preview_view(state))
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(Message::PreviewPressed)
    .on_right_press(Message::OpenMenu)
    .on_move(Message::PreviewCursorMoved)
    .interaction(if state.product.resize_mode {
        mouse::Interaction::ResizingDiagonallyDown
    } else {
        mouse::Interaction::Grab
    });

    let mut layers = stack![preview].width(Length::Fill).height(Length::Fill);
    match state.panel {
        Panel::StartupError => layers = layers.push(opaque(startup_error_view(state))),
        Panel::Preview if state.product.resize_mode => {
            layers = layers.push(opaque(resize_view(state)));
        }
        Panel::Preview | Panel::Menu | Panel::About => {}
    }
    layers.into()
}

fn root_menu_view(state: &Camlet) -> Element<'_, Message> {
    let catalog = catalog(
        state.product.settings.language,
        state.system_locale.as_deref(),
    );
    let content = column![
        submenu_button(catalog.theme, MenuPage::Theme),
        submenu_button(catalog.shape, MenuPage::Shape),
        submenu_button(catalog.language, MenuPage::Language),
        submenu_button(catalog.camera_device, MenuPage::Camera),
        rule::horizontal(1),
        menu_action_button(catalog.resize, Action::SetResizeMode(true)),
        submenu_button(catalog.advanced, MenuPage::Advanced),
        button(text(catalog.about).size(13))
            .width(Length::Fill)
            .height(32)
            .padding([4, 8])
            .style(button::subtle)
            .on_press(Message::Product(Action::OpenAbout)),
        rule::horizontal(1),
        menu_action_button(catalog.close_app, Action::Quit),
    ]
    .spacing(1)
    .padding(6);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(container::dark)
        .into()
}

fn submenu_button(label: &str, page: MenuPage) -> Element<'static, Message> {
    mouse_area(
        button(row![
            text(label.to_owned()).size(13),
            space::horizontal(),
            text("›").size(18)
        ])
        .width(Length::Fill)
        .height(32)
        .padding([4, 8])
        .style(button::subtle)
        .on_press(Message::OpenSubmenu(page)),
    )
    .on_enter(Message::OpenSubmenu(page))
    .interaction(mouse::Interaction::Pointer)
    .into()
}

fn menu_action_button(label: &str, action: Action) -> Element<'static, Message> {
    button(text(label.to_owned()).size(13))
        .width(Length::Fill)
        .height(32)
        .padding([4, 8])
        .style(button::subtle)
        .on_press(Message::MenuProduct(action))
        .into()
}

fn submenu_view(state: &Camlet) -> Element<'_, Message> {
    let catalog = catalog(
        state.product.settings.language,
        state.system_locale.as_deref(),
    );
    let model = MenuModel::from_state(&state.product);
    let content = match state.active_submenu.unwrap_or(MenuPage::Advanced) {
        MenuPage::Theme => {
            let mut content = column![].spacing(1).padding(6);
            for (choice, label) in model.themes.iter().zip(catalog.themes) {
                content = content.push(menu_choice_button(
                    label,
                    choice.selected,
                    Action::SetTheme(choice.value),
                ));
            }
            content
        }
        MenuPage::Shape => {
            let mut content = column![].spacing(1).padding(6);
            for (choice, label) in model.shapes.iter().zip(catalog.shapes) {
                content = content.push(menu_choice_button(
                    label,
                    choice.selected,
                    Action::SetShape(choice.value),
                ));
            }
            content
        }
        MenuPage::Language => {
            let mut content = column![].spacing(1).padding(6);
            for choice in &model.languages {
                content = content.push(menu_choice_button(
                    language_label(catalog, choice.value),
                    choice.selected,
                    Action::SetLanguage(choice.value),
                ));
            }
            content
        }
        MenuPage::Camera => {
            let mut content = column![].spacing(1).padding(6);
            if model.cameras.is_empty() {
                content = content.push(
                    container(text(catalog.no_devices).size(12))
                        .height(32)
                        .padding([7, 8]),
                );
            } else {
                for choice in &model.cameras {
                    content = content.push(menu_choice_button(
                        &choice.label,
                        choice.selected,
                        Action::SetCamera(choice.id.clone()),
                    ));
                }
            }
            content
        }
        MenuPage::Advanced => advanced_menu_view(state, catalog, &model),
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(container::dark)
        .into()
}

fn advanced_menu_view<'a>(
    state: &'a Camlet,
    catalog: &'a Catalog,
    model: &MenuModel,
) -> iced::widget::Column<'a, Message> {
    let fit = row![
        compact_choice_button(
            catalog.fit_modes[0],
            model.fit_modes[0].selected,
            Action::SetFit(model.fit_modes[0].value),
        ),
        compact_choice_button(
            catalog.fit_modes[1],
            model.fit_modes[1].selected,
            Action::SetFit(model.fit_modes[1].value),
        ),
    ]
    .spacing(3);

    column![
        text(format!(
            "{}: {}",
            catalog.preview_state,
            camera_status_text(catalog, model.camera_status)
        ))
        .size(11),
        text(format!(
            "{}: {}",
            catalog.active_camera,
            active_camera_label(state, catalog.none)
        ))
        .size(11),
        text(format!(
            "{}: {}",
            catalog.display_protocol,
            display_protocol()
        ))
        .size(11),
        rule::horizontal(1),
        text(catalog.camera_fps).size(12),
        compact_option_row(
            CAMERA_FPS_OPTIONS.map(|value| (format!("{value}"), Action::SetCameraFps(value))),
            state.product.settings.camera_fps,
        ),
        text(catalog.fit_mode).size(12),
        fit,
        text(catalog.ring_thickness).size(12),
        compact_option_row(
            RING_THICKNESS_OPTIONS
                .map(|value| (value.to_string(), Action::SetRingThickness(value))),
            state.product.settings.appearance.ring_thickness,
        ),
        text(catalog.corner_roundness).size(12),
        compact_option_row(
            CORNER_ROUNDNESS_OPTIONS
                .map(|value| (value.to_string(), Action::SetCornerRoundness(value))),
            state.product.settings.appearance.corner_roundness,
        ),
        rule::horizontal(1),
        menu_action_button(catalog.retry_camera, Action::RetryCamera),
        menu_action_button(catalog.reset_appearance, Action::ResetAppearance),
    ]
    .spacing(4)
    .padding(8)
}

fn active_camera_label<'a>(state: &'a Camlet, fallback: &'a str) -> &'a str {
    let active = state.product.active_camera_id.as_deref();
    state
        .product
        .cameras
        .iter()
        .find(|camera| Some(camera.id.as_str()) == active)
        .map_or(fallback, |camera| camera.label.as_str())
}

fn about_view(state: &Camlet) -> Element<'_, Message> {
    let catalog = catalog(
        state.product.settings.language,
        state.system_locale.as_deref(),
    );
    let channel = if release_channel() == "stable" {
        catalog.stable_channel
    } else {
        catalog.prerelease_channel
    };
    let copy_label = if state.diagnostics_state == DiagnosticsState::Copied {
        catalog.diagnostics_copied
    } else {
        catalog.copy_diagnostics
    };
    let content = column![
        row![
            text(catalog.about_window).size(18),
            button("×").on_press(Message::ClosePanel)
        ]
        .spacing(12),
        text(catalog.about_description).size(13),
        text(format!(
            "{}: {}",
            catalog.version,
            env!("CARGO_PKG_VERSION")
        )),
        text(format!("{}: {channel}", catalog.release_channel)),
        text(format!(
            "{}: {} / {}",
            catalog.platform,
            std::env::consts::OS,
            std::env::consts::ARCH
        )),
        text(format!(
            "{}: {}",
            catalog.display_protocol,
            display_protocol()
        )),
        text(format!("{}: Rust + Iced 0.14", catalog.runtime)),
        text(format!("{}: GPL-3.0-only", catalog.license)),
        button("rayan6ms GitHub").on_press(Message::OpenLink(AUTHOR_URL)),
        button("Camlet GitHub").on_press(Message::OpenLink(PROJECT_URL)),
        button("Camlet GitHub Issues").on_press(Message::OpenLink(ISSUES_URL)),
        button(copy_label).on_press(Message::CopyDiagnostics),
    ]
    .spacing(8)
    .padding(12);
    container(scrollable(content).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(container::dark)
        .into()
}

fn startup_error_view(state: &Camlet) -> Element<'_, Message> {
    let catalog = catalog(
        state.product.settings.language,
        state.system_locale.as_deref(),
    );
    let content = column![
        text(APP_NAME).size(13),
        text(catalog.startup_error_title).size(18),
        text(catalog.startup_error_message).size(13),
        button(catalog.reload).on_press(Message::RetryStartup),
        button(catalog.continue_with_defaults).on_press(Message::ContinueWithDefaults),
    ]
    .spacing(10)
    .padding(16);
    container(content)
        .center(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(container::dark)
        .into()
}

const fn camera_status_text(catalog: &Catalog, status: CameraStatus) -> &'static str {
    match status {
        CameraStatus::Loading => catalog.camera_status.loading,
        CameraStatus::Preview => catalog.camera_status.preview,
        CameraStatus::PermissionDenied => catalog.camera_status.permission_denied,
        CameraStatus::CameraInUse => catalog.camera_status.camera_in_use,
        CameraStatus::NoCamera => catalog.camera_status.no_camera,
        CameraStatus::SelectedDeviceUnavailable => {
            catalog.camera_status.selected_device_unavailable
        }
        CameraStatus::Error => catalog.camera_status.error,
    }
}

fn menu_choice_button(label: &str, selected: bool, action: Action) -> Element<'static, Message> {
    let caption = if selected {
        format!("✓  {label}")
    } else {
        format!("   {label}")
    };
    button(text(caption).size(12))
        .width(Length::Fill)
        .height(32)
        .padding([4, 8])
        .style(button::subtle)
        .on_press(Message::MenuProduct(action))
        .into()
}

fn compact_choice_button(label: &str, selected: bool, action: Action) -> Element<'static, Message> {
    button(text(label.to_owned()).size(11))
        .height(28)
        .padding([3, 7])
        .style(if selected {
            button::success
        } else {
            button::subtle
        })
        .on_press(Message::MenuProduct(action))
        .into()
}

fn compact_option_row<const N: usize>(
    options: [(String, Action); N],
    selected: u8,
) -> Element<'static, Message> {
    let mut controls = row![].spacing(3);
    for (label, action) in options {
        let is_selected = action_value(&action) == selected;
        controls = controls.push(
            button(text(label).size(11))
                .height(28)
                .padding([3, 7])
                .style(if is_selected {
                    button::success
                } else {
                    button::subtle
                })
                .on_press(Message::MenuProduct(action)),
        );
    }
    controls.into()
}

const fn action_value(action: &Action) -> u8 {
    match action {
        Action::SetRingThickness(value)
        | Action::SetCornerRoundness(value)
        | Action::SetCameraFps(value) => *value,
        _ => 0,
    }
}

const fn language_label(catalog: &Catalog, language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::System => catalog.language_system,
        AppLanguage::English => catalog.language_english,
        AppLanguage::PortugueseBrazil => catalog.language_portuguese_brazil,
    }
}

fn resize_view(state: &Camlet) -> Element<'_, Message> {
    let catalog = catalog(
        state.product.settings.language,
        state.system_locale.as_deref(),
    );
    let controls = column![
        text(catalog.resize).size(16),
        text(format!("{} px", state.product.settings.window.width)).size(13),
        row![
            button("−").on_press(Message::Product(Action::ResizeByStep {
                grow: false,
                maximum: state.monitor_maximum,
            })),
            button("+").on_press(Message::Product(Action::ResizeByStep {
                grow: true,
                maximum: state.monitor_maximum,
            })),
        ]
        .spacing(8),
        button(catalog.resize_done).on_press(Message::Product(Action::SetResizeMode(false))),
    ]
    .spacing(8)
    .padding(12);
    container(controls)
        .center(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(container::dark)
        .into()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use camlet_camera::{CameraDevice, CameraError, CameraWorkerEvent, VideoFrame};
    use camlet_core::appearance::ThemeId;
    use camlet_core::settings::AppSettings;
    use camlet_core::state::{AppState, CameraOption};
    use iced::keyboard::key::{Code, Named, Physical};
    use iced::keyboard::{Key, Modifiers};
    use iced::{Point, Size, event::Event, mouse};
    use iced_test::simulator;
    use num_traits::ToPrimitive;

    use super::{
        Action, AutomationMode, CameraPollResult, Camlet, DiagnosticsState, FrameSourceKind,
        Lifecycle, MENU_WINDOW_HEIGHT, MENU_WINDOW_WIDTH, MenuPage, Message, PROJECT_URL, Panel,
        ScreenshotState, about_view, apply_product_action, diagnostics_json, handle_camera_poll,
        handle_window_moved, handle_window_resized, keyboard_action, main_view,
        menu_window_position, popup_window_settings, root_menu_view, submenu_view,
    };

    fn test_state(menu_open: bool) -> Camlet {
        let mut product = AppState::new(AppSettings::default());
        product.cameras = vec![CameraOption {
            id: "synthetic".to_owned(),
            label: "Camlet synthetic camera".to_owned(),
        }];
        product.active_camera_id = Some("synthetic".to_owned());
        Camlet {
            product,
            automation: AutomationMode::None,
            frame_source_kind: FrameSourceKind::Synthetic,
            camera_worker: None,
            camera_events: None,
            source_frame: None,
            placeholder_frame: super::camera_placeholder_frame(),
            frame_revision: 0,
            capture_first_frame_at: None,
            capture_frame_count: 0,
            preview_error: Some("test preview".to_owned()),
            panel: if menu_open {
                Panel::Menu
            } else {
                Panel::Preview
            },
            screenshot_path: None,
            screenshot_state: ScreenshotState::Waiting,
            window_id: None,
            settings_window_id: None,
            submenu_window_id: None,
            active_submenu: None,
            preview_cursor: Point::new(112.0, 112.0),
            menu_origin: Point::ORIGIN,
            scale_factor: 1.0,
            monitor_maximum: 640,
            monitor_size: Some(iced::Size::new(1_920.0, 1_080.0)),
            profile_directory: None,
            settings_path: None,
            settings_writable: false,
            profile_origin: "test",
            persist_generation: 0,
            persist_scheduled: false,
            pending_window_position: None,
            pending_window_size: None,
            lifecycle: Lifecycle::Running,
            diagnostics_state: DiagnosticsState::Ready,
            automation_session: None,
            system_locale: Some("en-US".to_owned()),
        }
    }

    #[test]
    fn keyboard_parity_maps_small_large_moves_and_resize() {
        assert_eq!(
            keyboard_action(
                &Key::Named(Named::ArrowLeft),
                Physical::Code(Code::ArrowLeft),
                Modifiers::SHIFT,
                640,
            ),
            Some(Action::NudgeWindow {
                x: -1,
                y: 0,
                accelerated: true,
            })
        );
        assert_eq!(
            keyboard_action(
                &Key::Character("+".into()),
                Physical::Code(Code::Equal),
                Modifiers::NONE,
                480,
            ),
            Some(Action::ResizeByStep {
                grow: true,
                maximum: 480,
            })
        );
        assert_eq!(
            keyboard_action(
                &Key::Named(Named::F1),
                Physical::Code(Code::F1),
                Modifiers::NONE,
                640,
            ),
            None
        );
    }

    #[test]
    fn iced_menu_exposes_and_dispatches_appearance_actions() {
        let mut state = test_state(true);
        state.active_submenu = Some(MenuPage::Theme);
        let mut ui = simulator(submenu_view(&state));
        ui.click("   Ocean")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(ui.into_messages().into_iter().any(|message| matches!(
            message,
            Message::MenuProduct(Action::SetTheme(ThemeId::Ocean))
        )));
    }

    #[test]
    fn iced_menu_exposes_camera_selection_and_retry_actions() {
        let mut state = test_state(true);
        state.active_submenu = Some(MenuPage::Camera);
        let mut camera_ui = simulator(submenu_view(&state));
        camera_ui
            .click("✓  Camlet synthetic camera")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(camera_ui.into_messages().into_iter().any(|message| {
            matches!(message, Message::MenuProduct(Action::SetCamera(id)) if id == "synthetic")
        }));

        state.active_submenu = Some(MenuPage::Advanced);
        let mut retry_ui = simulator(submenu_view(&state));
        retry_ui
            .click("Retry camera")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            retry_ui
                .into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::MenuProduct(Action::RetryCamera)))
        );
    }

    #[test]
    fn advanced_menu_exposes_every_supported_camera_frame_rate() {
        let mut state = test_state(true);
        state.active_submenu = Some(MenuPage::Advanced);
        let mut ui = simulator(submenu_view(&state));
        ui.click("60")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            ui.into_messages().into_iter().any(|message| {
                matches!(message, Message::MenuProduct(Action::SetCameraFps(60)))
            })
        );
    }

    #[test]
    fn camera_failure_placeholder_exposes_a_visible_retry_action() {
        let mut state = test_state(false);
        state.product.camera_status = camlet_core::state::CameraStatus::Error;
        let mut ui = simulator(main_view(&state));
        ui.click("Retry camera")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            ui.into_messages()
                .into_iter()
                .any(|message| { matches!(message, Message::Product(Action::RetryCamera)) })
        );
    }

    #[test]
    fn camera_loading_placeholder_does_not_offer_a_premature_retry() {
        let mut state = test_state(false);
        state.product.camera_status = camlet_core::state::CameraStatus::Loading;
        let mut ui = simulator(main_view(&state));
        assert!(ui.click("Retry camera").is_err());
    }

    #[test]
    fn stale_programmatic_window_feedback_cannot_undo_newer_keyboard_input() {
        let mut state = test_state(false);
        let window_id = iced::window::Id::unique();
        state.window_id = Some(window_id);
        let initial_x = state.product.settings.window.x;
        let initial_y = state.product.settings.window.y;
        let initial_size = state.product.settings.window.width;

        let _ = apply_product_action(
            &mut state,
            Action::NudgeWindow {
                x: 1,
                y: 0,
                accelerated: false,
            },
        );
        let _ = apply_product_action(
            &mut state,
            Action::NudgeWindow {
                x: 1,
                y: 0,
                accelerated: false,
            },
        );
        let _ = handle_window_moved(
            &mut state,
            Point::new(
                (initial_x + 1).to_f32().unwrap_or(0.0),
                initial_y.to_f32().unwrap_or(0.0),
            ),
        );
        assert_eq!(state.product.settings.window.x, initial_x + 2);
        assert_eq!(
            state.pending_window_position,
            Some((initial_x + 2, initial_y))
        );
        let _ = handle_window_moved(
            &mut state,
            Point::new(
                (initial_x + 2).to_f32().unwrap_or(0.0),
                initial_y.to_f32().unwrap_or(0.0),
            ),
        );
        assert_eq!(state.pending_window_position, None);

        for _ in 0..2 {
            let _ = apply_product_action(
                &mut state,
                Action::ResizeByStep {
                    grow: true,
                    maximum: 640,
                },
            );
        }
        let first_step = initial_size + 24;
        let final_size = initial_size + 48;
        let _ = handle_window_resized(
            &mut state,
            window_id,
            Size::new(f32::from(first_step), f32::from(first_step)),
        );
        assert_eq!(state.product.settings.window.width, final_size);
        assert_eq!(state.pending_window_size, Some(final_size));
        let _ = handle_window_resized(
            &mut state,
            window_id,
            Size::new(f32::from(final_size), f32::from(final_size)),
        );
        assert_eq!(state.pending_window_size, None);
    }

    #[test]
    fn right_click_on_preview_opens_the_iced_menu_path() {
        let state = test_state(false);
        let mut ui = simulator(main_view(&state));
        ui.point_at(Point::new(112.0, 112.0));
        let _ = ui.simulate([
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)),
        ]);
        assert!(
            ui.into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::OpenMenu))
        );
    }

    #[test]
    fn camera_events_drive_the_same_product_and_rendering_path() {
        let mut state = test_state(false);
        state.product.cameras.clear();
        state.product.active_camera_id = None;
        let _ = handle_camera_poll(
            &mut state,
            CameraPollResult::Event(CameraWorkerEvent::Devices(Ok(vec![CameraDevice {
                id: "synthetic".to_owned(),
                label: "Synthetic".to_owned(),
            }]))),
        );
        assert_eq!(state.product.active_camera_id.as_deref(), Some("synthetic"));

        let _ = handle_camera_poll(
            &mut state,
            CameraPollResult::Event(CameraWorkerEvent::Frame(Ok(VideoFrame {
                width: 2,
                height: 2,
                sequence: 1,
                rgba: vec![64; 16],
            }))),
        );
        assert_eq!(
            state.product.camera_status,
            camlet_core::state::CameraStatus::Preview
        );
        assert!(state.source_frame.is_some());
    }

    #[test]
    fn camera_frames_and_settings_changes_do_not_dismiss_the_settings_window() {
        let mut state = test_state(true);
        let _ = handle_camera_poll(
            &mut state,
            CameraPollResult::Event(CameraWorkerEvent::Frame(Ok(VideoFrame {
                width: 2,
                height: 2,
                sequence: 1,
                rgba: vec![64; 16],
            }))),
        );
        assert_eq!(state.panel, Panel::Menu);

        let _ = apply_product_action(&mut state, Action::SetTheme(ThemeId::Ocean));
        assert_eq!(state.panel, Panel::Menu);
    }

    #[test]
    fn losing_all_devices_clears_the_last_camera_frame() {
        let mut state = test_state(false);
        state.source_frame = Some(Arc::new(VideoFrame {
            width: 1,
            height: 1,
            sequence: 1,
            rgba: vec![64; 4],
        }));
        state.product.settings.selected_camera_device_id = Some("synthetic".to_owned());

        let _ = handle_camera_poll(
            &mut state,
            CameraPollResult::Event(CameraWorkerEvent::Devices(Ok(Vec::new()))),
        );

        assert!(state.source_frame.is_none());
        assert_eq!(
            state.product.camera_status,
            camlet_core::state::CameraStatus::SelectedDeviceUnavailable
        );
    }

    #[test]
    fn menu_window_opens_at_the_pointer_and_stays_on_screen() {
        let mut state = test_state(false);
        state.product.settings.window.x = 100;
        state.product.settings.window.y = 120;
        state.product.settings.window.width = 224;
        state.preview_cursor = Point::new(80.0, 90.0);
        assert_eq!(
            menu_window_position(&state, MENU_WINDOW_WIDTH, MENU_WINDOW_HEIGHT),
            Point::new(180.0, 210.0)
        );

        state.product.settings.window.x = 1_700;
        assert_eq!(
            menu_window_position(&state, MENU_WINDOW_WIDTH, MENU_WINDOW_HEIGHT),
            Point::new(1_634.0, 210.0)
        );
    }

    #[test]
    fn context_menu_window_has_no_native_chrome_or_resize_controls() {
        let settings = popup_window_settings(
            Size::new(MENU_WINDOW_WIDTH, MENU_WINDOW_HEIGHT),
            Point::new(120.0, 140.0),
        );
        assert!(!settings.decorations);
        assert!(!settings.resizable);
        assert!(settings.transparent);
        assert_eq!(settings.min_size, None);
        assert_eq!(settings.max_size, None);
        #[cfg(target_os = "linux")]
        assert!(settings.platform_specific.override_redirect);
        #[cfg(target_os = "windows")]
        assert!(settings.platform_specific.skip_taskbar);
    }

    #[test]
    fn camera_backend_failures_become_stable_product_states() {
        for (error, expected) in [
            (
                CameraError::PermissionDenied,
                camlet_core::state::CameraStatus::PermissionDenied,
            ),
            (
                CameraError::Busy,
                camlet_core::state::CameraStatus::CameraInUse,
            ),
            (
                CameraError::DeviceNotFound,
                camlet_core::state::CameraStatus::SelectedDeviceUnavailable,
            ),
            (
                CameraError::Backend,
                camlet_core::state::CameraStatus::Error,
            ),
        ] {
            let mut state = test_state(false);
            let _ = handle_camera_poll(
                &mut state,
                CameraPollResult::Event(CameraWorkerEvent::Started(Err(error))),
            );
            assert_eq!(state.product.camera_status, expected);
            assert!(state.source_frame.is_none());
            assert!(state.preview_error.is_some());
        }
    }

    #[test]
    fn diagnostics_exclude_paths_camera_ids_and_frame_pixels() {
        let mut state = test_state(false);
        state.product.settings.selected_camera_device_id = Some("private-camera-id".to_owned());
        state.settings_path = Some("/private/home/user/settings.json".into());
        state.source_frame = Some(Arc::new(VideoFrame {
            width: 1,
            height: 1,
            sequence: 99,
            rgba: vec![11, 22, 33, 44],
        }));

        let diagnostics = diagnostics_json(&state);
        assert!(!diagnostics.contains("private-camera-id"));
        assert!(!diagnostics.contains("/private/home"));
        assert!(!diagnostics.contains("rgba"));
        assert!(!diagnostics.contains("sequence"));
        assert!(diagnostics.contains("\"schemaVersion\": 1"));
        assert!(serde_json::from_str::<serde_json::Value>(&diagnostics).is_ok());
    }

    #[test]
    fn about_and_startup_recovery_controls_are_reachable() {
        let mut about = test_state(false);
        about.panel = Panel::About;
        let mut about_ui = simulator(about_view(&about));
        about_ui
            .click("Copy diagnostics")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            about_ui
                .into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::CopyDiagnostics))
        );

        let mut link_ui = simulator(about_view(&about));
        link_ui
            .click("Camlet GitHub")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            link_ui
                .into_messages()
                .into_iter()
                .any(|message| { matches!(message, Message::OpenLink(PROJECT_URL)) })
        );

        let mut startup = test_state(false);
        startup.panel = Panel::StartupError;
        let mut startup_ui = simulator(main_view(&startup));
        startup_ui
            .click("Continue with safe defaults")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            startup_ui
                .into_messages()
                .into_iter()
                .any(|message| { matches!(message, Message::ContinueWithDefaults) })
        );

        let mut retry_ui = simulator(main_view(&startup));
        retry_ui
            .click("Reload Camlet")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            retry_ui
                .into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::RetryStartup))
        );
    }

    #[test]
    fn advanced_about_and_quit_menu_actions_are_reachable() {
        let state = test_state(true);
        let mut about_ui = simulator(root_menu_view(&state));
        about_ui
            .click("About")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            about_ui
                .into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::Product(Action::OpenAbout)))
        );

        let mut quit_ui = simulator(root_menu_view(&state));
        quit_ui
            .click("Close Camlet")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            quit_ui
                .into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::MenuProduct(Action::Quit)))
        );
    }
}
