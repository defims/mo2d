//! mo2d headless engine —万物皆线 (everything is a line).
//!
//! Layered per docs/architecture.md:
//! - L0 geometry/L1 document: `model`, `format`
//! - L2 evaluation: `eval` (relation DAG, track sampling, bind)
//! - L3 rendering: `render` (CPU oracle, 3D from day one) + `png`/`gif`,
//!   `inval` (reverse-dependency invalidation + damage rendering inputs, B1)
//! - L4 interaction: `command` (protocol v1), `mcp`, `stage`
//! - quality: `validate` (schema + determinism gate), `sha`

pub mod assets;
pub mod command;
pub mod eval;
pub mod export;
pub mod format;
pub mod gif;
#[cfg(target_os = "macos")]
pub mod gpuwgpu;
pub mod inval;
pub mod inflate;
pub mod json;
pub mod math;
pub mod mcp;
pub mod model;
pub mod png;
pub mod render;
pub mod schema;
pub mod sha;
pub mod stage;
pub mod validate;
