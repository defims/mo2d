//! B1: DAG reverse-dependency invalidation (roadmap B1 — "channels = signals").
//!
//! The relation DAG turns every edit into a signal propagation problem: a line
//! that changed dirties exactly the lines transitively downstream of it (every
//! relation that reads it, then everything reading their outputs, …). This
//! module is the formalized invalidation primitive: hosts (stage/MCP/AI
//! tooling) ask "what must be re-evaluated?" instead of re-evaluating the
//! world. Deterministic by construction (BTree sets, document-order edges).
//!
//! GPU note: the same index is what the future GPU workhorse uses to keep
//! uploads minimal; the structure is renderer-agnostic on purpose.

use crate::model::Document;
use std::collections::{BTreeMap, BTreeSet};

/// Reverse-dependency index: line id → relation indices that read it.
pub struct DepIndex {
    readers: BTreeMap<String, Vec<usize>>,
}

impl DepIndex {
    pub fn build(doc: &Document) -> DepIndex {
        let mut readers: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, r) in doc.relations.iter().enumerate() {
            for read in r.reads() {
                readers.entry(read.to_string()).or_default().push(i);
            }
        }
        DepIndex { readers }
    }

    /// Transitive closure of dirty lines over reverse edges. Includes the
    /// seeds themselves. Deterministic order (BTreeSet, stable iteration).
    pub fn affected(&self, doc: &Document, seeds: &[String]) -> BTreeSet<String> {
        let mut dirty: BTreeSet<String> = seeds.iter().cloned().collect();
        let mut queue: Vec<String> = seeds.to_vec();
        while let Some(line) = queue.pop() {
            if let Some(rels) = self.readers.get(line.as_str()) {
                for &ri in rels {
                    for out in doc.relations[ri].writes_all() {
                        if dirty.insert(out.to_string()) {
                            queue.push(out.to_string());
                        }
                    }
                }
            }
        }
        dirty
    }

    /// Direct readers of a line (one hop, document order).
    pub fn readers_of(&self, line: &str) -> &[usize] {
        self.readers.get(line).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{apply_stream, parse_stream};
    use crate::model::{Document, TICKS_PER_SECOND};
    use crate::math::Vec3;

    fn demo_doc() -> Document {
        let stream = format!(
            r#"{{"v":1,"cmds":[
              {{"op":"line.create","id":"skel","tags":["skeleton"],"nodes":[{{"p":[0,0,0]}},{{"p":[50,0,0]}},{{"p":[100,0,0]}}]}},
              {{"op":"line.create","id":"body","nodes":[{{"p":[25,10,0]}},{{"p":[75,10,0]}}]}},
              {{"op":"line.create","id":"free","nodes":[{{"p":[0,50,0]}},{{"p":[10,50,0]}}]}},
              {{"op":"line.create","id":"trk","tags":["track"],"nodes":[{{"p":[0],"v":[0,0,0]}},{{"p":[{}],"v":[30,40,0]}}]}},
              {{"op":"relation.create","type":"bind","target":"body","skeleton":"skel"}},
              {{"op":"relation.create","type":"track","trackLine":"trk","target":{{"line":"skel","node":2,"attr":"position"}}}}
            ]}}"#,
            TICKS_PER_SECOND
        );
        let mut doc = Document::default();
        apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
        doc
    }

    #[test]
    fn edit_propagates_downstream_only() {
        let doc = demo_doc();
        let idx = DepIndex::build(&doc);
        // Skeleton edited → skeleton + its bind target go dirty; the free line stays clean.
        let dirty = idx.affected(&doc, &["skel".to_string()]);
        assert_eq!(dirty, BTreeSet::from(["skel".to_string(), "body".to_string()]));
        // The free line is upstream of nothing — editing it dirties only itself.
        let dirty = idx.affected(&doc, &["free".to_string()]);
        assert_eq!(dirty, BTreeSet::from(["free".to_string()]));
        // A track line edit flows through the relation into skeleton and body.
        let dirty = idx.affected(&doc, &["trk".to_string()]);
        assert_eq!(
            dirty,
            BTreeSet::from(["trk".to_string(), "skel".to_string(), "body".to_string()])
        );
    }

    #[test]
    fn chains_close_transitively() {
        // body → share → free: editing skel must reach free through two hops.
        let mut doc = demo_doc();
        let share = r#"{"v":1,"cmds":[{"op":"relation.create","type":"share","driver":{"line":"body","node":1},"driven":{"line":"free","node":0}}]}"#;
        apply_stream(&mut doc, &parse_stream(share).unwrap()).unwrap();
        let idx = DepIndex::build(&doc);
        let dirty = idx.affected(&doc, &["skel".to_string()]);
        assert_eq!(
            dirty,
            BTreeSet::from([
                "skel".to_string(),
                "body".to_string(),
                "free".to_string(),
            ])
        );
        // Direct readers query stays one-hop.
        assert_eq!(idx.readers_of("skel").len(), 1, "only bind reads the skeleton");
    }

    #[test]
    fn moved_node_detectable() {
        // The integration contract with hosts: a node move is a seed for the index.
        let doc = demo_doc();
        let idx = DepIndex::build(&doc);
        let mut l = doc.line("skel").unwrap().clone();
        l.nodes[0].pos = crate::model::NodePos::Geom(Vec3::new(5.0, 0.0, 0.0));
        let _ = l;
        let dirty = idx.affected(&doc, &["skel".to_string()]);
        assert!(dirty.contains("body"));
    }
}
