<p align="center">
  <img src="assets/banner.svg" alt="RedLights: todas tus pantallas en rojo puro para astronomía" width="100%">
</p>

<p align="center">
  <a href="https://github.com/Vasallo94/redlights/releases/latest"><img src="https://img.shields.io/github/v/release/Vasallo94/redlights?color=d42010&label=versi%C3%B3n" alt="Última versión"></a>
  <img src="https://img.shields.io/badge/macOS-13%2B-d42010?logo=apple&logoColor=white" alt="macOS 13+">
  <img src="https://img.shields.io/badge/Apple%20Silicon%20%2B%20Intel-universal-d42010" alt="Binario universal">
  <a href="LICENSE"><img src="https://img.shields.io/badge/licencia-MIT-d42010" alt="Licencia MIT"></a>
</p>

**RedLights** es una app de barra de menú que pone **todas tus pantallas en rojo puro**: verde y azul a cero, y el rojo tan tenue como quieras, incluso por debajo del brillo mínimo del panel. Está pensada para usar el portátil junto al telescopio sin perder la visión nocturna: planetario, software de captura, guiado, mapas…

Un atajo (**⌃⌥⌘R**) y listo. No pide permisos, no hace capturas y no pone ventanas encima.

## ¿Por qué rojo?

De noche ves con los **bastones** de la retina, que tardan entre 20 y 30 minutos en adaptarse del todo a la oscuridad y lo pierden en segundos con un destello de luz blanca. Los bastones apenas responden a la luz roja de onda larga, así que una pantalla en rojo **tenue** te deja leerla sin estropear la adaptación. Por eso las linternas de astronomía son rojas.

Night Shift o el modo oscuro no bastan: siguen emitiendo verde y azul, que es justo lo que más daña la visión nocturna.

## Qué hace

- 🔴 **Rojo puro** en todas las pantallas a la vez, incluido el cursor.
- 🌑 **Brillo regulable** con curva cuadrática, para tener precisión en la zona más tenue. Se suma a las teclas de brillo del Mac: con las dos al mínimo, el rojo baja más de lo que permite el panel.
- ⌨️ **Atajo global ⌃⌥⌘R** desde cualquier app.
- 🔁 **Se mantiene activo**: lo reaplica al despertar el Mac, al conectar o desconectar monitores y cada 2 s por si Night Shift o un perfil de color lo pisan.
- 🛟 **Siempre se puede deshacer**: al salir, o incluso si la app se cierra de golpe, macOS restaura los colores normales.
- 🚀 Opción para **abrir al iniciar sesión**.

## Instalación

### Descargar

1. Baja `RedLights-x.y.z.zip` de la [última versión](https://github.com/Vasallo94/redlights/releases/latest), descomprímelo y arrastra `RedLights.app` a **Aplicaciones**.
2. La app está firmada ad-hoc, sin notarizar, así que macOS la bloquea la primera vez. Ve a **Ajustes del Sistema → Privacidad y seguridad** y pulsa **Abrir igualmente**. O, desde Terminal:

   ```bash
   xattr -dr com.apple.quarantine /Applications/RedLights.app
   ```

Cada versión publica también un `.sha256` para comprobar la descarga con `shasum -a 256 -c RedLights-x.y.z.zip.sha256`.

### Compilar

Necesitas macOS 13+ y las Command Line Tools (`xcode-select --install`). No hace falta Xcode.

```bash
git clone https://github.com/Vasallo94/redlights.git
cd redlights
./build.sh
open RedLights.app
```

`./build.sh --zip` genera además el zip listo para distribuir.

## Uso

| | |
|---|---|
| **⌃⌥⌘R** | Activa o desactiva el modo rojo |
| Luna en la barra de menú | Menú con el interruptor, el regulador de brillo y el inicio automático |
| Teclas de brillo del Mac | Siguen funcionando y se combinan con el regulador |

## Cómo funciona

macOS pasa cada color que se dibuja en pantalla por una **tabla gamma** antes de mandarlo al panel. RedLights sustituye esa tabla con `CGSetDisplayTransferByFormula`: el canal rojo va de 0 a tu nivel de brillo, y el verde y el azul se quedan siempre en 0. Como la transformación ocurre al final del todo, afecta a todo lo que ves (apps, vídeo, cursor) sin capturar la pantalla ni superponer ventanas. Por eso no necesita permisos de grabación ni de accesibilidad.

La tabla pertenece al proceso que la puso: si RedLights termina por la razón que sea, WindowServer devuelve la gamma original. No puedes quedarte con la pantalla en rojo para siempre.

Todo el código está en un único archivo, [`Sources/main.swift`](Sources/main.swift), de unas 200 líneas.

## Limitaciones conocidas

- Las capturas de pantalla salen con los colores normales: la tabla gamma se aplica después de componer la imagen.
- Las pantallas en modo HDR o XDR de referencia pueden ignorar o resetear la gamma (pendiente de probar).
- En Macs Intel antiguos, el cursor por hardware podría no pasar por la tabla gamma y verse blanco. En Apple Silicon sale rojo.

## Hoja de ruta

- [ ] Linux (X11, GNOME, KDE, wlroots)
- [ ] Elegir qué pantallas se ponen en rojo (p. ej. portátil en rojo y monitor externo apagado)
- [ ] CLI (`redlights on 0.1`) para automatizar sesiones de captura
- [ ] Activación automática con el crepúsculo astronómico de tu ubicación

## Publicar una versión

1. Añade los cambios a [`CHANGELOG.md`](CHANGELOG.md) bajo una sección `## [x.y.z] - AAAA-MM-DD`.
2. `git tag vx.y.z && git push origin vx.y.z`

GitHub Actions compila el binario universal, crea la release con el zip y su SHA-256, y usa como notas esa sección del CHANGELOG. El número de versión sale del tag: no hay que tocarlo en ningún archivo.

## Licencia

[MIT](LICENSE) © Vasallo94
