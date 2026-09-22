//! Replay a conversation, one line per turn, through model 3 and the demo 01
//! script, printing each reply with the Noul answers and the mechanism that
//! produced it.
//!
//! usage: cargo run -p eliza-sim --example talk [SNAPSHOT] < conversation.txt

use std::io::Read;

use tdm_model::{Bundle, Conversation, Script};

const BUNDLE: &str = include_str!("../../../../fixtures/bundles/demo01-model3.json");
const SCRIPT: &str = include_str!("../../../../fixtures/bundles/demo01-script.json");

#[expect(
    clippy::many_single_char_names,
    reason = "b, s, c, x, n are the bundle, script, conversation, exchange and Noul row"
)]
fn main() {
    let b = Bundle::parse(BUNDLE).expect("bundle");
    let s: Script = serde_json::from_str(SCRIPT).expect("script");
    let snap = std::env::args()
        .nth(1)
        .map_or(b.default_snapshot, |a| a.parse().expect("snapshot"));
    let mut c = Conversation::new(&b, &s, snap);
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("stdin");
    println!("ELIZA  {}", b.opening.clone().unwrap_or_default());
    for line in input.lines().filter(|l| !l.trim().is_empty()) {
        let x = c.say(line);
        let n: Vec<String> = x
            .turn
            .decision
            .nouls
            .iter()
            .map(|p| format!("{p:.2}"))
            .collect();
        println!("YOU    {line}");
        println!(
            "ELIZA  {}    [{} {:.2}, q/neg/pos {} -> {:?}]",
            x.reply.text,
            b.labels[x.turn.decision.selected],
            x.turn.decision.confidence,
            n.join("/"),
            x.reply.by
        );
    }
}
