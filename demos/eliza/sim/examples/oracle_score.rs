//! Score model 4 against the oracle it learned from: agreement on the held-out
//! split and on the 96 probes (both labelled by the DOCTOR script, never by
//! hand), per rule, against the majority-class baseline, with calibration, the
//! policy's outcomes, and how often the chosen rule's own pattern fits the
//! input -- which is what decides whether its reply can use the input's words.
//!
//! usage: `cargo run -p eliza-sim --example oracle_score -- [SNAPSHOT]`

use std::collections::BTreeMap;

use eliza_sim::{class_of, read_corpus, rule_classes};
use tdm_model::{Bundle, KeywordEngine, KeywordScript, Model, Outcome, respond_at};

const BUNDLE: &str = "fixtures/bundles/demo01-model4.json";
const SCRIPT: &str = "fixtures/bundles/demo01-doctor.json";
const CORPUS: &str = "demos/eliza/oracle/corpus.tsv";
const PROBES: &str = "demos/eliza/probe-labels.tsv";

#[derive(Default)]
struct PerRule {
    truth: usize,
    predicted: usize,
    right: usize,
}

#[derive(Default)]
struct Score {
    n: usize,
    right: usize,
    matched: usize,
    same_reply: usize,
    acted: usize,
    none: usize,
    escalated: usize,
    conf_right: f64,
    conf_wrong: f64,
    bins: [(usize, usize, f64); 10],
    rules: BTreeMap<String, PerRule>,
}

#[expect(clippy::cast_precision_loss, reason = "counts are far below 2^52")]
fn pct(a: usize, b: usize) -> f64 {
    100.0 * a as f64 / b.max(1) as f64
}

#[expect(clippy::cast_precision_loss, reason = "counts are far below 2^52")]
fn mean(sum: f64, n: usize) -> f64 {
    sum / n.max(1) as f64
}

fn score(
    bundle: &Bundle,
    snapshot: usize,
    script: &KeywordScript,
    rows: &[(String, String, String)],
) -> Score {
    let model = Model::at(bundle, snapshot);
    let mut s = Score::default();
    for (text, truth, oracle_reply) in rows {
        let d = model.decide(text);
        let chosen = &bundle.labels[d.selected];
        let ok = chosen == truth;
        s.n += 1;
        s.right += usize::from(ok);
        if ok {
            s.conf_right += d.confidence;
        } else {
            s.conf_wrong += d.confidence;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a probability times ten is 0..=10"
        )]
        let bin = ((d.confidence * 10.0) as usize).min(9);
        s.bins[bin].0 += 1;
        s.bins[bin].1 += usize::from(ok);
        s.bins[bin].2 += d.confidence;
        let a = KeywordEngine::new(script).respond_with(chosen, text);
        s.matched += usize::from(a.matched && &a.rule == chosen);
        s.same_reply += usize::from(&a.text == oracle_reply);
        match respond_at(bundle, snapshot, text, 0).reply.outcome {
            Outcome::Act => s.acted += 1,
            Outcome::NoneApplies => s.none += 1,
            Outcome::Escalate => s.escalated += 1,
        }
        s.rules.entry(truth.clone()).or_default().truth += 1;
        let p = s.rules.entry(chosen.clone()).or_default();
        p.predicted += 1;
        p.right += usize::from(ok);
    }
    s
}

fn report(name: &str, s: &Score) {
    let majority = s.rules.values().map(|r| r.truth).max().unwrap_or(0);
    println!("## {name}: {} inputs\n", s.n);
    println!(
        "- agreement with the oracle: {:.1}% (majority-class baseline {:.1}%)",
        pct(s.right, s.n),
        pct(majority, s.n)
    );
    println!(
        "- the chosen rule's pattern fits the input: {:.1}%",
        pct(s.matched, s.n)
    );
    println!(
        "- reply identical to the oracle's: {:.1}%",
        pct(s.same_reply, s.n)
    );
    println!(
        "- policy: act {:.1}%, none applies {:.1}%, escalate {:.1}%",
        pct(s.acted, s.n),
        pct(s.none, s.n),
        pct(s.escalated, s.n)
    );
    let wrong = s.n - s.right;
    let ece: f64 = s
        .bins
        .iter()
        .filter(|b| b.0 > 0)
        .map(|&(n, right, conf)| {
            #[expect(clippy::cast_precision_loss, reason = "small counts")]
            let (n, right, total) = (n as f64, right as f64, s.n as f64);
            (n / total) * (right / n - conf / n).abs()
        })
        .sum();
    println!(
        "- mean confidence when right {:.2}, when wrong {:.2}; expected calibration error {:.3} (10 bins)\n",
        mean(s.conf_right, s.right),
        mean(s.conf_wrong, wrong),
        ece
    );
    println!("| Rule | Oracle used | Model chose | Right | Recall | Precision |");
    println!("|---|---:|---:|---:|---:|---:|");
    let mut rules: Vec<_> = s.rules.iter().collect();
    rules.sort_by_key(|(_, r)| std::cmp::Reverse(r.truth));
    for (rule, r) in rules {
        println!(
            "| `{rule}` | {} | {} | {} | {:.0}% | {:.0}% |",
            r.truth,
            r.predicted,
            r.right,
            pct(r.right, r.truth),
            pct(r.right, r.predicted)
        );
    }
    println!();
}

fn main() {
    let bundle = Bundle::parse(&std::fs::read_to_string(BUNDLE).expect("bundle")).expect("parse");
    let script: KeywordScript =
        serde_json::from_str(&std::fs::read_to_string(SCRIPT).expect("script")).expect("parse");
    let snapshot = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(bundle.default_snapshot);
    let corpus = read_corpus(CORPUS);
    let (_, merged) = rule_classes(corpus.iter().filter(|c| !c.val).map(|c| c.rule.as_str()));
    let val: Vec<(String, String, String)> = corpus
        .iter()
        .filter(|c| c.val)
        .map(|c| {
            let class = class_of(&merged, &c.rule).to_owned();
            (c.text.clone(), class, c.reply.clone())
        })
        .collect();
    let probes: Vec<(String, String, String)> = std::fs::read_to_string(PROBES)
        .expect("probes")
        .lines()
        .filter_map(|l| l.split('\t').nth(1))
        .map(|t| {
            let a = KeywordEngine::new(&script).respond(t);
            (t.to_owned(), class_of(&merged, &a.rule).to_owned(), a.text)
        })
        .collect();
    println!(
        "# Model 4 against the oracle, snapshot {} ({})\n",
        snapshot,
        bundle.snapshot_label(snapshot)
    );
    report("validation split", &score(&bundle, snapshot, &script, &val));
    report("the 96 probes", &score(&bundle, snapshot, &script, &probes));
}
