//! Featurize demo 01's oracle corpus for model 4. MLPL trains; this only does
//! the bookkeeping the interpreter is slow at over eight thousand rows: the
//! class list, the vocabulary, and the id and mask matrices.
//!
//! - **Classes** are the DOCTOR rules the oracle used. A rule with fewer than
//!   `eliza_sim::MIN_RULE` training examples is merged into its keyword's most common
//!   rule, or into `xnone#0` when that keyword has none common enough.
//! - **Vocabulary** is exact: every word the script itself names (keywords,
//!   synonyms, pattern words) that occurs in training, then the training
//!   split's most frequent other words and bigrams (after a question mark
//!   becomes `qmark`) seen at least twice, `MAX_VOCAB` in all. Row 0 is
//!   reserved for unknown and padding. Without the first rule, "hello" -- a
//!   keyword, and most people's first line -- lost an alphabetical tie at the
//!   frequency cutoff.
//! - **Nouls** are masked. Every row knows whether it is a question: a
//!   generated sentence from its tag, anything else from whether its author
//!   ended it with a question mark. Only generated sentences carry negative
//!   and positive labels; elsewhere those two columns are masked out.
//!
//! usage: `cargo run -p eliza-sim --bin featurize -- CORPUS FRAME_NOULS PROBES PROBE_NOULS SCRIPT OUT`

use std::collections::{BTreeSet, HashMap};

use eliza_sim::{CATCH_ALL, class_of, read_corpus, rule_classes};
use serde_json::{Value, json};
use tdm_model::{KeywordEngine, KeywordScript, tokens, words};

const WIDTH: usize = 24;
const MAX_VOCAB: usize = 2000;
const QUESTION_TOKEN: &str = "qmark";
const EXAMPLES: [&str; 10] = [
    "I don't have any problems",
    "my mom never listens to me",
    "are you a computer?",
    "I don't know",
    "i lost my job last week",
    "i got promoted today",
    "everyone hates me",
    "what should i do?",
    "my job is killing me",
    "thank you",
];

struct Row {
    text: String,
    rule: String,
    nouls: [f64; 3],
    mask: [f64; 3],
}

fn prep(text: &str) -> String {
    text.replace('?', &format!(" {QUESTION_TOKEN} "))
}

fn norm(s: &str) -> String {
    words(s).join(" ")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn bits(fields: &[&str]) -> [f64; 3] {
    let b = |i: usize| f64::from(u8::from(fields[i] == "1"));
    [b(0), b(1), b(2)]
}

/// Every word the script matches on: keywords, synonym members, and the
/// literal words of decomposition patterns.
fn script_words(script: &KeywordScript) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for k in &script.keys {
        out.insert(k.key.clone());
        for d in &k.decomps {
            out.extend(
                d.pattern
                    .split_whitespace()
                    .filter(|w| *w != "*" && !w.starts_with('@'))
                    .map(str::to_owned),
            );
        }
    }
    out.extend(script.synonyms.values().flatten().cloned());
    out
}

fn vocabulary(train: &[Row], script: &KeywordScript) -> Vec<String> {
    let mut count: HashMap<String, usize> = HashMap::new();
    for r in train {
        for t in tokens(&prep(&r.text), WIDTH) {
            *count.entry(t).or_default() += 1;
        }
    }
    let named: Vec<String> = script_words(script)
        .into_iter()
        .filter(|w| count.contains_key(w))
        .collect();
    let mut rest: Vec<(String, usize)> = count
        .into_iter()
        .filter(|(t, n)| *n >= 2 && !named.contains(t))
        .collect();
    rest.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rest.truncate(MAX_VOCAB.saturating_sub(named.len()));
    named
        .into_iter()
        .chain(rest.into_iter().map(|(t, _)| t))
        .collect()
}

/// Ids and known-masks, [N, WIDTH] each, flattened row-major.
fn matrices(texts: &[&str], index: &HashMap<&str, usize>) -> (Vec<usize>, Vec<u8>) {
    let (mut ids, mut mask) = (Vec::new(), Vec::new());
    for text in texts {
        let toks = tokens(&prep(text), WIDTH);
        for i in 0..WIDTH {
            let row = toks.get(i).and_then(|t| index.get(t.as_str())).copied();
            ids.push(row.unwrap_or(0));
            mask.push(u8::from(row.is_some()));
        }
    }
    (ids, mask)
}

fn batch(rows: &[Row], index: &HashMap<&str, usize>, class: &HashMap<String, usize>) -> Value {
    let texts: Vec<&str> = rows.iter().map(|r| r.text.as_str()).collect();
    let (ids, mask) = matrices(&texts, index);
    json!({
        "n": rows.len(),
        "ids": ids,
        "mask": mask,
        "y": rows.iter().map(|r| class[&r.rule]).collect::<Vec<_>>(),
        "nouls": rows.iter().flat_map(|r| r.nouls).collect::<Vec<_>>(),
        "noul_mask": rows.iter().flat_map(|r| r.mask).collect::<Vec<_>>(),
    })
}

/// The frozen probes, labelled by the oracle, with their hand-tagged Nouls.
fn probes_labelled(probes: &str, probe_nouls: &str, script: &KeywordScript) -> Vec<Row> {
    let probe_tags: HashMap<String, [f64; 3]> = read(probe_nouls)
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f.len() == 4).then(|| (norm(f[3]), bits(&f[..3])))
        })
        .collect();
    read(probes)
        .lines()
        .filter_map(|l| l.split('\t').nth(1))
        .map(|text| Row {
            text: text.to_owned(),
            rule: KeywordEngine::new(script).respond(text).rule,
            nouls: probe_tags[&norm(text)],
            mask: [1.0; 3],
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [corpus, frame_nouls, probes, probe_nouls, script, out] = &args[..] else {
        panic!("usage: featurize CORPUS FRAME_NOULS PROBES PROBE_NOULS SCRIPT OUT");
    };
    let tagged: HashMap<String, [f64; 3]> = read(frame_nouls)
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f.len() == 4).then(|| (norm(f[0]), bits(&f[1..])))
        })
        .collect();
    let (mut train, mut val) = (Vec::new(), Vec::new());
    for c in read_corpus(corpus) {
        let (nouls, mask) = match tagged.get(&norm(&c.text)) {
            Some(n) if c.source == "frames" => (*n, [1.0; 3]),
            _ => {
                let q = f64::from(u8::from(c.text.trim_end().ends_with('?')));
                ([q, 0.0, 0.0], [1.0, 0.0, 0.0])
            }
        };
        let row = Row {
            text: c.text,
            rule: c.rule,
            nouls,
            mask,
        };
        if c.val {
            val.push(row);
        } else {
            train.push(row);
        }
    }
    let script: KeywordScript = serde_json::from_str(&read(script)).expect("parse script");
    let probe_rows = probes_labelled(probes, probe_nouls, &script);

    let (labels, merged) = rule_classes(train.iter().map(|r| r.rule.as_str()));
    let mut moves: Vec<_> = merged.iter().collect();
    moves.sort();
    for (rule, target) in moves {
        if rule != target {
            eprintln!("  merged {rule} into {target}");
        }
    }
    let position: HashMap<&str, usize> = labels
        .iter()
        .enumerate()
        .map(|(i, l)| (l.as_str(), i))
        .collect();
    let class: HashMap<String, usize> = train
        .iter()
        .chain(&val)
        .chain(&probe_rows)
        .map(|r| (r.rule.clone(), position[class_of(&merged, &r.rule)]))
        .collect();
    debug_assert!(position.contains_key(CATCH_ALL));
    let vocab = vocabulary(&train, &script);
    let index: HashMap<&str, usize> = vocab
        .iter()
        .enumerate()
        .map(|(i, t)| (t.as_str(), i + 1))
        .collect();
    let parity: Vec<&str> = probe_rows
        .iter()
        .map(|r| r.text.as_str())
        .chain(EXAMPLES)
        .collect();
    let (pids, pmask) = matrices(&parity, &index);
    let data = json!({
        "labels": labels,
        "vocab": vocab,
        "width": WIDTH,
        "train": batch(&train, &index, &class),
        "val": batch(&val, &index, &class),
        "probes": batch(&probe_rows, &index, &class),
        "parity": {"n": parity.len(), "texts": parity, "ids": pids, "mask": pmask},
        "examples": EXAMPLES,
    });
    std::fs::write(out, data.to_string()).expect("write features");
    eprintln!(
        "featurized {} train, {} val, {} probes into {out}: {} classes, vocab {}",
        train.len(),
        val.len(),
        probe_rows.len(),
        labels.len(),
        vocab.len()
    );
}
