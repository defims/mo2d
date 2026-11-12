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
    /// Node-level pressure (B2): stylus pressure shaping the width. Falls
    /// back to 1.0; an explicit node width wins over pressure.
    pub pressure: Option<f64>,
    /// Node-level color override [r,g,b,a], falls back to line color.
    pub color: Option<[u8; 4]>,
    /// Handle offsets stored on the node, acting on the adjacent segments.
    pub handle_in: Option<Vec3>,
    pub handle_out: Option<Vec3>,
    /// Texture anchor (B5 bitmap mesh, line-model mapping 5): UV within the
    /// line's texture, 0..1. Only meaningful on textured closed lines.
    pub uv: Option<[f64; 2]>,
    /// Keyframe value for track nodes (Num for scalars, Arr of 3 for Vec3).
    pub value: Option<Value>,
    pub unknown: Vec<(String, Value)>,
}

impl Node {
    pub fn geom(p: Vec3) -> Node {
        Node {
            pos: NodePos::Geom(p),
            width: None,
            pressure: None,
            color: None,
            handle_in: None,
            handle_out: None,
            uv: None,
            value: None,
            unknown: Vec::new(),
        }
    }
    pub fn track(tick: i64, value: Value) -> Node {
        Node {
            pos: NodePos::Tick(tick),
            width: None,
            pressure: None,
            color: None,
            handle_in: None,
            handle_out: None,
            uv: None,
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

/// Line-level brush style (B2 — the 15-parameter family from
/// moho-mapping.md §7.1). All fields optional: a brush may set any subset.
///
/// Engine-consumed today (deterministic CPU oracle):
/// - `size_amp` amplifies the pressure delta: p_eff = 1 + (p−1)·sizeAmp;
/// - `size_scale` scales all stroke widths;
/// - `tint` multiplies the stroke color component-wise (0..255 → factor/255).
/// The remaining params (jitter/scattering/angle drift/hue drift …) are
/// texture-stamp effects — stored faithfully, rendered once the GPU
/// uber-shader lands (roadmap B1 GPU half). `name` is an external-ref
/// endpoint (line-model.md §4 category 4), carried verbatim.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Brush {
    pub name: Option<String>,
    pub align: Option<f64>,
    pub jitter: Option<f64>,
    pub spacing: Option<f64>,
    pub angle_drift: Option<f64>,
    pub randomize: Option<bool>,
    pub merged_alpha: Option<bool>,
    pub tint: Option<[u8; 4]>,
    pub rand_order: Option<bool>,
    pub size_amp: Option<f64>,
    pub size_scale: Option<f64>,
    pub random_interval: Option<f64>,
    pub hue_drift: Option<f64>,
    pub sat_drift: Option<f64>,
    pub val_drift: Option<f64>,
    pub unknown: Vec<(String, Value)>,
}

impl Brush {
    /// Pressure→width amplifier: 1 + (p − 1)·sizeAmp.
    pub fn pressure_eff(&self, pressure: f64) -> f64 {
        1.0 + (pressure - 1.0) * self.size_amp.unwrap_or(1.0)
    }
    /// Global width scale.
    pub fn size_scale(&self) -> f64 {
        self.size_scale.unwrap_or(1.0)
    }
    /// Tint a stroke color component-wise.
    pub fn tint_color(&self, c: [u8; 4]) -> [u8; 4] {
        match self.tint {
            None => c,
            Some(t) => {
                let mut out = [0u8; 4];
                for i in 0..4 {
                    out[i] = (c[i] as f64 * (t[i] as f64 / 255.0)).round().clamp(0.0, 255.0) as u8;
                }
                out
            }
        }
    }
}

/// Named line-attribute template (B2: "styles = attribute template library").
/// `style.apply` copies the *defined* fields onto a line; undefined fields
/// leave the line untouched (templates merge, never blank).
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub id: String,
    pub width: Option<f64>,
    pub color: Option<[u8; 4]>,
    pub fill_color: Option<[u8; 4]>,
    pub stroke_on: Option<bool>,
    pub fill_on: Option<bool>,
    pub brush: Option<Brush>,
    pub unknown: Vec<(String, Value)>,
}

impl Style {
    pub fn new(id: &str) -> Style {
        Style {
            id: id.to_string(),
            width: None,
            color: None,
            fill_color: None,
            stroke_on: None,
            fill_on: None,
            brush: None,
            unknown: Vec::new(),
        }
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
    /// Line-level brush style (B2).
    pub brush: Option<Brush>,
    /// Texture reference (B5): path of an external image file (non-line
    /// boundary category 4 — the pixels live outside the document). Nodes
    /// carry `uv` anchors; the fill samples the texture. Relative paths
    /// resolve against the document's directory.
    pub texture: Option<String>,
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
            brush: None,
            texture: None,
            nodes: Vec::new(),
            segs: Vec::new(),
            unknown: Vec::new(),
        }
    }

    /// Effective stroke width at node `i` under the B2 vocabulary:
    /// explicit node width wins; else line width × pressure_eff(pressure);
    /// the whole result scales by brush sizeScale. Without brush/pressure
    /// this is exactly the MVP `eff_width` (byte-identical legacy behavior).
    pub fn stroke_width(&self, i: usize) -> f64 {
        let n = &self.nodes[i];
        let base = match n.width {
            Some(w) => w,
            None => {
                let p = n.pressure.unwrap_or(1.0);
                self.width * match &self.brush {
                    Some(b) => b.pressure_eff(p),
                    None => p,
                }
            }
        };
        match &self.brush {
            Some(b) => base * b.size_scale(),
            None => base,
        }
    }

    /// Effective stroke color at node `i` (line/node level, then brush tint).
    pub fn stroke_color(&self, i: usize) -> [u8; 4] {
        let c = self.nodes[i].eff_color(self.color);
        match &self.brush {
            Some(b) => b.tint_color(c),
            None => c,
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
    /// Node color channel (B3): keyframe value = [r,g,b] (alpha kept) or
    /// [r,g,b,a]; this is what makes gradient color stops animatable.
    Color,
}

impl TrackAttr {
    pub fn name(&self) -> &'static str {
        match self {
            TrackAttr::Position => "position",
            TrackAttr::Width => "width",
            TrackAttr::Color => "color",
        }
    }
}

/// What a gradient relation recolors on the target line (B3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientRole {
    Fill,
    Stroke,
}

impl GradientRole {
    pub fn name(&self) -> &'static str {
        match self {
            GradientRole::Fill => "fill",
            GradientRole::Stroke => "stroke",
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
    /// gradient (B3): the bar is a control-bar line; its nodes are the color
    /// stops (node position projected on the bar chord = stop position,
    /// node color = stop color). Target nodes are recolored by projecting
    /// them onto the bar and sampling the stop ramp. Animatable: drive the
    /// stop colors/positions with tracks.
    Gradient,
    /// group (B4): a transform-box line drives a set of member lines. The
    /// rest→current map of the box (anchor = first node, reference = last
    /// node; 2D similarity + z scaled) is applied to every member node, so
    /// dragging/animating the box scales and rotates the whole group.
    /// Layer/group = organizational predefined relation (relations.md §1);
    /// `visible` = the layer visibility parameter.
    Group,
    /// sim (B6): a dynamic relation — the sim line's free nodes integrate
    /// forward (Verlet + distance constraints = trailing tails/ropes).
    /// Anchored nodes ride driven positions (sampled from their tracks per
    /// substep). Deterministic by replay-from-seed: evaluating tick T
    /// re-integrates from tick 0 with fixed 240 Hz substeps — scrub-back and
    /// offline playback match by construction. Randomness has NO place here
    /// (fully determined by anchors + gravity + rest lengths).
    Sim,
    /// particles (B6): an emitter line/node spawns particles — each particle
    /// is a short directed segment (position + velocity vector, line-model
    /// mapping 9). Particles materialize as generated EvalLines; the seed is
    /// an explicit parameter (same document + same seed → same particles).
    Particles,
}

impl RelationKind {
    pub fn name(&self) -> &'static str {
        match self {
            RelationKind::Bind => "bind",
            RelationKind::Share => "share",
            RelationKind::Track => "track",
            RelationKind::Gradient => "gradient",
            RelationKind::Group => "group",
            RelationKind::Sim => "sim",
            RelationKind::Particles => "particles",
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
    // gradient
    pub gradient_bar: String,
    pub gradient_target: String,
    pub gradient_role: GradientRole,
    // group
    pub group_box: String,
    pub group_members: Vec<String>,
    pub group_visible: bool,
    // sim (B6)
    pub sim_line: String,
    /// Anchored nodes: (sim-line node index, driver line, driver node index).
    pub sim_anchors: Vec<(usize, String, usize)>,
    pub sim_gravity: Vec3,
    pub sim_damping: f64,
    pub sim_iterations: usize,
    /// B6 collisions: collider line ids — free sim nodes keep this clearance
    /// from every segment of these lines (projection + push-out).
    pub sim_collide_with: Vec<String>,
    /// Clearance radius (world units); 0 disables collision.
    pub sim_radius: f64,
    /// Chain self-collision: non-adjacent node pairs keep the same clearance.
    pub sim_self_collide: bool,
    // particles (B6)
    pub part_emitter: String,
    pub part_node: usize,
    pub part_rate: f64,
    pub part_speed: f64,
    pub part_dir: Vec3,
    pub part_spread_deg: f64,
    pub part_life_s: f64,
    pub part_gravity: Vec3,
    pub part_damping: f64,
    pub part_size: f64,
    pub part_color: [u8; 4],
    pub part_len: f64,
    pub part_seed: i64,
    /// B6 collisions: collider line ids + restitution (0 = stick, 1 = mirror).
    pub part_collide_with: Vec<String>,
    pub part_bounce: f64,
    pub part_radius: f64,
    pub unknown: Vec<(String, Value)>,
}

impl Relation {
    fn base(id: &str) -> Relation {
        Relation {
            id: id.to_string(),
            kind: RelationKind::Bind,
            bind_target: String::new(),
            bind_skeleton: String::new(),
            bind_falloff: 0.0,
            bind_stretch: true,
            share_driver: (String::new(), 0),
            share_driven: (String::new(), 0),
            track_line: String::new(),
            track_target_line: String::new(),
            track_target_node: None,
            track_attr: TrackAttr::Position,
            gradient_bar: String::new(),
            gradient_target: String::new(),
            gradient_role: GradientRole::Fill,
            group_box: String::new(),
            group_members: Vec::new(),
            group_visible: true,
            sim_line: String::new(),
            sim_anchors: Vec::new(),
            sim_gravity: Vec3::new(0.0, -900.0, 0.0),
            sim_damping: 0.99,
            sim_iterations: 4,
            sim_collide_with: Vec::new(),
            sim_radius: 0.0,
            sim_self_collide: false,
            part_emitter: String::new(),
            part_node: 0,
            part_rate: 30.0,
            part_speed: 120.0,
            part_dir: Vec3::new(0.0, 1.0, 0.0),
            part_spread_deg: 30.0,
            part_life_s: 1.0,
            part_gravity: Vec3::new(0.0, -300.0, 0.0),
            part_damping: 0.98,
            part_size: 2.0,
            part_color: [255, 120, 0, 255],
            part_len: 6.0,
            part_seed: 0,
            part_collide_with: Vec::new(),
            part_bounce: 0.5,
            part_radius: 4.0,
            unknown: Vec::new(),
        }
    }

    pub fn new_bind(id: &str, target: &str, skeleton: &str) -> Relation {
        let mut r = Relation::base(id);
        r.kind = RelationKind::Bind;
        r.bind_target = target.to_string();
        r.bind_skeleton = skeleton.to_string();
        r
    }
    pub fn new_share(id: &str, driver: (&str, usize), driven: (&str, usize)) -> Relation {
        let mut r = Relation::base(id);
        r.kind = RelationKind::Share;
        r.share_driver = (driver.0.to_string(), driver.1);
        r.share_driven = (driven.0.to_string(), driven.1);
        r
    }
    pub fn new_track(id: &str, track_line: &str, target_line: &str, node: Option<usize>, attr: TrackAttr) -> Relation {
        let mut r = Relation::base(id);
        r.kind = RelationKind::Track;
        r.track_line = track_line.to_string();
        r.track_target_line = target_line.to_string();
        r.track_target_node = node;
        r.track_attr = attr;
        r
    }
    pub fn new_gradient(id: &str, bar: &str, target: &str, role: GradientRole) -> Relation {
        let mut r = Relation::base(id);
        r.kind = RelationKind::Gradient;
        r.gradient_bar = bar.to_string();
        r.gradient_target = target.to_string();
        r.gradient_role = role;
        r
    }
    pub fn new_group(id: &str, box_line: &str, members: &[&str], visible: bool) -> Relation {
        let mut r = Relation::base(id);
        r.kind = RelationKind::Group;
        r.group_box = box_line.to_string();
        r.group_members = members.iter().map(|m| m.to_string()).collect();
        r.group_visible = visible;
        r
    }
    pub fn new_sim(id: &str, line: &str) -> Relation {
        let mut r = Relation::base(id);
        r.kind = RelationKind::Sim;
        r.sim_line = line.to_string();
        r
    }
    pub fn new_particles(id: &str, emitter: &str, node: usize) -> Relation {
        let mut r = Relation::base(id);
        r.kind = RelationKind::Particles;
        r.part_emitter = emitter.to_string();
        r.part_node = node;
        r
    }

    /// Lines this relation reads from (inputs).
    pub fn reads(&self) -> Vec<&str> {
        match self.kind {
            RelationKind::Bind => vec![self.bind_skeleton.as_str()],
            RelationKind::Share => vec![self.share_driver.0.as_str()],
            RelationKind::Track => vec![self.track_line.as_str()],
            RelationKind::Gradient => vec![self.gradient_bar.as_str()],
            RelationKind::Group => vec![self.group_box.as_str()],
            // sim reads its anchors (NOT its own line — rest state comes from
            // the document, avoiding a self-edge) plus its collider lines.
            RelationKind::Sim => self
                .sim_anchors
                .iter()
                .map(|(_, l, _)| l.as_str())
                .chain(self.sim_collide_with.iter().map(|l| l.as_str()))
                .collect(),
            // Particles read the emitter plus collider lines (B6 collisions).
            RelationKind::Particles => self
                .part_collide_with
                .iter()
                .map(|l| l.as_str())
                .chain(std::iter::once(self.part_emitter.as_str()))
                .collect(),
        }
    }

    /// Line this relation writes to (output). Relations writing more than one
    /// line (group → members) report the first here; use `writes_all` for the
    /// full set.
    pub fn writes(&self) -> &str {
        match self.kind {
            RelationKind::Bind => &self.bind_target,
            RelationKind::Share => &self.share_driven.0,
            RelationKind::Track => &self.track_target_line,
            RelationKind::Gradient => &self.gradient_target,
            RelationKind::Group => self.group_members.first().map(|s| s.as_str()).unwrap_or(&self.group_box),
            RelationKind::Sim => &self.sim_line,
            // Particles write only generated (virtual) lines — no document
            // line is claimed, so there is no primary write target.
            RelationKind::Particles => &self.part_emitter,
        }
    }

    /// Every line this relation writes (B4 groups write all members).
    pub fn writes_all(&self) -> Vec<&str> {
        match self.kind {
            RelationKind::Group => self.group_members.iter().map(|s| s.as_str()).collect(),
            // Particles claim no document line.
            RelationKind::Particles => vec![],
            _ => vec![self.writes()],
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
    /// Named attribute templates (B2). Absent in old files → empty.
    pub styles: Vec<Style>,
    pub unknown: Vec<(String, Value)>,
}

impl Default for Document {
    fn default() -> Self {
        Document { root: Root::default(), lines: Vec::new(), relations: Vec::new(), styles: Vec::new(), unknown: Vec::new() }
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
    pub fn style(&self, id: &str) -> Result<&Style, String> {
        self.styles.iter().find(|s| s.id == id).ok_or_else(|| format!("style '{}' not found", id))
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
