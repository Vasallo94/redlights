# RedLights en Linux

## Estado de la implementación (2026-10-12)

CLI en Rust en `rust/`, pensada para crecer a Windows: un núcleo común (`src/lib.rs`: curva de brillo, rampas gamma y matriz de color) y un backend por mecanismo (`src/backend/`).

```bash
redlights [on] [BRILLO]   # rojo hasta Ctrl-C o SIGTERM, y restaura
redlights reset           # recupera colores normales si algo se quedó en rojo
REDLIGHTS_BACKEND=x11|wlr|mutter redlights   # forzar backend
```

| Backend | Estado |
|---|---|
| X11 (RandR) | ✅ Verificado en Xvfb: aplica, restaura la rampa original y `reset` deja la identidad (test de integración) |
| GNOME (Mutter D-Bus) | ✅ Verificado con Mutter 48 en modo X11: aplica, restaura y resetea (test de integración) |
| wlroots (`wlr-gamma-control`) | ⚠️ Protocolo verificado con Sway 1.10 headless (se negocia y se maneja el `failed`), pero las salidas virtuales no admiten gamma: falta probarlo con pantalla real |
| KDE Plasma | ❌ Pendiente (perfil ICC con VCGT) |

El CI ejecuta Clippy, los tests y las integraciones de X11 y Mutter en x86_64 y ARM64.

Falta en hardware real: que se vea rojo, el cursor, Mutter en Wayland (no solo X11), wlroots con una salida real y si Night Light pisa el filtro.

---

Investigación original (octubre de 2026):

En macOS hay una sola API (`CGSetDisplayTransferByFormula`) que vale para todo. En Linux, quién controla la tabla gamma depende del **servidor gráfico o compositor**, y cada uno expone un mecanismo distinto. Una app de terceros no puede tocar el hardware directamente mientras el compositor lo tiene en uso.

## Mecanismos por entorno

| Entorno | Mecanismo | ¿Se restaura solo si la app muere? | Confianza |
|---|---|---|---|
| **X11** (cualquier escritorio) | XRandR `XRRSetCrtcGamma` con rampas a medida (verde y azul a 0) | **No.** La rampa se queda puesta: hay que guardar la original y restaurarla al salir | Alta: es lo que usan redshift y xcalib |
| **wlroots** (Sway, Hyprland, labwc, Wayfire, river, niri…) | Protocolo Wayland [`wlr-gamma-control-unstable-v1`](https://wayland.app/protocols/wlr-gamma-control-unstable-v1) | **Sí.** Al destruir el objeto se restaura la gamma, como en macOS. Obliga a tener un proceso vivo | Alta: es lo que usan wlsunset y gammastep |
| **GNOME** (Mutter, X11 y Wayland) | D-Bus `org.gnome.Mutter.DisplayConfig`: `SetCrtcGamma` (rampas) o `SetOutputCTM` (matriz de color 3×3) | Por comprobar | Media-alta: en Mutter 50.5 un tercero puede llamarlos (ver resultados abajo), pero es una interfaz interna de GNOME, no un contrato público |
| **KDE Plasma 6** (KWin Wayland) | Perfil ICC con etiqueta **VCGT** asignado a cada salida, la misma vía que [KGamma2](https://planet.kde.org/david-edmundson-2026-08-03-wayland-pain-points-kgamma/) | No (es configuración persistente) | Media: KWin aplica la VCGT y se combina con Night Color, pero falta ver cómo asignar el perfil desde fuera (¿`kscreen-doctor`?) |
| Hyprland (alternativa) | `hyprland-ctm-control-v1`, el de hyprsunset | Por comprobar | Media; Hyprland también soporta wlr-gamma-control, que basta |

KWin no implementa `wlr-gamma-control`, y GNOME tampoco: por eso wlsunset y gammastep no funcionan en esos escritorios.

## Matriz de color (CTM) en lugar de gamma: mejor rojo

La tabla gamma trabaja **canal a canal**: pone el verde y el azul a 0 y deja el rojo como está. La consecuencia es que algo puramente verde o azul (un texto azul, una línea verde de un gráfico de guiado) **se vuelve negro**. Así funciona hoy la versión de macOS.

Una matriz de color mezcla los tres canales antes de enviarlos. Con

```
R' = L · (0.2126 R + 0.7152 G + 0.0722 B)
G' = 0
B' = 0
```

cada píxel se pinta en rojo según su **luminancia**: todo sigue siendo legible y no desaparece nada. Mutter (`SetOutputCTM`) y Hyprland (CTM) permiten hacerlo. Si funciona bien, vale la pena buscar un equivalente en macOS. La API pública de gamma no lo permite, pero se podría simular con un overlay de Core Image.

## Raspberry Pi: probablemente el caso más importante

Muchos equipos de astrofotografía funcionan con una Raspberry Pi (Astroberry, StellarMate, KStars/Ekos con INDI). Raspberry Pi OS Bookworm usa **labwc, que es wlroots**, y X11 en los modelos antiguos o como opción. Con X11 y wlroots se cubre ese caso, que además es justo donde se usa una pantalla junto al telescopio.

## Prioridades propuestas

1. **X11 y wlroots**: son mecanismos estándar y bien documentados, y cubren la Raspberry Pi y los gestores de ventanas.
2. **GNOME**: depende de lo que salga del spike con D-Bus.
3. **KDE**: generar un ICC con VCGT roja y asignarlo; es el más incómodo.

## Lenguaje: propuesta

**Rust** para la versión de Linux: un único binario estático, fácil de empaquetar (`.deb`, AUR, Flatpak) y con crates maduros para cada backend:

- X11: `x11rb` (extensión RandR)
- wlroots: `wayland-client` + `wayland-protocols-wlr` (`gamma_control`)
- GNOME: `zbus`
- Bandeja: `ksni` (StatusNotifierItem, funciona en KDE, wlroots con waybar y GNOME con la extensión AppIndicator)

Alternativa: Python con PyGObject, más rápido de escribir pero más pesado de distribuir. Los spikes están en Python porque se ejecutan sin instalar nada en cualquier distro con escritorio.

Interfaz mínima, alineada con la hoja de ruta: una CLI `redlights on [nivel] | off | toggle` que se queda en primer plano mientras el filtro está activo. Así wlroots, que exige un proceso vivo, encaja de forma natural, y en X11 se restaura al recibir SIGINT o SIGTERM. La bandeja vendría después, encima de la CLI.

## Spikes para ejecutar en Linux

Estos son experimentos para responder preguntas, no código para el producto. Cada uno aplica el rojo, espera a que pulses Enter y restaura.

```bash
python3 spikes/x11_red.py 0.5                  # X11: rampas XRandR
python3 spikes/gnome_red.py --mode ctm 0.5     # GNOME: matriz de color por D-Bus
python3 spikes/gnome_red.py --mode gamma 0.5   # GNOME: rampas por D-Bus
```

Qué hay que anotar en cada uno:

- ¿Se pone la pantalla en rojo? ¿Y el **cursor**?
- ¿Se restaura al pulsar Enter? ¿Y con `kill -9` al proceso? Si no se restaura: en X11, `xrandr --output <salida> --gamma 1:1:1`; en GNOME, cerrar la sesión.
- GNOME: ¿da error de permisos? ¿Night Light lo pisa? ¿Funciona igual en X11 y en Wayland?
- Con `--mode ctm`: ¿el texto azul o verde sigue legible en rojo?

Si X11 y GNOME responden bien, el siguiente spike es wlroots (en una Raspberry Pi con labwc o en Sway).

## Resultados en contenedor (2026-10-11, Podman en el Mac)

Sin pantalla real no se puede ver el rojo, pero sí comprobar que las llamadas funcionan:

- **X11 (Debian 12 + Xvfb)**: `x11_red.py` aplica la rampa correcta (rojo 0→50 %, verde y azul a 0) y, **dentro de la misma conexión**, guardar → rojo → restaurar devuelve exactamente la rampa original. ✅ Leída desde otra conexión después de que el cliente se desconecte, la rampa de Xvfb sale corrupta. Es un fallo de Xvfb, no del spike, pero conviene revisarlo en hardware real.
- **GNOME (Fedora, Mutter 50.5 `--headless`)**: `org.gnome.Mutter.DisplayConfig` responde a un tercero sin `AccessDenied`. `SetOutputCTM` y `SetCrtcGamma` con ids inexistentes devuelven `InvalidArgs: Invalid output id / Invalid crtc id`, es decir, validan argumentos y no permisos. ✅ Los monitores virtuales del modo headless no aparecen en `GetResources`, así que el efecto real queda pendiente de hardware.

## Montar un equipo de pruebas

Para probar varios backends con una sola máquina (por ejemplo, un mini PC): instalar una distro con **GNOME** y añadir **Sway** (wlroots) y **XFCE** (X11) como sesiones alternativas, que se eligen en la pantalla de inicio de sesión. Con eso se prueban 3 de los 4 caminos. Debian 13 o Fedora sirven; en Fedora, GNOME ya no tiene sesión X11, así que X11 llegaría por XFCE. KDE Plasma se puede añadir igual.
