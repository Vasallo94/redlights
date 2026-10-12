//! RedLights para Linux y Windows.
//!
//!     redlights [on] [BRILLO]   pone las pantallas en rojo hasta Ctrl-C o SIGTERM
//!     redlights tray            icono en la bandeja (Windows: también con doble clic)
//!     redlights toggle|status   controla la bandeja en marcha (Linux)
//!     redlights reset           devuelve los colores normales si se quedaron en rojo

#[cfg(any(target_os = "linux", windows))]
mod app;
mod backend;
#[cfg(target_os = "linux")]
mod tray_linux;
#[cfg(windows)]
mod tray_windows;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Versión del tag de git, que pasa el CI al compilar; si no, la de Cargo.toml.
const VERSION: &str = match option_env!("REDLIGHTS_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

/// Cada cuánto se reafirma el filtro por si Night Light u otra app lo pisan.
const REAPPLY: Duration = Duration::from_secs(2);

const USAGE: &str = "\
Usage:
  redlights [on] [BRIGHTNESS]   turn every display red until Ctrl-C (BRIGHTNESS 0..1, default 0.5)
  redlights tray                run in the system tray (on Windows, also by double-clicking)
  redlights toggle              turn red mode on or off in the running tray app (Linux)
  redlights status              show whether red mode is on (Linux)
  redlights reset               restore normal colors if a display got stuck red
Env:
  REDLIGHTS_BACKEND=x11|wlr|mutter   force a backend instead of auto-detecting";

fn main() {
    // Windows: con doble clic, la consola es solo nuestra. Se oculta y arranca la bandeja.
    #[cfg(windows)]
    if std::env::args().len() == 1 && launched_by_double_click() {
        // SAFETY: suelta la consola de este proceso; no hay otros usuarios de ella.
        unsafe { windows_sys::Win32::System::Console::FreeConsole() };
        if let Err(e) = tray_windows::run() {
            message_box(&format!("RedLights couldn't start: {e}"));
        }
        return;
    }
    if let Err(e) = run() {
        eprintln!("redlights: {e}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn launched_by_double_click() -> bool {
    let mut pids = [0u32; 2];
    // SAFETY: el buffer tiene espacio para 2 ids, como se indica.
    unsafe { windows_sys::Win32::System::Console::GetConsoleProcessList(pids.as_mut_ptr(), 2) == 1 }
}

#[cfg(windows)]
fn message_box(text: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxW};
    let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
    // SAFETY: cadenas UTF-16 terminadas en nulo que viven durante la llamada.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            wide(text).as_ptr(),
            wide("RedLights").as_ptr(),
            MB_ICONERROR,
        )
    };
}

/// Idioma de la interfaz del usuario (p. ej. "es-ES" o "en_US.UTF-8").
#[cfg(windows)]
pub fn user_language() -> String {
    let mut buf = [0u16; 85]; // LOCALE_NAME_MAX_LENGTH
    // SAFETY: el buffer tiene el tamaño indicado.
    let n = unsafe {
        windows_sys::Win32::Globalization::GetUserDefaultLocaleName(
            buf.as_mut_ptr(),
            buf.len() as i32,
        )
    };
    String::from_utf16_lossy(&buf[..n.max(1) as usize - 1])
}

#[cfg(not(windows))]
pub fn user_language() -> String {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
        .unwrap_or_default()
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["-h" | "--help"] => println!("{USAGE}"),
        ["-V" | "--version"] => println!("redlights {VERSION}"),
        #[cfg(target_os = "linux")]
        ["tray"] => tray_linux::run()?,
        #[cfg(windows)]
        ["tray"] => tray_windows::run()?,
        #[cfg(target_os = "linux")]
        ["toggle"] => {
            tray_linux::call::<()>("Toggle")?;
            print_status()?;
        }
        #[cfg(target_os = "linux")]
        ["status"] => print_status()?,
        #[cfg(windows)]
        ["toggle" | "status"] => {
            return Err("on Windows, use the tray icon or Ctrl+Alt+Shift+R".into());
        }
        ["reset"] => {
            let mut b = backend::detect()?;
            b.reset()?;
            println!("Colors restored ({})", b.name());
        }
        [] | ["on"] => on(0.5)?,
        [v] | ["on", v] => on(v
            .parse()
            .map_err(|_| format!("invalid brightness '{v}'\n\n{USAGE}"))?)?,
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn print_status() -> Result<()> {
    std::thread::sleep(Duration::from_millis(200)); // la bandeja aplica la orden en su bucle
    let (on, slider): (bool, f64) = tray_linux::call("Status")?;
    if on {
        println!("on at {:.0}%", slider * 100.0);
    } else {
        println!("off");
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
    println!(
        "Red mode on at {:.0}% ({}). Ctrl-C to restore.",
        slider * 100.0,
        b.name()
    );

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
