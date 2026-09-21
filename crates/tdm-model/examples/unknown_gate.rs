//! Experiment: what if the model ignored every feature it never trained on, and
//! the policy abstained when no trained *content* word remained? Prints the
//! current decision beside the gated one, for inputs where they differ.
//!
//! usage: cargo run -p tdm-model --example `unknown_gate` -- BUNDLE < inputs.txt

use std::collections::HashSet;
use std::io::Read;

use tdm_model::{Bundle, Features, Model, featurize};

const TRAIN: usize = 232;
/// Words that carry no topic on their own; evidence from these alone is not
/// enough to act on.
const STOP: &[&str] = &[
    "i", "you", "me", "my", "a", "the", "is", "do", "to", "it", "am", "are", "so", "of", "was",
    "have", "been", "be", "that", "this", "and", "all", "what", "can",
];

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: unknown_gate BUNDLE < inputs");
    let b = Bundle::parse(&std::fs::read_to_string(path).expect("read")).expect("parse");
    let slots = b
        .slots
        .expect("this diagnostic reads a hashed (version 2) bundle");
    let vocab: HashSet<String> = b.parity.inputs[..TRAIN]
        .iter()
        .flat_map(|s| featurize(s, slots, b.width).tokens)
        .collect();
    let model = Model::new(&b);
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("stdin");
    let (mut changed, mut abstained) = (0, 0);
    for line in input.lines().filter(|l| !l.trim().is_empty()) {
        let before = model.decide(line);
        let f = featurize(line, slots, b.width);
        let (tokens, slots): (Vec<String>, Vec<usize>) = f
            .tokens
            .iter()
            .zip(&f.slots)
            .filter(|(t, _)| vocab.contains(*t))
            .map(|(t, s)| (t.clone(), *s))
            .unzip();
        let content = tokens
            .iter()
            .any(|t| !t.contains('_') && !STOP.contains(&t.as_str()));
        let after = if content {
            let logits = model.logits(&Features {
                known: vec![true; tokens.len()],
                slots,
                tokens: tokens.clone(),
            });
            let best = (0..logits.len())
                .max_by(|&x, &y| logits[x].total_cmp(&logits[y]))
                .unwrap_or(0);
            b.labels[best].clone()
        } else {
            abstained += 1;
            "ABSTAIN".to_owned()
        };
        if after != b.labels[before.selected] {
            changed += 1;
            println!(
                "{line:<34} {:<8} {:.2} -> {after:<8} [{}]",
                b.labels[before.selected],
                before.confidence,
                tokens.join(" ")
            );
        }
    }
    println!(
        "\n{changed} decisions changed, {abstained} abstained (would reply from the fallback table)"
    );
}
