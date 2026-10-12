//! GNOME (Mutter), en X11 o Wayland: D-Bus `org.gnome.Mutter.DisplayConfig`.
//!
//! Es una interfaz interna de GNOME, no un contrato público, pero Mutter 50 acepta
//! llamadas de terceros. Usamos `SetCrtcGamma`; `SetOutputCTM` (matriz de color,
//! ver `redlights::matrix`) queda para el modo luminancia.

use super::Backend;
use crate::Result;
use std::collections::HashMap;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedValue;

type Props = HashMap<String, OwnedValue>;
/// (id, winsys_id, x, y, ancho, alto, modo actual, transformación, transformaciones, props)
type Crtc = (u32, i64, i32, i32, i32, i32, i32, u32, Vec<u32>, Props);
type Output = (u32, i64, i32, Vec<u32>, String, Vec<u32>, Vec<u32>, Props);
type Mode = (u32, i64, u32, u32, f64, u32);
type Ramps = (Vec<u16>, Vec<u16>, Vec<u16>);

pub struct Mutter {
    proxy: Proxy<'static>,
    /// (crtc, rampas originales)
    saved: Vec<(u32, Ramps)>,
}

impl Mutter {
    pub fn new() -> Result<Self> {
        let conn = Connection::session()?;
        let proxy = Proxy::new(
            &conn,
            "org.gnome.Mutter.DisplayConfig",
            "/org/gnome/Mutter/DisplayConfig",
            "org.gnome.Mutter.DisplayConfig",
        )?;
        let mut me = Self {
            proxy,
            saved: Vec::new(),
        };
        let (serial, crtcs) = me.active_crtcs()?;
        if crtcs.is_empty() {
            return Err("Mutter: no active displays".into());
        }
        for crtc in crtcs {
            let ramps: Ramps = me.proxy.call("GetCrtcGamma", &(serial, crtc))?;
            me.saved.push((crtc, ramps));
        }
        Ok(me)
    }

    /// El serial cambia con cada reconfiguración de pantallas: se pide cada vez.
    fn active_crtcs(&self) -> Result<(u32, Vec<u32>)> {
        let (serial, crtcs, _, _, _, _): (u32, Vec<Crtc>, Vec<Output>, Vec<Mode>, i32, i32) =
            self.proxy.call("GetResources", &())?;
        Ok((
            serial,
            crtcs
                .into_iter()
                .filter(|c| c.6 >= 0)
                .map(|c| c.0)
                .collect(),
        ))
    }

    fn set_all(&self, ramps: impl Fn(u32, usize) -> Ramps) -> Result<()> {
        let (serial, crtcs) = self.active_crtcs()?;
        for crtc in crtcs {
            let size = self
                .saved
                .iter()
                .find(|(c, _)| *c == crtc)
                .map_or(256, |(_, r)| r.0.len());
            let (r, g, b) = ramps(crtc, size);
            let () = self.proxy.call("SetCrtcGamma", &(serial, crtc, r, g, b))?;
        }
        Ok(())
    }
}

impl Backend for Mutter {
    fn name(&self) -> &'static str {
        "GNOME Mutter"
    }

    fn apply(&mut self, level: f32) -> Result<()> {
        self.set_all(|_, size| redlights::red_ramps(size, level))
    }

    fn restore(&mut self) -> Result<()> {
        self.set_all(|crtc, size| {
            self.saved.iter().find(|(c, _)| *c == crtc).map_or_else(
                || redlights::identity_ramps(size), // pantalla conectada después
                |(_, r)| r.clone(),
            )
        })
    }

    fn reset(&mut self) -> Result<()> {
        self.set_all(|_, size| redlights::identity_ramps(size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Necesita Mutter en marcha (en CI: `mutter --x11` sobre Xvfb): `cargo test -- --ignored`.
    #[test]
    #[ignore = "needs a running Mutter"]
    fn apply_restore_reset_roundtrip() {
        let mut b = Mutter::new().unwrap();
        let read = |b: &Mutter| -> Vec<Ramps> {
            let (serial, crtcs) = b.active_crtcs().unwrap();
            crtcs
                .iter()
                .map(|c| b.proxy.call("GetCrtcGamma", &(serial, *c)).unwrap())
                .collect()
        };
        let original: Vec<Ramps> = b.saved.iter().map(|(_, r)| r.clone()).collect();

        b.apply(0.25).unwrap();
        for (r, g, bl) in read(&b) {
            assert_eq!(r, redlights::ramp(r.len(), 0.25));
            assert!(g.iter().chain(&bl).all(|&v| v == 0), "verde y azul a 0");
        }

        b.restore().unwrap();
        assert_eq!(read(&b), original, "restore devuelve la rampa original");

        b.reset().unwrap();
        for ramps in read(&b) {
            assert_eq!(
                ramps,
                redlights::identity_ramps(ramps.0.len()),
                "reset deja la identidad"
            );
        }
    }
}
