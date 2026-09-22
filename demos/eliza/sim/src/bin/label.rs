//! Label demo 01's oracle corpus: run every input through the DOCTOR script,
//! each in a fresh engine so no earlier line can colour it, and record which
//! rule fired and what it said. Inputs equal to a frozen probe are dropped; the
//! rest are split into training and validation by a stable hash, so the split
//! never depends on file order.
//!
//! usage: `cargo run -p eliza-sim --bin label -- OUT PROBES SCRIPT SOURCE=FILE...`

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;

use tdm_model::{KeywordEngine, KeywordScript, hash, words};

fn norm(s: &str) -> String {
    words(s).join(" ")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (out, probes, script_path) = (&args[0], &args[1], &args[2]);
    let script: KeywordScript =
        serde_json::from_str(&std::fs::read_to_string(script_path).expect("script"))
            .expect("parse script");
    let excluded: HashSet<String> = std::fs::read_to_string(probes)
        .expect("probes")
        .lines()
        .filter_map(|l| l.split('\t').next_back())
        .map(norm)
        .collect();
    let mut seen = HashSet::new();
    let mut rows = String::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let (mut total, mut val) = (0, 0);
    for spec in &args[3..] {
        let (source, file) = spec.split_once('=').expect("SOURCE=FILE");
        for line in std::fs::read_to_string(file).expect("source file").lines() {
            let text = line.rsplit('\t').next().unwrap_or(line).trim();
            let n = norm(text);
            if n.is_empty() || excluded.contains(&n) || !seen.insert(n.clone()) {
                continue;
            }
            let answer = KeywordEngine::new(&script).respond(text);
            let split = if hash(&n) % 10 == 0 { "val" } else { "train" };
            val += usize::from(split == "val");
            total += 1;
            *counts.entry(answer.rule.clone()).or_default() += 1;
            let _ = writeln!(
                rows,
                "{source}\t{split}\t{}\t{text}\t{}",
                answer.rule, answer.text
            );
        }
    }
    std::fs::write(out, rows).expect("write corpus");
    eprintln!("labelled {total} inputs ({val} validation) into {out}");
    let mut by_count: Vec<_> = counts.into_iter().collect();
    by_count.sort_by_key(|x| std::cmp::Reverse(x.1));
    for (rule, n) in &by_count {
        eprintln!("  {rule:<14} {n}");
    }
}
