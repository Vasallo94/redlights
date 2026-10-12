//! Bandeja en Windows: icono y menú nativos (tray-icon) y atajo global
//! Ctrl+Alt+Shift+R (global-hotkey). Ambos entregan sus eventos a través del bucle
//! de mensajes de Windows de este hilo, que se bombea aquí.

use crate::Result;
use crate::app::{App, Command, labels, nearest_preset};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::sync::mpsc;
use std::time::Duration;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
};

fn icon(on: bool) -> Result<Icon> {
    Ok(Icon::from_rgba(redlights::tray_icon_rgba(32, on), 32, 32)?)
}

pub fn run() -> Result<()> {
    let mut app = App::new();
    let l = labels(&crate::user_language());

    let red = CheckMenuItem::new(
        format!("{}\tCtrl+Alt+Shift+R", l.red_mode),
        true,
        false,
        None,
    );
    let presets: Vec<CheckMenuItem> = redlights::BRIGHTNESS_PRESETS
        .iter()
        .map(|&v| CheckMenuItem::new(l.percent(v), true, false, None))
        .collect();
    let preset_refs: Vec<&dyn tray_icon::menu::IsMenuItem> =
        presets.iter().map(|p| p as _).collect();
    let brightness = Submenu::with_items(l.brightness, true, &preset_refs)?;
    let quit = MenuItem::new(l.quit, true, None);
    let menu = Menu::with_items(&[&red, &brightness, &PredefinedMenuItem::separator(), &quit])?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false) // clic izquierdo = activar/desactivar; derecho = menú
        .with_tooltip("RedLights")
        .with_icon(icon(false)?)
        .build()?;

    let hotkeys = GlobalHotKeyManager::new()?;
    let hotkey = HotKey::new(
        Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
        Code::KeyR,
    );
    if let Err(e) = hotkeys.register(hotkey) {
        eprintln!("redlights: couldn't register Ctrl+Alt+Shift+R ({e}); use the tray icon");
    }

    let (quit_tx, quit_rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = quit_tx.send(());
    })?;

    let refresh = |app: &App| -> Result<()> {
        red.set_checked(app.enabled());
        let selected = nearest_preset(app.slider);
        presets
            .iter()
            .enumerate()
            .for_each(|(i, p)| p.set_checked(i == selected));
        tray.set_icon(Some(icon(app.enabled())?))?;
        Ok(())
    };
    refresh(&app)?;

    loop {
        // Bombear mensajes: así llegan los eventos del menú, del icono y del atajo.
        // SAFETY: MSG es una estructura plana; estas llamadas solo la leen y escriben.
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        let mut commands = Vec::new();
        while let Ok(e) = MenuEvent::receiver().try_recv() {
            if e.id() == red.id() {
                commands.push(Command::Toggle);
            } else if e.id() == quit.id() {
                commands.push(Command::Quit);
            } else if let Some(i) = presets.iter().position(|p| p.id() == e.id()) {
                commands.push(Command::SetBrightness(redlights::BRIGHTNESS_PRESETS[i]));
            }
        }
        while let Ok(e) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = e
            {
                commands.push(Command::Toggle);
            }
        }
        while let Ok(e) = GlobalHotKeyEvent::receiver().try_recv() {
            if e.id() == hotkey.id() && e.state() == HotKeyState::Pressed {
                commands.push(Command::Toggle);
            }
        }
        if quit_rx.try_recv().is_ok() {
            commands.push(Command::Quit);
        }

        let changed = !commands.is_empty();
        for cmd in commands {
            let keep = app.handle(cmd).unwrap_or_else(|e| {
                eprintln!("redlights: {e}");
                true
            });
            if !keep {
                return Ok(()); // App restaura los colores al soltarse
            }
        }
        if changed {
            refresh(&app)?;
        }
        app.tick();
        std::thread::sleep(Duration::from_millis(50));
    }
}
