//! Score a bundle against a labelled probe file (`LABEL<TAB>text` per line).
//! Reports what a visitor would see -- the label after the policy has applied
//! its threshold -- plus the model's raw choice, per-label false-positive
//! counts, and how confident the model was when it was wrong. A truth label the
//! bundle does not offer counts as wrong for any prediction, and an input the
//! policy abstains on counts as the bundle's fallback label.
//!
//! usage: cargo run -p tdm-model --example score -- BUNDLE LABELS.tsv [SNAPSHOT]

use std::collections::BTreeMap;

use tdm_model::{Bundle, respond_at};

fn main() {
    let mut args = std::env::args().skip(1);
    let b = Bundle::parse(&std::fs::read_to_string(args.next().expect("BUNDLE")).expect("read"))
        .expect("parse");
    let rows = std::fs::read_to_string(args.next().expect("LABELS.tsv")).expect("read labels");
    let snap = args
        .next()
        .map_or(b.default_snapshot, |s| s.parse().expect("snapshot"));
    let alias = |l: &str| {
        if l == "FALLBACK" {
            "NONE".to_owned()
        } else {
            l.to_owned()
        }
    };
    let (mut n, mut raw_ok, mut shown_ok) = (0u32, 0u32, 0u32);
    let (mut wrong_conf, mut wrong_n) = (0.0f64, 0u32);
    let mut false_pos: BTreeMap<String, u32> = BTreeMap::new();
    let mut shown_wrong: BTreeMap<String, u32> = BTreeMap::new();
    let (mut esc, mut esc_ok, mut acted, mut acted_ok) = (0u32, 0u32, 0u32, 0u32);
    for (i, row) in rows.lines().filter(|r| !r.trim().is_empty()).enumerate() {
        let (truth, text) = row.split_once('\t').expect("LABEL<TAB>text");
        let t = respond_at(&b, snap, text, i);
        let raw = alias(&b.labels[t.decision.selected]);
        // An escalation is not a reply the model chose; score it separately.
        let escalated = t.reply.outcome == tdm_model::Outcome::Escalate;
        esc += u32::from(escalated);
        esc_ok += u32::from(escalated && raw == truth);
        let shown = alias(&b.labels[t.reply.label]);
        if !escalated {
            acted += 1;
            acted_ok += u32::from(shown == truth);
            if shown != truth {
                *shown_wrong.entry(shown.clone()).or_default() += 1;
            }
        }
        n += 1;
        raw_ok += u32::from(raw == truth);
        shown_ok += u32::from(shown == truth);
        if raw != truth {
            wrong_conf += t.decision.confidence;
            wrong_n += 1;
            *false_pos.entry(raw).or_default() += 1;
        }
    }
    let pct = |a: u32| 100.0 * f64::from(a) / f64::from(n);
    println!("snapshot {snap}: {n} labelled inputs");
    println!(
        "  model's choice correct      {raw_ok:>3}  ({:.0}%)",
        pct(raw_ok)
    );
    println!(
        "  reply class correct         {shown_ok:>3}  ({:.0}%)  after the confidence policy",
        pct(shown_ok)
    );
    println!(
        "  mean confidence when wrong  {:.2}",
        if wrong_n > 0 {
            wrong_conf / f64::from(wrong_n)
        } else {
            0.0
        }
    );
    println!(
        "  escalated (would ask a larger decider)  {esc:>3}  ({:.0}%), of which the model's own choice was right {esc_ok}",
        pct(esc)
    );
    println!(
        "  replied without escalating  {acted:>3}, right {acted_ok} ({:.0}% of those)",
        if acted > 0 {
            100.0 * f64::from(acted_ok) / f64::from(acted)
        } else {
            0.0
        }
    );
    println!("  wrong replies a visitor sees, by the label replied from:");
    for (label, count) in &shown_wrong {
        println!("    {label:<10} {count}");
    }
    println!("  wrong answers by the label the model chose:");
    for (label, count) in &false_pos {
        println!("    {label:<10} {count}");
    }
}
