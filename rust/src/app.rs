//! Lógica de la app de bandeja, común a Linux y Windows: el estado (activo y
//! brillo), las órdenes que llegan del menú, del atajo o de `redlights toggle`,
//! y el brillo guardado entre arranques.

use crate::Result;
use crate::backend::{self, Backend};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Toggle,
    SetBrightness(f64),
    Quit,
}

pub struct App {
    backend: Option<Box<dyn Backend>>,
    pub slider: f64,
    last_apply: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            backend: None,
            slider: load_brightness(),
            last_apply: Instant::now(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.backend.is_some()
    }

    /// Aplica una orden. Devuelve `false` si hay que salir.
    pub fn handle(&mut self, cmd: Command) -> Result<bool> {
        match cmd {
            Command::Toggle if self.enabled() => self.disable()?,
            Command::Toggle => {
                // El backend se crea al activar y se suelta al desactivar: en wlroots,
                // soltar el control es lo que devuelve la gamma original.
                let mut b = backend::detect()?;
                b.apply(redlights::level_from_slider(self.slider))?;
                self.backend = Some(b);
                self.last_apply = Instant::now();
            }
            Command::SetBrightness(v) => {
                self.slider = v.clamp(0.0, 1.0);
                save_brightness(self.slider);
                if let Some(b) = &mut self.backend {
                    b.apply(redlights::level_from_slider(self.slider))?;
                }
            }
            Command::Quit => {
                self.disable()?;
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Reafirma el filtro cada 2 s por si Night Light u otra app lo pisan.
    pub fn tick(&mut self) {
        if self.last_apply.elapsed() < Duration::from_secs(2) {
            return;
        }
        self.last_apply = Instant::now();
        if let Some(b) = &mut self.backend
            && let Err(e) = b.apply(redlights::level_from_slider(self.slider))
        {
            eprintln!("redlights: reapply failed: {e}");
        }
    }

    fn disable(&mut self) -> Result<()> {
        if let Some(mut b) = self.backend.take() {
            b.restore()?;
        }
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.disable();
    }
}

/// Índice del valor de `BRIGHTNESS_PRESETS` más cercano al brillo actual.
pub fn nearest_preset(slider: f64) -> usize {
    let dist = |i: usize| (redlights::BRIGHTNESS_PRESETS[i] - slider).abs();
    (0..redlights::BRIGHTNESS_PRESETS.len())
        .min_by(|&a, &b| dist(a).total_cmp(&dist(b)))
        .unwrap_or(0)
}

fn config_file() -> Option<PathBuf> {
    let dir = if cfg!(windows) {
        PathBuf::from(std::env::var_os("APPDATA")?)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| Some(PathBuf::from(std::env::var_os("HOME")?).join(".config")))?
    };
    Some(dir.join("redlights").join("brightness"))
}

fn load_brightness() -> f64 {
    config_file()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| s.trim().parse().ok())
        .filter(|v: &f64| (0.0..=1.0).contains(v))
        .unwrap_or(0.5)
}

fn save_brightness(v: f64) {
    if let Some(p) = config_file() {
        let _ = p.parent().map(std::fs::create_dir_all);
        let _ = std::fs::write(p, v.to_string());
    }
}

/// Textos del menú en el idioma del sistema (español o inglés).
pub struct Labels {
    spanish: bool,
    pub red_mode: &'static str,
    pub brightness: &'static str,
    pub quit: &'static str,
}

pub fn labels(lang: &str) -> Labels {
    if lang.to_lowercase().starts_with("es") {
        Labels {
            spanish: true,
            red_mode: "Modo rojo",
            brightness: "Brillo",
            quit: "Salir",
        }
    } else {
        Labels {
            spanish: false,
            red_mode: "Red mode",
            brightness: "Brightness",
            quit: "Quit",
        }
    }
}

impl Labels {
    /// «50 %» en español, «50%» en inglés.
    pub fn percent(&self, slider: f64) -> String {
        let n = (slider * 100.0).round() as i32;
        if self.spanish {
            format!("{n} %")
        } else {
            format!("{n}%")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_and_labels() {
        assert_eq!(redlights::BRIGHTNESS_PRESETS[nearest_preset(0.5)], 0.5);
        assert_eq!(redlights::BRIGHTNESS_PRESETS[nearest_preset(0.27)], 0.3);
        assert_eq!(nearest_preset(-3.0), 0);
        assert_eq!(labels("es_ES.UTF-8").quit, "Salir");
        assert_eq!(labels("en-US").quit, "Quit");
        assert_eq!(labels("").quit, "Quit");
        assert_eq!(labels("es").percent(0.5), "50 %");
        assert_eq!(labels("en").percent(0.75), "75%");
    }
}
