//! Diagnostic: run one input per line through a bundle and print, for each, the
//! decision, the reply the page would give, the matcher's pick, and how much of
//! the input the model actually has evidence about. "Known" features are those
//! that appeared in the training sentences (the bundle's first `TRAIN` parity
//! inputs); a slot never seen in training contributes only noise to the pool.
//!
//! usage: cargo run -p tdm-model --example probe -- BUNDLE < inputs.txt

use std::collections::HashSet;
use std::io::Read;

use tdm_model::{Bundle, featurize, respond};

/// The exporter writes the 232 training sentences first in the parity set.
const TRAIN: usize = 232;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: probe BUNDLE < inputs");
    let bundle =
        Bundle::parse(&std::fs::read_to_string(path).expect("read bundle")).expect("parse bundle");
    let slots = bundle
        .slots
        .expect("this diagnostic reads a hashed (version 2) bundle");
    let known: HashSet<usize> = bundle.parity.inputs[..TRAIN]
        .iter()
        .flat_map(|s| featurize(s, slots, bundle.width).slots)
        .collect();
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("read stdin");
    for (i, line) in input.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let t = respond(&bundle, line, i);
        let d = &t.decision;
        let k = d
            .features
            .slots
            .iter()
            .filter(|s| known.contains(s))
            .count();
        let known_tokens: Vec<&str> = d
            .features
            .tokens
            .iter()
            .zip(&d.features.slots)
            .filter(|(_, s)| known.contains(s))
            .map(|(tok, _)| tok.as_str())
            .collect();
        println!(
            "{:<34} | {:<8} {:.2} | known {}/{} [{}] | matcher {:<8} | {}",
            line,
            bundle.labels[d.selected],
            d.confidence,
            k,
            d.features.slots.len(),
            known_tokens.join(" "),
            bundle.labels[t.matcher],
            t.reply.text
        );
    }
}
