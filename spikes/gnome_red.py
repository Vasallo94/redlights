#!/usr/bin/env python3
"""Spike: rojo puro en GNOME (Mutter) por D-Bus, en X11 o Wayland.

    python3 spikes/gnome_red.py --mode ctm   [nivel 0..1]   # matriz de color por salida
    python3 spikes/gnome_red.py --mode gamma [nivel 0..1]   # rampas gamma por CRTC

Usa org.gnome.Mutter.DisplayConfig, una interfaz interna de GNOME (no es un
contrato público): este spike sirve para ver si un tercero puede usarla.
"""
import argparse
import sys

LUMA = (0.2126, 0.7152, 0.0722)  # Rec. 709: cuánto aporta cada canal al brillo percibido


def s31_32(x: float) -> int:
    """Coma fija S31.32 en signo-magnitud, el formato de la propiedad CTM de DRM."""
    mag = round(abs(x) * (1 << 32))
    return mag | (1 << 63) if x < 0 else mag


def red_ctm(level: float) -> tuple[int, ...]:
    """R' = level·luma(R,G,B), G' = B' = 0: todo legible, solo en rojo."""
    return tuple(s31_32(v) for v in (*(level * k for k in LUMA), 0, 0, 0, 0, 0, 0))


IDENTITY = tuple(s31_32(v) for v in (1, 0, 0, 0, 1, 0, 0, 0, 1))


def main() -> None:
    from gi.repository import Gio, GLib

    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=["ctm", "gamma"], default="ctm")
    ap.add_argument("level", nargs="?", type=float, default=0.5)
    args = ap.parse_args()

    dc = Gio.DBusProxy.new_for_bus_sync(
        Gio.BusType.SESSION, Gio.DBusProxyFlags.NONE, None,
        "org.gnome.Mutter.DisplayConfig", "/org/gnome/Mutter/DisplayConfig",
        "org.gnome.Mutter.DisplayConfig", None)

    def call(method, sig=None, *values):
        params = GLib.Variant(sig, values) if sig else None
        return dc.call_sync(method, params, Gio.DBusCallFlags.NONE, -1, None).unpack()

    serial, crtcs, outputs, *_ = call("GetResources")
    active_crtcs = [c[0] for c in crtcs if c[6] >= 0]  # c[6] = modo actual, -1 si apagado
    output_ids = [o[0] for o in outputs]
    print(f"serial={serial}  crtcs activos={active_crtcs}  salidas={output_ids}")
    if not (output_ids if args.mode == "ctm" else active_crtcs):
        sys.exit("Mutter no expone salidas/CRTC aquí (¿sesión headless o monitores virtuales?)")

    restore = []
    if args.mode == "ctm":
        for out in output_ids:
            call("SetOutputCTM", "(uu(ttttttttt))", serial, out, red_ctm(args.level))
            restore.append(lambda out=out: call("SetOutputCTM", "(uu(ttttttttt))", serial, out, IDENTITY))
    else:
        for crtc in active_crtcs:
            r, g, b = call("GetCrtcGamma", "(uu)", serial, crtc)
            n = len(r)
            red = [round(65535 * args.level * i / max(n - 1, 1)) for i in range(n)]
            call("SetCrtcGamma", "(uuaqaqaq)", serial, crtc, red, [0] * n, [0] * n)
            restore.append(lambda crtc=crtc, r=r, g=g, b=b:
                           call("SetCrtcGamma", "(uuaqaqaq)", serial, crtc, r, g, b))

    print(f"Rojo al {args.level:.0%} con {args.mode}")
    try:
        input("Enter para restaurar… ")
    finally:
        for undo in restore:
            undo()
        print("Restaurado")


if __name__ == "__main__":
    # Autocomprobación de la codificación, sin GNOME: python3 gnome_red.py --test
    if sys.argv[1:] == ["--test"]:
        assert s31_32(1.0) == 1 << 32
        assert s31_32(-0.5) == (1 << 63) | (1 << 31)
        assert IDENTITY[0] == IDENTITY[4] == IDENTITY[8] == 1 << 32
        assert red_ctm(1.0)[3:] == (0,) * 6
        assert abs(sum(red_ctm(1.0)[:3]) - (1 << 32)) <= 2  # la fila roja suma 1 → blanco = rojo máximo
        print("ok")
    else:
        main()
