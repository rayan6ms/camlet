use std::path::PathBuf;
use std::time::Duration;

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
use camlet_core::settings::{AppSettings, write_settings};
use camlet_core::state::{Action, AppState, CameraOption, CameraStatus, Effect};
use camlet_core::{APP_NAME, FrameSourceKind};
use iced::futures::SinkExt;
use iced::keyboard::key::{Code, Named, Physical};
use iced::keyboard::{Key, Modifiers};
use iced::widget::{
    Column, button, column, container, mouse_area, opaque, row, scrollable, shader, stack, text,
};
use iced::{
    Color, Element, Length, Point, Size, Subscription, Task, Theme, clipboard, event, keyboard,
    stream, theme, window,
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
const AUTHOR_URL: &str = "https://github.com/rayan6ms";
const PROJECT_URL: &str = "https://github.com/rayan6ms/camlet";
const ISSUES_URL: &str = "https://github.com/rayan6ms/camlet/issues";

struct Camlet {
    product: AppState,
    automation: AutomationMode,
    frame_source_kind: FrameSourceKind,
    camera_worker: Option<CameraWorker>,
    camera_events: Option<CameraWorkerEvents>,
    source_frame: Option<VideoFrame>,
    placeholder_frame: VideoFrame,
    frame_revision: u64,
    preview_error: Option<String>,
    panel: Panel,
    screenshot_path: Option<PathBuf>,
    screenshot_state: ScreenshotState,
    window_id: Option<window::Id>,
    scale_factor: f32,
    monitor_maximum: u16,
    profile_directory: Option<PathBuf>,
    settings_path: Option<PathBuf>,
    settings_writable: bool,
    profile_origin: &'static str,
    persist_generation: u64,
    persist_scheduled: bool,
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
        key: Key,
        physical: Physical,
        modifiers: Modifiers,
    },
    PreviewPressed,
    ToggleMenu,
    ClosePanel,
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

    iced::application(
        move || {
            boot(
                frame_source,
                automation,
                screenshot_path.clone(),
                profile.clone(),
                profile_directory.clone(),
                startup_error,
                automation_session.clone(),
            )
        },
        update,
        view,
    )
    .title(APP_NAME)
    .theme(Theme::Dark)
    .style(|_, selected_theme| theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: selected_theme.palette().text,
    })
    .subscription(subscription)
    .antialiasing(true)
    .window(window::Settings {
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
    })
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
    let mut state = Camlet {
        product,
        automation,
        frame_source_kind,
        camera_worker,
        camera_events,
        source_frame: None,
        placeholder_frame: camera_placeholder_frame(),
        frame_revision: 0,
        preview_error,
        panel: if startup_error {
            Panel::StartupError
        } else {
            Panel::Preview
        },
        screenshot_path,
        screenshot_state: ScreenshotState::Waiting,
        window_id: None,
        scale_factor: 1.0,
        monitor_maximum: MAXIMUM_OVERLAY_SIZE,
        profile_directory,
        settings_path: profile.settings_path,
        settings_writable: profile.writable,
        profile_origin: profile.origin,
        persist_generation: 0,
        persist_scheduled: false,
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
            key,
            physical,
            modifiers,
        } => {
            if key == Key::Named(Named::Escape) && state.panel != Panel::Preview {
                state.panel = Panel::Preview;
                return Task::none();
            }
            keyboard_action(&key, physical, modifiers, state.monitor_maximum)
                .map_or_else(Task::none, |action| apply_product_action(state, action))
        }
        Message::PreviewPressed => state.window_id.map_or_else(Task::none, |id| {
            if state.product.resize_mode {
                window::drag_resize(id, window::Direction::SouthEast)
            } else {
                window::drag(id)
            }
        }),
        Message::ToggleMenu => {
            state.panel = if state.panel == Panel::Menu {
                Panel::Preview
            } else {
                Panel::Menu
            };
            Task::none()
        }
        Message::ClosePanel => {
            state.panel = Panel::Preview;
            Task::none()
        }
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
            state.window_id.map_or_else(Task::none, window::close)
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

fn handle_window_event(state: &mut Camlet, id: window::Id, event: &window::Event) -> Task<Message> {
    match event {
        window::Event::Opened { position, size } => {
            state.window_id = Some(id);
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
        window::Event::Moved(position) => {
            record_position(state, *position);
            request_persistence(state)
        }
        window::Event::Resized(size) => {
            record_size(state, *size);
            rerender(state);
            let square = state.product.settings.window.width;
            let resize_task = if size.width.round().to_u16() != Some(square)
                || size.height.round().to_u16() != Some(square)
            {
                window::resize(id, Size::new(f32::from(square), f32::from(square)))
            } else {
                Task::none()
            };
            Task::batch([request_persistence(state), resize_task])
        }
        window::Event::Rescaled(scale_factor) => {
            state.scale_factor = scale_factor.max(0.5);
            rerender(state);
            Task::none()
        }
        window::Event::CloseRequested => clean_shutdown(state, id),
        _ => Task::none(),
    }
}

fn subscription(state: &Camlet) -> Subscription<Message> {
    let window_events = event::listen_with(|event, status, id| match event {
        event::Event::Window(window_event) => Some(Message::WindowEvent(id, window_event)),
        event::Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if status == event::Status::Ignored => Some(Message::KeyPressed {
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
    } else if requested_backend.as_deref() == Some("x11") {
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
            state.source_frame = Some(frame);
            state.frame_revision = state.frame_revision.saturating_add(1);
            let ready_task = apply_product_action(state, Action::CameraReady);
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
    eprintln!("camlet: automation scenario failed");
    state
        .window_id
        .map_or_else(Task::none, |id| clean_shutdown(state, id))
}

fn apply_product_action(state: &mut Camlet, action: Action) -> Task<Message> {
    let resize_mode = match action {
        Action::SetResizeMode(enabled) => Some(enabled),
        _ => None,
    };
    let effects = state.product.update(action);
    if !effects.contains(&Effect::OpenAbout) {
        state.panel = Panel::Preview;
    }
    rerender(state);

    let mut tasks = effects
        .iter()
        .map(|effect| execute_effect(state, effect))
        .collect::<Vec<_>>();
    if let Some(enabled) = resize_mode
        && let Some(id) = state.window_id
    {
        tasks.push(window::set_resizable(id, enabled));
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
            window::move_to(
                id,
                Point::new(
                    window_state.x.to_f32().unwrap_or(0.0),
                    window_state.y.to_f32().unwrap_or(0.0),
                ),
            )
        }),
        Effect::ResizeWindow(window_state) => state.window_id.map_or_else(Task::none, |id| {
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
            send_camera_command(
                state,
                CameraWorkerCommand::Start {
                    device_id: Some(device_id.clone()),
                    request: CaptureRequest {
                        width: 640,
                        height: 480,
                        frame_interval: Duration::from_millis(33),
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
            Task::none()
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

fn clean_shutdown(state: &mut Camlet, id: window::Id) -> Task<Message> {
    if state.lifecycle == Lifecycle::ShuttingDown {
        return Task::none();
    }
    state.lifecycle = Lifecycle::ShuttingDown;
    persist_now(state);
    state.camera_events = None;
    state.camera_worker.take().map_or_else(
        || window::close(id),
        |worker| {
            Task::perform(
                async move { worker.shutdown() },
                Message::CameraShutdownFinished,
            )
        },
    )
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
    let message = column![
        text(camera_status_text(catalog, state.product.camera_status))
            .size(18)
            .color(Color::WHITE),
        text(detail).size(12).color(Color::from_rgb8(207, 218, 232)),
        button(text(format!("↻ {}", catalog.retry_camera)))
            .on_press(Message::Product(Action::RetryCamera)),
    ]
    .align_x(iced::Alignment::Center)
    .spacing(8)
    .padding(24);
    stack![
        overlay,
        container(message)
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
    ]
    .into()
}

fn camera_placeholder_frame() -> VideoFrame {
    const WIDTH: u32 = 4;
    const HEIGHT: u32 = 3;
    let mut rgba = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for _ in 0..WIDTH * HEIGHT {
        rgba.extend_from_slice(&[8, 13, 22, u8::MAX]);
    }
    VideoFrame {
        width: WIDTH,
        height: HEIGHT,
        sequence: 0,
        rgba,
    }
}

fn view(state: &Camlet) -> Element<'_, Message> {
    let preview = mouse_area(
        container(preview_view(state))
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(Message::PreviewPressed)
    .on_right_press(Message::ToggleMenu);

    let mut layers = stack![preview].width(Length::Fill).height(Length::Fill);
    match state.panel {
        Panel::Menu => layers = layers.push(opaque(menu_view(state))),
        Panel::About => layers = layers.push(opaque(about_view(state))),
        Panel::StartupError => layers = layers.push(opaque(startup_error_view(state))),
        Panel::Preview if state.product.resize_mode => {
            layers = layers.push(opaque(resize_view(state)));
        }
        Panel::Preview => {}
    }
    layers.into()
}

fn menu_view(state: &Camlet) -> Element<'_, Message> {
    let catalog = catalog(
        state.product.settings.language,
        state.system_locale.as_deref(),
    );
    let model = MenuModel::from_state(&state.product);
    let mut content = column![
        row![
            text(APP_NAME).size(18),
            button("×").on_press(Message::ClosePanel)
        ]
        .spacing(12),
    ]
    .spacing(6)
    .padding(10);

    content = camera_menu_controls(content, catalog, &model);
    content = appearance_menu_controls(content, state, catalog, &model);
    content = system_menu_controls(content, state, catalog, &model);

    container(scrollable(content).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(container::dark)
        .into()
}

fn camera_menu_controls<'a>(
    mut content: Column<'a, Message>,
    catalog: &'a Catalog,
    model: &MenuModel,
) -> Column<'a, Message> {
    content = content.push(text(catalog.camera_device).size(13));
    if model.cameras.is_empty() {
        content = content.push(text(catalog.no_devices).size(12));
    } else {
        for choice in &model.cameras {
            content = content.push(choice_button(
                &choice.label,
                choice.selected,
                Action::SetCamera(choice.id.clone()),
            ));
        }
    }
    content
        .push(text(camera_status_text(catalog, model.camera_status)).size(12))
        .push(button(catalog.retry_camera).on_press(Message::Product(Action::RetryCamera)))
}

fn appearance_menu_controls<'a>(
    mut content: Column<'a, Message>,
    state: &Camlet,
    catalog: &'a Catalog,
    model: &MenuModel,
) -> Column<'a, Message> {
    content = content.push(text(catalog.theme).size(13));
    for (choice, label) in model.themes.iter().zip(catalog.themes) {
        content = content.push(choice_button(
            label,
            choice.selected,
            Action::SetTheme(choice.value),
        ));
    }
    content = content.push(text(catalog.shape).size(13));
    for (choice, label) in model.shapes.iter().zip(catalog.shapes) {
        content = content.push(choice_button(
            label,
            choice.selected,
            Action::SetShape(choice.value),
        ));
    }
    content = content.push(text(catalog.fit_mode).size(13));
    for (choice, label) in model.fit_modes.iter().zip(catalog.fit_modes) {
        content = content.push(choice_button(
            label,
            choice.selected,
            Action::SetFit(choice.value),
        ));
    }
    content = content
        .push(text(catalog.ring_thickness).size(13))
        .push(option_row(
            RING_THICKNESS_OPTIONS
                .map(|value| (value.to_string(), Action::SetRingThickness(value))),
            state.product.settings.appearance.ring_thickness,
        ))
        .push(text(catalog.corner_roundness).size(13))
        .push(option_row(
            CORNER_ROUNDNESS_OPTIONS
                .map(|value| (value.to_string(), Action::SetCornerRoundness(value))),
            state.product.settings.appearance.corner_roundness,
        ))
        .push(text(catalog.language).size(13));
    for choice in &model.languages {
        content = content.push(choice_button(
            language_label(catalog, choice.value),
            choice.selected,
            Action::SetLanguage(choice.value),
        ));
    }
    content
        .push(button(catalog.resize).on_press(Message::Product(Action::SetResizeMode(true))))
        .push(button(catalog.reset_appearance).on_press(Message::Product(Action::ResetAppearance)))
}

fn system_menu_controls<'a>(
    content: Column<'a, Message>,
    state: &'a Camlet,
    catalog: &'a Catalog,
    model: &MenuModel,
) -> Column<'a, Message> {
    content
        .push(text(catalog.advanced).size(13))
        .push(
            text(format!(
                "{}: {}",
                catalog.preview_state,
                camera_status_text(catalog, model.camera_status)
            ))
            .size(11),
        )
        .push(
            text(format!(
                "{}: {}",
                catalog.active_camera,
                active_camera_label(state, catalog.none)
            ))
            .size(11),
        )
        .push(
            text(format!(
                "{}: {}",
                catalog.display_protocol,
                display_protocol()
            ))
            .size(11),
        )
        .push(button(catalog.about).on_press(Message::Product(Action::OpenAbout)))
        .push(button(catalog.close_app).on_press(Message::Product(Action::Quit)))
        .push(text(format!("Profile: {}", state.profile_origin)).size(10))
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

fn choice_button(label: &str, selected: bool, action: Action) -> Element<'static, Message> {
    let caption = if selected {
        format!("● {label}")
    } else {
        format!("  {label}")
    };
    button(text(caption).size(12))
        .width(Length::Fill)
        .style(if selected {
            button::success
        } else {
            button::subtle
        })
        .on_press(Message::Product(action))
        .into()
}

fn option_row<const N: usize>(
    options: [(String, Action); N],
    selected: u8,
) -> Element<'static, Message> {
    let mut controls = row![].spacing(3);
    for (label, action) in options {
        let is_selected = action_value(&action) == selected;
        controls = controls.push(
            button(text(label).size(11))
                .style(if is_selected {
                    button::success
                } else {
                    button::subtle
                })
                .on_press(Message::Product(action)),
        );
    }
    controls.into()
}

const fn action_value(action: &Action) -> u8 {
    match action {
        Action::SetRingThickness(value) | Action::SetCornerRoundness(value) => *value,
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
    use camlet_camera::{CameraDevice, CameraError, CameraWorkerEvent, VideoFrame};
    use camlet_core::appearance::ThemeId;
    use camlet_core::settings::AppSettings;
    use camlet_core::state::{AppState, CameraOption};
    use iced::keyboard::key::{Code, Named, Physical};
    use iced::keyboard::{Key, Modifiers};
    use iced::{Point, event::Event, mouse};
    use iced_test::simulator;

    use super::{
        Action, AutomationMode, CameraPollResult, Camlet, DiagnosticsState, FrameSourceKind,
        Lifecycle, Message, PROJECT_URL, Panel, ScreenshotState, diagnostics_json,
        handle_camera_poll, keyboard_action, view,
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
            preview_error: Some("test preview".to_owned()),
            panel: if menu_open {
                Panel::Menu
            } else {
                Panel::Preview
            },
            screenshot_path: None,
            screenshot_state: ScreenshotState::Waiting,
            window_id: None,
            scale_factor: 1.0,
            monitor_maximum: 640,
            profile_directory: None,
            settings_path: None,
            settings_writable: false,
            profile_origin: "test",
            persist_generation: 0,
            persist_scheduled: false,
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
        let state = test_state(true);
        let mut ui = simulator(view(&state));
        ui.click("  Ocean")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            ui.into_messages().into_iter().any(|message| matches!(
                message,
                Message::Product(Action::SetTheme(ThemeId::Ocean))
            ))
        );
    }

    #[test]
    fn iced_menu_exposes_camera_selection_and_retry_actions() {
        let state = test_state(true);
        let mut camera_ui = simulator(view(&state));
        camera_ui
            .click("● Camlet synthetic camera")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(camera_ui.into_messages().into_iter().any(|message| {
            matches!(message, Message::Product(Action::SetCamera(id)) if id == "synthetic")
        }));

        let mut retry_ui = simulator(view(&state));
        retry_ui
            .click("Retry camera")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            retry_ui
                .into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::Product(Action::RetryCamera)))
        );
    }

    #[test]
    fn camera_failure_placeholder_exposes_a_visible_retry_action() {
        let state = test_state(false);
        let mut ui = simulator(view(&state));
        ui.click("↻ Retry camera")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            ui.into_messages()
                .into_iter()
                .any(|message| { matches!(message, Message::Product(Action::RetryCamera)) })
        );
    }

    #[test]
    fn right_click_on_preview_opens_the_iced_menu_path() {
        let state = test_state(false);
        let mut ui = simulator(view(&state));
        ui.point_at(Point::new(112.0, 112.0));
        let _ = ui.simulate([
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)),
        ]);
        assert!(
            ui.into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::ToggleMenu))
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
        state.source_frame = Some(VideoFrame {
            width: 1,
            height: 1,
            sequence: 99,
            rgba: vec![11, 22, 33, 44],
        });

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
        let mut about_ui = simulator(view(&about));
        about_ui
            .click("Copy diagnostics")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            about_ui
                .into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::CopyDiagnostics))
        );

        let mut link_ui = simulator(view(&about));
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
        let mut startup_ui = simulator(view(&startup));
        startup_ui
            .click("Continue with safe defaults")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            startup_ui
                .into_messages()
                .into_iter()
                .any(|message| { matches!(message, Message::ContinueWithDefaults) })
        );

        let mut retry_ui = simulator(view(&startup));
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
        let mut about_ui = simulator(view(&state));
        about_ui.point_at(Point::new(100.0, 200.0));
        let _ = about_ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -60.0 },
        })]);
        about_ui
            .click("About")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            about_ui
                .into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::Product(Action::OpenAbout)))
        );

        let mut quit_ui = simulator(view(&state));
        quit_ui.point_at(Point::new(100.0, 200.0));
        let _ = quit_ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -60.0 },
        })]);
        quit_ui
            .click("Close Camlet")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            quit_ui
                .into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::Product(Action::Quit)))
        );
    }
}
