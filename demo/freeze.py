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

    # B5: textures are external assets resolved relative to the document —
    # the frozen .mo2d carries its assets/ alongside (project-directory form).
    import shutil
    (out / "assets").mkdir(exist_ok=True)
    shutil.copyfile(ROOT / "demo/assets/checker.png", out / "assets/checker.png")

    # Phase B demos (roadmap B2 brush / B3 gradient / B4 groups / B6 sim).
    b_demos = {
        "brush": "brush-styles.json",
        "gradient": "gradient-tentacle.json",
        "groups": "stage-groups.json",
        "sim": "sim-tail.json",
    }
    frozen_b = {}
    for key, name in b_demos.items():
        doc_path = out / f"{key}.mo2d"
        png_path = out / f"{key}.png"
        run("replay", "--stream", str(ROOT / "demo/cmd" / name), "--out", str(doc_path))
        run("render", "--doc", str(doc_path), "--tick", str(TPS // 2), "--out", str(png_path))
        frozen_b[key] = {"docSha256": sha(doc_path), "frameSha256": sha(png_path), "midTick": TPS // 2}

    # Phase C7 interactivity demos (roadmap C7: oscillate breathe/sway +
    # the respond/event-stream tap_jump — the N3 gate, offline no-AI playback
    # + record_wave, the perform-record transcript: gesture commands → track keys).
    c_demos = {
        "breathe": "breathe.json",
        "sway": "sway.json",
        "tapJump": "tap_jump.json",
        "recordWave": "record_wave.json",
        "tapVsHold": "tap_vs_hold.json",
    }
    for key, name in c_demos.items():
        doc_path = out / f"{key}.mo2d"
        png_path = out / f"{key}.png"
        run("replay", "--stream", str(ROOT / "demo/cmd" / name), "--out", str(doc_path))
        run("render", "--doc", str(doc_path), "--tick", str(TPS // 2), "--out", str(png_path))
        frozen_b[key] = {"docSha256": sha(doc_path), "frameSha256": sha(png_path), "midTick": TPS // 2}

    # B5+B7 mesh demo: the media project in BOTH container forms — the
    # project directory (document.mo2d.json + assets/) and the single-file
    # package (JSON header + tail BLOB area). Acceptance: the package renders
    # byte-identically to the directory form with no assets dir present.
    mesh_proj = out / "mesh-project"
    (mesh_proj / "assets").mkdir(parents=True, exist_ok=True)
    run("replay", "--stream", str(ROOT / "demo/cmd/mesh-bitmap.json"),
        "--out", str(mesh_proj / "document.mo2d.json"))
    shutil.copyfile(ROOT / "demo/assets/checker.png", mesh_proj / "assets/checker.png")
    run("render", "--doc", str(mesh_proj / "document.mo2d.json"), "--tick", str(TPS // 2),
        "--out", str(out / "mesh.png"))
    run("pack", "--dir", str(mesh_proj), "--out", str(out / "mesh-package.mo2d"))
    run("render", "--doc", str(out / "mesh-package.mo2d"), "--tick", str(TPS // 2),
        "--out", str(out / "mesh-package-frame.png"))
    if sha(out / "mesh-package-frame.png") != sha(out / "mesh.png"):
        sys.exit("B7 acceptance failed: the single-file package must render byte-identically to the project directory")
    frozen_b["mesh"] = {
        "docSha256": sha(mesh_proj / "document.mo2d.json"),
        "frameSha256": sha(out / "mesh.png"),
        "packageSha256": sha(out / "mesh-package.mo2d"),
        "midTick": TPS // 2,
    }

    # Hull probe demo (docs/log.md 2026-11-17): one turnaround timeline lifts
    # into a visual hull, then derives three pose trajectories as new 2D
    # contour timelines. Bespoke block: artifacts come from the `mo2d hull`
    # subcommand, hashed per derived timeline (purely additive).
    hull_doc = out / "hull_turnaround.mo2d"
    run("replay", "--stream", str(ROOT / "demo/cmd/hull_turnaround.json"), "--out", str(hull_doc))
    run("render", "--doc", str(hull_doc), "--tick", "0", "--out", str(out / "hull_turnaround.png"))
    hull_summary = json.loads(run("hull", "--doc", str(hull_doc), "--out", str(out / "hull")))
    frozen_b["hull"] = {
        "docSha256": sha(hull_doc),
        "frameSha256": sha(out / "hull_turnaround.png"),
        "midTick": 0,
        "derived": {
            "yawSwing": hull_summary["yawSwing"],
            "pitchNod": hull_summary["pitchNod"],
            "orbitDiagonal": hull_summary["orbitDiagonal"],
        },
        "slices": hull_summary["slices"],
        "grid": hull_summary["grid"],
    }

    manifest = json.loads((out / "tentacle-frames" / "manifest.json").read_text())
    frozen = {
        "note": "Frozen acceptance references (mvp.md A1 3D scene gate / P1-7 hash freeze; Phase B demos). Regenerate via demo/freeze.py.",
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
    frozen.update(frozen_b)
    (ROOT / "demo" / "frozen.json").write_text(json.dumps(frozen, indent=2) + "\n")
    print(json.dumps(frozen, indent=2))


if __name__ == "__main__":
    main()
