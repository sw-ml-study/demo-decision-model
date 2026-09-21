//! Diagnostic: for inputs the model assigns to one label, which features pushed
//! it there? A feature's contribution to a label's logit is its embedding row
//! dotted with that label's head column, divided by the feature count. Marks
//! whether each feature was a real training word or borrowed a slot by
//! collision.
//!
//! usage: cargo run -p tdm-model --example blame -- BUNDLE LABEL < inputs.txt

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::io::Read;

use tdm_model::{Bundle, Model, featurize};

const TRAIN: usize = 232;

#[expect(
    clippy::many_single_char_names,
    reason = "b, c, w, d, k follow the notation of model.rs"
)]
fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("usage: blame BUNDLE LABEL < inputs");
    let label = args.next().expect("usage: blame BUNDLE LABEL < inputs");
    let b = Bundle::parse(&std::fs::read_to_string(path).expect("read")).expect("parse");
    let slots = b
        .slots
        .expect("this diagnostic reads a hashed (version 2) bundle");
    let c = b.label_index(&label).expect("no such label");
    let w = b.weights(b.default_snapshot);
    let (d, k) = (b.dim, b.labels.len());
    let mut vocab = HashSet::new();
    let mut owner: HashMap<usize, String> = HashMap::new();
    for s in &b.parity.inputs[..TRAIN] {
        let f = featurize(s, slots, b.width);
        for (t, slot) in f.tokens.iter().zip(&f.slots) {
            vocab.insert(t.clone());
            owner.entry(*slot).or_insert_with(|| t.clone());
        }
    }
    let model = Model::new(&b);
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("stdin");
    let mut blamed: HashMap<String, (f64, usize)> = HashMap::new();
    for line in input.lines().filter(|l| !l.trim().is_empty()) {
        let dec = model.decide(line);
        if dec.selected != c {
            continue;
        }
        let f = featurize(line, slots, b.width);
        #[expect(clippy::cast_precision_loss, reason = "feature counts are tiny")]
        let n = f.slots.len().max(1) as f64;
        let mut parts: Vec<(String, f64)> = f
            .tokens
            .iter()
            .zip(&f.slots)
            .map(|(t, &s)| {
                let v: f64 = (0..d)
                    .map(|j| w.embedding[s * d + j] * w.head[j * k + c])
                    .sum::<f64>()
                    / n;
                let name = if vocab.contains(t) {
                    t.clone()
                } else {
                    format!(
                        "{t}(={})",
                        owner.get(&s).map_or("untrained", String::as_str)
                    )
                };
                (name, v)
            })
            .collect();
        parts.sort_by(|a, x| x.1.total_cmp(&a.1));
        let mut top = String::new();
        for (name, v) in parts.iter().take(3) {
            let _ = write!(top, "{name} {v:+.2}  ");
            let e = blamed
                .entry(name.split('(').next().unwrap_or(name).to_owned())
                .or_insert((0.0, 0));
            e.0 += v;
            e.1 += 1;
        }
        println!("{line:<38} {:.2}  {top}", dec.confidence);
    }
    let mut totals: Vec<_> = blamed.into_iter().filter(|(_, (_, n))| *n > 1).collect();
    totals.sort_by(|a, x| x.1.1.cmp(&a.1.1).then(x.1.0.total_cmp(&a.1.0)));
    println!("\nfeatures most often among the top three pushes toward {label}:");
    for (t, (v, n)) in totals.iter().take(8) {
        println!("  {t:<14} in {n} inputs, total push {v:+.2}");
    }
}
