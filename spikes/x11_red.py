#!/usr/bin/env python3
"""Spike: rojo puro en X11 con rampas gamma de XRandR (sin dependencias).

    python3 spikes/x11_red.py [nivel 0..1]

Aplica el rojo a todos los CRTC, espera a que pulses Enter y restaura las
rampas originales. En X11 la rampa NO se restaura sola si el proceso muere:
si se queda roja, `xrandr --output <salida> --gamma 1:1:1`.
"""
import ctypes as C
import signal
import sys

class ScreenResources(C.Structure):
    _fields_ = [("timestamp", C.c_ulong), ("configTimestamp", C.c_ulong),
                ("ncrtc", C.c_int), ("crtcs", C.POINTER(C.c_ulong))]  # resto de campos no usados


class CrtcGamma(C.Structure):
    _fields_ = [("size", C.c_int), ("red", C.POINTER(C.c_ushort)),
                ("green", C.POINTER(C.c_ushort)), ("blue", C.POINTER(C.c_ushort))]


def load_libs():
    """Carga libX11/libXrandr y declara las firmas que usamos."""
    X11, XR = C.CDLL("libX11.so.6"), C.CDLL("libXrandr.so.2")
    X11.XOpenDisplay.restype = C.c_void_p
    X11.XDefaultRootWindow.restype = C.c_ulong
    X11.XDefaultRootWindow.argtypes = [C.c_void_p]
    X11.XSync.argtypes = [C.c_void_p, C.c_int]
    XR.XRRGetScreenResourcesCurrent.restype = C.POINTER(ScreenResources)
    XR.XRRGetScreenResourcesCurrent.argtypes = [C.c_void_p, C.c_ulong]
    XR.XRRGetCrtcGammaSize.argtypes = [C.c_void_p, C.c_ulong]
    XR.XRRGetCrtcGamma.restype = C.POINTER(CrtcGamma)
    XR.XRRGetCrtcGamma.argtypes = [C.c_void_p, C.c_ulong]
    XR.XRRAllocGamma.restype = C.POINTER(CrtcGamma)
    XR.XRRSetCrtcGamma.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(CrtcGamma)]
    return X11, XR


def red_ramp(size: int, level: float) -> list[int]:
    """Rojo lineal de 0 a `level` (0..1) en una rampa de 16 bits."""
    if size < 2:
        return [0] * size
    return [round(65535 * level * i / (size - 1)) for i in range(size)]


def main() -> None:
    level = float(sys.argv[1]) if len(sys.argv) > 1 else 0.5
    X11, XR = load_libs()
    dpy = X11.XOpenDisplay(None)
    if not dpy:
        sys.exit("No hay display X11 (¿estás en Wayland? prueba gnome_red.py)")
    res = XR.XRRGetScreenResourcesCurrent(dpy, X11.XDefaultRootWindow(dpy)).contents
    crtcs = [res.crtcs[i] for i in range(res.ncrtc)]

    saved = {c: XR.XRRGetCrtcGamma(dpy, c) for c in crtcs if XR.XRRGetCrtcGammaSize(dpy, c) > 0}
    print(f"{len(saved)} CRTC con gamma; aplicando rojo al {level:.0%}")

    for crtc, orig in saved.items():
        size = orig.contents.size
        g = XR.XRRAllocGamma(size)
        ramp = red_ramp(size, level)
        for i in range(size):
            g.contents.red[i], g.contents.green[i], g.contents.blue[i] = ramp[i], 0, 0
        XR.XRRSetCrtcGamma(dpy, crtc, g)
    X11.XSync(dpy, 0)

    def restore(*_):
        for crtc, orig in saved.items():
            XR.XRRSetCrtcGamma(dpy, crtc, orig)
        X11.XSync(dpy, 0)
        print("Restaurado")
        sys.exit(0)

    signal.signal(signal.SIGTERM, restore)
    try:
        input("Enter para restaurar… ")
    finally:
        restore()


if __name__ == "__main__":
    # Autocomprobación de la rampa, sin necesidad de display: python3 x11_red.py --test
    if sys.argv[1:] == ["--test"]:
        assert red_ramp(256, 1.0)[0] == 0 and red_ramp(256, 1.0)[-1] == 65535
        assert red_ramp(256, 0.5)[-1] == 32768
        assert red_ramp(1, 1.0) == [0]
        print("ok")
    else:
        main()
