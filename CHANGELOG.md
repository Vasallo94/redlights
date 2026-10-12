# Changelog

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows [SemVer](https://semver.org/).

## [Unreleased]

### Added
- **About RedLights** item in the Mac app menu, showing the version.

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
