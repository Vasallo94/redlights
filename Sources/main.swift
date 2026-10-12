// RedLights — pone todas las pantallas en rojo puro para astronomía (macOS 13+)
//
// Cómo funciona: sustituye la tabla gamma de cada pantalla para que el verde y
// el azul salgan a cero y el rojo escale de 0 a `level`. Así se consigue un rojo
// más tenue que el brillo mínimo del panel. Si la app se cierra o se cuelga,
// macOS restaura la gamma original automáticamente.

import Cocoa
import Carbon.HIToolbox
import ServiceManagement

// MARK: - Filtro rojo vía tablas gamma

enum RedFilter {
    /// Verde y azul a cero; rojo lineal de 0 a `level` (0...1).
    static func apply(level: Float) {
        for display in onlineDisplays() {
            _ = CGSetDisplayTransferByFormula(display,
                                              0, level, 1,   // rojo: min, max, gamma
                                              0, 0, 1,       // verde
                                              0, 0, 1)       // azul
        }
    }

    /// Devuelve todas las pantallas a su perfil ColorSync.
    static func restore() {
        CGDisplayRestoreColorSyncSettings()
    }

    private static func onlineDisplays() -> [CGDirectDisplayID] {
        var count: UInt32 = 0
        guard CGGetOnlineDisplayList(0, nil, &count) == .success, count > 0 else { return [] }
        var displays = [CGDirectDisplayID](repeating: 0, count: Int(count))
        guard CGGetOnlineDisplayList(count, &displays, &count) == .success else { return [] }
        return Array(displays.prefix(Int(count)))
    }
}

// MARK: - Atajo de teclado global (Carbon, no necesita permisos de accesibilidad)

final class HotKey {
    private var hotKeyRef: EventHotKeyRef?
    private var handlerRef: EventHandlerRef?
    private let action: () -> Void

    init(keyCode: UInt32, modifiers: UInt32, action: @escaping () -> Void) {
        self.action = action

        var spec = EventTypeSpec(eventClass: OSType(kEventClassKeyboard),
                                 eventKind: UInt32(kEventHotKeyPressed))
        let selfPtr = Unmanaged.passUnretained(self).toOpaque()

        InstallEventHandler(GetApplicationEventTarget(), { _, _, userData in
            guard let userData else { return noErr }
            Unmanaged<HotKey>.fromOpaque(userData).takeUnretainedValue().action()
            return noErr
        }, 1, &spec, selfPtr, &handlerRef)

        let id = EventHotKeyID(signature: OSType(0x5244_4C54), id: 1) // 'RDLT'
        RegisterEventHotKey(keyCode, modifiers, id, GetApplicationEventTarget(), 0, &hotKeyRef)
    }

    deinit {
        if let hotKeyRef { UnregisterEventHotKey(hotKeyRef) }
        if let handlerRef { RemoveEventHandler(handlerRef) }
    }
}

// MARK: - App de barra de menú

final class AppDelegate: NSObject, NSApplicationDelegate {
    private let defaults = UserDefaults.standard

    private var statusItem: NSStatusItem!
    private var toggleItem: NSMenuItem!
    private var brightnessLabel: NSMenuItem!
    private var loginItem: NSMenuItem!
    private var hotKey: HotKey?
    private var timer: Timer?

    private var enabled = false

    /// Posición del regulador (0...1), persistida entre arranques.
    private var sliderValue: Double {
        get { defaults.object(forKey: "slider") as? Double ?? 0.5 }
        set { defaults.set(newValue, forKey: "slider") }
    }

    /// Curva cuadrática: más precisión en la zona baja, que es donde se usa de noche.
    private var level: Float {
        Float(max(0.02, sliderValue * sliderValue))
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
        statusItem.menu = buildMenu()
        updateUI()

        // ⌃⌥⌘R activa/desactiva desde cualquier app
        hotKey = HotKey(keyCode: UInt32(kVK_ANSI_R),
                        modifiers: UInt32(cmdKey | optionKey | controlKey)) { [weak self] in
            self?.toggle()
        }

        // El sistema resetea la gamma al despertar o al cambiar de pantalla: reaplicamos.
        let ws = NSWorkspace.shared.notificationCenter
        ws.addObserver(self, selector: #selector(reapply(_:)),
                       name: NSWorkspace.didWakeNotification, object: nil)
        ws.addObserver(self, selector: #selector(reapply(_:)),
                       name: NSWorkspace.screensDidWakeNotification, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(reapply(_:)),
                       name: NSApplication.didChangeScreenParametersNotification, object: nil)
    }

    func applicationWillTerminate(_ notification: Notification) {
        RedFilter.restore()
    }

    // MARK: Menú

    private func buildMenu() -> NSMenu {
        let menu = NSMenu()

        toggleItem = NSMenuItem(title: NSLocalizedString("Red Mode", comment: "Menu item"), action: #selector(toggle), keyEquivalent: "r")
        toggleItem.keyEquivalentModifierMask = [.command, .option, .control]
        toggleItem.target = self
        menu.addItem(toggleItem)
        menu.addItem(.separator())

        brightnessLabel = NSMenuItem(title: "", action: nil, keyEquivalent: "")
        brightnessLabel.isEnabled = false
        menu.addItem(brightnessLabel)

        let slider = NSSlider(value: sliderValue, minValue: 0, maxValue: 1,
                              target: self, action: #selector(sliderChanged(_:)))
        slider.isContinuous = true
        slider.frame = NSRect(x: 20, y: 4, width: 180, height: 22)
        let container = NSView(frame: NSRect(x: 0, y: 0, width: 220, height: 30))
        container.addSubview(slider)
        let sliderItem = NSMenuItem()
        sliderItem.view = container
        menu.addItem(sliderItem)
        menu.addItem(.separator())

        loginItem = NSMenuItem(title: NSLocalizedString("Open at Login", comment: "Menu item"),
                               action: #selector(toggleLogin), keyEquivalent: "")
        loginItem.target = self
        menu.addItem(loginItem)
        menu.addItem(.separator())

        let aboutItem = NSMenuItem(title: NSLocalizedString("About RedLights", comment: "Menu item"),
                                   action: #selector(showAbout), keyEquivalent: "")
        aboutItem.target = self
        menu.addItem(aboutItem)
        menu.addItem(NSMenuItem(title: NSLocalizedString("Quit", comment: "Menu item"),
                                action: #selector(NSApplication.terminate(_:)),
                                keyEquivalent: "q"))
        return menu
    }

    // MARK: Acciones

    @objc private func toggle() {
        enabled.toggle()
        if enabled {
            RedFilter.apply(level: level)
            // Red de seguridad por si Night Shift, un perfil de color u otra app pisan la gamma.
            timer = Timer.scheduledTimer(timeInterval: 2, target: self,
                                         selector: #selector(reapply(_:)),
                                         userInfo: nil, repeats: true)
        } else {
            timer?.invalidate()
            timer = nil
            RedFilter.restore()
        }
        updateUI()
    }

    @objc private func sliderChanged(_ sender: NSSlider) {
        sliderValue = sender.doubleValue
        if enabled { RedFilter.apply(level: level) }
        updateUI()
    }

    @objc private func reapply(_ sender: Any?) {
        if enabled { RedFilter.apply(level: level) }
    }

    @objc private func showAbout() {
        // Panel estándar de macOS: nombre, icono y versión del Info.plist.
        NSApp.activate(ignoringOtherApps: true)
        NSApp.orderFrontStandardAboutPanel(nil)
    }

    @objc private func toggleLogin() {
        let service = SMAppService.mainApp
        do {
            if service.status == .enabled {
                try service.unregister()
            } else {
                try service.register()
            }
        } catch {
            NSApp.activate(ignoringOtherApps: true)
            let alert = NSAlert()
            alert.messageText = NSLocalizedString("Couldn’t change the login item", comment: "Alert title")
            alert.informativeText = error.localizedDescription
            alert.runModal()
        }
        updateUI()
    }

    private func updateUI() {
        toggleItem.state = enabled ? .on : .off
        brightnessLabel.title = String(format: NSLocalizedString("Brightness: %ld%%", comment: "Slider label"),
                                       Int((sliderValue * 100).rounded()))
        loginItem.state = SMAppService.mainApp.status == .enabled ? .on : .off

        let image = NSImage(systemSymbolName: enabled ? "moon.fill" : "moon",
                            accessibilityDescription: NSLocalizedString("Red Mode", comment: "Menu item"))
        image?.isTemplate = true
        statusItem.button?.image = image
    }
}

// MARK: - Arranque

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.setActivationPolicy(.accessory)   // sin icono en el Dock
app.run()
