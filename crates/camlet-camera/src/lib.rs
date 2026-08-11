#![doc = "Camera abstractions for Camlet."]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{
    self, Receiver, RecvTimeoutError, Sender, SyncSender, TryRecvError, TrySendError,
};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use nokhwa::Camera;
use nokhwa::pixel_format::{FormatDecoder, RgbAFormat};
use nokhwa::utils::{
    ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType,
};

const CAMERA_START_ATTEMPTS: u8 = 5;
const CAMERA_START_RETRY_DELAY: Duration = Duration::from_millis(500);
const TRANSIENT_FRAME_ERROR_LIMIT: u8 = 3;

/// A camera that can be shown in the selection menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraDevice {
    /// Backend identifier. It must not be written to diagnostics.
    pub id: String,
    /// User-visible device label.
    pub label: String,
}

/// Capture dimensions and cadence requested by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRequest {
    /// Desired frame width in physical pixels.
    pub width: u32,
    /// Desired frame height in physical pixels.
    pub height: u32,
    /// Desired time between frames.
    pub frame_interval: Duration,
}

/// One tightly packed RGBA frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoFrame {
    /// Frame width in physical pixels.
    pub width: u32,
    /// Frame height in physical pixels.
    pub height: u32,
    /// Monotonic sequence assigned by the frame source.
    pub sequence: u64,
    /// RGBA pixels in row-major order.
    pub rgba: Vec<u8>,
}

/// Stable error categories shared by real and synthetic sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CameraError {
    /// The user or system denied camera access.
    #[error("camera permission was denied")]
    PermissionDenied,
    /// The requested camera is no longer available.
    #[error("camera device was not found")]
    DeviceNotFound,
    /// Another application or backend owns the camera.
    #[error("camera is busy")]
    Busy,
    /// The backend could not produce a usable frame.
    #[error("camera backend failed")]
    Backend,
}

/// Cross-platform boundary between application state and frame capture.
pub trait FrameSource {
    /// Lists devices available to this source.
    ///
    /// # Errors
    ///
    /// Returns a stable category when the backend cannot enumerate devices.
    fn devices(&mut self) -> Result<Vec<CameraDevice>, CameraError>;

    /// Starts or replaces the active capture.
    ///
    /// # Errors
    ///
    /// Returns a stable category when the requested capture cannot start.
    fn start(
        &mut self,
        device_id: Option<&str>,
        request: CaptureRequest,
    ) -> Result<(), CameraError>;

    /// Takes the newest available frame, if one exists.
    ///
    /// # Errors
    ///
    /// Returns a stable category when capture or decoding fails after startup.
    fn latest_frame(&mut self) -> Result<Option<VideoFrame>, CameraError>;

    /// Stops capture and releases associated resources.
    fn stop(&mut self);
}

/// Deterministic, privacy-safe source used by tests, diagnostics, and automation.
#[derive(Debug, Default)]
pub struct SyntheticFrameSource {
    request: Option<CaptureRequest>,
    base_rgba: Vec<u8>,
    latest: Option<VideoFrame>,
    next_sequence: u64,
}

impl SyntheticFrameSource {
    /// Stable non-hardware device identifier.
    pub const DEVICE_ID: &'static str = "synthetic";

    /// Generates one new frame and replaces any unconsumed frame.
    ///
    /// # Errors
    ///
    /// Returns [`CameraError::Backend`] if the source has not started or dimensions overflow.
    pub fn advance(&mut self) -> Result<(), CameraError> {
        let request = self.request.ok_or(CameraError::Backend)?;
        let mut rgba = self.base_rgba.clone();
        animate_synthetic_marker(&mut rgba, request, self.next_sequence)?;
        let frame = VideoFrame {
            width: request.width,
            height: request.height,
            sequence: self.next_sequence,
            rgba,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.latest = Some(frame);
        Ok(())
    }
}

impl FrameSource for SyntheticFrameSource {
    fn devices(&mut self) -> Result<Vec<CameraDevice>, CameraError> {
        Ok(vec![CameraDevice {
            id: Self::DEVICE_ID.to_owned(),
            label: "Camlet synthetic camera".to_owned(),
        }])
    }

    fn start(
        &mut self,
        device_id: Option<&str>,
        request: CaptureRequest,
    ) -> Result<(), CameraError> {
        if device_id.is_some_and(|id| id != Self::DEVICE_ID)
            || request.width == 0
            || request.height == 0
            || request.width > 4_096
            || request.height > 4_096
            || request.frame_interval.is_zero()
        {
            return Err(CameraError::Backend);
        }

        self.base_rgba = generate_synthetic_frame(request, 0)?.rgba;
        self.request = Some(request);
        self.latest = None;
        self.next_sequence = 0;
        self.advance()
    }

    fn latest_frame(&mut self) -> Result<Option<VideoFrame>, CameraError> {
        if self.latest.is_none() && self.request.is_some() {
            self.advance()?;
        }
        Ok(self.latest.take())
    }

    fn stop(&mut self) {
        self.request = None;
        self.base_rgba.clear();
        self.latest = None;
        self.next_sequence = 0;
    }
}

/// Fully deterministic source for lifecycle and failure-path tests.
#[derive(Debug, Default)]
pub struct ScriptedFrameSource {
    devices: Vec<CameraDevice>,
    devices_error: Option<CameraError>,
    start_error: Option<CameraError>,
    latest: Option<VideoFrame>,
    running: bool,
    stop_count: u64,
}

impl ScriptedFrameSource {
    /// Creates a source with the devices returned by enumeration.
    #[must_use]
    pub fn new(devices: Vec<CameraDevice>) -> Self {
        Self {
            devices,
            ..Self::default()
        }
    }

    /// Configures device enumeration to fail until cleared.
    pub const fn set_devices_error(&mut self, error: Option<CameraError>) {
        self.devices_error = error;
    }

    /// Configures the next start attempt to fail.
    pub const fn fail_next_start(&mut self, error: CameraError) {
        self.start_error = Some(error);
    }

    /// Replaces the pending frame, matching the bounded production handoff.
    pub fn push_frame(&mut self, frame: VideoFrame) {
        self.latest = Some(frame);
    }

    /// Returns the number of stop calls observed by the fake.
    #[must_use]
    pub const fn stop_count(&self) -> u64 {
        self.stop_count
    }
}

impl FrameSource for ScriptedFrameSource {
    fn devices(&mut self) -> Result<Vec<CameraDevice>, CameraError> {
        match self.devices_error {
            Some(error) => Err(error),
            None => Ok(self.devices.clone()),
        }
    }

    fn start(
        &mut self,
        device_id: Option<&str>,
        _request: CaptureRequest,
    ) -> Result<(), CameraError> {
        if let Some(error) = self.start_error.take() {
            return Err(error);
        }
        if let Some(device_id) = device_id
            && !self.devices.iter().any(|device| device.id == device_id)
        {
            return Err(CameraError::DeviceNotFound);
        }
        self.running = true;
        Ok(())
    }

    fn latest_frame(&mut self) -> Result<Option<VideoFrame>, CameraError> {
        if self.running {
            Ok(self.latest.take())
        } else {
            Ok(None)
        }
    }

    fn stop(&mut self) {
        self.running = false;
        self.latest = None;
        self.stop_count = self.stop_count.saturating_add(1);
    }
}

/// Cross-platform native camera source backed by Nokhwa.
#[derive(Default)]
pub struct NokhwaFrameSource {
    camera: Option<Camera>,
    active_index: Option<CameraIndex>,
    devices: Vec<CachedCameraDevice>,
    next_sequence: u64,
    consecutive_frame_errors: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CachedCameraDevice {
    public: CameraDevice,
    index: CameraIndex,
}

impl std::fmt::Debug for NokhwaFrameSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NokhwaFrameSource")
            .field("running", &self.camera.is_some())
            .field("has_active_index", &self.active_index.is_some())
            .field("device_count", &self.devices.len())
            .field("next_sequence", &self.next_sequence)
            .field("consecutive_frame_errors", &self.consecutive_frame_errors)
            .finish()
    }
}

impl FrameSource for NokhwaFrameSource {
    fn devices(&mut self) -> Result<Vec<CameraDevice>, CameraError> {
        let started = Instant::now();
        self.refresh_devices()?;
        performance_trace("camera.enumerate", started);
        Ok(self
            .devices
            .iter()
            .map(|device| device.public.clone())
            .collect())
    }

    fn start(
        &mut self,
        device_id: Option<&str>,
        request: CaptureRequest,
    ) -> Result<(), CameraError> {
        let started = Instant::now();
        if request.width == 0 || request.height == 0 || request.frame_interval.is_zero() {
            return Err(CameraError::Backend);
        }
        ensure_nokhwa_initialized()?;
        performance_trace("camera.initialize", started);
        let index = self.resolve_device_index(device_id)?;
        retry_camera_start(
            || self.start_once(index.clone(), request, started),
            || thread::sleep(CAMERA_START_RETRY_DELAY),
        )
    }

    fn latest_frame(&mut self) -> Result<Option<VideoFrame>, CameraError> {
        let Some(camera) = self.camera.as_mut() else {
            return Ok(None);
        };
        let first_frame = self.next_sequence == 0;
        let capture_started = Instant::now();
        let buffer = match camera.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                let error = map_nokhwa_error(&error);
                tolerate_transient_frame_error(&mut self.consecutive_frame_errors, error)?;
                return Ok(None);
            }
        };
        if first_frame {
            performance_trace("camera.first-frame-capture", capture_started);
        }
        let resolution = buffer.resolution();
        let decode_started = Instant::now();
        let image = match buffer.decode_image::<RgbAFormat>() {
            Ok(image) => image,
            Err(error) => {
                let error = map_nokhwa_error(&error);
                tolerate_transient_frame_error(&mut self.consecutive_frame_errors, error)?;
                return Ok(None);
            }
        };
        if first_frame {
            performance_trace("camera.first-frame-decode", decode_started);
        }
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.consecutive_frame_errors = 0;
        Ok(Some(VideoFrame {
            width: resolution.width(),
            height: resolution.height(),
            sequence,
            rgba: image.into_raw(),
        }))
    }

    fn stop(&mut self) {
        self.active_index = None;
        self.next_sequence = 0;
        self.consecutive_frame_errors = 0;
        let Some(mut camera) = self.camera.take() else {
            return;
        };
        if camera.is_stream_open() {
            let _ = camera.stop_stream();
        }
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(camera)));
    }
}

impl NokhwaFrameSource {
    fn start_once(
        &mut self,
        index: CameraIndex,
        request: CaptureRequest,
        overall_started: Instant,
    ) -> Result<(), CameraError> {
        let restarting = self.active_index.as_ref() == Some(&index) && self.camera.is_some();
        let mut camera = if restarting {
            let mut camera = self.camera.take().ok_or(CameraError::Backend)?;
            self.active_index = None;
            self.next_sequence = 0;
            if camera.is_stream_open() {
                camera
                    .stop_stream()
                    .map_err(|error| map_nokhwa_error(&error))?;
            }
            configure_camera(&mut camera, request)?;
            camera
        } else {
            self.stop();
            let open_started = Instant::now();
            let camera = open_configured_camera(index.clone(), request)?;
            performance_trace("camera.open", open_started);
            camera
        };

        let stream_started = Instant::now();
        camera
            .open_stream()
            .map_err(|error| map_nokhwa_error(&error))?;
        performance_trace("camera.stream", stream_started);
        performance_trace(
            if restarting {
                "camera.restart-total"
            } else {
                "camera.start-total"
            },
            overall_started,
        );
        self.camera = Some(camera);
        self.active_index = Some(index);
        self.next_sequence = 0;
        self.consecutive_frame_errors = 0;
        Ok(())
    }
}

const fn tolerate_transient_frame_error(
    consecutive_errors: &mut u8,
    error: CameraError,
) -> Result<(), CameraError> {
    if !matches!(error, CameraError::Busy | CameraError::Backend) {
        return Err(error);
    }
    *consecutive_errors = consecutive_errors.saturating_add(1);
    if *consecutive_errors <= TRANSIENT_FRAME_ERROR_LIMIT {
        Ok(())
    } else {
        Err(error)
    }
}

fn retry_camera_start<T>(
    mut operation: impl FnMut() -> Result<T, CameraError>,
    mut wait: impl FnMut(),
) -> Result<T, CameraError> {
    for attempt in 1..=CAMERA_START_ATTEMPTS {
        match operation() {
            Ok(value) => return Ok(value),
            Err(error)
                if attempt < CAMERA_START_ATTEMPTS
                    && matches!(error, CameraError::Busy | CameraError::Backend) =>
            {
                if std::env::var_os("CAMLET_TRACE_CAMERA").is_some() {
                    eprintln!(
                        "camlet-camera: retrying-start category={} attempt={attempt}/{CAMERA_START_ATTEMPTS}",
                        camera_error_code(error)
                    );
                }
                wait();
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("the bounded camera start loop always returns")
}

fn open_configured_camera(
    index: CameraIndex,
    request: CaptureRequest,
) -> Result<Camera, CameraError> {
    let exact_format = CameraFormat::new_from(
        request.width,
        request.height,
        FrameFormat::YUYV,
        requested_fps(request),
    );
    let exact_request =
        RequestedFormat::new::<RgbAFormat>(RequestedFormatType::Exact(exact_format));
    match Camera::new(index.clone(), exact_request) {
        Ok(camera) => Ok(camera),
        Err(error)
            if matches!(
                map_nokhwa_error(&error),
                CameraError::Backend | CameraError::DeviceNotFound
            ) =>
        {
            let fallback = RequestedFormat::new::<RgbAFormat>(RequestedFormatType::None);
            let mut camera =
                Camera::new(index, fallback).map_err(|error| map_nokhwa_error(&error))?;
            configure_camera(&mut camera, request)?;
            Ok(camera)
        }
        Err(error) => Err(map_nokhwa_error(&error)),
    }
}

fn performance_trace(stage: &str, started: Instant) {
    if std::env::var_os("CAMLET_TRACE_PERFORMANCE").is_some() {
        eprintln!(
            "camlet-performance: {stage}={}us",
            started.elapsed().as_micros()
        );
    }
}

fn configure_camera(camera: &mut Camera, request: CaptureRequest) -> Result<(), CameraError> {
    let formats_started = Instant::now();
    if let Ok(formats) = camera.compatible_camera_formats()
        && let Some(format) = choose_camera_format(&formats, request)
    {
        performance_trace("camera.formats", formats_started);
        if camera.camera_format() != format {
            let configure_started = Instant::now();
            let exact = RequestedFormat::new::<RgbAFormat>(RequestedFormatType::Exact(format));
            camera
                .set_camera_requset(exact)
                .map_err(|error| map_nokhwa_error(&error))?;
            performance_trace("camera.configure", configure_started);
        }
    }
    if std::env::var_os("CAMLET_TRACE_PERFORMANCE").is_some() {
        let format = camera.camera_format();
        eprintln!(
            "camlet-performance: camera.selected-format={}x{}@{} {:?}",
            format.width(),
            format.height(),
            format.frame_rate(),
            format.format()
        );
    }
    Ok(())
}

impl NokhwaFrameSource {
    fn refresh_devices(&mut self) -> Result<(), CameraError> {
        ensure_nokhwa_initialized()?;
        let devices = nokhwa::query(ApiBackend::Auto).map_err(|error| map_nokhwa_error(&error))?;
        self.devices = cache_camera_devices(&devices);
        Ok(())
    }

    fn resolve_device_index(
        &mut self,
        device_id: Option<&str>,
    ) -> Result<CameraIndex, CameraError> {
        if self.devices.is_empty()
            || device_id.is_some_and(|id| !self.devices.iter().any(|device| device.public.id == id))
        {
            self.refresh_devices()?;
        }

        device_id.map_or_else(
            || {
                self.devices
                    .first()
                    .map(|device| device.index.clone())
                    .ok_or(CameraError::DeviceNotFound)
            },
            |id| {
                self.devices
                    .iter()
                    .find(|device| device.public.id == id)
                    .map(|device| device.index.clone())
                    .ok_or(CameraError::DeviceNotFound)
            },
        )
    }
}

impl Drop for NokhwaFrameSource {
    fn drop(&mut self) {
        self.stop();
    }
}

fn ensure_nokhwa_initialized() -> Result<(), CameraError> {
    if nokhwa::nokhwa_check() {
        return Ok(());
    }
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    nokhwa::nokhwa_initialize(move |granted| {
        let _ = sender.send(granted);
    });
    match receiver.recv_timeout(Duration::from_secs(30)) {
        Ok(true) => Ok(()),
        Ok(false) | Err(_) => Err(CameraError::PermissionDenied),
    }
}

fn stable_device_id(device: &nokhwa::utils::CameraInfo) -> String {
    let stable = device.misc();
    if stable.trim().is_empty() {
        device.index().as_string()
    } else {
        stable
    }
}

fn cache_camera_devices(devices: &[nokhwa::utils::CameraInfo]) -> Vec<CachedCameraDevice> {
    let mut devices = devices
        .iter()
        .filter(|device| is_capture_device(device))
        .collect::<Vec<_>>();
    devices.sort_by(|first, second| first.index().cmp(second.index()));

    devices
        .into_iter()
        .enumerate()
        .map(|(position, device)| {
            let label = device.human_name();
            CachedCameraDevice {
                public: CameraDevice {
                    id: stable_device_id(device),
                    label: if label.trim().is_empty() {
                        format!("Camera {}", position.saturating_add(1))
                    } else {
                        label
                    },
                },
                index: device.index().clone(),
            }
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn is_capture_device(device: &nokhwa::utils::CameraInfo) -> bool {
    let CameraIndex::Index(index) = device.index() else {
        return true;
    };
    let device_number_path = format!("/sys/class/video4linux/video{index}/dev");
    let Ok(device_number) = std::fs::read_to_string(device_number_path) else {
        return true;
    };
    let udev_path = format!("/run/udev/data/c{}", device_number.trim());
    let Ok(properties) = std::fs::read_to_string(udev_path) else {
        return true;
    };
    udev_capture_capability(&properties).unwrap_or(true)
}

#[cfg(not(target_os = "linux"))]
const fn is_capture_device(_device: &nokhwa::utils::CameraInfo) -> bool {
    true
}

#[cfg(target_os = "linux")]
fn udev_capture_capability(properties: &str) -> Option<bool> {
    properties.lines().find_map(|line| {
        line.strip_prefix("E:ID_V4L_CAPABILITIES=")
            .map(|capabilities| {
                capabilities.contains(":capture:") || capabilities.contains(":capture_mplane:")
            })
    })
}

fn choose_camera_format(formats: &[CameraFormat], request: CaptureRequest) -> Option<CameraFormat> {
    let requested_fps = requested_fps(request);
    formats
        .iter()
        .filter(|format| RgbAFormat::FORMATS.contains(&format.format()))
        .min_by_key(|format| {
            let resolution = format.resolution();
            let width_delta = i64::from(resolution.width()) - i64::from(request.width);
            let height_delta = i64::from(resolution.height()) - i64::from(request.height);
            let resolution_error = width_delta
                .unsigned_abs()
                .saturating_mul(width_delta.unsigned_abs())
                + height_delta
                    .unsigned_abs()
                    .saturating_mul(height_delta.unsigned_abs());
            let frame_rate_error = format.frame_rate().abs_diff(requested_fps);
            (
                resolution_error,
                frame_rate_error,
                frame_format_preference(format.format()),
            )
        })
        .copied()
}

fn requested_fps(request: CaptureRequest) -> u32 {
    (Duration::from_secs(1).as_nanos() / request.frame_interval.as_nanos()).clamp(1, 240) as u32
}

const fn frame_format_preference(format: FrameFormat) -> u8 {
    match format {
        FrameFormat::NV12 => 0,
        FrameFormat::YUYV => 1,
        FrameFormat::MJPEG => 2,
        FrameFormat::RAWRGB | FrameFormat::RAWBGR => 3,
        FrameFormat::GRAY => 4,
    }
}

fn map_nokhwa_error(error: &nokhwa::NokhwaError) -> CameraError {
    let message = error.to_string().to_lowercase();
    let category = if message.contains("permission")
        || message.contains("access denied")
        || message.contains("not authorized")
    {
        CameraError::PermissionDenied
    } else if message.contains("busy")
        || message.contains("in use")
        || message.contains("resource temporarily unavailable")
    {
        CameraError::Busy
    } else if message.contains("not found")
        || message.contains("no device")
        || message.contains("no such")
    {
        CameraError::DeviceNotFound
    } else {
        CameraError::Backend
    };
    if std::env::var_os("CAMLET_TRACE_CAMERA").is_some() {
        eprintln!(
            "camlet-camera: backend-error kind={} category={}",
            nokhwa_error_kind(error),
            camera_error_code(category)
        );
    }
    category
}

const fn nokhwa_error_kind(error: &nokhwa::NokhwaError) -> &'static str {
    match error {
        nokhwa::NokhwaError::UnitializedError => "uninitialized",
        nokhwa::NokhwaError::InitializeError { .. } => "initialize",
        nokhwa::NokhwaError::ShutdownError { .. } => "shutdown",
        nokhwa::NokhwaError::GeneralError(_) => "general",
        nokhwa::NokhwaError::StructureError { .. } => "structure",
        nokhwa::NokhwaError::OpenDeviceError(..) => "open-device",
        nokhwa::NokhwaError::GetPropertyError { .. } => "get-property",
        nokhwa::NokhwaError::SetPropertyError { .. } => "set-property",
        nokhwa::NokhwaError::OpenStreamError(_) => "open-stream",
        nokhwa::NokhwaError::ReadFrameError(_) => "read-frame",
        nokhwa::NokhwaError::ProcessFrameError { .. } => "process-frame",
        nokhwa::NokhwaError::StreamShutdownError(_) => "stream-shutdown",
        nokhwa::NokhwaError::UnsupportedOperationError(_) => "unsupported",
        nokhwa::NokhwaError::NotImplementedError(_) => "not-implemented",
    }
}

const fn camera_error_code(error: CameraError) -> &'static str {
    match error {
        CameraError::PermissionDenied => "permission-denied",
        CameraError::DeviceNotFound => "device-not-found",
        CameraError::Busy => "busy",
        CameraError::Backend => "backend",
    }
}

/// Commands accepted by the dedicated camera thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CameraWorkerCommand {
    /// Refresh the device list.
    Enumerate,
    /// Stop the old stream and start the selected device.
    Start {
        /// Stable device ID, or the backend default.
        device_id: Option<String>,
        /// Requested capture geometry and cadence.
        request: CaptureRequest,
    },
    /// Stop the active stream while retaining the worker.
    Stop,
    /// Stop the stream and terminate the worker.
    Shutdown,
}

/// Events emitted by the dedicated camera thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CameraWorkerEvent {
    /// Result of device enumeration.
    Devices(Result<Vec<CameraDevice>, CameraError>),
    /// Result of starting or switching a stream.
    Started(Result<(), CameraError>),
    /// Newest decoded frame or an asynchronous capture failure.
    Frame(Result<VideoFrame, CameraError>),
    /// The source was stopped and released.
    Stopped,
}

/// Cloneable receiving end used by an async UI adapter.
#[derive(Debug, Clone)]
pub struct CameraWorkerEvents {
    control: Arc<Mutex<Receiver<CameraWorkerEvent>>>,
    frames: Arc<Mutex<Receiver<CameraWorkerEvent>>>,
    notifications: async_channel::Receiver<()>,
}

impl CameraWorkerEvents {
    /// Waits asynchronously for one worker event without occupying an executor thread.
    pub async fn recv(&self) -> Option<CameraWorkerEvent> {
        loop {
            match self.try_recv() {
                Ok(event) => return Some(event),
                Err(TryRecvError::Disconnected) => return None,
                Err(TryRecvError::Empty) => {}
            }
            if self.notifications.recv().await.is_err() {
                return self.try_recv().ok();
            }
        }
    }

    /// Waits for one worker event.
    ///
    /// # Errors
    ///
    /// Returns the standard timeout/disconnection category.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<CameraWorkerEvent, RecvTimeoutError> {
        let deadline = Instant::now() + timeout;
        loop {
            match self.try_recv() {
                Ok(event) => return Ok(event),
                Err(TryRecvError::Disconnected) => return Err(RecvTimeoutError::Disconnected),
                Err(TryRecvError::Empty) => {}
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(RecvTimeoutError::Timeout);
            }
            thread::sleep((deadline - now).min(Duration::from_millis(2)));
        }
    }

    fn try_recv(&self) -> Result<CameraWorkerEvent, TryRecvError> {
        let control = self
            .control
            .lock()
            .map_err(|_| TryRecvError::Disconnected)?
            .try_recv();
        match control {
            Ok(event) => return Ok(event),
            Err(TryRecvError::Disconnected) => return Err(TryRecvError::Disconnected),
            Err(TryRecvError::Empty) => {}
        }
        self.frames
            .lock()
            .map_err(|_| TryRecvError::Disconnected)?
            .try_recv()
    }
}

/// Handle to a camera source that is created, used, and dropped on one dedicated thread.
#[derive(Debug)]
pub struct CameraWorker {
    commands: SyncSender<CameraWorkerCommand>,
    events: CameraWorkerEvents,
    join: Option<JoinHandle<()>>,
    dropped_frames: Arc<AtomicU64>,
}

impl CameraWorker {
    /// Starts a worker. The factory itself crosses the thread boundary; the resulting source does
    /// not, allowing backends whose native handles are intentionally not `Send`.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the operating system cannot create the camera thread.
    pub fn spawn(
        factory: impl FnOnce() -> Box<dyn FrameSource> + Send + 'static,
    ) -> Result<Self, std::io::Error> {
        let (command_sender, command_receiver) = mpsc::sync_channel(8);
        let (control_sender, control_receiver) = mpsc::channel();
        let (frame_sender, frame_receiver) = mpsc::sync_channel(1);
        let (notification_sender, notification_receiver) = async_channel::bounded(1);
        let dropped_frames = Arc::new(AtomicU64::new(0));
        let worker_dropped_frames = Arc::clone(&dropped_frames);
        let join = thread::Builder::new()
            .name("camlet-camera".to_owned())
            .spawn(move || {
                worker_loop(
                    factory(),
                    &command_receiver,
                    &control_sender,
                    &frame_sender,
                    &notification_sender,
                    &worker_dropped_frames,
                );
            })?;
        Ok(Self {
            commands: command_sender,
            events: CameraWorkerEvents {
                control: Arc::new(Mutex::new(control_receiver)),
                frames: Arc::new(Mutex::new(frame_receiver)),
                notifications: notification_receiver,
            },
            join: Some(join),
            dropped_frames,
        })
    }

    /// Returns a cloneable event receiver.
    #[must_use]
    pub fn events(&self) -> CameraWorkerEvents {
        self.events.clone()
    }

    /// Sends a command, returning false if the worker has already stopped.
    #[must_use]
    pub fn send(&self, command: CameraWorkerCommand) -> bool {
        self.commands.send(command).is_ok()
    }

    /// Returns the number of frames discarded because the capacity-one event slot was occupied.
    #[must_use]
    pub fn dropped_frames(&self) -> u64 {
        self.dropped_frames.load(Ordering::Relaxed)
    }

    /// Stops, joins, and proves source release.
    #[must_use]
    pub fn shutdown(mut self) -> bool {
        let sent = self.send(CameraWorkerCommand::Shutdown);
        let joined = self.join.take().is_none_or(|join| join.join().is_ok());
        sent && joined
    }
}

impl Drop for CameraWorker {
    fn drop(&mut self) {
        let _ = self.commands.try_send(CameraWorkerCommand::Shutdown);
    }
}

fn worker_loop(
    mut source: Box<dyn FrameSource>,
    commands: &Receiver<CameraWorkerCommand>,
    control_events: &Sender<CameraWorkerEvent>,
    frame_events: &SyncSender<CameraWorkerEvent>,
    event_notifications: &async_channel::Sender<()>,
    dropped_frames: &AtomicU64,
) {
    let mut running = false;
    let mut frame_interval = Duration::from_millis(33);
    let mut frames_until_refresh = 150_u16;
    loop {
        let command = if running {
            match commands.try_recv() {
                Ok(command) => Some(command),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => break,
            }
        } else {
            match commands.recv() {
                Ok(command) => Some(command),
                Err(_) => break,
            }
        };

        if let Some(command) = command {
            match command {
                CameraWorkerCommand::Enumerate => {
                    if control_events
                        .send(CameraWorkerEvent::Devices(source.devices()))
                        .is_err()
                    {
                        break;
                    }
                    notify_event(event_notifications);
                }
                CameraWorkerCommand::Start { device_id, request } => {
                    frame_interval = request.frame_interval;
                    let result = source.start(device_id.as_deref(), request);
                    running = result.is_ok();
                    if result.is_err() {
                        source.stop();
                    }
                    frames_until_refresh = 150;
                    if control_events
                        .send(CameraWorkerEvent::Started(result))
                        .is_err()
                    {
                        break;
                    }
                    notify_event(event_notifications);
                }
                CameraWorkerCommand::Stop => {
                    running = false;
                    source.stop();
                    if control_events.send(CameraWorkerEvent::Stopped).is_err() {
                        break;
                    }
                    notify_event(event_notifications);
                }
                CameraWorkerCommand::Shutdown => break,
            }
        }

        if running {
            let capture_started = Instant::now();
            match source.latest_frame() {
                Ok(Some(frame)) => match frame_events.try_send(CameraWorkerEvent::Frame(Ok(frame)))
                {
                    Ok(()) => notify_event(event_notifications),
                    Err(TrySendError::Full(_)) => {
                        dropped_frames.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(TrySendError::Disconnected(_)) => break,
                },
                Ok(None) => thread::sleep(Duration::from_millis(5)),
                Err(error) => {
                    running = false;
                    source.stop();
                    if control_events
                        .send(CameraWorkerEvent::Frame(Err(error)))
                        .is_err()
                    {
                        break;
                    }
                    notify_event(event_notifications);
                }
            }
            frames_until_refresh = frames_until_refresh.saturating_sub(1);
            if frames_until_refresh == 0 {
                if control_events
                    .send(CameraWorkerEvent::Devices(source.devices()))
                    .is_err()
                {
                    break;
                }
                notify_event(event_notifications);
                frames_until_refresh = 150;
            }
            if let Some(remaining) = frame_interval.checked_sub(capture_started.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }
    source.stop();
    let _ = control_events.send(CameraWorkerEvent::Stopped);
    notify_event(event_notifications);
}

fn notify_event(notifications: &async_channel::Sender<()>) {
    let _ = notifications.try_send(());
}

fn generate_synthetic_frame(
    request: CaptureRequest,
    sequence: u64,
) -> Result<VideoFrame, CameraError> {
    let pixel_count = usize::try_from(request.width)
        .ok()
        .and_then(|width| {
            usize::try_from(request.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(CameraError::Backend)?;
    let mut rgba = vec![0_u8; pixel_count];
    let center_x = i64::from(request.width / 2);
    let center_y = i64::from(request.height / 2);
    let maximum_radius = center_x.min(center_y).max(1);
    let animated_blue = u8::try_from(sequence % 96).unwrap_or(0);

    for y in 0..request.height {
        for x in 0..request.width {
            let checker = (x / 16 + y / 16).is_multiple_of(2);
            let gradient = if request.width > 1 {
                u8::try_from(x.saturating_mul(127) / (request.width - 1)).unwrap_or(127)
            } else {
                0
            };
            let mut red = if checker { 52 } else { 34 };
            let mut green = if checker { 64 } else { 46 };
            let mut blue = 82_u8
                .saturating_add(gradient / 3)
                .saturating_add(animated_blue / 4);
            let delta_x = i64::from(x) - center_x;
            let delta_y = i64::from(y) - center_y;
            let radius_squared = delta_x * delta_x + delta_y * delta_y;

            if x.is_multiple_of(32) || y.is_multiple_of(32) {
                red = 84;
                green = 102;
                blue = 128;
            }

            for ring in [1_i64, 2, 3] {
                let radius = maximum_radius * ring / 4;
                let target = radius * radius;
                if (radius_squared - target).abs() <= maximum_radius {
                    red = 124;
                    green = 226;
                    blue = 198;
                }
            }

            if i64::from(x) == center_x || i64::from(y) == center_y {
                red = 255;
                green = 108;
                blue = 77;
            }

            let index =
                usize::try_from((u64::from(y) * u64::from(request.width) + u64::from(x)) * 4)
                    .map_err(|_| CameraError::Backend)?;
            rgba[index..index + 4].copy_from_slice(&[red, green, blue, u8::MAX]);
        }
    }

    Ok(VideoFrame {
        width: request.width,
        height: request.height,
        sequence,
        rgba,
    })
}

fn animate_synthetic_marker(
    rgba: &mut [u8],
    request: CaptureRequest,
    sequence: u64,
) -> Result<(), CameraError> {
    if sequence == 0 {
        return Ok(());
    }

    let width = usize::try_from(request.width).map_err(|_| CameraError::Backend)?;
    let height = usize::try_from(request.height).map_err(|_| CameraError::Backend)?;
    let expected = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(CameraError::Backend)?;
    if rgba.len() != expected {
        return Err(CameraError::Backend);
    }

    let marker_width = width.clamp(1, 16);
    let marker_height = height.clamp(1, 8);
    let travel = width.saturating_sub(marker_width).saturating_add(1);
    let start_x = usize::try_from(sequence % u64::try_from(travel).unwrap_or(1)).unwrap_or(0);
    let start_y = height.saturating_sub(marker_height);
    for y in start_y..height {
        for x in start_x..start_x + marker_width {
            let index = (y * width + x) * 4;
            rgba[index..index + 4].copy_from_slice(&[255, 176, 103, u8::MAX]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use nokhwa::utils::{CameraFormat, CameraIndex, CameraInfo, FrameFormat};

    #[cfg(target_os = "linux")]
    use super::udev_capture_capability;
    use super::{
        CameraDevice, CameraError, CameraWorker, CameraWorkerCommand, CameraWorkerEvent,
        CaptureRequest, FrameSource, NokhwaFrameSource, ScriptedFrameSource, SyntheticFrameSource,
        VideoFrame, cache_camera_devices, choose_camera_format, map_nokhwa_error,
        retry_camera_start, tolerate_transient_frame_error,
    };
    use std::time::Duration;

    fn request() -> CaptureRequest {
        CaptureRequest {
            width: 64,
            height: 48,
            frame_interval: Duration::from_millis(33),
        }
    }

    #[test]
    fn synthetic_source_never_needs_hardware() {
        let mut source = SyntheticFrameSource::default();
        let devices = source
            .devices()
            .unwrap_or_else(|error| unreachable!("{error}"));
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, SyntheticFrameSource::DEVICE_ID);
        source
            .start(Some(SyntheticFrameSource::DEVICE_ID), request())
            .unwrap_or_else(|error| unreachable!("{error}"));
        let frame = source
            .latest_frame()
            .unwrap_or_else(|error| unreachable!("{error}"))
            .unwrap_or_else(|| unreachable!());
        assert_eq!((frame.width, frame.height, frame.sequence), (64, 48, 0));
        assert_eq!(frame.rgba.len(), 64 * 48 * 4);
    }

    #[test]
    fn synthetic_animation_changes_only_the_bounded_marker() {
        let mut source = SyntheticFrameSource::default();
        source
            .start(None, request())
            .unwrap_or_else(|error| unreachable!("{error}"));
        let first = source
            .latest_frame()
            .unwrap_or_else(|error| unreachable!("{error}"))
            .unwrap_or_else(|| unreachable!());
        let second = source
            .latest_frame()
            .unwrap_or_else(|error| unreachable!("{error}"))
            .unwrap_or_else(|| unreachable!());

        let changed_pixels = first
            .rgba
            .chunks_exact(4)
            .zip(second.rgba.chunks_exact(4))
            .filter(|(first, second)| first != second)
            .count();
        assert_eq!((first.sequence, second.sequence), (0, 1));
        assert!((1..=16 * 8).contains(&changed_pixels));
    }

    #[test]
    fn latest_frame_buffer_has_capacity_one() {
        let mut source = SyntheticFrameSource::default();
        source
            .start(None, request())
            .unwrap_or_else(|error| unreachable!("{error}"));
        for _ in 0..100 {
            source
                .advance()
                .unwrap_or_else(|error| unreachable!("{error}"));
        }
        let frame = source
            .latest_frame()
            .unwrap_or_else(|error| unreachable!("{error}"))
            .unwrap_or_else(|| unreachable!());
        assert_eq!(frame.sequence, 100);
        assert_eq!(
            source
                .latest_frame()
                .map(|frame| frame.map(|frame| frame.sequence)),
            Ok(Some(101))
        );
    }

    #[test]
    fn stop_releases_and_clears_pending_frame() {
        let mut source = SyntheticFrameSource::default();
        source
            .start(None, request())
            .unwrap_or_else(|error| unreachable!("{error}"));
        source.stop();
        assert_eq!(source.latest_frame(), Ok(None));
        assert!(source.advance().is_err());
    }

    #[test]
    fn scripted_source_exercises_failures_and_release() {
        let mut source = ScriptedFrameSource::new(vec![CameraDevice {
            id: "test-1".to_owned(),
            label: "Test camera".to_owned(),
        }]);
        source.fail_next_start(CameraError::Busy);
        assert_eq!(
            source.start(Some("test-1"), request()),
            Err(CameraError::Busy)
        );
        assert!(source.start(Some("test-1"), request()).is_ok());
        source.push_frame(VideoFrame {
            width: 2,
            height: 2,
            sequence: 7,
            rgba: vec![0; 16],
        });
        assert_eq!(
            source
                .latest_frame()
                .map(|frame| frame.map(|frame| frame.sequence)),
            Ok(Some(7))
        );
        source.stop();
        assert_eq!(source.stop_count(), 1);
        assert_eq!(source.latest_frame(), Ok(None));
    }

    #[test]
    fn scripted_source_replaces_pending_frames() {
        let mut source = ScriptedFrameSource::default();
        assert!(source.start(None, request()).is_ok());
        for sequence in 0..10 {
            source.push_frame(VideoFrame {
                width: 1,
                height: 1,
                sequence,
                rgba: vec![0; 4],
            });
        }
        assert_eq!(
            source
                .latest_frame()
                .map(|frame| frame.map(|frame| frame.sequence)),
            Ok(Some(9))
        );
        assert_eq!(source.latest_frame(), Ok(None));
    }

    #[test]
    fn format_negotiation_prefers_requested_geometry() {
        let formats = [
            CameraFormat::new_from(320, 240, FrameFormat::MJPEG, 30),
            CameraFormat::new_from(640, 480, FrameFormat::YUYV, 60),
            CameraFormat::new_from(1_280, 720, FrameFormat::NV12, 30),
        ];
        let selected = choose_camera_format(&formats, request()).unwrap_or_else(|| unreachable!());
        assert_eq!((selected.width(), selected.height()), (320, 240));

        let selected = choose_camera_format(
            &formats,
            CaptureRequest {
                width: 640,
                height: 480,
                frame_interval: Duration::from_millis(17),
            },
        )
        .unwrap_or_else(|| unreachable!());
        assert_eq!((selected.width(), selected.height()), (640, 480));
    }

    #[test]
    fn format_negotiation_honors_requested_resolution_before_fps() {
        let formats = [
            CameraFormat::new_from(640, 480, FrameFormat::YUYV, 30),
            CameraFormat::new_from(320, 240, FrameFormat::MJPEG, 60),
        ];
        let selected = choose_camera_format(
            &formats,
            CaptureRequest {
                width: 640,
                height: 480,
                frame_interval: Duration::from_nanos(1_000_000_000 / 60),
            },
        )
        .unwrap_or_else(|| unreachable!());

        assert_eq!((selected.width(), selected.height()), (640, 480));
        assert_eq!(selected.frame_rate(), 30);
    }

    #[test]
    fn enumerated_device_ids_reopen_with_the_original_backend_index() {
        let devices = cache_camera_devices(&[
            CameraInfo::new("", "", "", CameraIndex::Index(11)),
            CameraInfo::new("USB camera", "", "persistent-id", CameraIndex::Index(7)),
        ]);

        assert_eq!(devices[0].public.id, "persistent-id");
        assert_eq!(devices[0].index, CameraIndex::Index(7));
        assert_eq!(devices[1].public.id, "11");
        assert_eq!(devices[1].public.label, "Camera 2");
        assert_eq!(devices[1].index, CameraIndex::Index(11));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_inventory_excludes_nodes_without_capture_capability() {
        assert_eq!(
            udev_capture_capability("E:ID_V4L_CAPABILITIES=:capture:\n"),
            Some(true)
        );
        assert_eq!(
            udev_capture_capability("E:ID_V4L_CAPABILITIES=:\n"),
            Some(false)
        );
        assert_eq!(udev_capture_capability("E:OTHER=value\n"), None);
    }

    #[test]
    fn backend_errors_map_without_exposing_identifiers() {
        assert_eq!(
            map_nokhwa_error(&nokhwa::NokhwaError::OpenStreamError(
                "Device or resource busy".to_owned()
            )),
            CameraError::Busy
        );
        assert_eq!(
            map_nokhwa_error(&nokhwa::NokhwaError::OpenDeviceError(
                "private-id".to_owned(),
                "not found".to_owned()
            )),
            CameraError::DeviceNotFound
        );
    }

    #[test]
    fn transient_camera_start_failures_are_retried_but_permissions_are_not() {
        let mut attempts = 0_u8;
        let mut waits = 0_u8;
        let recovered = retry_camera_start(
            || {
                attempts = attempts.saturating_add(1);
                if attempts < 3 {
                    Err(CameraError::Backend)
                } else {
                    Ok("preview")
                }
            },
            || waits = waits.saturating_add(1),
        );
        assert_eq!(recovered, Ok("preview"));
        assert_eq!((attempts, waits), (3, 2));

        attempts = 0;
        waits = 0;
        let denied = retry_camera_start(
            || {
                attempts = attempts.saturating_add(1);
                Err::<(), _>(CameraError::PermissionDenied)
            },
            || waits = waits.saturating_add(1),
        );
        assert_eq!(denied, Err(CameraError::PermissionDenied));
        assert_eq!((attempts, waits), (1, 0));
    }

    #[test]
    fn stopping_without_a_live_handle_still_clears_camera_identity() {
        let mut source = NokhwaFrameSource {
            camera: None,
            active_index: Some(CameraIndex::Index(7)),
            devices: Vec::new(),
            next_sequence: 42,
            consecutive_frame_errors: 2,
        };

        source.stop();

        assert_eq!(source.active_index, None);
        assert_eq!(source.next_sequence, 0);
        assert_eq!(source.consecutive_frame_errors, 0);
    }

    #[test]
    fn isolated_frame_errors_do_not_destroy_a_healthy_stream() {
        let mut consecutive_errors = 0;
        for _ in 0..3 {
            assert_eq!(
                tolerate_transient_frame_error(&mut consecutive_errors, CameraError::Backend),
                Ok(())
            );
        }
        assert_eq!(consecutive_errors, 3);
        assert_eq!(
            tolerate_transient_frame_error(&mut consecutive_errors, CameraError::Backend),
            Err(CameraError::Backend)
        );

        consecutive_errors = 0;
        assert_eq!(
            tolerate_transient_frame_error(&mut consecutive_errors, CameraError::PermissionDenied),
            Err(CameraError::PermissionDenied)
        );
        assert_eq!(consecutive_errors, 0);
    }

    #[test]
    fn worker_uses_capacity_one_and_joins_after_release() {
        struct ReleasingSource {
            inner: SyntheticFrameSource,
            stops: Arc<AtomicU64>,
        }

        impl FrameSource for ReleasingSource {
            fn devices(&mut self) -> Result<Vec<CameraDevice>, CameraError> {
                self.inner.devices()
            }

            fn start(
                &mut self,
                device_id: Option<&str>,
                request: CaptureRequest,
            ) -> Result<(), CameraError> {
                self.inner.start(device_id, request)
            }

            fn latest_frame(&mut self) -> Result<Option<VideoFrame>, CameraError> {
                self.inner.latest_frame()
            }

            fn stop(&mut self) {
                self.stops.fetch_add(1, Ordering::Relaxed);
                self.inner.stop();
            }
        }

        let stops = Arc::new(AtomicU64::new(0));
        let worker_stops = Arc::clone(&stops);
        let worker = CameraWorker::spawn(move || {
            Box::new(ReleasingSource {
                inner: SyntheticFrameSource::default(),
                stops: worker_stops,
            })
        })
        .unwrap_or_else(|error| unreachable!("{error}"));
        let events = worker.events();
        assert!(worker.send(CameraWorkerCommand::Enumerate));
        assert!(matches!(
            events.recv_timeout(Duration::from_secs(1)),
            Ok(CameraWorkerEvent::Devices(Ok(devices))) if devices.len() == 1
        ));
        assert!(worker.send(CameraWorkerCommand::Start {
            device_id: None,
            request: request(),
        }));
        assert_eq!(
            events.recv_timeout(Duration::from_secs(1)),
            Ok(CameraWorkerEvent::Started(Ok(())))
        );
        std::thread::sleep(Duration::from_millis(120));
        assert!(worker.dropped_frames() > 0);
        assert!(matches!(
            events.recv_timeout(Duration::from_secs(1)),
            Ok(CameraWorkerEvent::Frame(Ok(_)))
        ));
        assert!(worker.send(CameraWorkerCommand::Start {
            device_id: Some(SyntheticFrameSource::DEVICE_ID.to_owned()),
            request: request(),
        }));
        assert_eq!(
            events.recv_timeout(Duration::from_secs(1)),
            Ok(CameraWorkerEvent::Started(Ok(())))
        );
        assert!(worker.shutdown());
        assert_eq!(stops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn worker_releases_a_source_after_a_failed_start() {
        struct FailingStartSource {
            stops: Arc<AtomicU64>,
        }

        impl FrameSource for FailingStartSource {
            fn devices(&mut self) -> Result<Vec<CameraDevice>, CameraError> {
                Ok(Vec::new())
            }

            fn start(
                &mut self,
                _device_id: Option<&str>,
                _request: CaptureRequest,
            ) -> Result<(), CameraError> {
                Err(CameraError::Backend)
            }

            fn latest_frame(&mut self) -> Result<Option<VideoFrame>, CameraError> {
                Ok(None)
            }

            fn stop(&mut self) {
                self.stops.fetch_add(1, Ordering::Relaxed);
            }
        }

        let stops = Arc::new(AtomicU64::new(0));
        let worker_stops = Arc::clone(&stops);
        let worker = CameraWorker::spawn(move || {
            Box::new(FailingStartSource {
                stops: worker_stops,
            })
        })
        .unwrap_or_else(|error| unreachable!("{error}"));
        let events = worker.events();

        assert!(worker.send(CameraWorkerCommand::Start {
            device_id: None,
            request: request(),
        }));
        assert_eq!(
            events.recv_timeout(Duration::from_secs(1)),
            Ok(CameraWorkerEvent::Started(Err(CameraError::Backend)))
        );
        assert_eq!(stops.load(Ordering::Relaxed), 1);
        assert!(worker.shutdown());
    }
}
