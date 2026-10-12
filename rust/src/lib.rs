//! Núcleo de RedLights, común a todas las plataformas.
//!
//! El filtro se expresa de dos formas porque cada sistema admite una u otra:
//! - **rampas gamma** por canal (X11, wlroots, Mutter, macOS): rojo de 0 a `level`,
//!   verde y azul a 0;
//! - **matriz de color** 3×3 (Windows, Mutter CTM, Hyprland): permite además el modo
//!   luminancia, en el que lo puramente verde o azul no se vuelve negro.

/// Brillo mínimo: por debajo, la pantalla queda prácticamente negra.
pub const MIN_LEVEL: f32 = 0.02;

/// Pasa la posición del regulador (0..=1) a nivel de rojo con curva cuadrática,
/// para tener más precisión en la zona tenue. Igual que la app de macOS.
pub fn level_from_slider(slider: f64) -> f32 {
    let v = slider.clamp(0.0, 1.0);
    ((v * v) as f32).max(MIN_LEVEL)
}

/// Rampa lineal de 16 bits de 0 a `level` (0..=1) con `size` entradas.
pub fn ramp(size: usize, level: f32) -> Vec<u16> {
    if size < 2 {
        return vec![0; size];
    }
    let max = f64::from(level.clamp(0.0, 1.0)) * 65535.0;
    (0..size)
        .map(|i| (max * i as f64 / (size - 1) as f64).round() as u16)
        .collect()
}

/// Rampas (rojo, verde, azul) del filtro rojo.
pub fn red_ramps(size: usize, level: f32) -> (Vec<u16>, Vec<u16>, Vec<u16>) {
    (ramp(size, level), vec![0; size], vec![0; size])
}

/// Rampas sin filtro, para recuperar una pantalla que se quedó roja.
pub fn identity_ramps(size: usize) -> (Vec<u16>, Vec<u16>, Vec<u16>) {
    let r = ramp(size, 1.0);
    (r.clone(), r.clone(), r)
}

/// Cómo se reparte el color de entrada en el rojo de salida.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// Solo el canal rojo, como las rampas gamma: el azul puro sale negro.
    Channel,
    /// Rojo según la luminancia (Rec. 709): todo sigue siendo legible.
    Luma,
}

/// Matriz 3×3 por filas (salida = M · entrada RGB) del filtro rojo.
pub fn matrix(level: f32, mode: Mode) -> [[f32; 3]; 3] {
    let l = level.clamp(0.0, 1.0);
    let red_row = match mode {
        Mode::Channel => [l, 0.0, 0.0],
        Mode::Luma => [0.2126 * l, 0.7152 * l, 0.0722 * l],
    };
    [red_row, [0.0; 3], [0.0; 3]]
}

/// Matriz 5×5 en el formato de GDI+ y de la API de Magnificación de Windows:
/// vector fila (salida = entrada · M), así que cada **fila** es un canal de
/// entrada. Es la traspuesta de [`matrix`], ampliada con alfa y traslación.
pub fn row_vector_5x5(m: [[f32; 3]; 3]) -> [f32; 25] {
    let mut t = [0.0; 25];
    for input in 0..3 {
        for output in 0..3 {
            t[input * 5 + output] = m[output][input];
        }
    }
    t[18] = 1.0; // alfa → alfa
    t[24] = 1.0; // fila de traslación
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_curve() {
        assert_eq!(level_from_slider(1.0), 1.0);
        assert_eq!(level_from_slider(0.5), 0.25);
        assert_eq!(level_from_slider(0.0), MIN_LEVEL);
        assert_eq!(level_from_slider(7.0), 1.0); // fuera de rango se recorta
    }

    #[test]
    fn ramp_ends() {
        let r = ramp(256, 1.0);
        assert_eq!((r[0], r[255]), (0, 65535));
        assert_eq!(ramp(256, 0.5)[255], 32768);
        assert_eq!(ramp(1, 1.0), vec![0]);
        assert!(ramp(1024, 1.0).windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn red_ramps_zero_green_blue() {
        let (r, g, b) = red_ramps(256, 0.3);
        assert!(r[255] > 0);
        assert!(g.iter().chain(&b).all(|&v| v == 0));
    }

    #[test]
    fn row_vector_is_transposed() {
        let id = row_vector_5x5([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
        let expected_id: [f32; 25] = std::array::from_fn(|i| if i % 6 == 0 { 1.0 } else { 0.0 });
        assert_eq!(id, expected_id);
        // Luminancia → rojo: la columna 0 (salida roja) lleva los pesos; el resto a 0
        let t = row_vector_5x5(matrix(1.0, Mode::Luma));
        assert_eq!([t[0], t[5], t[10]], [0.2126, 0.7152, 0.0722]);
        assert_eq!([t[1], t[2], t[6], t[7], t[11], t[12]], [0.0; 6]);
    }

    #[test]
    fn luma_row_sums_to_level() {
        let m = matrix(0.5, Mode::Luma);
        assert!((m[0].iter().sum::<f32>() - 0.5).abs() < 1e-6); // blanco → rojo al 50 %
        assert_eq!(m[1], [0.0; 3]);
        assert_eq!(m[2], [0.0; 3]);
        assert_eq!(matrix(1.0, Mode::Channel)[0], [1.0, 0.0, 0.0]);
    }
}
