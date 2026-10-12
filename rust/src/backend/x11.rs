//! X11: rampas gamma por CRTC con la extensión RandR.
//!
//! En X11 la rampa NO se restaura sola si el proceso muere: por eso guardamos la
//! original y `redlights reset` existe para recuperarse de un cierre brusco.

use super::Backend;
use crate::Result;
use x11rb::connection::Connection;
use x11rb::protocol::randr::{ConnectionExt as _, Crtc};
use x11rb::rust_connection::RustConnection;

pub struct X11 {
    conn: RustConnection,
    /// (crtc, tamaño de la rampa, rampas originales r/g/b)
    crtcs: Vec<(Crtc, usize, [Vec<u16>; 3])>,
}

impl X11 {
    pub fn new() -> Result<Self> {
        let (conn, screen) = x11rb::connect(None)?;
        let root = conn.setup().roots[screen].root;
        let res = conn.randr_get_screen_resources_current(root)?.reply()?;
        let mut crtcs = Vec::new();
        for crtc in res.crtcs {
            let size = conn.randr_get_crtc_gamma_size(crtc)?.reply()?.size as usize;
            if size == 0 {
                continue;
            }
            let g = conn.randr_get_crtc_gamma(crtc)?.reply()?;
            crtcs.push((crtc, size, [g.red, g.green, g.blue]));
        }
        if crtcs.is_empty() {
            return Err("X11: no display supports gamma ramps".into());
        }
        Ok(Self { conn, crtcs })
    }

    fn set_all(
        &self,
        ramps: impl Fn(usize, usize) -> (Vec<u16>, Vec<u16>, Vec<u16>),
    ) -> Result<()> {
        for (i, (crtc, size, _)) in self.crtcs.iter().enumerate() {
            let (r, g, b) = ramps(i, *size);
            self.conn.randr_set_crtc_gamma(*crtc, &r, &g, &b)?.check()?;
        }
        Ok(())
    }
}

impl Backend for X11 {
    fn name(&self) -> &'static str {
        "X11 RandR"
    }

    fn apply(&mut self, level: f32) -> Result<()> {
        self.set_all(|_, size| redlights::red_ramps(size, level))
    }

    fn restore(&mut self) -> Result<()> {
        self.set_all(|i, _| {
            let [r, g, b] = &self.crtcs[i].2;
            (r.clone(), g.clone(), b.clone())
        })
    }

    fn reset(&mut self) -> Result<()> {
        self.set_all(|_, size| redlights::identity_ramps(size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Necesita un servidor X (p. ej. Xvfb): `cargo test -- --ignored`.
    /// Todo en una conexión: Xvfb devuelve basura si se lee la rampa después de
    /// que el cliente que la fijó se haya desconectado.
    #[test]
    #[ignore = "needs an X server"]
    fn apply_restore_reset_roundtrip() {
        let mut b = X11::new().unwrap();
        let read = |b: &X11| -> Vec<[Vec<u16>; 3]> {
            b.crtcs
                .iter()
                .map(|(c, _, _)| {
                    let g = b.conn.randr_get_crtc_gamma(*c).unwrap().reply().unwrap();
                    [g.red, g.green, g.blue]
                })
                .collect()
        };

        // Punto de partida conocido: una rampa que no es ni la identidad ni roja.
        b.set_all(|_, n| {
            (
                redlights::ramp(n, 0.9),
                redlights::ramp(n, 0.8),
                redlights::ramp(n, 0.7),
            )
        })
        .unwrap();
        b.crtcs = X11::new().unwrap().crtcs; // re-guardar esa rampa como "original"
        let original = read(&b);

        b.apply(0.25).unwrap();
        for (crtc, [r, g, bl]) in b.crtcs.iter().zip(read(&b)) {
            assert_eq!(r, redlights::ramp(crtc.1, 0.25));
            assert!(g.iter().chain(&bl).all(|&v| v == 0), "verde y azul a 0");
        }

        b.restore().unwrap();
        assert_eq!(read(&b), original, "restore devuelve la rampa original");

        b.reset().unwrap();
        for (crtc, [r, g, bl]) in b.crtcs.iter().zip(read(&b)) {
            let id = redlights::ramp(crtc.1, 1.0);
            assert_eq!(
                (r, g, bl),
                (id.clone(), id.clone(), id),
                "reset deja la identidad"
            );
        }
    }
}
