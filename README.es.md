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
- **Linux y Windows**: una pequeña herramienta de línea de comandos, `redlights`.

## Instalación

### macOS

Requiere macOS 13 o posterior. Binario universal para Apple Silicon e Intel.

```bash
brew install --cask vasallo94/tap/redlights
```

Sin Homebrew, descarga `RedLights-x.y.z.zip` de la [última versión](https://github.com/Vasallo94/redlights/releases/latest) y arrastra `RedLights.app` a Aplicaciones. La app no está notarizada, así que macOS la bloquea la primera vez: abre **Ajustes del Sistema → Privacidad y seguridad** y pulsa **Abrir igualmente**, o ejecuta `xattr -dr com.apple.quarantine /Applications/RedLights.app`.

### Linux

Funciona en X11 (cualquier escritorio, incluido KDE Plasma en X11), GNOME en Wayland o X11 y compositores wlroots: Sway, Hyprland, Wayfire, river, niri y labwc, el escritorio de Raspberry Pi OS.

```bash
curl -L https://github.com/Vasallo94/redlights/releases/latest/download/redlights-linux-x86_64.tar.gz | tar -xz
sudo install redlights /usr/local/bin/
```

En ARM64, como una Raspberry Pi 4 o 5 con sistema de 64 bits, descarga `redlights-linux-aarch64.tar.gz`.

### Windows

Windows 10 u 11, de 64 bits. Descarga [`redlights-windows-x64.zip`](https://github.com/Vasallo94/redlights/releases/latest/download/redlights-windows-x64.zip), descomprímelo y ejecuta `redlights.exe`. El ejecutable no está firmado, así que SmartScreen puede avisar la primera vez: elige **Más información → Ejecutar de todas formas**.

### Verificar las descargas

Cada archivo de una versión trae su suma `.sha256` y una attestation de procedencia de GitHub, que demuestra que lo compiló el CI de este repositorio:

```bash
gh attestation verify redlights-linux-x86_64.tar.gz --repo Vasallo94/redlights
```

## Uso

### macOS

| Acción | Cómo |
|---|---|
| Activar o desactivar el modo rojo | **⌃⌥⌘R** desde cualquier app, o la luna de la barra de menú |
| Ajustar el brillo | Regulador del menú. Las teclas de brillo del Mac siguen funcionando y se combinan con él |
| Arrancar al iniciar sesión | **Abrir al iniciar sesión** en el menú |

### Linux y Windows

```bash
redlights          # rojo al 50 % de brillo hasta pulsar Ctrl-C
redlights 0.3      # brillo de 0 a 1, con la misma curva que el regulador del Mac
redlights reset    # colores normales si una pantalla se quedó en rojo
```

En Windows también puedes hacer doble clic en `redlights.exe`: la pantalla se queda en rojo mientras su ventana esté abierta, y al cerrarla vuelven los colores normales.

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

La app de Mac es un único archivo Swift, [`Sources/main.swift`](Sources/main.swift). La herramienta de Linux y Windows, en [`rust/`](rust/), tiene un núcleo común y un backend por mecanismo; las notas de diseño están en [`docs/linux.md`](docs/linux.md).

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
- [ ] Icono en la bandeja y atajo global en Linux y Windows
- [ ] Modo luminancia: el rojo sigue el brillo de cada color, para que el texto azul o verde siga siendo legible
- [ ] Elegir qué pantallas se ponen en rojo, por ejemplo el portátil en rojo y el monitor externo apagado
- [ ] Activación automática con el crepúsculo astronómico de tu ubicación

## Desarrollo

### Compilar

```bash
./build.sh                         # app de macOS; solo necesita las Command Line Tools (xcode-select --install)
cd rust && cargo build --release   # herramienta de Linux y Windows
```

### Tests

El CI se ejecuta en cada pull request:

- macOS: compila la app, revisa el bundle (binario universal, firma, versión, traducciones) y comprueba que arranca.
- Linux, x86_64 y ARM64: formato, Clippy, tests unitarios y tests de integración contra un servidor X real y GNOME Mutter.
- Windows: Clippy, tests unitarios y un test de integración que aplica la matriz de color y la vuelve a leer.

### Publicar una versión

1. Añade una sección `## [x.y.z] - AAAA-MM-DD` a [`CHANGELOG.md`](CHANGELOG.md) y fusiónala en `main` con un pull request.
2. Crea el tag sobre ese commit: `git tag vx.y.z && git push origin vx.y.z`.

El CI compila todas las plataformas, publica la versión con sumas de verificación y attestations, y usa la sección del CHANGELOG como notas. El número de versión sale del tag y no está escrito en ningún archivo. El [tap de Homebrew](https://github.com/Vasallo94/homebrew-tap) recoge las versiones nuevas solo en menos de un día.

## Licencia

[MIT](LICENSE) © Vasallo94
