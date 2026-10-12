<p align="center">
  <img src="assets/banner.es.svg" alt="RedLights: todas tus pantallas en rojo puro para astronomía" width="100%">
</p>

<p align="center">
  <a href="https://github.com/Vasallo94/redlights/releases/latest"><img src="https://img.shields.io/github/v/release/Vasallo94/redlights?color=d42010&label=versi%C3%B3n" alt="Última versión"></a>
  <img src="https://img.shields.io/badge/macOS%20%C2%B7%20Linux%20%C2%B7%20Windows-d42010" alt="macOS · Linux · Windows">
  <a href="LICENSE"><img src="https://img.shields.io/badge/licencia-MIT-d42010" alt="Licencia MIT"></a>
</p>

<p align="center"><a href="README.md">English</a> · <b>Español</b></p>

**RedLights** pone **todas tus pantallas en rojo puro**: verde y azul a cero, y el rojo tan tenue como quieras, incluso por debajo del brillo mínimo del panel. Está pensada para usar el portátil junto al telescopio sin perder la visión nocturna: planetario, software de captura, guiado, mapas…

En macOS es una app de barra de menú: un atajo (**⌃⌥⌘R**) y listo. En Linux y Windows es una pequeña herramienta de línea de comandos. No pide permisos, no hace capturas y no pone ventanas encima.

## ¿Por qué rojo?

De noche ves con los **bastones** de la retina, que tardan entre 20 y 30 minutos en adaptarse del todo a la oscuridad y lo pierden en segundos con un destello de luz blanca. Los bastones apenas responden a la luz roja de onda larga, así que una pantalla en rojo **tenue** te deja leerla sin estropear la adaptación. Por eso las linternas de astronomía son rojas.

Night Shift o el modo oscuro no bastan: siguen emitiendo verde y azul, que es justo lo que más daña la visión nocturna.

## Qué hace

La app de Mac:

- 🔴 **Rojo puro** en todas las pantallas a la vez, incluido el cursor.
- 🌑 **Brillo regulable** con curva cuadrática, para tener precisión en la zona más tenue. Se suma a las teclas de brillo del Mac: con las dos al mínimo, el rojo baja más de lo que permite el panel.
- ⌨️ **Atajo global ⌃⌥⌘R** desde cualquier app.
- 🔁 **Se mantiene activo**: lo reaplica al despertar el Mac, al conectar o desconectar monitores y cada 2 s por si Night Shift o un perfil de color lo pisan.
- 🛟 **Siempre se puede deshacer**: al salir, o incluso si la app se cierra de golpe, macOS restaura los colores normales.
- 🚀 Opción para **abrir al iniciar sesión**.
- 🌍 En español e inglés, según el idioma del sistema.

En Linux y Windows el mismo filtro llega como herramienta de línea de comandos, `redlights`, que también restaura los colores al cerrarse. El icono en la bandeja y el atajo global están en la hoja de ruta.

## Instalación

### macOS

#### Homebrew (recomendado)

```bash
brew install --cask vasallo94/tap/redlights
```

Se instala en Aplicaciones, ya preparada para abrirse sin el aviso de Gatekeeper, y se actualiza con `brew upgrade`.

#### Descargar

1. Baja `RedLights-x.y.z.zip` de la [última versión](https://github.com/Vasallo94/redlights/releases/latest), descomprímelo y arrastra `RedLights.app` a **Aplicaciones**.
2. La app está firmada ad-hoc, sin notarizar, así que macOS la bloquea la primera vez. Ve a **Ajustes del Sistema → Privacidad y seguridad** y pulsa **Abrir igualmente**. O, desde Terminal:

   ```bash
   xattr -dr com.apple.quarantine /Applications/RedLights.app
   ```

Cada versión publica también un `.sha256` para comprobar la descarga con `shasum -a 256 -c RedLights-x.y.z.zip.sha256`. Además, GitHub certifica cada release, así que puedes verificar que el zip lo compiló el CI de este repo:

```bash
gh attestation verify RedLights-x.y.z.zip --repo Vasallo94/redlights
```

#### Compilar

Necesitas macOS 13+ y las Command Line Tools (`xcode-select --install`). No hace falta Xcode.

```bash
git clone https://github.com/Vasallo94/redlights.git
cd redlights
./build.sh
open RedLights.app
```

`./build.sh --zip` genera además el zip listo para distribuir.

### Linux

Funciona en X11 (cualquier escritorio, incluido KDE Plasma en X11), GNOME (Wayland o X11) y compositores wlroots como Sway, Hyprland, Wayfire, river, niri o labwc (el escritorio de Raspberry Pi OS). KDE Plasma en Wayland todavía no está soportado.

```bash
curl -L https://github.com/Vasallo94/redlights/releases/latest/download/redlights-linux-x86_64.tar.gz | tar -xz
sudo install redlights /usr/local/bin/
```

En ARM64 (Raspberry Pi 4/5 con sistema de 64 bits), usa `redlights-linux-aarch64.tar.gz`. Para compilarlo: `cd rust && cargo build --release`.

### Windows

Windows 10 u 11, de 64 bits.

1. Descarga [`redlights-windows-x64.zip`](https://github.com/Vasallo94/redlights/releases/latest/download/redlights-windows-x64.zip) y descomprímelo.
2. Haz doble clic en `redlights.exe`: la pantalla se pone roja mientras su ventana esté abierta. Ciérrala (o pulsa Ctrl-C) para volver a los colores normales.
3. El exe no está firmado, así que SmartScreen puede avisar la primera vez: **Más información → Ejecutar de todas formas**.

## Uso

En el Mac:

| | |
|---|---|
| **⌃⌥⌘R** | Activa o desactiva el modo rojo |
| Luna en la barra de menú | Menú con el interruptor, el regulador de brillo y el inicio automático |
| Teclas de brillo del Mac | Siguen funcionando y se combinan con el regulador |

En Linux y Windows:

```bash
redlights          # rojo al 50 % hasta Ctrl-C
redlights 0.3      # brillo de 0 a 1 (misma curva que el regulador del Mac)
redlights reset    # colores normales si una pantalla se quedó en rojo
```

## Cómo funciona

macOS pasa cada color que se dibuja en pantalla por una **tabla gamma** antes de mandarlo al panel. RedLights sustituye esa tabla con `CGSetDisplayTransferByFormula`: el canal rojo va de 0 a tu nivel de brillo, y el verde y el azul se quedan siempre en 0. Como la transformación ocurre al final del todo, afecta a todo lo que ves (apps, vídeo, cursor) sin capturar la pantalla ni superponer ventanas. Por eso no necesita permisos de grabación ni de accesibilidad.

La tabla pertenece al proceso que la puso: si RedLights termina por la razón que sea, WindowServer devuelve la gamma original. No puedes quedarte con la pantalla en rojo para siempre.

Todo el código de la app de Mac está en un único archivo, [`Sources/main.swift`](Sources/main.swift), de unas 200 líneas.

Cada sistema lo expone de forma distinta, así que la herramienta de Linux y Windows ([`rust/`](rust/)) tiene un backend por mecanismo sobre un núcleo común:

| Plataforma | Mecanismo | ¿Se restaura solo si la app muere? |
|---|---|---|
| macOS | Tabla gamma (`CGSetDisplayTransferByFormula`) | Sí |
| Linux, wlroots | Protocolo Wayland `wlr-gamma-control` | Sí |
| Linux, GNOME | API D-Bus de pantallas de Mutter (`SetCrtcGamma`) | No, usa `redlights reset` |
| Linux, X11 | Rampas gamma de XRandR | No, usa `redlights reset` |
| Windows | Matriz de color de la API de Magnificación (`MagSetFullscreenColorEffect`) | Sí |

Las notas de diseño están en [`docs/linux.md`](docs/linux.md).

## Limitaciones conocidas

- Las capturas de pantalla salen con los colores normales: la tabla gamma se aplica después de componer la imagen.
- Las pantallas en modo HDR o XDR de referencia pueden ignorar o resetear la gamma (pendiente de probar).
- En Macs Intel antiguos, el cursor por hardware podría no pasar por la tabla gamma y verse blanco. En Apple Silicon sale rojo.
- Linux: KDE Plasma en Wayland todavía no está soportado (funciona en una sesión X11 de KDE).
- Linux y Windows: probado en las máquinas del CI y con pantallas virtuales; cualquier prueba en hardware real es bienvenida.

## Hoja de ruta

- [x] Linux: X11, GNOME, wlroots
- [x] Windows
- [ ] KDE Plasma en Wayland
- [ ] Icono en la bandeja y atajo global en Linux y Windows
- [ ] Modo luminancia: el rojo sigue el brillo, para que el texto azul o verde siga siendo legible
- [ ] Elegir qué pantallas se ponen en rojo (p. ej. portátil en rojo y monitor externo apagado)
- [ ] Activación automática con el crepúsculo astronómico de tu ubicación

## Publicar una versión

1. Añade los cambios a [`CHANGELOG.md`](CHANGELOG.md) bajo una sección `## [x.y.z] - AAAA-MM-DD`.
2. `git tag vx.y.z && git push origin vx.y.z`

GitHub Actions compila la app de Mac (binario universal) y las herramientas de Linux y Windows, crea la release con todos los archivos y su SHA-256, y usa como notas esa sección del CHANGELOG. El número de versión sale del tag: no hay que tocarlo en ningún archivo. El [tap de Homebrew](https://github.com/Vasallo94/homebrew-tap) recoge la nueva versión solo en menos de un día.

## Licencia

[MIT](LICENSE) © Vasallo94
