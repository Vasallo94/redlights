<p align="center">
  <img src="assets/banner.svg" alt="RedLights: every display, pure red, for astronomy" width="100%">
</p>

<p align="center">
  <a href="https://github.com/Vasallo94/redlights/releases/latest"><img src="https://img.shields.io/github/v/release/Vasallo94/redlights?color=d42010&label=release" alt="Latest release"></a>
  <img src="https://img.shields.io/badge/macOS-13%2B-d42010?logo=apple&logoColor=white" alt="macOS 13+">
  <img src="https://img.shields.io/badge/Apple%20Silicon%20%2B%20Intel-universal-d42010" alt="Universal binary">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-d42010" alt="MIT license"></a>
</p>

<p align="center"><b>English</b> · <a href="README.es.md">Español</a></p>

**RedLights** is a menu bar app that turns **every display pure red**: green and blue go to zero, and red dims as low as you want, even below the panel's minimum brightness. It's made for using a laptop next to the telescope without ruining your night vision: planetarium apps, capture software, guiding, charts…

One shortcut (**⌃⌥⌘R**) and you're done. No permissions, no screen capture, no overlay windows.

## Why red?

At night you see with the **rods** in your retina. They take 20–30 minutes to fully adapt to the dark, and a flash of white light undoes that in seconds. Rods barely respond to long-wavelength red light, so a **dim** red screen lets you read it without spoiling your dark adaptation. That's why astronomy flashlights are red.

Night Shift and dark mode aren't enough: they still emit green and blue, which is exactly what hurts night vision the most.

## Features

- 🔴 **Pure red** on every display at once, cursor included.
- 🌑 **Adjustable brightness** with a quadratic curve, for fine control in the dimmest range. It stacks with the Mac's brightness keys: with both at minimum, red goes lower than the panel alone allows.
- ⌨️ **Global shortcut ⌃⌥⌘R** from any app.
- 🔁 **Stays on**: reapplied when the Mac wakes, when displays are connected or disconnected, and every 2 s in case Night Shift or a color profile overrides it.
- 🛟 **Always reversible**: when you quit, or even if the app crashes, macOS restores normal colors.
- 🚀 **Open at login** option.
- 🌍 English and Spanish, following your system language.

## Install

### Homebrew (recommended)

```bash
brew install --cask vasallo94/tap/redlights
```

It installs into Applications, ready to open without the Gatekeeper warning, and updates with `brew upgrade`.

### Download

1. Get `RedLights-x.y.z.zip` from the [latest release](https://github.com/Vasallo94/redlights/releases/latest), unzip it and drag `RedLights.app` to **Applications**.
2. The app is ad-hoc signed and not notarized, so macOS blocks it the first time. Go to **System Settings → Privacy & Security** and click **Open Anyway**. Or, from Terminal:

   ```bash
   xattr -dr com.apple.quarantine /Applications/RedLights.app
   ```

Each release also ships a `.sha256` so you can check the download with `shasum -a 256 -c RedLights-x.y.z.zip.sha256`.

### Build from source

You need macOS 13+ and the Command Line Tools (`xcode-select --install`). Xcode is not required.

```bash
git clone https://github.com/Vasallo94/redlights.git
cd redlights
./build.sh
open RedLights.app
```

`./build.sh --zip` also produces a zip ready to distribute.

## Usage

| | |
|---|---|
| **⌃⌥⌘R** | Toggle red mode |
| Moon icon in the menu bar | Menu with the toggle, brightness slider and open at login |
| Mac brightness keys | Keep working and stack with the slider |

## How it works

macOS passes every color drawn on screen through a **gamma table** before sending it to the panel. RedLights replaces that table with `CGSetDisplayTransferByFormula`: the red channel ramps from 0 to your brightness level, and green and blue stay at 0. Because the transform happens at the very end, it affects everything you see (apps, video, cursor) without capturing the screen or drawing windows on top. That's why it needs no screen recording or accessibility permissions.

The table belongs to the process that set it: if RedLights exits for any reason, WindowServer restores the original gamma. You can't get stuck with a red screen.

All the code lives in a single file, [`Sources/main.swift`](Sources/main.swift), about 200 lines.

## Known limitations

- Screenshots come out with normal colors: the gamma table is applied after the image is composed.
- Displays in HDR or XDR reference modes may ignore or reset the gamma (not yet tested).
- On older Intel Macs the hardware cursor might bypass the gamma table and stay white. On Apple Silicon it turns red.

## Roadmap

- [ ] Linux (X11, GNOME, KDE, wlroots)
- [ ] Choose which displays go red (e.g. laptop red, external monitor off)
- [ ] CLI (`redlights on 0.1`) to script capture sessions
- [ ] Turn on automatically at astronomical twilight for your location

## Releasing

1. Add the changes to [`CHANGELOG.md`](CHANGELOG.md) under a `## [x.y.z] - YYYY-MM-DD` section.
2. `git tag vx.y.z && git push origin vx.y.z`

GitHub Actions builds the universal binary, creates the release with the zip and its SHA-256, and uses that CHANGELOG section as release notes. The version number comes from the tag; it isn't hardcoded anywhere. The [Homebrew tap](https://github.com/Vasallo94/homebrew-tap) picks up the new release automatically within a day.

## License

[MIT](LICENSE) © Vasallo94
