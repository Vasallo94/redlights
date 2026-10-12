//! Bandeja en Linux: icono StatusNotifierItem (ksni, D-Bus puro, sin GTK) y un
//! servicio D-Bus propio para `redlights toggle` y `redlights status`, pensado
//! para asignarlo a una tecla del escritorio (en Wayland las apps no pueden
//! registrar atajos globales).

use crate::Result;
use crate::app::{App, Command, Labels, labels, nearest_preset};
use ksni::blocking::TrayMethods;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const NAME: &str = "io.github.Vasallo94.RedLights";
const PATH: &str = "/io/github/Vasallo94/RedLights";

/// Servicio D-Bus de control. `status` devuelve (activo, brillo 0..1).
struct Control {
    tx: Sender<Command>,
    status: Arc<Mutex<(bool, f64)>>,
}

#[zbus::interface(name = "io.github.Vasallo94.RedLights")]
impl Control {
    fn toggle(&self) {
        let _ = self.tx.send(Command::Toggle);
    }
    fn set_brightness(&self, value: f64) {
        let _ = self.tx.send(Command::SetBrightness(value));
    }
    fn quit(&self) {
        let _ = self.tx.send(Command::Quit);
    }
    fn status(&self) -> (bool, f64) {
        *self.status.lock().unwrap()
    }
}

struct Tray {
    tx: Sender<Command>,
    enabled: bool,
    preset: usize,
    labels: Labels,
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        "redlights".into()
    }
    fn title(&self) -> String {
        "RedLights".into()
    }
    fn category(&self) -> ksni::Category {
        ksni::Category::Hardware
    }
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        [22, 32, 48]
            .into_iter()
            .map(|size| {
                // ksni quiere ARGB (orden de red); el núcleo da RGBA.
                let mut data = redlights::tray_icon_rgba(size, self.enabled);
                data.as_chunks_mut::<4>()
                    .0
                    .iter_mut()
                    .for_each(|p| p.rotate_right(1));
                ksni::Icon {
                    width: size as i32,
                    height: size as i32,
                    data,
                }
            })
            .collect()
    }
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(Command::Toggle); // clic en el icono = activar/desactivar
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let options = redlights::BRIGHTNESS_PRESETS
            .iter()
            .map(|&v| RadioItem {
                label: self.labels.percent(v),
                ..Default::default()
            })
            .collect();
        vec![
            CheckmarkItem {
                label: self.labels.red_mode.into(),
                checked: self.enabled,
                activate: Box::new(|t: &mut Self| {
                    let _ = t.tx.send(Command::Toggle);
                }),
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: self.labels.brightness.into(),
                submenu: vec![
                    RadioGroup {
                        selected: self.preset,
                        select: Box::new(|t: &mut Self, i| {
                            let _ =
                                t.tx.send(Command::SetBrightness(redlights::BRIGHTNESS_PRESETS[i]));
                        }),
                        options,
                    }
                    .into(),
                ],
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: self.labels.quit.into(),
                activate: Box::new(|t: &mut Self| {
                    let _ = t.tx.send(Command::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn run() -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let mut app = App::new();
    let status = Arc::new(Mutex::new((false, app.slider)));

    // Reservar el nombre en D-Bus sin hacer cola ni quitárselo a nadie: si ya hay
    // otra bandeja abierta, esta no arranca.
    let control = zbus::blocking::connection::Builder::session()?
        .serve_at(
            PATH,
            Control {
                tx: tx.clone(),
                status: status.clone(),
            },
        )?
        .build()?;
    let already = "RedLights is already running in the tray";
    match control.request_name_with_flags(NAME, zbus::fdo::RequestNameFlags::DoNotQueue.into()) {
        Ok(zbus::fdo::RequestNameReply::PrimaryOwner) => {}
        Ok(_) | Err(zbus::Error::NameTaken) => return Err(already.into()),
        Err(e) => return Err(e.into()),
    }

    let lang = crate::user_language();
    // Sin panel con bandeja (p. ej. GNOME sin la extensión AppIndicator) el icono no
    // se ve, pero `redlights toggle` sigue funcionando.
    let tray = Tray {
        tx: tx.clone(),
        enabled: false,
        preset: nearest_preset(app.slider),
        labels: labels(&lang),
    }
    .assume_sni_available(true)
    .spawn()?;

    let quit = tx.clone();
    ctrlc::set_handler(move || {
        let _ = quit.send(Command::Quit);
    })?;
    println!(
        "RedLights is running in the system tray. Toggle with the tray icon or `redlights toggle`."
    );

    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(cmd) => {
                let keep = app.handle(cmd).unwrap_or_else(|e| {
                    eprintln!("redlights: {e}");
                    true
                });
                let (enabled, slider) = (app.enabled(), app.slider);
                *status.lock().unwrap() = (enabled, slider);
                tray.update(|t| {
                    t.enabled = enabled;
                    t.preset = nearest_preset(slider);
                });
                if !keep {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => app.tick(),
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    drop(app); // restaura los colores antes de cerrar
    tray.shutdown().wait();
    Ok(())
}

fn proxy() -> Result<zbus::blocking::Proxy<'static>> {
    let conn = zbus::blocking::Connection::session()?;
    Ok(zbus::blocking::Proxy::new(&conn, NAME, PATH, NAME)?)
}

/// Llama a un método de la bandeja en marcha, con un error claro si no lo está.
pub fn call<R: serde::de::DeserializeOwned + zbus::zvariant::Type>(method: &str) -> Result<R> {
    proxy()?.call(method, &()).map_err(|e| match e {
        zbus::Error::MethodError(name, _, _)
            if name.as_str() == "org.freedesktop.DBus.Error.ServiceUnknown" =>
        {
            "RedLights isn't running in the tray. Start it with `redlights tray`.".into()
        }
        e => e.into(),
    })
}
