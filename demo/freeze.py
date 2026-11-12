#!/usr/bin/env python3
"""Freeze the acceptance-reference hashes (demo/frozen.json).

Replays the frozen command streams, regenerates every demo output with the
release CLI, and records the hashes the gate tests compare against
(mvp.md A1 3D scene gate; P1-7 hash-freeze + human eyeball policy).

Run after intentional visual/semantic changes only:
    cd engine && cargo build --release && python3 ../demo/freeze.py
"""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLI = ROOT / "engine" / "target" / "release" / "mo2d"
TPS = 254_016_000_000
CAMERA = {"type": "perspective", "eye": [320, 200, 600], "target": [320, 190, 120], "fovY": 42}


def run(*args: str) -> str:
    r = subprocess.run([str(CLI), *args], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"mo2d {' '.join(args)} failed:\n{r.stderr}")
    return r.stdout.strip()


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    out = ROOT / "demo" / "out"
    out.mkdir(parents=True, exist_ok=True)

    run("replay", "--stream", str(ROOT / "demo/cmd/tentacle-ai.json"), "--out", str(out / "tentacle.mo2d"))
    run("replay", "--stream", str(ROOT / "demo/cmd/scene3d.json"), "--out", str(out / "scene3d.mo2d"))
    run("export", "--doc", str(out / "tentacle.mo2d"), "--dir", str(out / "tentacle-frames"),
        "--gif", str(out / "tentacle.gif"), "--seconds", "2")
    run("render", "--doc", str(out / "tentacle.mo2d"), "--tick", str(TPS // 2),
        "--out", str(out / "tentacle_mid.png"))
    run("render", "--doc", str(out / "scene3d.mo2d"), "--tick", "0", "--out", str(out / "scene3d.png"),
        "--camera", json.dumps(CAMERA))

    manifest = json.loads((out / "tentacle-frames" / "manifest.json").read_text())
    frozen = {
        "note": "Frozen acceptance references (mvp.md A1 3D scene gate / P1-7 hash freeze). Regenerate via demo/freeze.py.",
        "tentacle": {
            "docSha256": sha(out / "tentacle.mo2d"),
            "frameCount": manifest["frameCount"],
            "firstFrameSha256": manifest["frames"][0]["sha256"],
            "loopFrameSha256": manifest["frames"][24]["sha256"],
            "midFrameSha256": sha(out / "tentacle_mid.png"),
            "midTick": TPS // 2,
        },
        "scene3d": {
            "docSha256": sha(out / "scene3d.mo2d"),
            "frameSha256": sha(out / "scene3d.png"),
            "camera": CAMERA,
        },
    }
    (ROOT / "demo" / "frozen.json").write_text(json.dumps(frozen, indent=2) + "\n")
    print(json.dumps(frozen, indent=2))


if __name__ == "__main__":
    main()
