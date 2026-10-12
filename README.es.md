<p align="center">
  <img src="assets/banner.es.svg" alt="RedLights: todas tus pantallas en rojo puro para astronomía" width="100%">
</p>

<p align="center">
  <a href="https://github.com/Vasallo94/redlights/releases/latest"><img src="https://img.shields.io/github/v/release/Vasallo94/redlights?color=d42010&label=versi%C3%B3n" alt="Última versión"></a>
  <img src="https://img.shields.io/badge/macOS%20%C2%B7%20Linux%20%C2%B7%20Windows-d42010" alt="macOS, Linux y Windows">
  <a href="LICENSE"><img src="https://img.shields.io/badge/licencia-MIT-d42010" alt="Licencia MIT"></a>
</p>

<p align="center"><a href="README.md">English</a> · <b>Español</b></p>

**RedLights** pone todas tus pantallas en rojo puro para astronomía y astrofotografía. El verde y el azul bajan a cero y el rojo se atenúa tanto como necesites, incluso por debajo del brillo mínimo del panel, para usar el portátil junto al telescopio (planetario, captura, guiado, mapas) sin perder la visión nocturna.

Cambia los colores que emite la propia pantalla: no hace capturas, no pone ventanas encima y no necesita permisos especiales. Todo se vuelve rojo, incluido el cursor, y los colores normales vuelven al desactivarlo.

- **macOS**: app de barra de menú con atajo global (⌃⌥⌘R) y regulador de brillo.
- **Linux y Windows**: una app de bandeja del sistema (con atajo global en Windows) que también funciona como herramienta de línea de comandos, `redlights`.

## Instalación

### macOS

Requiere macOS 13 o posterior. Binario universal para Apple Silicon e Intel.

```bash
brew install --cask vasallo94/tap/redlights
```

Sin Homebrew, descarga `RedLights-x.y.z.zip` de la [última versión](https://github.com/Vasallo94/redlights/releases/latest) y arrastra `RedLights.app` a Aplicaciones. La app no está notarizada, así que macOS la bloquea la primera vez: abre **Ajustes del Sistema → Privacidad y seguridad** y pulsa **Abrir igualmente**, o ejecuta `xattr -dr com.apple.quarantine /Applications/RedLights.app`.

### Linux

Funciona en X11 (cualquier escritorio, incluido KDE Plasma en X11), GNOME en Wayland o X11 y compositores wlroots: Sway, Hyprland, Wayfire, river, niri y labwc, el escritorio de Raspberry Pi OS.

En Debian, Ubuntu, Raspberry Pi OS y derivadas, instala el paquete `.deb`:

```bash
curl -LO https://github.com/Vasallo94/redlights/releases/latest/download/redlights-linux-amd64.deb
sudo apt install ./redlights-linux-amd64.deb
```

En cualquier otra distribución, usa el binario independiente:

```bash
curl -L https://github.com/Vasallo94/redlights/releases/latest/download/redlights-linux-x86_64.tar.gz | tar -xz
sudo install redlights /usr/local/bin/
```

En ARM64, como una Raspberry Pi 4 o 5 con sistema de 64 bits, usa `redlights-linux-arm64.deb` o `redlights-linux-aarch64.tar.gz`. El binario está enlazado estáticamente, así que funciona en cualquier distribución de Linux, sea cual sea la versión de su librería C.

### Windows

Windows 10 u 11, de 64 bits. Con [Scoop](https://scoop.sh):

```powershell
scoop bucket add vasallo94 https://github.com/Vasallo94/scoop-bucket
scoop install redlights
```

O descarga [`redlights-windows-x64.zip`](https://github.com/Vasallo94/redlights/releases/latest/download/redlights-windows-x64.zip), descomprímelo y haz doble clic en `redlights.exe`. El ejecutable no está firmado, así que SmartScreen puede avisar la primera vez: elige **Más información → Ejecutar de todas formas**.

### Verificar las descargas

Cada archivo de una versión trae una suma SHA-256 y una attestation de procedencia de GitHub que lo vincula con el código y la compilación que lo generaron. En [SECURITY.md](SECURITY.md#verifying-downloads) se explica cómo comprobarlas y en qué punto está la firma de código.

## Uso

### macOS

| Acción | Cómo |
|---|---|
| Activar o desactivar el modo rojo | **⌃⌥⌘R** desde cualquier app, o la luna de la barra de menú |
| Ajustar el brillo | Regulador del menú. Las teclas de brillo del Mac siguen funcionando y se combinan con él |
| Arrancar al iniciar sesión | **Abrir al iniciar sesión** en el menú |

### Windows

Haz doble clic en `redlights.exe` y aparecerá una luna en la bandeja del sistema, junto al reloj:

| Acción | Cómo |
|---|---|
| Activar o desactivar el modo rojo | **Ctrl+Alt+Shift+R** desde cualquier app, o clic en el icono |
| Brillo y salir | Clic derecho en el icono |

### Linux

Abre **RedLights** desde el menú de aplicaciones (lo instala el `.deb`) o ejecuta `redlights tray`. Aparecerá una luna en la bandeja del sistema: clic para activar o desactivar, clic derecho para el brillo. En GNOME, los iconos de bandeja necesitan la [extensión AppIndicator](https://extensions.gnome.org/extension/615/appindicator-support/), que Ubuntu ya trae.

Wayland no deja que las apps registren atajos globales, así que asigna `redlights toggle` a una tecla en los ajustes de teclado de tu escritorio:

| Escritorio | Dónde |
|---|---|
| GNOME | Configuración → Teclado → Atajos personalizados → comando `redlights toggle` |
| KDE Plasma | Preferencias del sistema → Atajos → Añadir nuevo → Orden |
| Sway, i3 | `bindsym Ctrl+Alt+Shift+r exec redlights toggle` |
| Hyprland | `bind = CTRL ALT SHIFT, R, exec, redlights toggle` |

### Línea de comandos (Linux y Windows)

```bash
redlights          # rojo al 50 % de brillo hasta pulsar Ctrl-C
redlights 0.3      # brillo de 0 a 1, con la misma curva que el regulador del Mac
redlights tray     # en la bandeja del sistema
redlights toggle   # activa o desactiva el modo rojo de la bandeja en marcha (Linux)
redlights reset    # colores normales si una pantalla se quedó en rojo
```

## ¿Por qué rojo?

De noche ves con los bastones de la retina. Necesitan entre 20 y 30 minutos para adaptarse del todo a la oscuridad, y un destello de luz blanca lo deshace en segundos. Los bastones apenas son sensibles a la luz roja de onda larga, así que una pantalla en rojo tenue se puede leer sin estropear la adaptación. Por eso las linternas de astronomía son rojas.

Night Shift y el modo oscuro no bastan: siguen emitiendo verde y azul, que es lo que más daña la visión nocturna.

## Cómo funciona

Las pantallas pasan cada color por una tabla de corrección (tabla gamma) o por una matriz de color justo antes de enviarlo al panel. RedLights la sustituye para que el rojo vaya de 0 a tu nivel de brillo y el verde y el azul se queden siempre en 0. Como el cambio ocurre después de dibujarlo todo, afecta a todas las apps, al vídeo y al cursor.

Cada sistema operativo lo expone de forma distinta:

| Plataforma | Mecanismo | Se restaura solo si RedLights se cierra de golpe |
|---|---|---|
| macOS | Tabla gamma, `CGSetDisplayTransferByFormula` | Sí |
| Linux, wlroots | Protocolo Wayland `wlr-gamma-control` | Sí |
| Linux, GNOME | API D-Bus de pantallas de Mutter, `SetCrtcGamma` | No, ejecuta `redlights reset` |
| Linux, X11 | Rampas gamma de XRandR | No, ejecuta `redlights reset` |
| Windows | Matriz de color de la API de Magnificación, `MagSetFullscreenColorEffect` | Sí |

La app de Mac es un único archivo Swift, [`Sources/main.swift`](Sources/main.swift). La herramienta de Linux y Windows, en [`rust/`](rust/), tiene un núcleo común y un backend por mecanismo; las notas de diseño están en [`docs/design.md`](docs/design.md) (en inglés).

## Limitaciones conocidas

- Las capturas de pantalla salen con los colores normales, porque el cambio se aplica después de componer la imagen.
- KDE Plasma en Wayland todavía no está soportado. Una sesión X11 de KDE sí funciona.
- Los modos de referencia HDR y XDR pueden ignorar o resetear la tabla gamma. Pendiente de probar.
- En Macs Intel antiguos el cursor por hardware puede saltarse la tabla gamma y verse blanco. En Apple Silicon sale rojo.
- Las versiones de Linux y Windows se han probado en las máquinas del CI y con pantallas virtuales. Cualquier prueba en hardware real es bienvenida: [abre una issue](https://github.com/Vasallo94/redlights/issues).

## Hoja de ruta

- [x] App de barra de menú para macOS
- [x] Linux: X11, GNOME, wlroots
- [x] Windows
- [ ] KDE Plasma en Wayland
- [x] Icono en la bandeja en Linux y Windows, atajo global en Windows
- [ ] Modo luminancia: el rojo sigue el brillo de cada color, para que el texto azul o verde siga siendo legible
- [ ] Elegir qué pantallas se ponen en rojo, por ejemplo el portátil en rojo y el monitor externo apagado
- [ ] Activación automática con el crepúsculo astronómico de tu ubicación

## Contribuir

Los informes de errores, las pruebas en hardware real y los pull requests son bienvenidos. [Abre una issue](https://github.com/Vasallo94/redlights/issues/new/choose) con uno de los formularios, o lee [CONTRIBUTING.md](CONTRIBUTING.md) (en inglés) para preparar el proyecto, ejecutar los tests y enviar un pull request. Se espera que todo el mundo respete el [código de conducta](CODE_OF_CONDUCT.md).

## Licencia

[MIT](LICENSE) © Vasallo94
