#!/usr/bin/env python3
"""Generate demo/cmd/hull_turnaround.json — the self-made turnaround fixture.

The analytic truth is an asymmetric star-shaped meridian solid (about the
vertical y axis): ellipsoid base + front nose bump + two ear bumps + a
flattened back and a neck pinch — deliberately NOT a solid of revolution, so
the lifted hull cannot accidentally agree with a lathe profile. For each of
the 12 poses (30° apart, a full loop) the projected silhouette row-intervals
are exact (orthographic projection of a star-shaped cross-section), and they
are emitted as one closed vector contour per pose — the "hand-drawn"
turnaround as pure lines, in document coordinates fitted to 640x360.

Deterministic: pure f64 math, fixed iteration order, no randomness.
Run: python3 -I demo/gen_hull_fixture.py
"""
import json
import math
from pathlib import Path

N_FRAMES = 12
ROWS = 240       # silhouette rows per frame (t samples)
U_SAMPLES = 720  # meridian samples per row
Y0, HEIGHT = 60.0, 240.0
CENTER_X = 320.0  # document center: keeps every silhouette inside 0..640

OUT = Path(__file__).resolve().parent / "cmd" / "hull_turnaround.json"


def angdiff(u: float, c: float) -> float:
    return (u - c + math.pi) % (2.0 * math.pi) - math.pi


def gauss(u: float, c: float, s: float) -> float:
    return math.exp(-(angdiff(u, c) / s) ** 2)


def radius(u: float, t: float) -> float:
    base = 100.0 * math.sin(math.pi * t) ** 0.7
    neck = 1.0 - 0.30 * math.exp(-((t - 0.18) / 0.07) ** 2)
    nose = 0.30 * gauss(u, 0.0, 0.45) * math.exp(-((t - 0.55) / 0.18) ** 2)
    ear_r = 0.12 * gauss(u, math.pi / 2.0, 0.30) * math.exp(-((t - 0.60) / 0.20) ** 2)
    ear_l = 0.12 * gauss(u, -math.pi / 2.0, 0.30) * math.exp(-((t - 0.60) / 0.20) ** 2)
    back = -0.05 * (1.0 + math.cos(u)) / 2.0
    return base * (1.0 + nose + ear_r + ear_l + back) * neck


def frame_contour(theta_deg: float) -> list[tuple[float, float]]:
    """Silhouette outline at pose theta (rotating the object about y).

    Engine convention: rotating the object by R(theta), screen-x of a
    surface point (r cos u, y, r sin u) is r·cos(u − theta).
    """
    th = math.radians(theta_deg)
    left: list[tuple[float, float]] = []
    right: list[tuple[float, float]] = []
    for k in range(ROWS):
        t = k / (ROWS - 1)
        y = Y0 + HEIGHT * t
        xmin = xmax = None
        for m in range(U_SAMPLES):
            u = 2.0 * math.pi * m / U_SAMPLES
            x = radius(u, t) * math.cos(u - th)
            if xmin is None or x < xmin:
                xmin = x
            if xmax is None or x > xmax:
                xmax = x
        left.append((xmin, y))
        right.append((xmax, y))
    poly = left + right[::-1]
    # Drop consecutive duplicates (the caps pinch to the axis at t=0/1).
    out: list[tuple[float, float]] = []
    for p in poly:
        if not out or (abs(p[0] - out[-1][0]) > 1e-9 or abs(p[1] - out[-1][1]) > 1e-9):
            out.append(p)
    if len(out) > 1 and abs(out[0][0] - out[-1][0]) <= 1e-9 and abs(out[0][1] - out[-1][1]) <= 1e-9:
        out.pop()
    return out


def main() -> None:
    cmds: list[dict] = [{"op": "doc.setRoot", "width": 640, "height": 360, "fps": 24}]
    for i in range(N_FRAMES):
        theta = 30.0 * i
        poly = frame_contour(theta)
        nodes = [{"p": [round(CENTER_X + x, 4), round(y, 4), 0]} for x, y in poly]
        cmds.append({
            "op": "line.create",
            "id": f"turn{i}",
            "tags": ["hull-frame", f"frame={i}", f"angle={theta:g}"],
            "closed": True,
            "strokeOn": True,
            "fillOn": False,
            "width": 1.5,
            "color": [30, 30, 45, 255],
            "nodes": nodes,
        })
    stream = {"v": 1, "cmds": cmds}
    OUT.write_text(json.dumps(stream, indent=1) + "\n")
    total = sum(len(c.get("nodes", [])) for c in cmds)
    print(f"wrote {OUT} ({len(cmds) - 1} frames, {total} nodes)")


if __name__ == "__main__":
    main()
