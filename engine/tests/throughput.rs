//! Completion definition 5 — evaluation throughput: a synthetic stress
//! document (command-stream-generated, 10k nodes) evaluates a single frame in
//! < 16 ms on the CPU.

mod common;

use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::model::{Document, TICKS_PER_SECOND};
use std::time::Instant;

fn stress_doc(nodes: usize) -> Document {
    let mut node_json = Vec::new();
    for i in 0..nodes {
        let x = (i as f64) * 0.1;
        let y = ((i as f64) * 0.37).sin() * 80.0 + 180.0;
        node_json.push(format!("{{\"p\":[{:.4},{:.4},0],\"w\":{}}}", x, y, 2.0 + (i % 20) as f64 * 0.5));
    }
    let stream = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"stress","width":3,"nodes":[{}]}},
          {{"op":"line.create","id":"trk","tags":["track"],"nodes":[{{"p":[0],"v":2.0}},{{"p":[{}],"v":8.0}}]}},
          {{"op":"relation.create","type":"track","trackLine":"trk","target":{{"line":"stress","attr":"width"}}}}
        ]}}"#,
        node_json.join(","),
        TICKS_PER_SECOND
    );
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    doc
}

#[test]
fn throughput_10k_nodes_under_16ms() {
    let doc = stress_doc(10_000);
    assert_eq!(doc.line("stress").unwrap().nodes.len(), 10_000);
    assert!(mo2d::validate::validate(&doc).is_empty(), "stress doc must validate");

    // Warm up, then median of 31 single-frame evaluations.
    eval::evaluate(&doc, 0).unwrap();
    let mut samples = Vec::new();
    for run in 0..31 {
        let tick = TICKS_PER_SECOND / 3 + run as i64;
        let t = Instant::now();
        let lines = eval::evaluate(&doc, tick).unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        std::hint::black_box(&lines);
        samples.push(ms);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = samples[samples.len() / 2];
    eprintln!("throughput: 10k nodes, median single-frame eval = {:.3} ms", median);
    assert!(median < 16.0, "median single-frame evaluation must be < 16 ms, got {:.3} ms", median);
}

/// Roadmap B1 acceptance class: 100k nodes. The GPU workhorse owns the 60 fps
/// render target (tech-stack decision pending); this gate pins the *engine*
/// half — single-frame evaluation of the 100k-node class stays far inside the
/// 16 ms frame budget even on the CPU.
#[test]
fn throughput_100k_eval_under_16ms() {
    let doc = stress_doc(100_000);
    assert_eq!(doc.line("stress").unwrap().nodes.len(), 100_000);

    eval::evaluate(&doc, 0).unwrap();
    let mut samples = Vec::new();
    for run in 0..15 {
        let tick = TICKS_PER_SECOND / 3 + run as i64;
        let t = Instant::now();
        let lines = eval::evaluate(&doc, tick).unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        std::hint::black_box(&lines);
        samples.push(ms);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = samples[samples.len() / 2];
    eprintln!("throughput: 100k nodes, median single-frame eval = {:.3} ms", median);
    assert!(median < 16.0, "100k-node single-frame evaluation must be < 16 ms, got {:.3} ms", median);
}
