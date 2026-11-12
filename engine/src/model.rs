//! Data model: Line / Node / SegmentAttrs / Relation / Document.
//!
//! Implements the frozen decisions of docs/mvp.md §8:
//! - D1: track = line (node position = Tick, keyframe value = node attr, interp = segment attr)
//! - D3: node position = Vec3 (2D always z=0)
//! - D5: 3-node closed line = triangle face
//! - D7: time = raw Tick integers (254,016,000,000 per second)
//!
//! Every struct keeps an `unknown` list so unknown JSON keys pass through
//! round-trips untouched (file-format.md §2.5: never strip foreign keys).

use crate::json::Value;
use crate::math::Vec3;

/// Ticks per second (docs/file-format.md §2: 254,016,000,000 = 504,000²).
/// 24/25/30/48/50/60/120/240 fps, ms/µs, NTSC and common audio rates all divide it exactly.
pub const TICKS_PER_SECOND: i64 = 254_016_000_000;

/// Tick range guard: ±105 days, leaving 2 bits of i64 headroom (file-format.md §2.1).
pub const TICK_LIMIT: i64 = i64::MAX / 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodePos {
    /// Geometric line node (2D content keeps z = 0).
    Geom(Vec3),
    /// Track line node: raw Tick position (D1/D7).
    Tick(i64),
}

impl NodePos {
    pub fn tick(&self) -> Result<i64, String> {
        match self {
            NodePos::Tick(t) => Ok(*t),
            NodePos::Geom(_) => Err("node is geometric (Vec3), not a track (Tick) node".into()),
        }
    }
    pub fn geom(&self) -> Result<Vec3, String> {
        match self {
            NodePos::Geom(v) => Ok(*v),
            NodePos::Tick(_) => Err("node is a track (Tick) node, not geometric (Vec3)".into()),
        }
    }
}

/// Per-component cubic-bezier handle for Bezier interpolation (im=9),
/// Moho InterpSetting semantics: out handle from key i, in handle into key i+1.
/// ao/ai = angles (radians), po/pi = handle lengths as fractions (0..1) of the
/// key interval in t and of the value delta in v.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezierHandle {
    pub ao: f64,
    pub ai: f64,
    pub po: f64,
    pub pi: f64,
}

impl BezierHandle {
    pub fn default_handle() -> BezierHandle {
        BezierHandle { ao: 0.0, ai: 0.0, po: 1.0 / 3.0, pi: 1.0 / 3.0 }
    }
}

/// Segment-level interpolation attribute (only meaningful on track lines).
#[derive(Debug, Clone, PartialEq)]
pub struct Interp {
    /// 0 = Linear, 9 = Bezier (MVP minimal set, moho-mapping.md §5.2 im codes).
    pub im: i64,
    /// Per-component bezier handles; length must equal the component count of
    /// the track value (1 for scalars, 3 for Vec3). Missing → default handles.
    pub b: Option<Vec<BezierHandle>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegType {
    Straight,
    Curve,
}

/// Segment attrs: the segment's own direct state (never interpolated between segments).
#[derive(Debug, Clone, PartialEq)]
pub struct SegAttrs {
    pub seg_type: SegType,
    pub visible: bool,
    pub interp: Option<Interp>,
    pub unknown: Vec<(String, Value)>,
}

impl Default for SegAttrs {
    fn default() -> Self {
        SegAttrs { seg_type: SegType::Straight, visible: true, interp: None, unknown: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub pos: NodePos,
    /// Node-level width override (world units), falls back to line width.
    pub width: Option<f64>,
    /// Node-level color override [r,g,b,a], falls back to line color.
    pub color: Option<[u8; 4]>,
    /// Handle offsets stored on the node, acting on the adjacent segments.
    pub handle_in: Option<Vec3>,
    pub handle_out: Option<Vec3>,
    /// Keyframe value for track nodes (Num for scalars, Arr of 3 for Vec3).
    pub value: Option<Value>,
    pub unknown: Vec<(String, Value)>,
}

impl Node {
    pub fn geom(p: Vec3) -> Node {
        Node {
            pos: NodePos::Geom(p),
            width: None,
            color: None,
            handle_in: None,
            handle_out: None,
            value: None,
            unknown: Vec::new(),
        }
    }
    pub fn track(tick: i64, value: Value) -> Node {
        Node {
            pos: NodePos::Tick(tick),
            width: None,
            color: None,
            handle_in: None,
            handle_out: None,
            value: Some(value),
            unknown: Vec::new(),
        }
    }
    /// Effective width: node-level override, else the line default.
    pub fn eff_width(&self, line_width: f64) -> f64 {
        self.width.unwrap_or(line_width)
    }
    pub fn eff_color(&self, line_color: [u8; 4]) -> [u8; 4] {
        self.color.unwrap_or(line_color)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub id: String,
    pub tags: Vec<String>,
    pub closed: bool,
    /// Line-level default width (world units).
    pub width: f64,
    /// Line-level stroke color.
    pub color: [u8; 4],
    /// Fill color for closed lines.
    pub fill_color: [u8; 4],
    pub stroke_on: bool,
    pub fill_on: bool,
    pub nodes: Vec<Node>,
    /// Segment attrs: open line → nodes.len()-1 entries, closed → nodes.len().
    pub segs: Vec<SegAttrs>,
    pub unknown: Vec<(String, Value)>,
}

impl Line {
    pub fn new(id: &str) -> Line {
        Line {
            id: id.to_string(),
            tags: Vec::new(),
            closed: false,
            width: 4.0,
            color: [30, 30, 30, 255],
            fill_color: [200, 200, 200, 255],
            stroke_on: true,
            fill_on: false,
            nodes: Vec::new(),
            segs: Vec::new(),
            unknown: Vec::new(),
        }
    }

    pub fn is_track(&self) -> bool {
        !self.nodes.is_empty() && matches!(self.nodes[0].pos, NodePos::Tick(_))
    }

    /// Ensure `segs` has the right length for the node count, filling defaults.
    pub fn sync_segs(&mut self) {
        let want = if self.closed { self.nodes.len() } else { self.nodes.len().saturating_sub(1) };
        while self.segs.len() < want {
            self.segs.push(SegAttrs::default());
        }
        self.segs.truncate(want);
    }

    pub fn seg(&self, i: usize) -> &SegAttrs {
        self.segs.get(i).unwrap_or(&FALLBACK_SEG)
    }
}

static FALLBACK_SEG: SegAttrs = SegAttrs {
    seg_type: SegType::Straight,
    visible: true,
    interp: None,
    unknown: Vec::new(),
};

/// What a track relation writes into.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackAttr {
    Position,
    Width,
}

impl TrackAttr {
    pub fn name(&self) -> &'static str {
        match self {
            TrackAttr::Position => "position",
            TrackAttr::Width => "width",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelationKind {
    /// bind: body line follows skeleton polyline (linear weights via
    /// point-to-segment projection distance; 3d-extension.md decision #4).
    Bind,
    /// node sharing: driven node position = driver node position.
    Share,
    /// track: sample the track line at the current tick, write into target.
    Track,
}

impl RelationKind {
    pub fn name(&self) -> &'static str {
        match self {
            RelationKind::Bind => "bind",
            RelationKind::Share => "share",
            RelationKind::Track => "track",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Relation {
    pub id: String,
    pub kind: RelationKind,
    // bind
    pub bind_target: String,
    pub bind_skeleton: String,
    /// Linear falloff radius in world units. 0 = rigid (all weights 1).
    pub bind_falloff: f64,
    /// Scale the rest offset when the bone stretches.
    pub bind_stretch: bool,
    // share
    pub share_driver: (String, usize),
    pub share_driven: (String, usize),
    // track
    pub track_line: String,
    pub track_target_line: String,
    pub track_target_node: Option<usize>,
    pub track_attr: TrackAttr,
    pub unknown: Vec<(String, Value)>,
}

impl Relation {
    pub fn new_bind(id: &str, target: &str, skeleton: &str) -> Relation {
        Relation {
            id: id.to_string(),
            kind: RelationKind::Bind,
            bind_target: target.to_string(),
            bind_skeleton: skeleton.to_string(),
            bind_falloff: 0.0,
            bind_stretch: true,
            share_driver: (String::new(), 0),
            share_driven: (String::new(), 0),
            track_line: String::new(),
            track_target_line: String::new(),
            track_target_node: None,
            track_attr: TrackAttr::Position,
            unknown: Vec::new(),
        }
    }
    pub fn new_share(id: &str, driver: (&str, usize), driven: (&str, usize)) -> Relation {
        Relation {
            id: id.to_string(),
            kind: RelationKind::Share,
            bind_target: String::new(),
            bind_skeleton: String::new(),
            bind_falloff: 0.0,
            bind_stretch: true,
            share_driver: (driver.0.to_string(), driver.1),
            share_driven: (driven.0.to_string(), driven.1),
            track_line: String::new(),
            track_target_line: String::new(),
            track_target_node: None,
            track_attr: TrackAttr::Position,
            unknown: Vec::new(),
        }
    }
    pub fn new_track(id: &str, track_line: &str, target_line: &str, node: Option<usize>, attr: TrackAttr) -> Relation {
        Relation {
            id: id.to_string(),
            kind: RelationKind::Track,
            bind_target: String::new(),
            bind_skeleton: String::new(),
            bind_falloff: 0.0,
            bind_stretch: true,
            share_driver: (String::new(), 0),
            share_driven: (String::new(), 0),
            track_line: track_line.to_string(),
            track_target_line: target_line.to_string(),
            track_target_node: node,
            track_attr: attr,
            unknown: Vec::new(),
        }
    }

    /// Lines this relation reads from (inputs).
    pub fn reads(&self) -> Vec<&str> {
        match self.kind {
            RelationKind::Bind => vec![self.bind_skeleton.as_str()],
            RelationKind::Share => vec![self.share_driver.0.as_str()],
            RelationKind::Track => vec![self.track_line.as_str()],
        }
    }

    /// Line this relation writes to (output).
    pub fn writes(&self) -> &str {
        match self.kind {
            RelationKind::Bind => &self.bind_target,
            RelationKind::Share => &self.share_driven.0,
            RelationKind::Track => &self.track_target_line,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Root {
    pub width: u32,
    pub height: u32,
    pub ticks_per_second: i64,
    /// Display/export parameter only — time truth is Ticks (D7).
    pub fps: f64,
    pub unknown: Vec<(String, Value)>,
}

impl Default for Root {
    fn default() -> Self {
        Root {
            width: 640,
            height: 360,
            ticks_per_second: TICKS_PER_SECOND,
            fps: 24.0,
            unknown: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub root: Root,
    pub lines: Vec<Line>,
    pub relations: Vec<Relation>,
    pub unknown: Vec<(String, Value)>,
}

impl Default for Document {
    fn default() -> Self {
        Document { root: Root::default(), lines: Vec::new(), relations: Vec::new(), unknown: Vec::new() }
    }
}

impl Document {
    pub fn line(&self, id: &str) -> Result<&Line, String> {
        self.lines.iter().find(|l| l.id == id).ok_or_else(|| format!("line '{}' not found", id))
    }
    pub fn line_mut(&mut self, id: &str) -> Result<&mut Line, String> {
        self.lines.iter_mut().find(|l| l.id == id).ok_or_else(|| format!("line '{}' not found", id))
    }
    pub fn relation(&self, id: &str) -> Result<&Relation, String> {
        self.relations.iter().find(|r| r.id == id).ok_or_else(|| format!("relation '{}' not found", id))
    }

    /// Frame index n (at root fps) → tick. Exact integer math by design.
    pub fn frame_tick(&self, frame: i64) -> i64 {
        let tps = self.root.ticks_per_second.max(1);
        // ticks = frame * tps / fps, rounded; fps is a display parameter.
        ((frame as f64) * (tps as f64) / self.root.fps).round() as i64
    }

    /// Tick → frame index.
    pub fn tick_frame(&self, tick: i64) -> i64 {
        let tps = self.root.ticks_per_second.max(1);
        ((tick as f64) * self.root.fps / (tps as f64)).round() as i64
    }
}
