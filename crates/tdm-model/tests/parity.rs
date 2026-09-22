//! The Rust port must run the same model MLPL trained. The bundle carries every
//! corpus sentence, the hand-written set, and the demo transcript, with the
//! probabilities and matcher picks MLPL computed; these tests require the port
//! to reproduce all of them.

use tdm_model::{Bundle, KeywordMatcher, Model, featurize, hash, words};

const BUNDLE: &str = include_str!("../../../fixtures/bundles/demo01.json");

fn bundle() -> Bundle {
    Bundle::parse(BUNDLE).expect("the committed bundle must parse")
}

#[test]
fn every_parity_probability_matches_mlpl_at_every_snapshot() {
    let b = bundle();
    let k = b.labels.len();
    for snap in 0..b.snapshot_count() {
        let model = Model::at(&b, snap);
        let mut worst: f64 = 0.0;
        for (i, input) in b.parity.inputs.iter().enumerate() {
            let d = model.decide(input);
            for c in 0..k {
                worst = worst.max((d.probs[c] - b.parity.probs[snap][i * k + c]).abs());
            }
        }
        assert!(
            worst < 1e-9,
            "snapshot {snap}: largest disagreement with MLPL over {} inputs was {worst}",
            b.parity.inputs.len()
        );
    }
}

#[test]
#[expect(
    clippy::cast_precision_loss,
    reason = "label and input counts are tiny"
)]
fn the_first_snapshot_is_untrained_and_the_last_is_not() {
    // "Random weights, random results": at step 0 every decision is close to a
    // uniform guess over the nine labels. By the last snapshot it is not.
    let b = bundle();
    assert_eq!(b.snapshots.steps[0], 0);
    let untrained = Model::at(&b, 0);
    let trained = Model::at(&b, b.snapshot_count() - 1);
    let uniform = 1.0 / b.labels.len() as f64;
    for input in &b.parity.inputs {
        assert!(
            untrained.decide(input).confidence < 3.0 * uniform,
            "step 0 was already sure about {input:?}"
        );
    }
    let mean = |m: &Model| {
        b.parity
            .inputs
            .iter()
            .map(|i| m.decide(i).confidence)
            .sum::<f64>()
            / b.parity.inputs.len() as f64
    };
    assert!(mean(&trained) > 0.8 && mean(&untrained) < 0.2);
}

#[test]
fn the_timeline_reports_what_training_did_and_did_not_buy() {
    // The measured story, pinned so a re-export cannot quietly change it:
    // accuracy jumps in the first second and then stops moving, while
    // confidence on the unlabelled probe inputs keeps climbing.
    let b = bundle();
    let val = |s| b.metric(s, "val accuracy").expect("val accuracy");
    let probe = |s| b.metric(s, "probe confidence").expect("probe confidence");
    let last = b.snapshot_count() - 1;
    assert!(
        val(0) < 0.15 && val(1) > 0.8,
        "the first second is where the learning happens"
    );
    assert!(
        (val(last) - val(2)).abs() < 1e-9,
        "after that, validation accuracy does not move"
    );
    assert!(
        probe(last) > probe(1) + 0.2,
        "but confidence on inputs it knows nothing about keeps rising"
    );
}

#[test]
fn the_keyword_matcher_matches_mlpl_on_every_input() {
    let b = bundle();
    let matcher = KeywordMatcher::new(&b);
    for (i, input) in b.parity.inputs.iter().enumerate() {
        assert_eq!(
            matcher.classify(input),
            b.parity.matcher[i],
            "matcher disagreed with MLPL on {input:?}"
        );
    }
}

#[test]
fn the_featurizer_matches_lib_text_on_the_documented_cases() {
    assert_eq!(
        words("  The QUICK fox doesn't stop!! "),
        ["the", "quick", "fox", "doesnt", "stop"]
    );
    assert!(words("   ").is_empty());
    let f = featurize("the fox runs", 512, 24);
    assert_eq!(f.tokens, ["the", "fox", "runs", "the_fox", "fox_runs"]);
    assert_eq!(hash("fox"), hash("fox"));
    assert!(f.slots.iter().all(|&s| s < 512));
}

#[test]
fn the_bundle_is_the_measured_model() {
    let b = bundle();
    assert_eq!(b.param_count(), 33_065);
    assert_eq!(b.labels.len(), 9);
    assert_eq!(b.snapshots.steps, [0, 11, 22, 55, 110]);
    assert!(b.parity.inputs.len() >= 310);
    // Every reply is one of the offered candidates; the port adds none.
    for label in 0..b.labels.len() {
        assert!(!b.replies(label).is_empty());
    }
}

#[test]
fn an_empty_input_falls_back_to_the_bias() {
    let b = bundle();
    let d = Model::new(&b).decide("!!!");
    assert!(d.features.slots.is_empty());
    assert!((d.probs.iter().sum::<f64>() - 1.0).abs() < 1e-12);
}

#[test]
fn every_reply_is_quoted_verbatim_from_the_table_it_names() {
    // The no-generation invariant, in the browser port: across every parity
    // input and several turns, the reply is exactly an entry of its table.
    let b = bundle();
    for (i, input) in b.parity.inputs.iter().enumerate() {
        let t = tdm_model::respond(&b, input, i);
        assert_eq!(b.replies(t.reply.label)[t.reply.index], t.reply.text);
    }
}

#[test]
fn below_the_threshold_the_policy_abstains_to_the_fallback() {
    let mut b = bundle();
    b.threshold = 1.1; // no confidence can reach this
    let t = tdm_model::respond(&b, "my mom never listens to me", 0);
    assert!(!t.reply.acted);
    assert_eq!(Some(t.reply.label), b.label_index(&b.fallback));
}

#[test]
fn a_curly_apostrophe_is_an_apostrophe() {
    // Phones and novels type ’; before this fix "I don’t know" became
    // "i don t know" and missed every rule written for "dont".
    assert_eq!(words("I don’t know"), ["i", "dont", "know"]);
    assert_eq!(words("it’s ‘fine’"), ["its", "fine"]);
}
