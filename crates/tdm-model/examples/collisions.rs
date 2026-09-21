//! Diagnostic: how many features of an input are words the model genuinely
//! trained on, versus words it never saw that happen to hash onto a slot some
//! training word owns (and so silently inherit that word's learned meaning).
//!
//! usage: cargo run -p tdm-model --example collisions -- BUNDLE < inputs.txt

use std::collections::{HashMap, HashSet};
use std::io::Read;

use tdm_model::{Bundle, featurize};

const TRAIN: usize = 232;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: collisions BUNDLE < inputs");
    let b = Bundle::parse(&std::fs::read_to_string(path).expect("read")).expect("parse");
    let slots = b
        .slots
        .expect("this diagnostic reads a hashed (version 2) bundle");
    let mut owner: HashMap<usize, String> = HashMap::new();
    let mut vocab: HashSet<String> = HashSet::new();
    for s in &b.parity.inputs[..TRAIN] {
        let f = featurize(s, slots, b.width);
        for (tok, slot) in f.tokens.iter().zip(&f.slots) {
            vocab.insert(tok.clone());
            owner.entry(*slot).or_insert_with(|| tok.clone());
        }
    }
    println!(
        "training vocabulary: {} distinct features in {} of {} slots\n",
        vocab.len(),
        owner.len(),
        slots
    );
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("stdin");
    let (mut seen, mut borrowed, mut empty) = (0, 0, 0);
    for line in input.lines().filter(|l| !l.trim().is_empty()) {
        let f = featurize(line, slots, b.width);
        let mut notes = Vec::new();
        for (tok, slot) in f.tokens.iter().zip(&f.slots) {
            if vocab.contains(tok) {
                seen += 1;
            } else if let Some(o) = owner.get(slot) {
                borrowed += 1;
                notes.push(format!("{tok}->{o}"));
            } else {
                empty += 1;
            }
        }
        if !notes.is_empty() {
            println!("{line:<34} borrows: {}", notes.join(", "));
        }
    }
    let total = seen + borrowed + empty;
    println!("\nfeatures in the probe set: {total}");
    println!(
        "  trained on          {seen:>4} ({:.0}%)",
        100.0 * f64::from(seen) / f64::from(total)
    );
    println!(
        "  unseen, borrowed    {borrowed:>4} ({:.0}%)  <- hash collision with a training feature",
        100.0 * f64::from(borrowed) / f64::from(total)
    );
    println!(
        "  unseen, empty slot  {empty:>4} ({:.0}%)  <- contributes an untrained random vector",
        100.0 * f64::from(empty) / f64::from(total)
    );
}
