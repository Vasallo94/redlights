# Design notes

How RedLights turns displays red on each platform, how the code is organized, and how it is tested without real hardware.

## The filter

Every platform offers a last-stage color transform between the composed image and the panel. RedLights uses it in one of two forms:

- **Gamma ramps**: one lookup table per channel. Red goes linearly from 0 to the brightness level; green and blue are always 0. Pure green or pure blue content therefore turns black.
- **Color matrix**: a 3×3 matrix applied to every pixel. Besides the channel filter above, it allows a luminance mode, `R' = level · (0.2126 R + 0.7152 G + 0.0722 B)`, in which every color becomes red according to its brightness and nothing disappears. Not exposed to users yet.

Brightness uses the same quadratic curve everywhere, `level = max(0.02, slider²)`, which gives finer control in the dim range where it is used at night.

## Platforms

| Platform | Mechanism | Form | Restored if the process dies |
|---|---|---|---|
| macOS | `CGSetDisplayTransferByFormula` | ramps | yes, WindowServer resets it |
| Windows 8+ | `MagSetFullscreenColorEffect` (Magnification API) | 5×5 matrix | yes, verified on CI |
| Linux, wlroots | `wlr-gamma-control-unstable-v1` | ramps | yes, the compositor resets it |
| Linux, GNOME | `org.gnome.Mutter.DisplayConfig.SetCrtcGamma` (D-Bus) | ramps | no |
| Linux, X11 | XRandR `SetCrtcGamma` | ramps | no |
| Linux, KDE Plasma (Wayland) | not supported yet | | |

Notes:

- **Windows** uses the GDI+ row-vector convention (`out = in · M`, rows are input channels), the transpose of the core's matrix; see `row_vector_5x5`. The API only works in 64-bit processes.
- **GNOME**'s D-Bus interface is internal to GNOME rather than a public contract, but Mutter accepts calls from third parties. It also offers `SetOutputCTM`, a color matrix, which would enable the luminance mode on GNOME.
- **KDE Plasma** on Wayland implements neither `wlr-gamma-control` nor a public gamma API. The likely route is an ICC profile with a red-only VCGT tag assigned to each output, as [KGamma2](https://planet.kde.org/david-edmundson-2026-08-03-wayland-pain-points-kgamma/) does. A KDE X11 session already works through the X11 backend.
- **Android** apps cannot change the display's color transform. An overlay can tint the screen but cannot remove green and blue; a true red filter needs SurfaceFlinger's color matrix through adb or root.

## Code layout

- `Sources/main.swift`: the macOS menu bar app, independent from the rest.
- `rust/src/lib.rs`: platform-independent core: brightness curve, ramps, matrices.
- `rust/src/backend/`: one backend per mechanism behind the `Backend` trait (`apply`, `restore`, `reset`). `backend::detect()` picks one from the session (`WAYLAND_DISPLAY`, `DISPLAY`) or from `REDLIGHTS_BACKEND`.
- `rust/src/app.rs`: tray app state shared by Linux and Windows: on/off, brightness (saved between launches), and the commands coming from the menu, the shortcut or `redlights toggle`. The backend is created when red mode turns on and dropped when it turns off, which is what restores the original gamma on wlroots.
- `rust/src/tray_linux.rs`: StatusNotifierItem tray icon through `ksni` (pure D-Bus, no GTK, so the binary stays static) and a D-Bus service, `io.github.Vasallo94.RedLights`, used by `redlights toggle` and `status`. Owning that bus name also prevents a second instance. Wayland apps cannot register global shortcuts, so users bind `redlights toggle` in their desktop instead.
- `rust/src/tray_windows.rs`: native tray icon and menu (`tray-icon`) and the Ctrl+Alt+Shift+R hotkey (`global-hotkey`), both driven by the thread's Windows message loop. Double-clicking the exe detaches the console and starts the tray.
- `rust/src/main.rs`: command-line interface. It reapplies the filter every 2 s in case Night Light or another program overrides it, and restores on Ctrl-C, SIGTERM or console close.

Linux binaries are statically linked against musl so they run on any distribution regardless of its glibc version.

## Testing without hardware

| What | How |
|---|---|
| Core math | unit tests in `lib.rs` |
| X11 | integration test against Xvfb; everything happens in one connection, because Xvfb returns garbage when a ramp is read after the client that set it has disconnected |
| GNOME | integration test against Mutter running in X11 mode on Xvfb, through the real D-Bus API |
| Windows | integration test on a GitHub Windows runner: apply the matrix and read it back with `MagGetFullscreenColorEffect` |
| wlroots | manual: load the `vkms` kernel module (a virtual DRM driver) in a VM, run Sway on it with `WLR_BACKENDS=drm LIBSEAT_BACKEND=noop`, and read the CRTC's `GAMMA_LUT` property through `DRM_IOCTL_MODE_GETPROPBLOB` |
| Tray app | Linux CI starts the tray, toggles it over D-Bus, checks the single-instance guard and quits it; Windows CI starts it and checks it stays alive |
| Released binaries | CI runs the exact static Linux binary against X11 and Mutter (apply, SIGTERM, reset) and installs the Windows package with Scoop and winget |

What only real hardware can tell: that the screen actually looks red, whether the cursor follows, HDR modes, and interactions with Night Light or other color tools.
