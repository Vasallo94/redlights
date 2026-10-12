//! Windows 8+: matriz de color de pantalla completa de la API de Magnificación
//! (`MagSetFullscreenColorEffect`). Solo 64 bits: la API no funciona bajo WOW64.

use super::Backend;
use crate::Result;
use windows_sys::Win32::UI::Magnification::{
    MAGCOLOREFFECT, MagInitialize, MagSetFullscreenColorEffect, MagUninitialize,
};

const IDENTITY: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

pub struct Magnifier;

impl Magnifier {
    pub fn new() -> Result<Self> {
        // SAFETY: sin argumentos; se empareja con MagUninitialize en Drop.
        if unsafe { MagInitialize() } == 0 {
            return Err(
                format!("MagInitialize failed: {}", std::io::Error::last_os_error()).into(),
            );
        }
        Ok(Self)
    }

    fn set(&self, m: [[f32; 3]; 3]) -> Result<()> {
        let effect = MAGCOLOREFFECT {
            transform: redlights::row_vector_5x5(m),
        };
        // SAFETY: `effect` vive durante la llamada y la función solo lo lee.
        if unsafe { MagSetFullscreenColorEffect(&effect) } == 0 {
            return Err(format!(
                "MagSetFullscreenColorEffect failed: {}",
                std::io::Error::last_os_error()
            )
            .into());
        }
        Ok(())
    }

    #[cfg(test)]
    fn current(&self) -> Result<[f32; 25]> {
        use windows_sys::Win32::UI::Magnification::MagGetFullscreenColorEffect;
        let mut effect = MAGCOLOREFFECT {
            transform: [0.0; 25],
        };
        // SAFETY: `effect` es un MAGCOLOREFFECT válido donde escribir.
        if unsafe { MagGetFullscreenColorEffect(&mut effect) } == 0 {
            return Err(format!(
                "MagGetFullscreenColorEffect failed: {}",
                std::io::Error::last_os_error()
            )
            .into());
        }
        Ok(effect.transform)
    }
}

impl Drop for Magnifier {
    fn drop(&mut self) {
        // SAFETY: emparejado con el MagInitialize de `new`.
        unsafe { MagUninitialize() };
    }
}

impl Backend for Magnifier {
    fn name(&self) -> &'static str {
        "Windows Magnification"
    }

    fn apply(&mut self, level: f32) -> Result<()> {
        self.set(redlights::matrix(level, redlights::Mode::Channel))
    }

    fn restore(&mut self) -> Result<()> {
        self.set(IDENTITY)
    }

    fn reset(&mut self) -> Result<()> {
        self.set(IDENTITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Necesita una sesión de escritorio de Windows (los runners de GitHub la tienen).
    #[test]
    #[ignore = "needs a Windows desktop session"]
    fn apply_restore_roundtrip() {
        let mut b = Magnifier::new().unwrap();
        b.apply(0.25).unwrap();
        let expected = redlights::row_vector_5x5(redlights::matrix(0.25, redlights::Mode::Channel));
        assert_eq!(b.current().unwrap(), expected);

        b.restore().unwrap();
        assert_eq!(b.current().unwrap(), redlights::row_vector_5x5(IDENTITY));
    }

    /// Informativo: muestra el efecto activo (para ver si sobrevive a un proceso muerto).
    #[test]
    #[ignore = "needs a Windows desktop session"]
    fn print_current_effect() {
        let b = Magnifier::new().unwrap();
        println!("current effect: {:?}", b.current().unwrap());
    }
}
