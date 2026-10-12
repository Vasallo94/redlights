# Changelog

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows [SemVer](https://semver.org/).

## [Unreleased]

### Added
- **System tray app on Linux and Windows**: `redlights tray` shows a moon icon; click it to toggle red mode, right-click for brightness presets. Brightness is remembered between launches, and menus follow the system language (English or Spanish).
- **Windows**: double-clicking `redlights.exe` starts the tray app without a console window. **Ctrl+Alt+Shift+R** toggles red mode from any app.
- **Linux**: `redlights toggle` and `redlights status` control the running tray app, so red mode can be bound to a key in any desktop. The `.deb` adds RedLights to the applications menu.

### Changed
- Dependabot now also watches Rust dependencies, and CI checks them with cargo-deny for known vulnerabilities, licenses and sources.

## [0.3.1] - 2026-10-12

### Added
- **About RedLights** item in the Mac app menu, showing the version.
- Debian packages (`.deb`) for x86_64 and ARM64, for Debian, Ubuntu and Raspberry Pi OS.
- Scoop bucket for Windows: `scoop bucket add vasallo94 https://github.com/Vasallo94/scoop-bucket`.

### Fixed
- Linux binaries are now statically linked (musl). Version 0.3.0 required glibc 2.39 and did not start on Raspberry Pi OS Bookworm or Ubuntu 22.04.

## [0.3.0] - 2026-10-12

### Added
- **Linux**: `redlights` command-line tool for X11, GNOME (Wayland and X11) and wlroots compositors (Sway, Hyprland, labwc, Wayfire…), for x86_64 and ARM64. `redlights [BRIGHTNESS]` turns displays red until Ctrl-C; `redlights reset` recovers a display stuck red.
- **Windows** 10/11 (64-bit): the same tool, using the Magnification API color matrix. Normal colors come back when it exits, even if it crashes.
- Release assets for every platform, each with its SHA-256 and a GitHub provenance attestation.

## [0.2.0] - 2026-10-12

### Added
- English and Spanish localization, following the system language (English is the fallback).
- Install with Homebrew: `brew install --cask vasallo94/tap/redlights`.

## [0.1.0] - 2026-10-11

### Added
- Menu bar app that turns every display pure red (green and blue at zero).
- Brightness slider with a quadratic curve, remembered across launches.
- Global shortcut ⌃⌥⌘R.
- Automatic reapply on wake, on display changes and every 2 s.
- Open at login option.
- Universal binary (Apple Silicon + Intel) and app icon.
