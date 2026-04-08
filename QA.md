# QA Checklist

The goal is to validate the packaged app on real desktops, confirm that translated UI remains complete and readable across the supported locales, and keep the existing camera, startup, and persistence fallbacks understandable.

## Automated Baseline

Run these before packaging or distributing a candidate:

```bash
pnpm verify
pnpm package:linux
```

Run `pnpm package:win` on a Windows host when validating the NSIS installer path.

## Release-Candidate Gate

Do not call a build release-candidate ready until all of the following are true:

- `pnpm verify` passes locally.
- At least one Linux AppImage launch has been validated on a real desktop session.
- The in-app About surface shows the intended version, release channel, runtime mode, platform, arch, display protocol, and Electron/Chrome runtime versions.
- The diagnostics summary can be copied from the About panel and attached to a bug report.
- Camera permission denial, camera-busy behavior, no-camera fallback, and saved-device fallback are understandable without reading logs.
- Startup notices for repaired settings or unavailable persistence are visible and translated.
- The language selector lists every shipped locale and switching between them does not break the overlay UI.

## Platform Matrix

Validate the relevant rows before widening distribution:

| Platform       | Minimum checks                                                                                                                                       |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ubuntu Wayland | transparency, hit testing, drag-to-move, always-on-top focus behavior, camera permission prompt, device hot-plug                                     |
| Ubuntu X11     | transparency, drag persistence, resize persistence, camera start/retry, saved-device fallback                                                        |
| Windows        | frameless overlay input, startup/build info visibility, camera permission flow, persisted language and appearance, installer launch/uninstall sanity |

## Core Overlay Checklist

- Launch the packaged build and confirm the overlay window opens without a startup failure screen.
- Confirm the transparent frameless shell accepts pointer input correctly.
- Drag the overlay with the left mouse button and confirm the saved bounds restore after restart.
- Resize the overlay and confirm the saved size restores after restart.
- Right-click a non-interactive overlay area and confirm the settings panel toggles.
- Confirm `Escape` closes the settings panel.
- Confirm the preview starts after camera permission is granted.
- Confirm the selected camera persists across restarts.
- Confirm appearance and language changes persist across restarts.
- Confirm the About card shows the exact build metadata being tested.
- Confirm each shipped language renders the startup UI, settings panel, camera state copy, and About surface without missing labels.

## Camera and Runtime Resilience

- Deny camera permission once and confirm the app stays usable with a clear retry path.
- Start while another app is holding the camera and confirm Camlet reports the camera as busy instead of failing silently.
- Disconnect or disable the saved camera and confirm Camlet either falls back safely or explains that the selected device is unavailable.
- Reconnect a camera and confirm the preview recovers after retrying.
- Confirm blank or missing camera labels still render as safe fallback labels.
- If media devices are unavailable in the environment, confirm the overlay remains usable and shows a clear error state.

## Startup and Settings Resilience

- Start from a fresh settings file and confirm safe defaults.
- Start from a malformed or partially legacy settings file and confirm the app repairs values and shows a startup notice.
- Confirm malformed settings do not block startup.
- Simulate or force a persistence failure if practical and confirm Camlet keeps running while warning that recent changes may not persist after exit.
- Confirm window bounds are clamped back onto the current display work area after restoring invalid saved bounds.

## Localization Coverage

- Switch through each shipped locale: `en`, `pt-BR`, `es`, `fr`, `de`, `it`, and `ja`.
- Confirm the language selector remains readable even before selecting a language you understand.
- Confirm the effective-language summary updates immediately after switching.
- If possible, launch the app with the OS locale set to a supported language family and confirm `system` resolves to the expected shipped locale.
- Confirm translated startup failure screens still show the reload action and developer-only debug details in development builds.

## Packaged Diagnostics and Bug Reports

When filing a packaged-app issue, include:

- the exact artifact name
- the host OS and desktop session
- whether the run was Wayland, X11, or Windows desktop
- the copied diagnostics summary from the About panel
- whether the problem reproduces after restarting Camlet
- whether the problem reproduces with a clean settings profile

## Known Headless Limits

The automated suite does not verify:

- compositor-specific transparency behavior
- desktop permission prompts
- actual webcam hardware compatibility
- always-on-top behavior after real focus changes
- packaged startup on every desktop environment

At least one real-desktop validation pass is still required before promoting a beta or stable build.
