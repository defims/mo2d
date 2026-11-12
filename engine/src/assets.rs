//! B5 asset store: decode-on-demand texture loading with a budgeted cache.
//!
//! Textures are **external assets** (line-model.md §4 category 4): the
//! document carries only the reference; pixels live in image files. The
//! store decodes a referenced PNG the first time a render asks for it
//! (decode-on-demand, following the BUI design idea) and evicts
//! least-recently-used entries when the byte budget is exceeded.
//!
//! Determinism: the cache never affects output pixels — a decoded image's
//! bytes are fully determined by the file, and eviction only touches memory.
//! Load failures are recorded in `errors` (in first-touch order) and render
//! as the line's plain fill color.

use crate::png::decode_png;
use crate::render::Image;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct AssetStore {
    /// Base directory for relative texture references (the host sets it to
    /// the document's directory).
    pub root: PathBuf,
    cache: HashMap<PathBuf, Arc<Image>>,
    order: Vec<PathBuf>, // front = least recently used
    bytes_used: usize,
    pub budget_bytes: usize,
    /// B7: raw blob payloads from a loaded package, keyed by their document
    /// reference (e.g. "assets/checker.png"). Texture refs that match a blob
    /// resolve here FIRST (the package is self-contained), then filesystem.
    blobs_raw: HashMap<String, Vec<u8>>,
    /// Load failures in first-touch order, deduplicated per path
    /// (informational; the host reports them, the renderer falls back to
    /// flat fill).
    pub errors: Vec<String>,
    failed: std::collections::BTreeSet<PathBuf>,
}

/// Default texture cache budget: 256 MiB of decoded pixels.
pub const DEFAULT_BUDGET_BYTES: usize = 256 * 1024 * 1024;

impl Default for AssetStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetStore {
    pub fn new() -> AssetStore {
        AssetStore {
            root: PathBuf::from("."),
            cache: HashMap::new(),
            order: Vec::new(),
            bytes_used: 0,
            budget_bytes: DEFAULT_BUDGET_BYTES,
            errors: Vec::new(),
            failed: std::collections::BTreeSet::new(),
            blobs_raw: HashMap::new(),
        }
    }

    /// B7: attach a package's blob area — texture references that match a
    /// blob name resolve to the embedded payload instead of the filesystem.
    pub fn attach_blobs(&mut self, blobs: &[(String, Vec<u8>)]) {
        for (name, data) in blobs {
            self.blobs_raw.insert(name.clone(), data.clone());
        }
    }

    /// Resolve a texture reference: absolute paths pass through; relative
    /// paths join the store root (the document's directory).
    pub fn resolve(&self, raw: &str) -> PathBuf {
        let p = Path::new(raw);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.root.join(p)
        }
    }

    /// Look up a texture by its document reference (raw string from the
    /// line): blob area first (B7 packages are self-contained), then the
    /// filesystem under `root`.
    pub fn get_ref(&mut self, raw: &str) -> Option<Arc<Image>> {
        if self.blobs_raw.contains_key(raw) {
            let key = PathBuf::from(format!("\u{0000}blob:{}", raw)); // map key only, never a real path
            if self.cache.contains_key(&key) {
                self.touch(&key);
                return self.cache.get(&key).cloned();
            }
            let bytes = self.blobs_raw.get(raw).cloned().unwrap_or_default();
            return self.decode_and_cache(key, &bytes);
        }
        let path = self.resolve(raw);
        self.get(&path)
    }

    fn decode_and_cache(&mut self, key: PathBuf, bytes: &[u8]) -> Option<Arc<Image>> {
        let img = match decode_png(bytes) {
            Ok(i) => Arc::new(i),
            Err(e) => {
                self.record_failure(&key, &format!("texture '{}' undecodable: {}", key.display(), e));
                return None;
            }
        };
        let entry_bytes = img.px.len();
        self.cache.insert(key.clone(), Arc::clone(&img));
        self.order.push(key);
        self.bytes_used += entry_bytes;
        while self.bytes_used > self.budget_bytes && self.order.len() > 1 {
            let evict = self.order.remove(0);
            if let Some(img) = self.cache.remove(&evict) {
                self.bytes_used -= img.px.len();
            }
        }
        Some(img)
    }

    /// Decoded image for `path`, loading on first touch. Returns None when
    /// the file is missing or not a decodable PNG (the error is recorded
    /// once per path).
    pub fn get(&mut self, path: &Path) -> Option<Arc<Image>> {
        let key = path.to_path_buf();
        if self.cache.contains_key(&key) {
            self.touch(&key);
            return self.cache.get(&key).cloned();
        }
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                self.record_failure(&key, &format!("texture '{}' unreadable: {}", path.display(), e));
                return None;
            }
        };
        let img = match decode_png(&bytes) {
            Ok(i) => Arc::new(i),
            Err(e) => {
                self.record_failure(&key, &format!("texture '{}' undecodable: {}", path.display(), e));
                return None;
            }
        };
        let entry_bytes = img.px.len();
        self.cache.insert(key.clone(), Arc::clone(&img));
        self.order.push(key);
        self.bytes_used += entry_bytes;
        // Evict LRU entries until within budget (never evicts the just-loaded
        // entry unless it alone exceeds the budget — then the cache stays at
        // one entry).
        while self.bytes_used > self.budget_bytes && self.order.len() > 1 {
            let evict = self.order.remove(0);
            if let Some(img) = self.cache.remove(&evict) {
                self.bytes_used -= img.px.len();
            }
        }
        Some(img)
    }

    fn record_failure(&mut self, key: &PathBuf, msg: &str) {
        if self.failed.insert(key.clone()) {
            self.errors.push(msg.to_string());
        }
    }

    fn touch(&mut self, key: &PathBuf) {
        if let Some(pos) = self.order.iter().position(|p| p == key) {
            let k = self.order.remove(pos);
            self.order.push(k);
        }
    }

    /// Number of live cache entries (exposed for tests/diagnostics).
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("demo/assets/checker.png")
    }

    #[test]
    fn decodes_on_demand_and_caches() {
        let mut s = AssetStore::new();
        assert!(s.is_empty());
        let a = s.get(&fixture()).expect("checker.png decodes");
        assert_eq!((a.w, a.h), (128, 128));
        let b = s.get(&fixture()).expect("second touch");
        assert!(Arc::ptr_eq(&a, &b), "same entry reused");
        assert_eq!(s.len(), 1);
        assert!(s.errors.is_empty());
    }

    #[test]
    fn missing_and_undecodable_are_recorded() {
        let mut s = AssetStore::new();
        let missing = std::path::Path::new("/nonexistent/no-such-texture.png");
        assert!(s.get(missing).is_none());
        assert_eq!(s.errors.len(), 1);
        assert!(s.get(missing).is_none());
        assert_eq!(s.errors.len(), 1, "no duplicate error entries for the same repeated miss");
        // Not-a-PNG file.
        let bad = std::env::temp_dir().join("mo2d-test-not-a-png.png");
        std::fs::write(&bad, b"definitely not png").unwrap();
        assert!(s.get(&bad).is_none());
        assert!(s.errors.iter().any(|e| e.contains("undecodable")));
        let _ = std::fs::remove_file(&bad);
    }

    #[test]
    fn budget_evicts_lru_but_never_the_only_entry() {
        let mut s = AssetStore::new();
        s.budget_bytes = 0; // force eviction on every insert
        let a = s.get(&fixture()).expect("load");
        assert_eq!(s.len(), 1, "the only entry survives even over budget");
        let _ = a;
    }
}
