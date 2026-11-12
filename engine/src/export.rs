//! Export: PNG frame sequence (primary, per-frame hash verifiable) + GIF
//! (bonus) + manifest with frozen hashes (mvp.md §2.5).

use crate::eval;
use crate::gif::{encode_gif, GifFrame};
use crate::json::Value;
use crate::model::Document;
use crate::png::encode_png;
use crate::assets::AssetStore;
use crate::render::{render_with_assets, Camera, RenderOpts};
use crate::sha::sha256_hex;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ExportOpts {
    /// Inclusive tick range.
    pub from_tick: i64,
    pub to_tick: i64,
    /// Ticks per output frame.
    pub step: i64,
    pub camera: Camera,
    /// Output pixel size (defaults to root size).
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Base directory for B5 texture references (set to the doc's dir).
    pub asset_root: std::path::PathBuf,
}

impl ExportOpts {
    /// One second of animation at the document's fps.
    pub fn one_second(doc: &Document) -> ExportOpts {
        let tps = doc.root.ticks_per_second;
        ExportOpts {
            from_tick: 0,
            to_tick: tps - 1,
            step: ((tps as f64 / doc.root.fps.max(0.001)).round() as i64).max(1),
            camera: Camera::Ortho2d,
            width: None,
            height: None,
            asset_root: std::path::PathBuf::from("."),
        }
    }
}

pub struct ExportResult {
    pub frames: Vec<(String, String)>, // (file name, sha256)
    pub manifest: Value,
}

/// Render and write the PNG sequence + manifest.json into `dir`.
pub fn export_frames(doc: &Document, opts: &ExportOpts, dir: &Path) -> Result<ExportResult, String> {
    fs::create_dir_all(dir).map_err(|e| format!("create {}: {}", dir.display(), e))?;
    let width = opts.width.unwrap_or(doc.root.width);
    let height = opts.height.unwrap_or(doc.root.height);

    let mut frames = Vec::new();
    let mut tick = opts.from_tick;
    let mut index = 0u64;
    while tick <= opts.to_tick {
        let lines = eval::evaluate(doc, tick)?;
        let mut assets = AssetStore::new();
        assets.root = opts.asset_root.clone();
        let img = render_with_assets(
            &lines,
            doc.root.width as f64,
            doc.root.height as f64,
            &RenderOpts { width, height, camera: opts.camera.clone(), samples: 1 },
            &mut assets,
        );
        let png = encode_png(&img);
        let name = format!("frame_{:06}.png", index);
        fs::write(dir.join(&name), &png).map_err(|e| format!("write {}: {}", name, e))?;
        frames.push((name.clone(), sha256_hex(&png)));
        tick += opts.step;
        index += 1;
    }
    if frames.is_empty() {
        return Err("empty frame range (from_tick > to_tick)".into());
    }

    let doc_bytes = crate::format::doc_to_bytes(doc)?;
    let mut manifest = Value::obj();
    manifest.set("format", Value::Str("mo2d-export".into()));
    manifest.set("version", Value::Int(1));
    manifest.set("docSha256", Value::Str(sha256_hex(&doc_bytes)));
    manifest.set("generator", Value::Str(crate::format::GENERATOR.into()));
    let mut root = Value::obj();
    root.set("width", Value::Int(doc.root.width as i64));
    root.set("height", Value::Int(doc.root.height as i64));
    root.set("ticksPerSecond", Value::Int(doc.root.ticks_per_second));
    root.set("fps", Value::Num(doc.root.fps));
    manifest.set("root", root);
    manifest.set("renderWidth", Value::Int(width as i64));
    manifest.set("renderHeight", Value::Int(height as i64));
    let mut fr = Vec::new();
    for (i, (name, hash)) in frames.iter().enumerate() {
        let mut f = Value::obj();
        f.set("file", Value::Str(name.clone()));
        f.set("tick", Value::Int(opts.from_tick + i as i64 * opts.step));
        f.set("frame", Value::Int(i as i64));
        f.set("sha256", Value::Str(hash.clone()));
        fr.push(f);
    }
    manifest.set("frames", Value::Arr(fr));
    manifest.set("frameCount", Value::Int(frames.len() as i64));

    let manifest_text = crate::json::to_string(&manifest)? + "\n";
    fs::write(dir.join("manifest.json"), manifest_text).map_err(|e| format!("write manifest: {}", e))?;

    Ok(ExportResult { frames, manifest })
}

/// Encode the same range as an animated GIF at `path`.
pub fn export_gif(doc: &Document, opts: &ExportOpts, path: &Path) -> Result<(), String> {
    let width = opts.width.unwrap_or(doc.root.width);
    let height = opts.height.unwrap_or(doc.root.height);
    let tps = doc.root.ticks_per_second;
    let delay_cs = ((opts.step as f64) * 100.0 / tps as f64).round().clamp(1.0, 65535.0) as u16;

    let mut images = Vec::new();
    let mut tick = opts.from_tick;
    while tick <= opts.to_tick {
        let lines = eval::evaluate(doc, tick)?;
        let mut assets = AssetStore::new();
        assets.root = opts.asset_root.clone();
        let img = render_with_assets(
            &lines,
            doc.root.width as f64,
            doc.root.height as f64,
            &RenderOpts { width, height, camera: opts.camera.clone(), samples: 1 },
            &mut assets,
        );
        images.push(img);
        tick += opts.step;
    }
    if images.is_empty() {
        return Err("empty frame range".into());
    }
    let frames: Vec<GifFrame<'_>> = images.iter().map(|img| GifFrame { img, delay_cs }).collect();
    let bytes = encode_gif(&frames, width, height);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {}", parent.display(), e))?;
    }
    fs::write(path, bytes).map_err(|e| format!("write {}: {}", path.display(), e))?;
    Ok(())
}
