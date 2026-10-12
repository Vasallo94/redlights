//! CLI de RedLights para Linux (Windows, más adelante).
//!
//!     redlights [on] [BRILLO]   pone las pantallas en rojo hasta Ctrl-C o SIGTERM
//!     redlights reset           devuelve los colores normales si se quedaron en rojo

mod backend;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Cada cuánto se reafirma el filtro por si Night Light u otra app lo pisan.
const REAPPLY: Duration = Duration::from_secs(2);

const USAGE: &str = "\
Usage:
  redlights [on] [BRIGHTNESS]   turn every display red until Ctrl-C (BRIGHTNESS 0..1, default 0.5)
  redlights reset               restore normal colors if a display got stuck red
Env:
  REDLIGHTS_BACKEND=x11|wlr|mutter   force a backend instead of auto-detecting";

fn main() {
    if let Err(e) = run() {
        eprintln!("redlights: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["-h" | "--help"] => println!("{USAGE}"),
        ["-V" | "--version"] => println!("redlights {}", env!("CARGO_PKG_VERSION")),
        ["reset"] => {
            let mut b = backend::detect()?;
            b.reset()?;
            println!("Colors restored ({})", b.name());
        }
        [] | ["on"] => on(0.5)?,
        [v] | ["on", v] => on(v.parse().map_err(|_| format!("invalid brightness '{v}'\n\n{USAGE}"))?)?,
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

fn on(slider: f64) -> Result<()> {
    if !(0.0..=1.0).contains(&slider) {
        return Err(format!("brightness must be between 0 and 1, got {slider}").into());
    }
    let level = redlights::level_from_slider(slider);
    let mut b = backend::detect()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst))?;

    b.apply(level)?;
    println!("Red mode on at {:.0}% ({}). Ctrl-C to restore.", slider * 100.0, b.name());

    let mut last = Instant::now();
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
        if last.elapsed() >= REAPPLY {
            if let Err(e) = b.apply(level) {
                eprintln!("redlights: reapply failed: {e}");
            }
            last = Instant::now();
        }
    }

    b.restore()?;
    println!("\nColors restored.");
    Ok(())
}
