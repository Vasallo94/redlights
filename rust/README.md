# redlights (Linux CLI)

Rust port of RedLights for Linux (Windows later). See [`docs/linux.md`](../docs/linux.md) for the design and status.

```bash
cargo build --release
./target/release/redlights 0.5     # red until Ctrl-C, then restore
./target/release/redlights reset   # recover if a display got stuck red
```

Backends: X11 (RandR), wlroots compositors (`wlr-gamma-control`) and GNOME (Mutter D-Bus). KDE Plasma is planned.

Tests: `cargo test` runs the core tests. `cargo test -- --ignored` also runs the X11 and Mutter integration tests and needs an X server with Mutter running (see the `linux` job in `.github/workflows/build.yml`).
