# redlights (Linux and Windows)

Command-line tool that turns every display red, for Linux (X11, GNOME, wlroots compositors) and Windows. Installation and usage are in the [main README](../README.md); design and testing notes in [`docs/design.md`](../docs/design.md).

```bash
cargo build --release
./target/release/redlights 0.5     # red until Ctrl-C, then restore
./target/release/redlights reset   # recover if a display was left red
```

`cargo test` runs the unit tests. `cargo test -- --ignored` also runs the integration tests, which need a display server: on Linux, an X server with Mutter running; on Windows, a desktop session. See [CONTRIBUTING.md](../CONTRIBUTING.md#linux-and-windows-tool).
