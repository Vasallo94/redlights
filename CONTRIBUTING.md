# Contributing to RedLights

Thanks for your interest. Contributions in English or Spanish are welcome.

## Reporting bugs and testing on real hardware

The most useful contribution right now is trying RedLights on hardware we have not tested: Linux desktops (GNOME on Wayland, Sway, Hyprland, labwc on a Raspberry Pi…), Windows machines, external monitors, HDR displays.

[Open an issue](https://github.com/Vasallo94/redlights/issues/new/choose) with the bug report form. It asks for the details we need: operating system, desktop environment or compositor, RedLights version (`redlights --version`, or **About** in the Mac app) and what you saw. A photo of the screen helps, since screenshots always show normal colors.

Security problems go through private reporting instead; see [SECURITY.md](SECURITY.md).

## Project layout

| Path | What |
|---|---|
| `Sources/main.swift` | macOS menu bar app (Swift, AppKit) |
| `build.sh` | Builds and packages `RedLights.app` |
| `Resources/` | App translations |
| `rust/` | Linux and Windows command-line tool |
| `rust/src/lib.rs` | Shared core: brightness curve, gamma ramps, color matrix |
| `rust/src/backend/` | One backend per platform mechanism |
| `rust/src/app.rs`, `tray_*.rs` | Tray app on Linux and Windows |
| `rust/packaging/` | Linux desktop entry for the `.deb` |
| `docs/design.md` | Design notes: mechanisms per platform, layout, testing |
| `scripts/` | Bundle checks and icon generation |

## Development setup

### macOS app

You need macOS 13 or later and the Command Line Tools (`xcode-select --install`); Xcode is not required.

```bash
./build.sh                 # builds RedLights.app
./scripts/check-bundle.sh  # verifies the bundle like CI does
open RedLights.app
```

User-facing strings use `NSLocalizedString` with English text as the key. Add the Spanish translation to `Resources/es.lproj/Localizable.strings`; the bundle check fails if a key is missing or unused.

### Linux and Windows tool

You need a recent stable Rust toolchain ([rustup](https://rustup.rs)).

```bash
cd rust
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

The X11 and GNOME backends also have integration tests that need a display server. On Linux:

```bash
sudo apt install xvfb mutter dbus-x11
Xvfb :99 &
DISPLAY=:99 dbus-run-session -- sh -c 'mutter --x11 --replace & sleep 5; cargo test -- --ignored --test-threads=1'
```

From macOS you can type-check other platforms with `cargo check --target x86_64-unknown-linux-gnu` or `--target x86_64-pc-windows-msvc`, and run the Linux tests in a container.

A new platform mechanism goes in its own file under `rust/src/backend/`, implementing the `Backend` trait (`apply`, `restore`, `reset`), and is selected in `backend::detect()`.

## Pull requests

1. Fork the repository and create a branch from `main`.
2. Keep the change focused; one topic per pull request.
3. Add an entry under `## [Unreleased]` in [`CHANGELOG.md`](CHANGELOG.md) if users will notice the change.
4. Make sure CI passes. It builds and tests macOS, Linux (x86_64 and ARM64) and Windows, and a pull request can only be merged when it is green.

For larger changes, such as a new platform or a new feature, open an issue first so we can agree on the approach.

## Releasing (maintainers)

1. Move the `## [Unreleased]` entries in [`CHANGELOG.md`](CHANGELOG.md) to a new `## [x.y.z] - YYYY-MM-DD` section and merge it into `main` through a pull request.
2. Tag the merge commit: `git tag vx.y.z && git push origin vx.y.z`.

CI builds every platform, publishes the release with checksums and attestations, and uses that CHANGELOG section as release notes. The version comes from the tag and is not hardcoded anywhere. The [Homebrew tap](https://github.com/Vasallo94/homebrew-tap) picks up new releases automatically within a day.

## Code of conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). Please be kind.
