//! Prototype, not the shipped policy: replay a conversation under today's policy
//! and under a proposed one, side by side.
//!
//! Proposed policy, all ordinary code around the same model:
//! 1. ignore features the model never trained on;
//! 2. act on the model's choice only when a trained content word remains and
//!    confidence is at least `ACT`;
//! 3. otherwise recall an earlier "my ..." statement, once each, reflected
//!    ("Earlier you said your mom never listens to you."), in the manner of
//!    Weizenbaum's MEMORY rule, which fired when no keyword matched;
//! 4. otherwise reflect the input itself ("Why do you say you can't sleep?");
//! 5. otherwise a generic prompt.
//!
//! Every reply is still a fixed frame plus the user's own words, transformed
//! deterministically, so the no-generation invariant holds.
//!
//! usage: cargo run -p eliza-sim -- BUNDLE < conversation.txt
//!
//! Lives under demos/ rather than crates/ because its reply frames are demo 01's
//! words: the crates stay demo-neutral, and the abstraction gate enforces it.

use std::collections::HashSet;
use std::io::Read;

use tdm_model::{Bundle, Features, Model, featurize, respond};

const TRAIN: usize = 232;
const ACT: f64 = 0.6;
const STOP: &[&str] = &[
    "i", "you", "me", "my", "a", "the", "is", "do", "to", "it", "am", "are", "so", "of", "was",
    "have", "been", "be", "that", "this", "and", "all", "what", "can", "last", "week",
];
const RECALL: &[&str] = &[
    "Earlier you said {}. Tell me more about that.",
    "You mentioned before that {}. Does that still bother you?",
    "Does that have anything to do with the fact that {}?",
];
const REFLECT: &[&str] = &[
    "Why do you say {}?",
    "What makes you say {}?",
    "How does it feel that {}?",
];
const GENERIC: &[&str] = &[
    "Please go on.",
    "I'm not sure I understand you fully.",
    "Can you say more?",
];

/// Pronoun reflection over the user's own words: the deterministic transform
/// ELIZA applied before splicing input into a frame.
fn reflect(text: &str) -> String {
    let lower = text.trim().trim_end_matches(['.', '!', '?']).to_lowercase();
    lower
        .split_whitespace()
        .map(|w| match w {
            "i" => "you",
            "me" | "myself" => {
                if w == "me" {
                    "you"
                } else {
                    "yourself"
                }
            }
            "my" => "your",
            "am" => "are",
            "i'm" => "you're",
            "you" => "I",
            "your" => "my",
            "yours" => "mine",
            "you're" => "I'm",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn fill(frame: &str, slot: &str) -> String {
    frame.replacen("{}", slot, 1)
}

struct Proposed<'a> {
    b: &'a Bundle,
    vocab: HashSet<String>,
    memory: Vec<String>,
    turn: usize,
}

impl Proposed<'_> {
    fn reply(&mut self, line: &str) -> (String, &'static str) {
        let b = self.b;
        let f = featurize(line, b.slots.expect("hashed bundle"), b.width);
        let (tokens, slots): (Vec<String>, Vec<usize>) = f
            .tokens
            .iter()
            .zip(&f.slots)
            .filter(|(t, _)| self.vocab.contains(*t))
            .map(|(t, s)| (t.clone(), *s))
            .unzip();
        let content = tokens
            .iter()
            .any(|t| !t.contains('_') && !STOP.contains(&t.as_str()));
        let words: Vec<String> = tdm_model::words(line);
        let has_my = words.iter().any(|w| w == "my");
        let t = self.turn;
        self.turn += 1;
        let out = if content {
            let logits = Model::new(b).logits(&Features {
                known: vec![true; tokens.len()],
                slots,
                tokens,
            });
            let probs = softmax(&logits);
            let best = (0..probs.len())
                .max_by(|&x, &y| probs[x].total_cmp(&probs[y]))
                .unwrap_or(0);
            if probs[best] >= ACT {
                let r = b.replies(best);
                Some((r[t % r.len()].to_owned(), "model"))
            } else {
                None
            }
        } else {
            None
        };
        let out = out.or_else(|| {
            (!self.memory.is_empty()).then(|| {
                let m = self.memory.remove(0);
                (fill(RECALL[t % RECALL.len()], &reflect(&m)), "memory")
            })
        });
        let out = out.unwrap_or_else(|| {
            if words.len() >= 3 {
                (fill(REFLECT[t % REFLECT.len()], &reflect(line)), "reflect")
            } else {
                (GENERIC[t % GENERIC.len()].to_owned(), "generic")
            }
        });
        if has_my && words.len() >= 3 {
            self.memory.push(line.to_owned());
        }
        out
    }
}

fn softmax(l: &[f64]) -> Vec<f64> {
    let m = l.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let e: Vec<f64> = l.iter().map(|x| (x - m).exp()).collect();
    let s: f64 = e.iter().sum();
    e.iter().map(|x| x / s).collect()
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: dialog_sim BUNDLE < conversation");
    let b = Bundle::parse(&std::fs::read_to_string(path).expect("read")).expect("parse");
    let vocab = b.parity.inputs[..TRAIN]
        .iter()
        .flat_map(|s| featurize(s, b.slots.expect("hashed bundle"), b.width).tokens)
        .collect();
    let mut p = Proposed {
        b: &b,
        vocab,
        memory: Vec::new(),
        turn: 0,
    };
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("stdin");
    for (i, line) in input.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let now = respond(&b, line, i);
        let (next, why) = p.reply(line);
        println!("YOU   {line}");
        println!("  now       {}", now.reply.text);
        println!("  proposed  {next}   [{why}]");
    }
}
