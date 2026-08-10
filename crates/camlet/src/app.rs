use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use camlet_camera::{
    CameraDevice, CameraError, CameraWorker, CameraWorkerCommand, CameraWorkerEvent,
    CameraWorkerEvents, CaptureRequest, NokhwaFrameSource, SyntheticFrameSource, VideoFrame,
};
use camlet_core::appearance::MAXIMUM_OVERLAY_SIZE;
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
    button, column, container, mouse_area, opaque, row, rule, shader, space, stack, text,
};
use iced::{
    Background, Border, Color, Element, Length, Point, Rectangle, Shadow, Size, Subscription, Task,
    Theme, Vector, clipboard, event, keyboard, mouse, stream, theme, window,
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
const SUBMENU_HOVER_DELAY: Duration = Duration::from_millis(60);
const MENU_AIM_DELAY: Duration = Duration::from_millis(300);
const MENU_AIM_VERTICAL_TOLERANCE: f32 = 10.0;
const MENU_WINDOW_WIDTH: f32 = 260.0;
const MENU_ROW_HEIGHT: f32 = 28.0;
const MENU_PADDING: f32 = 5.0;
const MENU_SEPARATOR_HEIGHT: f32 = 9.0;
const MENU_WINDOW_HEIGHT: f32 = 252.0;
const ABOUT_WINDOW_WIDTH: f32 = 420.0;
const ABOUT_WINDOW_HEIGHT: f32 = 330.0;
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
    submenu_windows: Vec<MenuPopup>,
    menu_root_position: Point,
    menu_monitor_bounds: Option<Rectangle>,
    focused_menu_window: Option<window::Id>,
    menu_focus_generation: u64,
    submenu_hover_generation: u64,
    menu_pointer: Option<MenuPointer>,
    menu_intent_generation: u64,
    deferred_menu_parent: Option<window::Id>,
    preview_cursor: Point,
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
    CameraFps,
    CameraResolution,
    Fit,
    RingThickness,
    CornerRoundness,
}

#[derive(Debug, Clone, Copy)]
struct MenuPopup {
    id: window::Id,
    page: MenuPage,
    position: Point,
    size: Size,
    aim_origin: Point,
}

#[derive(Debug, Clone, Copy)]
struct MenuPointer {
    window_id: window::Id,
    previous: Point,
    current: Point,
}

#[derive(Debug, Clone, Copy)]
enum MenuIntent {
    Open {
        parent: window::Id,
        page: MenuPage,
        anchor_y: f32,
    },
    Close {
        parent: window::Id,
    },
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
    BeginWindowResize(window::Direction),
    PreviewCursorMoved(Point),
    MenuCursorMoved {
        window_id: window::Id,
        position: Point,
    },
    OpenMenu,
    OpenSubmenu {
        parent: window::Id,
        page: MenuPage,
        anchor_y: f32,
    },
    HoverSubmenu {
        parent: window::Id,
        page: MenuPage,
        anchor_y: f32,
    },
    OpenHoveredSubmenu {
        parent: window::Id,
        page: MenuPage,
        anchor_y: f32,
        generation: u64,
    },
    EvaluateSubmenuClose {
        parent: window::Id,
        generation: u64,
    },
    CloseSubmenusAfter(window::Id),
    ResolveMenuIntent {
        intent: MenuIntent,
        generation: u64,
    },
    DismissMenuIfUnfocused(u64),
    MenuProduct(Action),
    ClosePanel,
    WindowOpened(window::Id),
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
    AutomationMenuScreenshotReady(PathBuf, window::Screenshot),
}

#[derive(Debug, Clone)]
enum CameraPollResult {
    Event(CameraWorkerEvent),
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
        icon: app_icon(),
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
                Task::batch([boot_task, open_task.map(Message::WindowOpened)]),
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
    // The overlay shader performs analytic edge antialiasing itself. Multisampling every
    // transparent window adds resize/repaint cost without improving the camera edge.
    .antialiasing(false)
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
        submenu_windows: Vec::new(),
        menu_root_position: Point::ORIGIN,
        menu_monitor_bounds: None,
        focused_menu_window: None,
        menu_focus_generation: 0,
        submenu_hover_generation: 0,
        menu_pointer: None,
        menu_intent_generation: 0,
        deferred_menu_parent: None,
        preview_cursor,
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
    let camera_task = if startup_error {
        Task::none()
    } else {
        send_camera_command(&state, CameraWorkerCommand::Enumerate)
    };
    (state, camera_task)
}

fn update(state: &mut Camlet, message: Message) -> Task<Message> {
    match message {
        Message::WindowEvent(id, event) => handle_window_event(state, id, &event),
        Message::MonitorSize(size) => record_monitor_size(state, size),
        Message::KeyPressed {
            window_id,
            key,
            physical,
            modifiers,
        } => handle_key_pressed(state, window_id, &key, physical, modifiers),
        Message::PreviewPressed => handle_preview_pressed(state),
        Message::BeginWindowResize(direction) => state
            .window_id
            .map_or_else(Task::none, |id| window::drag_resize(id, direction)),
        Message::PreviewCursorMoved(position) => {
            state.preview_cursor = position;
            Task::none()
        }
        Message::MenuCursorMoved {
            window_id,
            position,
        } => track_menu_cursor(state, window_id, position),
        Message::OpenMenu => open_settings_window(state),
        Message::OpenSubmenu {
            parent,
            page,
            anchor_y,
        } => open_submenu_immediately(state, parent, page, anchor_y),
        Message::HoverSubmenu {
            parent,
            page,
            anchor_y,
        } => handle_submenu_hover(state, parent, page, anchor_y),
        Message::OpenHoveredSubmenu {
            parent,
            page,
            anchor_y,
            generation,
        } => open_hovered_submenu(state, parent, page, anchor_y, generation),
        Message::EvaluateSubmenuClose { parent, generation } => {
            evaluate_submenu_close(state, parent, generation)
        }
        Message::CloseSubmenusAfter(parent) => handle_submenu_close_hover(state, parent),
        Message::ResolveMenuIntent { intent, generation } => {
            resolve_menu_intent(state, intent, generation)
        }
        Message::DismissMenuIfUnfocused(generation) => dismiss_menu_if_unfocused(state, generation),
        Message::MenuProduct(action) => apply_menu_product(state, action),
        Message::ClosePanel => close_settings_window(state),
        Message::WindowOpened(id) => handle_window_opened(state, id),
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
        Message::AutomationMenuScreenshotReady(path, screenshot) => {
            finish_automation_menu_screenshot(state, &path, &screenshot)
        }
    }
}

fn dismiss_menu_if_unfocused(state: &mut Camlet, generation: u64) -> Task<Message> {
    if generation == state.menu_focus_generation && state.focused_menu_window.is_none() {
        close_settings_window(state)
    } else {
        Task::none()
    }
}

fn apply_menu_product(state: &mut Camlet, action: Action) -> Task<Message> {
    Task::batch([
        apply_product_action(state, action),
        close_settings_window(state),
    ])
}

fn open_submenu_immediately(
    state: &mut Camlet,
    parent: window::Id,
    page: MenuPage,
    anchor_y: f32,
) -> Task<Message> {
    cancel_menu_intent(state);
    cancel_submenu_hover(state);
    open_submenu_window(state, parent, page, anchor_y)
}

fn handle_submenu_hover(
    state: &mut Camlet,
    parent: window::Id,
    page: MenuPage,
    anchor_y: f32,
) -> Task<Message> {
    let intent = MenuIntent::Open {
        parent,
        page,
        anchor_y,
    };
    if cursor_is_aiming_at_child(state, parent) {
        defer_menu_intent(state, intent)
    } else {
        cancel_menu_intent(state);
        schedule_submenu_hover(state, parent, page, anchor_y)
    }
}

fn handle_submenu_close_hover(state: &mut Camlet, parent: window::Id) -> Task<Message> {
    let intent = MenuIntent::Close { parent };
    if cursor_is_aiming_at_child(state, parent) {
        defer_menu_intent(state, intent)
    } else {
        cancel_menu_intent(state);
        state.submenu_hover_generation = state.submenu_hover_generation.saturating_add(1);
        let generation = state.submenu_hover_generation;
        Task::perform(
            async move {
                futures_timer::Delay::new(SUBMENU_HOVER_DELAY).await;
                Message::EvaluateSubmenuClose { parent, generation }
            },
            std::convert::identity,
        )
    }
}

fn open_hovered_submenu(
    state: &mut Camlet,
    parent: window::Id,
    page: MenuPage,
    anchor_y: f32,
    generation: u64,
) -> Task<Message> {
    if generation != state.submenu_hover_generation {
        return Task::none();
    }
    if cursor_is_aiming_at_child(state, parent) {
        defer_menu_intent(
            state,
            MenuIntent::Open {
                parent,
                page,
                anchor_y,
            },
        )
    } else {
        open_submenu_window(state, parent, page, anchor_y)
    }
}

fn evaluate_submenu_close(
    state: &mut Camlet,
    parent: window::Id,
    generation: u64,
) -> Task<Message> {
    if generation != state.submenu_hover_generation {
        return Task::none();
    }
    if cursor_is_aiming_at_child(state, parent) {
        defer_menu_intent(state, MenuIntent::Close { parent })
    } else {
        close_submenus_immediately(state, parent)
    }
}

fn close_submenus_immediately(state: &mut Camlet, parent: window::Id) -> Task<Message> {
    cancel_menu_intent(state);
    cancel_submenu_hover(state);
    close_submenus_after(state, parent)
}

fn defer_menu_intent(state: &mut Camlet, intent: MenuIntent) -> Task<Message> {
    cancel_submenu_hover(state);
    state.menu_intent_generation = state.menu_intent_generation.saturating_add(1);
    let generation = state.menu_intent_generation;
    state.deferred_menu_parent = Some(menu_intent_parent(intent));
    Task::perform(
        async move {
            futures_timer::Delay::new(MENU_AIM_DELAY).await;
            Message::ResolveMenuIntent { intent, generation }
        },
        std::convert::identity,
    )
}

fn resolve_menu_intent(state: &mut Camlet, intent: MenuIntent, generation: u64) -> Task<Message> {
    if generation != state.menu_intent_generation
        || state.deferred_menu_parent != Some(menu_intent_parent(intent))
    {
        return Task::none();
    }
    state.deferred_menu_parent = None;
    match intent {
        MenuIntent::Open {
            parent,
            page,
            anchor_y,
        } => open_submenu_immediately(state, parent, page, anchor_y),
        MenuIntent::Close { parent } => close_submenus_immediately(state, parent),
    }
}

const fn menu_intent_parent(intent: MenuIntent) -> window::Id {
    match intent {
        MenuIntent::Open { parent, .. } | MenuIntent::Close { parent } => parent,
    }
}

fn track_menu_cursor(
    state: &mut Camlet,
    window_id: window::Id,
    local_position: Point,
) -> Task<Message> {
    let Some(window_position) = menu_window_position_by_id(state, window_id) else {
        return Task::none();
    };
    let current = Point::new(
        window_position.x + local_position.x,
        window_position.y + local_position.y,
    );
    let window_changed = state
        .menu_pointer
        .is_some_and(|pointer| pointer.window_id != window_id);
    let previous = state
        .menu_pointer
        .filter(|pointer| pointer.window_id == window_id)
        .map_or(current, |pointer| pointer.current);
    state.menu_pointer = Some(MenuPointer {
        window_id,
        previous,
        current,
    });

    if window_changed {
        cancel_submenu_hover(state);
    }

    if state
        .deferred_menu_parent
        .is_some_and(|parent| parent != window_id)
    {
        cancel_menu_intent(state);
    }
    Task::none()
}

fn menu_window_position_by_id(state: &Camlet, id: window::Id) -> Option<Point> {
    if state.settings_window_id == Some(id) {
        Some(state.menu_root_position)
    } else {
        state
            .submenu_windows
            .iter()
            .find(|popup| popup.id == id)
            .map(|popup| popup.position)
    }
}

fn cursor_is_aiming_at_child(state: &Camlet, parent: window::Id) -> bool {
    let Some((level, parent_position, parent_size)) = menu_parent_geometry(state, parent) else {
        return false;
    };
    let Some(child) = state.submenu_windows.get(level) else {
        return false;
    };
    let Some(pointer) = state
        .menu_pointer
        .filter(|pointer| pointer.window_id == parent)
    else {
        return false;
    };

    let opens_right = child.position.x >= parent_position.x + parent_size.width;
    let horizontal_motion = pointer.current.x - pointer.previous.x;
    if (opens_right && horizontal_motion <= 0.25) || (!opens_right && horizontal_motion >= -0.25) {
        return false;
    }

    let near_x = if opens_right {
        child.position.x
    } else {
        child.position.x + child.size.width
    };
    let top = Point::new(near_x, child.position.y - MENU_AIM_VERTICAL_TOLERANCE);
    let bottom = Point::new(
        near_x,
        child.position.y + child.size.height + MENU_AIM_VERTICAL_TOLERANCE,
    );
    point_in_triangle(pointer.current, child.aim_origin, top, bottom)
}

fn point_in_triangle(point: Point, first: Point, second: Point, third: Point) -> bool {
    let first_sign = triangle_sign(point, first, second);
    let second_sign = triangle_sign(point, second, third);
    let third_sign = triangle_sign(point, third, first);
    let has_negative = first_sign < 0.0 || second_sign < 0.0 || third_sign < 0.0;
    let has_positive = first_sign > 0.0 || second_sign > 0.0 || third_sign > 0.0;
    !(has_negative && has_positive)
}

fn triangle_sign(point: Point, first: Point, second: Point) -> f32 {
    (point.x - second.x).mul_add(
        first.y - second.y,
        -(first.x - second.x) * (point.y - second.y),
    )
}

const fn cancel_menu_intent(state: &mut Camlet) {
    state.menu_intent_generation = state.menu_intent_generation.saturating_add(1);
    state.deferred_menu_parent = None;
}

fn handle_window_opened(state: &Camlet, id: window::Id) -> Task<Message> {
    if state.settings_window_id == Some(id)
        || state.submenu_windows.iter().any(|popup| popup.id == id)
    {
        window::gain_focus(id)
    } else if state.window_id == Some(id) {
        Task::none()
    } else {
        // A popup can finish opening after a rapid hover already closed its branch.
        // Never leave that now-untracked native window alive.
        window::close(id)
    }
}

fn record_monitor_size(state: &mut Camlet, size: Option<Size>) -> Task<Message> {
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

fn handle_preview_pressed(state: &mut Camlet) -> Task<Message> {
    if state.settings_window_id.is_some() {
        return close_settings_window(state);
    }
    state.window_id.map_or_else(Task::none, |id| {
        if state.product.resize_mode {
            window::drag_resize(id, window::Direction::SouthEast)
        } else {
            window::drag(id)
        }
    })
}

fn handle_key_pressed(
    state: &mut Camlet,
    window_id: window::Id,
    key: &Key,
    physical: Physical,
    modifiers: Modifiers,
) -> Task<Message> {
    if *key == Key::Named(Named::Escape) {
        if state.settings_window_id.is_some() {
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
    state.focused_menu_window = None;
    state.menu_focus_generation = state.menu_focus_generation.saturating_add(1);
    cancel_menu_intent(state);
    cancel_submenu_hover(state);
    state.menu_pointer = None;
    let (position, monitor) = menu_window_geometry(state, MENU_WINDOW_WIDTH, MENU_WINDOW_HEIGHT);
    state.menu_root_position = position;
    state.menu_monitor_bounds = Some(monitor);
    let settings =
        popup_window_settings(Size::new(MENU_WINDOW_WIDTH, MENU_WINDOW_HEIGHT), position);
    let mut tasks = state
        .submenu_windows
        .drain(..)
        .map(|popup| window::close(popup.id))
        .collect::<Vec<_>>();
    if let Some(id) = state.settings_window_id.take() {
        tasks.push(window::close(id));
    }
    let (id, open_task) = window::open(settings);
    state.settings_window_id = Some(id);
    tasks.push(open_task.map(Message::WindowOpened));
    Task::batch(tasks)
}

fn open_submenu_window(
    state: &mut Camlet,
    parent: window::Id,
    page: MenuPage,
    anchor_y: f32,
) -> Task<Message> {
    let Some((level, parent_position, parent_size)) = menu_parent_geometry(state, parent) else {
        return Task::none();
    };
    let mut tasks = Vec::new();
    if state
        .submenu_windows
        .get(level)
        .is_some_and(|popup| popup.page == page)
    {
        tasks.extend(close_submenus_from_level(state, level + 1));
        return Task::batch(tasks);
    }

    let size = Size::new(MENU_WINDOW_WIDTH, menu_page_height(state, page));
    let monitor = state.menu_monitor_bounds.unwrap_or_else(|| {
        Rectangle::new(
            Point::ORIGIN,
            state.monitor_size.unwrap_or(Size::new(1_920.0, 1_080.0)),
        )
    });
    let position = place_submenu(parent_position, parent_size, monitor, size, anchor_y);
    let aim_origin = state
        .menu_pointer
        .filter(|pointer| pointer.window_id == parent)
        .map_or_else(
            || {
                Point::new(
                    parent_position.x + parent_size.width / 2.0,
                    parent_position.y + anchor_y + MENU_ROW_HEIGHT / 2.0,
                )
            },
            |pointer| pointer.current,
        );

    if let Some(popup) = state.submenu_windows.get(level).copied() {
        // A level owns one native popup for the lifetime of that menu branch. Swapping
        // its contents avoids an open/close race when the pointer crosses rows quickly.
        tasks.extend(close_submenus_from_level(state, level + 1));
        state.submenu_windows[level] = MenuPopup {
            id: popup.id,
            page,
            position,
            size,
            aim_origin,
        };
        tasks.push(window::resize(popup.id, size));
        tasks.push(window::move_to(popup.id, position));
        return Task::batch(tasks);
    }

    tasks.extend(close_submenus_from_level(state, level));
    let (id, open_task) = window::open(popup_window_settings(size, position));
    state.submenu_windows.push(MenuPopup {
        id,
        page,
        position,
        size,
        aim_origin,
    });
    tasks.push(open_task.map(Message::WindowOpened));
    Task::batch(tasks)
}

fn schedule_submenu_hover(
    state: &mut Camlet,
    parent: window::Id,
    page: MenuPage,
    anchor_y: f32,
) -> Task<Message> {
    state.submenu_hover_generation = state.submenu_hover_generation.saturating_add(1);
    let generation = state.submenu_hover_generation;
    Task::perform(
        async move {
            futures_timer::Delay::new(SUBMENU_HOVER_DELAY).await;
            Message::OpenHoveredSubmenu {
                parent,
                page,
                anchor_y,
                generation,
            }
        },
        std::convert::identity,
    )
}

const fn cancel_submenu_hover(state: &mut Camlet) {
    state.submenu_hover_generation = state.submenu_hover_generation.saturating_add(1);
}

fn menu_parent_geometry(state: &Camlet, parent: window::Id) -> Option<(usize, Point, Size)> {
    if state.settings_window_id == Some(parent) {
        return Some((
            0,
            state.menu_root_position,
            Size::new(MENU_WINDOW_WIDTH, MENU_WINDOW_HEIGHT),
        ));
    }
    state
        .submenu_windows
        .iter()
        .position(|popup| popup.id == parent)
        .map(|index| {
            let popup = state.submenu_windows[index];
            (index + 1, popup.position, popup.size)
        })
}

fn close_submenus_after(state: &mut Camlet, parent: window::Id) -> Task<Message> {
    let Some((level, _, _)) = menu_parent_geometry(state, parent) else {
        return Task::none();
    };
    Task::batch(close_submenus_from_level(state, level))
}

fn close_submenus_from_level(state: &mut Camlet, level: usize) -> Vec<Task<Message>> {
    let level = level.min(state.submenu_windows.len());
    state
        .submenu_windows
        .drain(level..)
        .map(|popup| window::close(popup.id))
        .collect()
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
        icon: app_icon(),
        exit_on_close_request: false,
        platform_specific: popup_platform_settings(),
        ..window::Settings::default()
    }
}

fn app_icon() -> Option<window::Icon> {
    static ICON: OnceLock<Option<window::Icon>> = OnceLock::new();
    ICON.get_or_init(load_app_icon).clone()
}

fn load_app_icon() -> Option<window::Icon> {
    let decoder = png::Decoder::new(std::io::Cursor::new(include_bytes!(
        "../../../assets/icons/128x128.png"
    )));
    let mut reader = decoder.read_info().ok()?;
    let mut rgba = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut rgba).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    rgba.truncate(info.buffer_size());
    window::icon::from_rgba(rgba, info.width, info.height).ok()
}

#[cfg(target_os = "linux")]
fn popup_platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific {
        application_id: "camlet-menu".to_owned(),
        override_redirect: false,
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
    state.focused_menu_window = None;
    state.menu_focus_generation = state.menu_focus_generation.saturating_add(1);
    cancel_menu_intent(state);
    cancel_submenu_hover(state);
    state.menu_pointer = None;
    let mut tasks = close_submenus_from_level(state, 0);
    if let Some(id) = state.settings_window_id.take() {
        tasks.push(window::close(id));
    }
    Task::batch(tasks)
}

#[cfg(test)]
fn menu_window_position(state: &Camlet, width: f32, height: f32) -> Point {
    menu_window_geometry(state, width, height).0
}

fn menu_window_geometry(state: &Camlet, width: f32, height: f32) -> (Point, Rectangle) {
    #[cfg(target_os = "linux")]
    if !cfg!(test)
        && display_protocol() == "x11"
        && let Some((pointer, monitor)) = x11_pointer_and_monitor(state.scale_factor)
    {
        return (
            place_popup_at_pointer(pointer, monitor, width, height),
            monitor,
        );
    }

    let overlay = state.product.settings.window;
    let monitor = Rectangle::new(
        Point::ORIGIN,
        state.monitor_size.unwrap_or(Size::new(1_920.0, 1_080.0)),
    );
    let position = Point::new(
        overlay.x.to_f32().unwrap_or(0.0) + state.preview_cursor.x,
        overlay.y.to_f32().unwrap_or(0.0) + state.preview_cursor.y,
    );
    (position, monitor)
}

fn place_popup_at_pointer(pointer: Point, monitor: Rectangle, width: f32, height: f32) -> Point {
    let gap = 4.0;
    let x = if pointer.x + gap + width <= monitor.x + monitor.width {
        pointer.x + gap
    } else {
        (pointer.x - width - gap).max(monitor.x)
    };
    let y = if pointer.y + gap + height <= monitor.y + monitor.height {
        pointer.y + gap
    } else {
        (pointer.y - height - gap).max(monitor.y)
    };
    Point::new(x, y)
}

fn place_submenu(
    parent_position: Point,
    parent_size: Size,
    monitor: Rectangle,
    submenu_size: Size,
    anchor_y: f32,
) -> Point {
    let gap = 4.0;
    let right = parent_position.x + parent_size.width + gap;
    let x = if right + submenu_size.width <= monitor.x + monitor.width {
        right
    } else {
        (parent_position.x - submenu_size.width - gap).max(monitor.x)
    };
    let y = (parent_position.y + anchor_y).clamp(
        monitor.y,
        (monitor.y + monitor.height - submenu_size.height).max(monitor.y),
    );
    Point::new(x, y)
}

fn keep_window_on_monitor(position: Point, monitor: Rectangle, size: Size) -> Point {
    Point::new(
        position.x.clamp(
            monitor.x,
            (monitor.x + monitor.width - size.width).max(monitor.x),
        ),
        position.y.clamp(
            monitor.y,
            (monitor.y + monitor.height - size.height).max(monitor.y),
        ),
    )
}

#[cfg(target_os = "linux")]
fn x11_pointer_and_monitor(scale_factor: f32) -> Option<(Point, Rectangle)> {
    use x11rb::connection::Connection;
    use x11rb::protocol::randr::ConnectionExt as _;
    use x11rb::protocol::xproto::ConnectionExt;

    let (connection, screen_index) = x11rb::connect(None).ok()?;
    let screen = connection.setup().roots.get(screen_index)?;
    let pointer = connection.query_pointer(screen.root).ok()?.reply().ok()?;
    let scale = scale_factor.max(0.5);
    let monitor = connection
        .randr_get_monitors(screen.root, true)
        .ok()
        .and_then(|cookie| cookie.reply().ok())
        .and_then(|reply| {
            reply.monitors.into_iter().find(|monitor| {
                pointer.root_x >= monitor.x
                    && pointer.root_y >= monitor.y
                    && i32::from(pointer.root_x) < i32::from(monitor.x) + i32::from(monitor.width)
                    && i32::from(pointer.root_y) < i32::from(monitor.y) + i32::from(monitor.height)
            })
        });
    let bounds = monitor.map_or_else(
        || {
            Rectangle::new(
                Point::ORIGIN,
                Size::new(
                    f32::from(screen.width_in_pixels) / scale,
                    f32::from(screen.height_in_pixels) / scale,
                ),
            )
        },
        |monitor| {
            Rectangle::new(
                Point::new(f32::from(monitor.x) / scale, f32::from(monitor.y) / scale),
                Size::new(
                    f32::from(monitor.width) / scale,
                    f32::from(monitor.height) / scale,
                ),
            )
        },
    );
    Some((
        Point::new(
            f32::from(pointer.root_x) / scale,
            f32::from(pointer.root_y) / scale,
        ),
        bounds,
    ))
}

fn menu_page_height(state: &Camlet, page: MenuPage) -> f32 {
    let (rows, separators) = match page {
        MenuPage::Theme | MenuPage::Shape | MenuPage::RingThickness | MenuPage::CornerRoundness => {
            (6, 0)
        }
        MenuPage::Language => (3, 0),
        MenuPage::Camera => (state.product.cameras.len().max(1), 0),
        MenuPage::Advanced => (7, 1),
        MenuPage::CameraFps | MenuPage::CameraResolution => (4, 0),
        MenuPage::Fit => (2, 0),
    };
    let rows = f32::from(u16::try_from(rows).unwrap_or(u16::MAX));
    let separators = f32::from(u16::try_from(separators).unwrap_or(u16::MAX));
    rows.mul_add(
        MENU_ROW_HEIGHT,
        separators.mul_add(MENU_SEPARATOR_HEIGHT, MENU_PADDING * 2.0),
    )
}

fn handle_window_event(state: &mut Camlet, id: window::Id, event: &window::Event) -> Task<Message> {
    let is_menu_window = state.settings_window_id == Some(id)
        || state.submenu_windows.iter().any(|popup| popup.id == id);
    if is_menu_window {
        return match event {
            window::Event::Focused => {
                state.focused_menu_window = Some(id);
                state.menu_focus_generation = state.menu_focus_generation.saturating_add(1);
                Task::none()
            }
            window::Event::Unfocused => {
                cancel_menu_intent(state);
                cancel_submenu_hover(state);
                if state.focused_menu_window == Some(id) {
                    state.focused_menu_window = None;
                }
                state.menu_focus_generation = state.menu_focus_generation.saturating_add(1);
                let generation = state.menu_focus_generation;
                Task::perform(
                    async move {
                        futures_timer::Delay::new(Duration::from_millis(90)).await;
                        generation
                    },
                    Message::DismissMenuIfUnfocused,
                )
            }
            window::Event::CloseRequested | window::Event::Closed => {
                if state.settings_window_id == Some(id) {
                    if matches!(event, window::Event::CloseRequested) {
                        close_settings_window(state)
                    } else {
                        state.settings_window_id = None;
                        state.panel = Panel::Preview;
                        Task::batch(close_submenus_from_level(state, 0))
                    }
                } else {
                    let index = state
                        .submenu_windows
                        .iter()
                        .position(|popup| popup.id == id)
                        .unwrap_or(state.submenu_windows.len());
                    let tasks = if matches!(event, window::Event::CloseRequested) {
                        close_submenus_from_level(state, index)
                    } else if index < state.submenu_windows.len() {
                        state.submenu_windows.remove(index);
                        close_submenus_from_level(state, index)
                    } else {
                        Vec::new()
                    };
                    Task::batch(tasks)
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
            let startup_task = if state.panel == Panel::StartupError {
                if state.automation == AutomationMode::ExitAfterLaunch {
                    Task::done(Message::ContinueWithDefaults)
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            };
            Task::batch([monitor_task, startup_task])
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
        while let Some(event) = events.recv().await {
            if output.send(CameraPollResult::Event(event)).await.is_err() {
                return;
            }
        }
        let _ = output.send(CameraPollResult::Disconnected).await;
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
    let active_resolution = state
        .source_frame
        .as_ref()
        .map(|frame| format!("{} × {}", frame.width, frame.height));
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
            "requestedResolution": state.product.settings.camera_resolution.label(),
            "activeResolution": active_resolution,
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
        ) => Task::none(),
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
                futures_timer::Delay::new(Duration::from_millis(100)).await;
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
                futures_timer::Delay::new(Duration::from_millis(100)).await;
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
        AutomationAction::WaitForPreview => wait_for_automation_preview(state),
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
        AutomationAction::SetCameraResolution(value) => {
            automation_product_action(state, Action::SetCameraResolution(value))
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
            state.settings_window_id.map_or_else(Task::none, |parent| {
                open_submenu_window(
                    state,
                    parent,
                    MenuPage::Advanced,
                    root_menu_anchor(MenuPage::Advanced),
                )
            }),
            delayed_automation_step(Duration::from_millis(250)),
        ]),
        AutomationAction::OpenAbout => Task::batch([
            apply_product_action(state, Action::OpenAbout),
            delayed_automation_step(Duration::from_millis(250)),
        ]),
        AutomationAction::OpenResize => Task::batch([
            apply_product_action(state, Action::SetResizeMode(true)),
            delayed_automation_step(Duration::from_millis(250)),
        ]),
        AutomationAction::Screenshot(filename) => automation_screenshot(state, &filename),
        AutomationAction::WindowScreenshot(filename) => {
            automation_window_screenshot(state, &filename)
        }
        AutomationAction::MenuScreenshot(filename) => automation_menu_screenshot(state, &filename),
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

fn automation_screenshot(state: &mut Camlet, filename: &str) -> Task<Message> {
    let Some(session) = state.automation_session.as_ref() else {
        return fail_automation(state);
    };
    let path = session.output_path(filename);
    automation_delay(Duration::from_millis(100), Message::AutomationCapture(path))
}

fn automation_menu_screenshot(state: &mut Camlet, filename: &str) -> Task<Message> {
    let Some(session) = state.automation_session.as_ref() else {
        return fail_automation(state);
    };
    let Some(id) = state
        .submenu_windows
        .last()
        .map(|popup| popup.id)
        .or(state.settings_window_id)
    else {
        return fail_automation(state);
    };
    let path = session.output_path(filename);
    window::screenshot(id)
        .map(move |screenshot| Message::AutomationMenuScreenshotReady(path.clone(), screenshot))
}

fn automation_window_screenshot(state: &mut Camlet, filename: &str) -> Task<Message> {
    let Some(session) = state.automation_session.as_ref() else {
        return fail_automation(state);
    };
    let Some(id) = state.window_id else {
        return fail_automation(state);
    };
    let path = session.output_path(filename);
    window::screenshot(id)
        .map(move |screenshot| Message::AutomationMenuScreenshotReady(path.clone(), screenshot))
}

fn wait_for_automation_preview(state: &mut Camlet) -> Task<Message> {
    if state.product.camera_status == CameraStatus::Preview {
        Task::done(Message::AutomationStep)
    } else {
        if let Some(session) = state.automation_session.as_mut() {
            session.retry(AutomationAction::WaitForPreview);
        }
        delayed_automation_step(Duration::from_millis(50))
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
            futures_timer::Delay::new(duration).await;
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

fn finish_automation_menu_screenshot(
    state: &mut Camlet,
    path: &std::path::Path,
    screenshot: &window::Screenshot,
) -> Task<Message> {
    if write_white_background_ppm(path, screenshot).is_ok() {
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
            state.pending_window_size = Some(window_state.width);
            window::resize(
                id,
                Size::new(
                    f32::from(window_state.width),
                    f32::from(window_state.height),
                ),
            )
        }),
        Effect::EnumerateCameras => send_camera_command(state, CameraWorkerCommand::Enumerate),
        Effect::StartCamera(device_id) => {
            // Keep the last valid frame visible while the same device changes mode. Physical UVC
            // cameras can take hundreds of milliseconds to deliver their next frame; replacing it
            // with a loading panel makes a fast native reconfiguration feel much slower.
            state.preview_error = None;
            state.capture_first_frame_at = None;
            state.capture_frame_count = 0;
            let (width, height) = state.product.settings.camera_resolution.dimensions();
            send_camera_command(
                state,
                CameraWorkerCommand::Start {
                    device_id: Some(device_id.clone()),
                    request: CaptureRequest {
                        width,
                        height,
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
            let submenu_closes = close_submenus_from_level(state, 0);
            let about_size = Size::new(ABOUT_WINDOW_WIDTH, ABOUT_WINDOW_HEIGHT);
            let monitor = state.menu_monitor_bounds.unwrap_or_else(|| {
                Rectangle::new(
                    Point::ORIGIN,
                    state.monitor_size.unwrap_or(Size::new(1_920.0, 1_080.0)),
                )
            });
            let position = keep_window_on_monitor(state.menu_root_position, monitor, about_size);
            state.menu_root_position = position;
            let window_tasks = state.settings_window_id.map_or_else(Vec::new, |id| {
                vec![
                    window::resize(id, about_size),
                    window::move_to(id, position),
                ]
            });
            Task::batch(submenu_closes.into_iter().chain(window_tasks))
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
            futures_timer::Delay::new(PERSISTENCE_DEBOUNCE).await;
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
    if state.settings_window_id == Some(id) {
        return match state.panel {
            Panel::About => about_view(state),
            _ => root_menu_view(state, id),
        };
    }
    if let Some(popup) = state.submenu_windows.iter().find(|popup| popup.id == id) {
        return submenu_view(state, popup.page, id);
    }
    if state.window_id == Some(id) {
        main_view(state)
    } else {
        // Native popup creation is asynchronous. A popup that was superseded before
        // WindowOpened arrives must stay transparent instead of inheriting the camera.
        container(space::horizontal())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
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

fn root_menu_view(state: &Camlet, id: window::Id) -> Element<'_, Message> {
    let catalog = catalog(
        state.product.settings.language,
        state.system_locale.as_deref(),
    );
    let content = column![
        submenu_button(
            state,
            catalog.theme,
            id,
            MenuPage::Theme,
            root_menu_anchor(MenuPage::Theme)
        ),
        submenu_button(
            state,
            catalog.shape,
            id,
            MenuPage::Shape,
            root_menu_anchor(MenuPage::Shape)
        ),
        submenu_button(
            state,
            catalog.language,
            id,
            MenuPage::Language,
            root_menu_anchor(MenuPage::Language)
        ),
        submenu_button(
            state,
            catalog.camera_device,
            id,
            MenuPage::Camera,
            root_menu_anchor(MenuPage::Camera)
        ),
        menu_separator(),
        menu_action_button(catalog.resize, Action::SetResizeMode(true), id),
        submenu_button(
            state,
            catalog.advanced,
            id,
            MenuPage::Advanced,
            root_menu_anchor(MenuPage::Advanced)
        ),
        panel_menu_button(catalog.about, Action::OpenAbout, id),
        menu_separator(),
        destructive_menu_button(catalog.close_app, Action::Quit, id),
    ]
    .spacing(0);

    tracked_menu_surface(content, id)
}

fn submenu_button(
    state: &Camlet,
    label: &str,
    parent: window::Id,
    page: MenuPage,
    anchor_y: f32,
) -> Element<'static, Message> {
    let active = active_child_page(state, parent) == Some(page);
    mouse_area(
        button(
            row![
                menu_row_label(label),
                space::horizontal(),
                submenu_arrow_icon(),
            ]
            .height(Length::Fill)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(MENU_ROW_HEIGHT)
        .padding([0, 10])
        .style(if active {
            active_submenu_row_style
        } else {
            menu_row_style
        })
        .on_press(Message::OpenSubmenu {
            parent,
            page,
            anchor_y,
        }),
    )
    .on_enter(Message::HoverSubmenu {
        parent,
        page,
        anchor_y,
    })
    .interaction(mouse::Interaction::Pointer)
    .into()
}

fn menu_row_anchor(row: u8, separators_before: u8) -> f32 {
    f32::from(separators_before).mul_add(
        MENU_SEPARATOR_HEIGHT,
        f32::from(row).mul_add(MENU_ROW_HEIGHT, MENU_PADDING),
    )
}

fn root_menu_anchor(page: MenuPage) -> f32 {
    match page {
        MenuPage::Shape => menu_row_anchor(1, 0),
        MenuPage::Language => menu_row_anchor(2, 0),
        MenuPage::Camera => menu_row_anchor(3, 0),
        MenuPage::Advanced => menu_row_anchor(5, 1),
        MenuPage::Theme
        | MenuPage::CameraFps
        | MenuPage::CameraResolution
        | MenuPage::Fit
        | MenuPage::RingThickness
        | MenuPage::CornerRoundness => menu_row_anchor(0, 0),
    }
}

fn active_child_page(state: &Camlet, parent: window::Id) -> Option<MenuPage> {
    let (level, _, _) = menu_parent_geometry(state, parent)?;
    state.submenu_windows.get(level).map(|popup| popup.page)
}

fn menu_label(label: &str) -> iced::widget::Text<'static> {
    text(label.to_owned()).size(13)
}

fn menu_row_label(label: &str) -> Element<'static, Message> {
    container(menu_label(label))
        .height(Length::Fill)
        .align_y(iced::Alignment::Center)
        .into()
}

fn menu_row_icon(icon: &str, color: Color) -> Element<'static, Message> {
    container(text(icon.to_owned()).size(15).color(color))
        .width(16)
        .height(Length::Fill)
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center)
        .into()
}

fn submenu_arrow_icon() -> Element<'static, Message> {
    container(
        container(text("›").size(15).color(Color::from_rgb8(151, 163, 181)))
            // The glyph's typographic baseline sits optically low even when its line box
            // is mathematically centered. Extra bottom space raises only the chevron.
            .padding(iced::padding::bottom(2)),
    )
    .width(16)
    .height(Length::Fill)
    .align_x(iced::Alignment::Center)
    .align_y(iced::Alignment::Center)
    .into()
}

fn menu_separator() -> Element<'static, Message> {
    container(rule::horizontal(1))
        .height(MENU_SEPARATOR_HEIGHT)
        .center_y(Length::Fill)
        .into()
}

fn menu_action_button(
    label: &str,
    action: Action,
    parent: window::Id,
) -> Element<'static, Message> {
    mouse_area(
        button(menu_row_label(label))
            .width(Length::Fill)
            .height(MENU_ROW_HEIGHT)
            .padding([0, 10])
            .style(menu_row_style)
            .on_press(Message::MenuProduct(action)),
    )
    .on_enter(Message::CloseSubmenusAfter(parent))
    .interaction(mouse::Interaction::Pointer)
    .into()
}

fn destructive_menu_button(
    label: &str,
    action: Action,
    parent: window::Id,
) -> Element<'static, Message> {
    mouse_area(
        button(menu_row_label(label))
            .width(Length::Fill)
            .height(MENU_ROW_HEIGHT)
            .padding([0, 10])
            .style(destructive_menu_row_style)
            .on_press(Message::MenuProduct(action)),
    )
    .on_enter(Message::CloseSubmenusAfter(parent))
    .interaction(mouse::Interaction::Pointer)
    .into()
}

fn panel_menu_button(label: &str, action: Action, parent: window::Id) -> Element<'static, Message> {
    mouse_area(
        button(menu_row_label(label))
            .width(Length::Fill)
            .height(MENU_ROW_HEIGHT)
            .padding([0, 10])
            .style(menu_row_style)
            .on_press(Message::Product(action)),
    )
    .on_enter(Message::CloseSubmenusAfter(parent))
    .interaction(mouse::Interaction::Pointer)
    .into()
}

fn submenu_view(state: &Camlet, page: MenuPage, id: window::Id) -> Element<'_, Message> {
    let catalog = catalog(
        state.product.settings.language,
        state.system_locale.as_deref(),
    );
    let model = MenuModel::from_state(&state.product);
    let content = match page {
        MenuPage::Theme => {
            let mut content = column![].spacing(0);
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
            let mut content = column![].spacing(0);
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
            let mut content = column![].spacing(0);
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
            let mut content = column![].spacing(0);
            if model.cameras.is_empty() {
                content = content.push(
                    container(text(catalog.no_devices).size(12))
                        .height(MENU_ROW_HEIGHT)
                        .padding([5, 10]),
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
        MenuPage::Advanced => advanced_menu_view(state, catalog, id),
        MenuPage::CameraFps => camera_fps_menu(&model),
        MenuPage::CameraResolution => camera_resolution_menu(&model),
        MenuPage::Fit => {
            let mut content = column![].spacing(0);
            for (choice, label) in model.fit_modes.iter().zip(catalog.fit_modes) {
                content = content.push(menu_choice_button(
                    label,
                    choice.selected,
                    Action::SetFit(choice.value),
                ));
            }
            content
        }
        MenuPage::RingThickness => {
            let mut content = column![].spacing(0);
            for choice in &model.ring_thicknesses {
                content = content.push(menu_choice_button(
                    &format!("{} px", choice.value),
                    choice.selected,
                    Action::SetRingThickness(choice.value),
                ));
            }
            content
        }
        MenuPage::CornerRoundness => {
            let mut content = column![].spacing(0);
            for choice in &model.corner_roundnesses {
                content = content.push(menu_choice_button(
                    &format!("{} px", choice.value),
                    choice.selected,
                    Action::SetCornerRoundness(choice.value),
                ));
            }
            content
        }
    };

    tracked_menu_surface(content, id)
}

fn camera_fps_menu(model: &MenuModel) -> iced::widget::Column<'static, Message> {
    let mut content = column![].spacing(0);
    for choice in &model.camera_fps {
        content = content.push(menu_choice_button(
            &format!("{} FPS", choice.value),
            choice.selected,
            Action::SetCameraFps(choice.value),
        ));
    }
    content
}

fn camera_resolution_menu(model: &MenuModel) -> iced::widget::Column<'static, Message> {
    let mut content = column![].spacing(0);
    for choice in &model.camera_resolutions {
        content = content.push(menu_choice_button(
            choice.value.label(),
            choice.selected,
            Action::SetCameraResolution(choice.value),
        ));
    }
    content
}

fn advanced_menu_view(
    state: &Camlet,
    catalog: &Catalog,
    id: window::Id,
) -> iced::widget::Column<'static, Message> {
    column![
        submenu_button(
            state,
            catalog.camera_resolution,
            id,
            MenuPage::CameraResolution,
            menu_row_anchor(0, 0)
        ),
        submenu_button(
            state,
            catalog.camera_fps,
            id,
            MenuPage::CameraFps,
            menu_row_anchor(1, 0)
        ),
        submenu_button(
            state,
            catalog.fit_mode,
            id,
            MenuPage::Fit,
            menu_row_anchor(2, 0)
        ),
        submenu_button(
            state,
            catalog.ring_thickness,
            id,
            MenuPage::RingThickness,
            menu_row_anchor(3, 0)
        ),
        submenu_button(
            state,
            catalog.corner_roundness,
            id,
            MenuPage::CornerRoundness,
            menu_row_anchor(4, 0)
        ),
        menu_separator(),
        menu_action_button(catalog.retry_camera, Action::RetryCamera, id),
        menu_action_button(catalog.reset_appearance, Action::ResetAppearance, id),
    ]
    .spacing(0)
}

fn menu_surface<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(MENU_PADDING)
        .style(menu_surface_style)
        .into()
}

fn tracked_menu_surface<'a>(
    content: impl Into<Element<'a, Message>>,
    id: window::Id,
) -> Element<'a, Message> {
    mouse_area(menu_surface(content))
        .on_move(move |position| Message::MenuCursorMoved {
            window_id: id,
            position,
        })
        .into()
}

fn menu_surface_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(Color::from_rgb8(237, 241, 247)),
        background: Some(Background::Color(Color::from_rgba8(22, 25, 31, 0.98))),
        border: Border {
            color: Color::from_rgba8(126, 143, 168, 0.28),
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.38),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 12.0,
        },
        ..container::Style::default()
    }
}

fn menu_row_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Some(Background::Color(Color::from_rgba8(75, 87, 106, 0.58))),
        button::Status::Pressed => Some(Background::Color(Color::from_rgba8(56, 67, 84, 0.78))),
        button::Status::Active | button::Status::Disabled => None,
    };
    button::Style {
        background,
        text_color: Color::from_rgb8(237, 241, 247),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

fn active_submenu_row_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgba8(82, 96, 117, 0.78),
        button::Status::Pressed => Color::from_rgba8(56, 67, 84, 0.86),
        button::Status::Active | button::Status::Disabled => Color::from_rgba8(67, 79, 98, 0.72),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::from_rgb8(242, 245, 250),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

fn selected_menu_row_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgba8(24, 139, 111, 0.72),
        button::Status::Pressed => Color::from_rgba8(18, 105, 84, 0.82),
        button::Status::Active | button::Status::Disabled => Color::from_rgba8(22, 121, 97, 0.52),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::from_rgb8(238, 255, 250),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

fn destructive_menu_row_style(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = menu_row_style(theme, status);
    style.text_color = if status == button::Status::Hovered {
        Color::from_rgb8(255, 211, 211)
    } else {
        Color::from_rgb8(240, 191, 191)
    };
    style
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
    let facts = container(
        column![
            row![
                about_fact(catalog.version, env!("CARGO_PKG_VERSION")),
                about_fact(catalog.release_channel, channel),
            ]
            .spacing(8),
            row![
                about_fact(
                    catalog.platform,
                    format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH)
                ),
                about_fact(catalog.display_protocol, display_protocol()),
            ]
            .spacing(8),
            row![
                about_fact(catalog.runtime, "Rust + Iced 0.14"),
                about_fact(catalog.license, "GPL-3.0-only"),
            ]
            .spacing(8),
        ]
        .spacing(6),
    )
    .width(Length::Fill)
    .padding(8)
    .style(about_card_style);
    let content = column![
        row![
            text(catalog.about_window).size(19),
            space::horizontal(),
            button(
                container(text("×").size(18))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center)
            )
            .width(32)
            .height(32)
            .padding(0)
            .style(about_close_button_style)
            .on_press(Message::ClosePanel)
        ]
        .height(32)
        .align_y(iced::Alignment::Center),
        container(text(catalog.about_description).size(12).line_height(1.35))
            .width(Length::Fill)
            .padding([8, 10])
            .style(about_card_style),
        facts,
        row![
            about_link_button("rayan6ms", AUTHOR_URL),
            about_link_button("GitHub", PROJECT_URL),
            about_link_button("Issues", ISSUES_URL),
        ]
        .spacing(7),
        button(
            container(text(copy_label).size(12))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center)
        )
        .width(Length::Fill)
        .height(32)
        .padding(0)
        .style(about_diagnostics_button_style)
        .on_press(Message::CopyDiagnostics),
    ]
    .spacing(9)
    .padding(12);
    menu_surface(content)
}

fn about_fact(label: &str, value: impl Into<String>) -> Element<'static, Message> {
    container(
        column![
            text(label.to_owned())
                .size(10)
                .color(Color::from_rgb8(151, 166, 187)),
            text(value.into()).size(12),
        ]
        .spacing(1),
    )
    .width(Length::FillPortion(1))
    .into()
}

fn about_link_button(label: &str, url: &'static str) -> Element<'static, Message> {
    button(
        container(text(label.to_owned()).size(11))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center),
    )
    .width(Length::FillPortion(1))
    .height(32)
    .padding(0)
    .style(about_link_button_style)
    .on_press(Message::OpenLink(url))
    .into()
}

fn about_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba8(255, 255, 255, 0.035))),
        border: Border {
            color: Color::from_rgba8(151, 168, 194, 0.16),
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    }
}

fn about_close_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgba8(80, 94, 115, 0.72),
        button::Status::Pressed => Color::from_rgba8(57, 69, 88, 0.86),
        button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::from_rgb8(225, 232, 242),
        border: Border {
            radius: 8.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

fn about_link_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgba8(75, 91, 116, 0.72),
        button::Status::Pressed => Color::from_rgba8(53, 66, 86, 0.86),
        button::Status::Active | button::Status::Disabled => Color::from_rgba8(55, 67, 86, 0.58),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::from_rgb8(222, 231, 244),
        border: Border {
            color: Color::from_rgba8(151, 168, 194, 0.16),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..button::Style::default()
    }
}

fn about_diagnostics_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = about_link_button_style(theme, status);
    style.background = Some(Background::Color(match status {
        button::Status::Hovered => Color::from_rgba8(37, 167, 133, 0.88),
        button::Status::Pressed => Color::from_rgba8(24, 119, 96, 0.94),
        button::Status::Active | button::Status::Disabled => Color::from_rgba8(28, 139, 111, 0.72),
    }));
    style.text_color = Color::WHITE;
    style
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
    button(
        row![
            menu_row_label(label),
            space::horizontal(),
            menu_row_icon(
                if selected { "✓" } else { "" },
                Color::from_rgb8(132, 236, 205)
            ),
        ]
        .height(Length::Fill)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(MENU_ROW_HEIGHT)
    .padding([0, 10])
    .style(if selected {
        selected_menu_row_style
    } else {
        menu_row_style
    })
    .on_press(Message::MenuProduct(action))
    .into()
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
    let done = button(
        container(text(catalog.resize_done).size(11))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center),
    )
    .width(58)
    .height(30)
    .padding(0)
    .style(resize_control_style)
    .on_press(Message::Product(Action::SetResizeMode(false)));
    let card: Element<'_, Message> = if state.product.settings.window.width >= 260 {
        container(
            row![resize_status(state, catalog, true), done]
                .spacing(14)
                .align_y(iced::Alignment::Center),
        )
        .padding([8, 11])
        .style(resize_card_style)
        .into()
    } else {
        container(
            column![resize_status(state, catalog, false), done]
                .spacing(7)
                .align_x(iced::Alignment::Center),
        )
        .width(146)
        .padding([9, 10])
        .style(resize_card_style)
        .into()
    };
    let resize_handle = mouse_area(
        button(
            container(text("↘").size(17))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center),
        )
        .width(34)
        .height(34)
        .padding(0)
        .style(resize_handle_style)
        .on_press(Message::BeginWindowResize(window::Direction::SouthEast)),
    )
    .interaction(mouse::Interaction::ResizingDiagonallyDown);
    stack![
        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Start)
            .padding(8),
        container(resize_handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::Alignment::End)
            .align_y(iced::Alignment::End)
            .padding(7),
    ]
    .into()
}

fn resize_status(
    state: &Camlet,
    catalog: &Catalog,
    show_hint: bool,
) -> iced::widget::Column<'static, Message> {
    let mut status = column![
        text(catalog.resize).size(13),
        text(format!(
            "{} × {} px",
            state.product.settings.window.width, state.product.settings.window.height
        ))
        .size(11)
        .color(Color::from_rgb8(185, 200, 220)),
    ]
    .spacing(1);
    if show_hint {
        status = status.push(
            text(catalog.resize_hint)
                .size(10)
                .color(Color::from_rgb8(145, 163, 188)),
        );
    }
    status
}

fn resize_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(Color::WHITE),
        background: Some(Background::Color(Color::from_rgba8(12, 16, 22, 0.86))),
        border: Border {
            color: Color::from_rgba8(255, 255, 255, 0.16),
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.32),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..container::Style::default()
    }
}

fn resize_control_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgba8(255, 255, 255, 0.15),
        button::Status::Pressed => Color::from_rgba8(255, 255, 255, 0.09),
        button::Status::Active | button::Status::Disabled => Color::from_rgba8(255, 255, 255, 0.06),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: Border {
            color: Color::from_rgba8(255, 255, 255, 0.13),
            width: 1.0,
            radius: 999.0.into(),
        },
        ..button::Style::default()
    }
}

fn resize_handle_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (background, border) = match status {
        button::Status::Hovered => (
            Color::from_rgba8(24, 119, 96, 0.94),
            Color::from_rgba8(176, 255, 232, 0.72),
        ),
        button::Status::Pressed => (
            Color::from_rgba8(18, 88, 72, 0.98),
            Color::from_rgba8(176, 255, 232, 0.84),
        ),
        button::Status::Active | button::Status::Disabled => (
            Color::from_rgba8(12, 16, 22, 0.84),
            Color::from_rgba8(132, 236, 205, 0.72),
        ),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::from_rgb8(176, 255, 232),
        border: Border {
            color: border,
            width: 1.0,
            radius: 9.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.3),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 5.0,
        },
        ..button::Style::default()
    }
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
    use iced::{Point, Rectangle, Size, event::Event, mouse};
    use iced_test::simulator;
    use num_traits::ToPrimitive;

    use super::{
        Action, AutomationMode, CameraPollResult, Camlet, DiagnosticsState, FrameSourceKind,
        Lifecycle, MENU_WINDOW_HEIGHT, MENU_WINDOW_WIDTH, MenuPage, MenuPopup, Message,
        PROJECT_URL, Panel, ScreenshotState, about_view, app_icon, apply_product_action,
        cursor_is_aiming_at_child, diagnostics_json, handle_camera_poll, handle_window_event,
        handle_window_moved, handle_window_resized, keyboard_action, main_view,
        menu_window_position, open_submenu_window, place_popup_at_pointer, place_submenu,
        popup_window_settings, resize_view, root_menu_anchor, root_menu_view, submenu_view, update,
        window_view,
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
            submenu_windows: Vec::new(),
            menu_root_position: Point::ORIGIN,
            menu_monitor_bounds: None,
            focused_menu_window: None,
            menu_focus_generation: 0,
            submenu_hover_generation: 0,
            menu_pointer: None,
            menu_intent_generation: 0,
            deferred_menu_parent: None,
            preview_cursor: Point::new(112.0, 112.0),
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
    fn resize_panel_starts_native_window_resize_and_can_finish() {
        let state = test_state(false);
        let mut resize_ui = simulator(resize_view(&state));
        resize_ui
            .click("↘")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(resize_ui.into_messages().into_iter().any(|message| {
            matches!(
                message,
                Message::BeginWindowResize(iced::window::Direction::SouthEast)
            )
        }));

        let mut done_ui = simulator(resize_view(&state));
        done_ui
            .click("Done")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            done_ui.into_messages().into_iter().any(|message| {
                matches!(message, Message::Product(Action::SetResizeMode(false)))
            })
        );
    }

    #[test]
    fn iced_menu_exposes_and_dispatches_appearance_actions() {
        let state = test_state(true);
        let mut ui = simulator(submenu_view(
            &state,
            MenuPage::Theme,
            iced::window::Id::unique(),
        ));
        ui.click("Ocean")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(ui.into_messages().into_iter().any(|message| matches!(
            message,
            Message::MenuProduct(Action::SetTheme(ThemeId::Ocean))
        )));
    }

    #[test]
    fn iced_menu_exposes_camera_selection_and_retry_actions() {
        let state = test_state(true);
        let mut camera_ui = simulator(submenu_view(
            &state,
            MenuPage::Camera,
            iced::window::Id::unique(),
        ));
        camera_ui
            .click("Camlet synthetic camera")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(camera_ui.into_messages().into_iter().any(|message| {
            matches!(message, Message::MenuProduct(Action::SetCamera(id)) if id == "synthetic")
        }));

        let mut retry_ui = simulator(submenu_view(
            &state,
            MenuPage::Advanced,
            iced::window::Id::unique(),
        ));
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
        let state = test_state(true);
        let mut ui = simulator(submenu_view(
            &state,
            MenuPage::CameraFps,
            iced::window::Id::unique(),
        ));
        ui.click("60 FPS")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            ui.into_messages().into_iter().any(|message| {
                matches!(message, Message::MenuProduct(Action::SetCameraFps(60)))
            })
        );
    }

    #[test]
    fn resolution_menu_dispatches_a_camera_restart_choice() {
        let state = test_state(true);
        let mut ui = simulator(submenu_view(
            &state,
            MenuPage::CameraResolution,
            iced::window::Id::unique(),
        ));
        ui.click("1280 × 720")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(ui.into_messages().into_iter().any(|message| {
            matches!(
                message,
                Message::MenuProduct(Action::SetCameraResolution(
                    camlet_core::settings::CameraResolution::R1280x720
                ))
            )
        }));
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
    fn camera_mode_change_keeps_the_last_frame_visible_until_replacement() {
        let mut state = test_state(false);
        state.source_frame = Some(Arc::new(VideoFrame {
            width: 2,
            height: 2,
            sequence: 1,
            rgba: vec![64; 16],
        }));

        let _ = apply_product_action(
            &mut state,
            Action::SetCameraResolution(camlet_core::settings::CameraResolution::R1280x720),
        );

        assert!(state.source_frame.is_some());
        assert_eq!(
            state.product.camera_status,
            camlet_core::state::CameraStatus::Loading
        );
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
    fn menu_fallback_uses_the_overlay_pointer_without_wrong_monitor_clamping() {
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
            Point::new(1_780.0, 210.0)
        );
    }

    #[test]
    fn popup_placement_uses_global_pointer_and_monitor_bounds() {
        assert_eq!(
            place_popup_at_pointer(
                Point::new(2_500.0, 400.0),
                Rectangle::new(Point::new(1_920.0, 0.0), Size::new(1_920.0, 1_080.0)),
                MENU_WINDOW_WIDTH,
                MENU_WINDOW_HEIGHT,
            ),
            Point::new(2_504.0, 404.0)
        );
        assert_eq!(
            place_popup_at_pointer(
                Point::new(3_820.0, 1_060.0),
                Rectangle::new(Point::new(1_920.0, 0.0), Size::new(1_920.0, 1_080.0)),
                MENU_WINDOW_WIDTH,
                MENU_WINDOW_HEIGHT,
            ),
            Point::new(3_556.0, 1_056.0 - MENU_WINDOW_HEIGHT)
        );
        assert_eq!(
            place_popup_at_pointer(
                Point::new(1_910.0, 300.0),
                Rectangle::new(Point::ORIGIN, Size::new(1_920.0, 1_080.0)),
                MENU_WINDOW_WIDTH,
                MENU_WINDOW_HEIGHT,
            ),
            Point::new(1_646.0, 304.0)
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
        assert!(!settings.platform_specific.override_redirect);
        #[cfg(target_os = "windows")]
        assert!(settings.platform_specific.skip_taskbar);
    }

    #[test]
    fn submenus_flip_at_monitor_edges_and_stay_vertically_visible() {
        let monitor = Rectangle::new(Point::new(1_920.0, 0.0), Size::new(1_920.0, 1_080.0));
        let submenu = Size::new(260.0, 720.0);

        assert_eq!(
            place_submenu(
                Point::new(2_000.0, 200.0),
                Size::new(260.0, 252.0),
                monitor,
                submenu,
                28.0,
            ),
            Point::new(2_264.0, 228.0)
        );
        assert_eq!(
            place_submenu(
                Point::new(3_560.0, 900.0),
                Size::new(260.0, 252.0),
                monitor,
                submenu,
                154.0,
            ),
            Point::new(3_296.0, 360.0)
        );
    }

    #[test]
    fn opening_a_peer_submenu_replaces_the_existing_branch() {
        let mut state = test_state(true);
        let root = iced::window::Id::unique();
        state.settings_window_id = Some(root);
        state.menu_monitor_bounds =
            Some(Rectangle::new(Point::ORIGIN, Size::new(1_920.0, 1_080.0)));

        let _ = open_submenu_window(
            &mut state,
            root,
            MenuPage::Advanced,
            root_menu_anchor(MenuPage::Advanced),
        );
        assert_eq!(state.submenu_windows.len(), 1);
        assert_eq!(state.submenu_windows[0].page, MenuPage::Advanced);
        let popup_id = state.submenu_windows[0].id;

        let _ = open_submenu_window(
            &mut state,
            root,
            MenuPage::Theme,
            root_menu_anchor(MenuPage::Theme),
        );
        assert_eq!(state.submenu_windows.len(), 1);
        assert_eq!(state.submenu_windows[0].page, MenuPage::Theme);
        assert_eq!(state.submenu_windows[0].id, popup_id);
    }

    #[test]
    fn safety_triangle_tracks_real_direction_and_cancels_after_reaching_the_child() {
        let mut state = test_state(true);
        let root = iced::window::Id::unique();
        state.settings_window_id = Some(root);
        state.menu_monitor_bounds =
            Some(Rectangle::new(Point::ORIGIN, Size::new(1_920.0, 1_080.0)));

        let _ = update(
            &mut state,
            Message::MenuCursorMoved {
                window_id: root,
                position: Point::new(120.0, 168.0),
            },
        );
        let _ = open_submenu_window(
            &mut state,
            root,
            MenuPage::Advanced,
            root_menu_anchor(MenuPage::Advanced),
        );
        let child = state.submenu_windows[0].id;

        let _ = update(
            &mut state,
            Message::MenuCursorMoved {
                window_id: root,
                position: Point::new(180.0, 190.0),
            },
        );
        assert!(cursor_is_aiming_at_child(&state, root));

        let _ = update(
            &mut state,
            Message::HoverSubmenu {
                parent: root,
                page: MenuPage::Theme,
                anchor_y: root_menu_anchor(MenuPage::Theme),
            },
        );
        let deferred_generation = state.menu_intent_generation;
        assert_eq!(state.deferred_menu_parent, Some(root));
        assert_eq!(state.submenu_windows[0].page, MenuPage::Advanced);

        let _ = update(
            &mut state,
            Message::MenuCursorMoved {
                window_id: child,
                position: Point::new(8.0, 20.0),
            },
        );
        assert_eq!(state.deferred_menu_parent, None);
        let _ = update(
            &mut state,
            Message::ResolveMenuIntent {
                intent: super::MenuIntent::Open {
                    parent: root,
                    page: MenuPage::Theme,
                    anchor_y: root_menu_anchor(MenuPage::Theme),
                },
                generation: deferred_generation,
            },
        );
        assert_eq!(state.submenu_windows[0].page, MenuPage::Advanced);

        state.menu_pointer = Some(super::MenuPointer {
            window_id: root,
            previous: Point::new(180.0, 190.0),
            current: Point::new(160.0, 190.0),
        });
        assert!(!cursor_is_aiming_at_child(&state, root));
    }

    #[test]
    fn safety_triangle_supports_submenus_that_open_to_the_left() {
        let mut state = test_state(true);
        let root = iced::window::Id::unique();
        state.settings_window_id = Some(root);
        state.menu_root_position = Point::new(600.0, 100.0);
        state.menu_monitor_bounds = Some(Rectangle::new(Point::ORIGIN, Size::new(900.0, 700.0)));

        let _ = update(
            &mut state,
            Message::MenuCursorMoved {
                window_id: root,
                position: Point::new(100.0, 168.0),
            },
        );
        let _ = open_submenu_window(
            &mut state,
            root,
            MenuPage::Advanced,
            root_menu_anchor(MenuPage::Advanced),
        );
        assert!(state.submenu_windows[0].position.x < state.menu_root_position.x);

        let _ = update(
            &mut state,
            Message::MenuCursorMoved {
                window_id: root,
                position: Point::new(70.0, 180.0),
            },
        );
        assert!(cursor_is_aiming_at_child(&state, root));
    }

    #[test]
    fn embedded_window_icon_is_the_generated_rust_icon() {
        let (rgba, size) = app_icon()
            .unwrap_or_else(|| unreachable!("generated icon must decode"))
            .into_raw();
        assert_eq!(size, Size::new(128, 128));
        assert_eq!(rgba.len(), 128 * 128 * 4);
        assert!(rgba.chunks_exact(4).any(|pixel| pixel[3] > 0));
    }

    #[test]
    fn only_the_latest_rapid_hover_can_open_a_submenu() {
        let mut state = test_state(true);
        let root = iced::window::Id::unique();
        state.settings_window_id = Some(root);
        state.menu_monitor_bounds =
            Some(Rectangle::new(Point::ORIGIN, Size::new(1_920.0, 1_080.0)));

        let _ = update(
            &mut state,
            Message::HoverSubmenu {
                parent: root,
                page: MenuPage::Advanced,
                anchor_y: root_menu_anchor(MenuPage::Advanced),
            },
        );
        let stale_generation = state.submenu_hover_generation;
        let _ = update(
            &mut state,
            Message::HoverSubmenu {
                parent: root,
                page: MenuPage::Theme,
                anchor_y: root_menu_anchor(MenuPage::Theme),
            },
        );
        let current_generation = state.submenu_hover_generation;

        let _ = update(
            &mut state,
            Message::OpenHoveredSubmenu {
                parent: root,
                page: MenuPage::Advanced,
                anchor_y: root_menu_anchor(MenuPage::Advanced),
                generation: stale_generation,
            },
        );
        assert!(state.submenu_windows.is_empty());

        let _ = update(
            &mut state,
            Message::OpenHoveredSubmenu {
                parent: root,
                page: MenuPage::Theme,
                anchor_y: root_menu_anchor(MenuPage::Theme),
                generation: current_generation,
            },
        );
        assert_eq!(state.submenu_windows.len(), 1);
        assert_eq!(state.submenu_windows[0].page, MenuPage::Theme);
    }

    #[test]
    fn an_untracked_native_window_never_falls_back_to_the_camera_view() {
        let mut state = test_state(false);
        let main = iced::window::Id::unique();
        state.window_id = Some(main);
        state.product.camera_status = camlet_core::state::CameraStatus::Error;

        let mut main_ui = simulator(window_view(&state, main));
        assert!(main_ui.click("Retry camera").is_ok());

        let stale_popup = iced::window::Id::unique();
        let mut stale_ui = simulator(window_view(&state, stale_popup));
        assert!(stale_ui.click("Retry camera").is_err());
    }

    #[test]
    fn menu_focus_handoff_survives_but_an_outside_click_dismisses_every_popup() {
        let mut state = test_state(true);
        let root = iced::window::Id::unique();
        let child = iced::window::Id::unique();
        state.settings_window_id = Some(root);
        state.submenu_windows.push(MenuPopup {
            id: child,
            page: MenuPage::Advanced,
            position: Point::new(264.0, 154.0),
            size: Size::new(260.0, 215.0),
            aim_origin: Point::new(240.0, 168.0),
        });
        state.focused_menu_window = Some(root);

        let _ = handle_window_event(&mut state, root, &iced::window::Event::Unfocused);
        let stale_generation = state.menu_focus_generation;
        let _ = handle_window_event(&mut state, child, &iced::window::Event::Focused);
        let _ = update(
            &mut state,
            Message::DismissMenuIfUnfocused(stale_generation),
        );
        assert_eq!(state.settings_window_id, Some(root));
        assert_eq!(state.submenu_windows.len(), 1);

        let _ = handle_window_event(&mut state, child, &iced::window::Event::Unfocused);
        let outside_click_generation = state.menu_focus_generation;
        let _ = update(
            &mut state,
            Message::DismissMenuIfUnfocused(outside_click_generation),
        );
        assert_eq!(state.settings_window_id, None);
        assert!(state.submenu_windows.is_empty());
        assert_eq!(state.panel, Panel::Preview);
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
            .click("GitHub")
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
        let mut about_ui = simulator(root_menu_view(&state, iced::window::Id::unique()));
        about_ui
            .click("About")
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert!(
            about_ui
                .into_messages()
                .into_iter()
                .any(|message| matches!(message, Message::Product(Action::OpenAbout)))
        );

        let mut quit_ui = simulator(root_menu_view(&state, iced::window::Id::unique()));
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
