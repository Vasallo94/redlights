//! wlroots (Sway, Hyprland, labwc, Wayfire, river, niri…): protocolo
//! `wlr-gamma-control-unstable-v1`.
//!
//! El compositor restaura la gamma original en cuanto se destruye el control o
//! el proceso muere, igual que en macOS: no hace falta guardar nada.

use super::Backend;
use crate::Result;
use rustix::fs::{MemfdFlags, memfd_create};
use std::io::{Seek, SeekFrom, Write};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::{wl_output::WlOutput, wl_registry::WlRegistry};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_wlr::gamma_control::v1::client::{
    zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1,
    zwlr_gamma_control_v1::{self, ZwlrGammaControlV1},
};

struct Control {
    ctrl: ZwlrGammaControlV1,
    size: usize,
    failed: bool,
}

#[derive(Default)]
struct State {
    controls: Vec<Control>,
}

pub struct Wlr {
    queue: EventQueue<State>,
    state: State,
}

impl Wlr {
    /// `Ok(None)` si el compositor no implementa el protocolo (no es wlroots).
    pub fn new() -> Result<Option<Self>> {
        let conn = Connection::connect_to_env()?;
        let (globals, mut queue) = registry_queue_init::<State>(&conn)?;
        let qh = queue.handle();
        let Ok(manager) = globals.bind::<ZwlrGammaControlManagerV1, _, _>(&qh, 1..=1, ()) else {
            return Ok(None);
        };

        let mut state = State::default();
        let outputs: Vec<WlOutput> = globals.contents().with_list(|list| {
            list.iter()
                .filter(|g| g.interface == "wl_output")
                .map(|g| globals.registry().bind(g.name, g.version.min(4), &qh, ()))
                .collect()
        });
        for output in &outputs {
            let idx = state.controls.len();
            let ctrl = manager.get_gamma_control(output, &qh, idx);
            state.controls.push(Control { ctrl, size: 0, failed: false });
        }
        queue.roundtrip(&mut state)?; // recibe gamma_size o failed de cada salida

        let usable = state.controls.iter().filter(|c| !c.failed && c.size > 0).count();
        if usable == 0 {
            return Err("the compositor supports wlr-gamma-control, but no display accepts gamma tables".into());
        }
        Ok(Some(Self { queue, state }))
    }
}

impl Backend for Wlr {
    fn name(&self) -> &'static str {
        "wlr-gamma-control"
    }

    fn apply(&mut self, level: f32) -> Result<()> {
        for c in self.state.controls.iter().filter(|c| !c.failed && c.size > 0) {
            // Tabla: size×u16 rojo, luego verde, luego azul, en orden de bytes nativo.
            let (r, g, b) = redlights::red_ramps(c.size, level);
            let bytes: Vec<u8> = r.iter().chain(&g).chain(&b).flat_map(|v| v.to_ne_bytes()).collect();
            let fd = memfd_create("redlights-gamma", MemfdFlags::CLOEXEC)?;
            let mut file = std::fs::File::from(fd);
            file.write_all(&bytes)?;
            file.seek(SeekFrom::Start(0))?; // el compositor comparte este offset al leer
            c.ctrl.set_gamma(std::os::fd::AsFd::as_fd(&file));
        }
        self.queue.roundtrip(&mut self.state)?;
        if self.state.controls.iter().any(|c| c.failed && c.size > 0) {
            return Err("compositor rejected the gamma table (another app may own it)".into());
        }
        Ok(())
    }

    fn restore(&mut self) -> Result<()> {
        for c in &self.state.controls {
            c.ctrl.destroy(); // el compositor vuelve a la gamma original
        }
        self.state.controls.clear();
        self.queue.roundtrip(&mut self.state)?;
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        self.restore() // aquí no puede quedarse en rojo: basta con soltar el control
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for State {
    fn event(_: &mut Self, _: &WlRegistry, _: <WlRegistry as wayland_client::Proxy>::Event,
             _: &GlobalListContents, _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<WlOutput, ()> for State {
    fn event(_: &mut Self, _: &WlOutput, _: <WlOutput as wayland_client::Proxy>::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<ZwlrGammaControlManagerV1, ()> for State {
    fn event(_: &mut Self, _: &ZwlrGammaControlManagerV1,
             _: <ZwlrGammaControlManagerV1 as wayland_client::Proxy>::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<ZwlrGammaControlV1, usize> for State {
    fn event(state: &mut Self, _: &ZwlrGammaControlV1, event: zwlr_gamma_control_v1::Event,
             idx: &usize, _: &Connection, _: &QueueHandle<Self>) {
        let Some(c) = state.controls.get_mut(*idx) else { return };
        match event {
            zwlr_gamma_control_v1::Event::GammaSize { size } => c.size = size as usize,
            zwlr_gamma_control_v1::Event::Failed => c.failed = true,
            _ => {}
        }
    }
}
