//! Un backend por mecanismo de cada sistema. Todos hacen lo mismo: aplicar el
//! filtro, restaurar lo que había y, para recuperarse, volver a colores normales.

use crate::Result;

#[cfg(target_os = "linux")]
mod mutter;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod wlr;
#[cfg(target_os = "linux")]
mod x11;

pub trait Backend {
    fn name(&self) -> &'static str;
    /// Aplica el filtro rojo con nivel `level` (0..=1). Debe poder repetirse.
    fn apply(&mut self, level: f32) -> Result<()>;
    /// Deja la pantalla como estaba antes del primer `apply`.
    fn restore(&mut self) -> Result<()>;
    /// Pone colores normales sin depender de un estado guardado (recuperación).
    fn reset(&mut self) -> Result<()>;
}

/// Elige el backend: `REDLIGHTS_BACKEND` si está definido; si no, lo detecta.
#[cfg(target_os = "linux")]
pub fn detect() -> Result<Box<dyn Backend>> {
    let env = |k| std::env::var_os(k).is_some_and(|v| !v.is_empty());
    match std::env::var("REDLIGHTS_BACKEND").as_deref() {
        Ok("x11") => return Ok(Box::new(x11::X11::new()?)),
        Ok("wlr") => {
            return Ok(Box::new(
                wlr::Wlr::new()?.ok_or("compositor doesn't support wlr-gamma-control")?,
            ));
        }
        Ok("mutter") => return Ok(Box::new(mutter::Mutter::new()?)),
        Ok(other) => return Err(format!("unknown backend '{other}' (x11, wlr, mutter)").into()),
        Err(_) => {}
    }

    if env("WAYLAND_DISPLAY") {
        // wlroots (Sway, Hyprland, labwc, Wayfire, niri…) primero; luego GNOME.
        if let Some(b) = wlr::Wlr::new()? {
            return Ok(Box::new(b));
        }
        return match mutter::Mutter::new() {
            Ok(b) => Ok(Box::new(b)),
            Err(_) => Err("this Wayland compositor isn't supported yet. \
                           Supported: wlroots-based compositors and GNOME; KDE Plasma is planned."
                .into()),
        };
    }
    if env("DISPLAY") {
        return Ok(Box::new(x11::X11::new()?));
    }
    Err("no graphical session found (neither WAYLAND_DISPLAY nor DISPLAY is set)".into())
}

#[cfg(windows)]
pub fn detect() -> Result<Box<dyn Backend>> {
    Ok(Box::new(windows::Magnifier::new()?))
}

#[cfg(not(any(target_os = "linux", windows)))]
pub fn detect() -> Result<Box<dyn Backend>> {
    Err("this platform isn't supported yet (on macOS, use the RedLights app)".into())
}
