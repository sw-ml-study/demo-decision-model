//! Replay a conversation, one line per turn, through model 2 and the demo 01
//! script, printing each reply with the mechanism that produced it.
//!
//! usage: cargo run -p eliza-sim --example talk < conversation.txt

use std::io::Read;

use tdm_model::{Bundle, Conversation, Script};

const BUNDLE: &str = include_str!("../../../../fixtures/bundles/demo01-model2.json");
const SCRIPT: &str = include_str!("../../../../fixtures/bundles/demo01-script.json");

fn main() {
    let b = Bundle::parse(BUNDLE).expect("bundle");
    let s: Script = serde_json::from_str(SCRIPT).expect("script");
    let mut c = Conversation::new(&b, &s, b.default_snapshot);
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("stdin");
    println!("ELIZA  {}", b.opening.clone().unwrap_or_default());
    for line in input.lines().filter(|l| !l.trim().is_empty()) {
        let x = c.say(line);
        println!("YOU    {line}");
        println!(
            "ELIZA  {}    [{} {:.2} -> {:?}]",
            x.reply.text,
            b.labels[x.turn.decision.selected],
            x.turn.decision.confidence,
            x.reply.by
        );
    }
}
