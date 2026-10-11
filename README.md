# RedScreen

App de barra de menú para macOS que pone **todas las pantallas en rojo puro** para astronomía: verde y azul a cero y el rojo regulable por debajo del brillo mínimo del panel.

## Uso

- Icono de luna en la barra de menú → **Modo rojo**.
- **⌃⌥⌘R** activa o desactiva desde cualquier app.
- Regulador de brillo con curva cuadrática, para tener más precisión en la zona tenue. Se recuerda entre arranques.
- Opción para abrir al iniciar sesión.

Al salir (o si la app se cierra de golpe) macOS restaura los colores normales.

## Compilar

Requiere macOS 13+ y las Command Line Tools (`xcode-select --install`).

```bash
chmod +x build.sh
./build.sh
open RedScreen.app
```

Si la compilas tú, Gatekeeper no pone pegas. Si descargas una versión compilada por otra persona, va firmada ad-hoc (sin notarizar) y macOS la bloquea la primera vez: Ajustes del Sistema → Privacidad y seguridad → **Abrir igualmente**, o bien `xattr -dr com.apple.quarantine RedScreen.app`.

## Cómo funciona

Usa `CGSetDisplayTransferByFormula` para reemplazar la tabla gamma de cada pantalla. No hace capturas ni superpone ventanas, así que no necesita permisos de grabación de pantalla ni de accesibilidad. Reaplica el filtro al despertar, al conectar o desconectar monitores y cada 2 s por si Night Shift u otra app lo pisa.

## Limitaciones conocidas

- En Apple Silicon el cursor también sale rojo. En Macs más antiguos el cursor por hardware podría no pasar por la tabla gamma y verse blanco.
- Las pantallas en modo HDR / XDR de referencia pueden ignorar o resetear la gamma.
- Las capturas de pantalla salen con colores normales (es esperable).

## Licencia

[MIT](LICENSE)
