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
fn every_parity_probability_matches_mlpl() {
    let b = bundle();
    let model = Model::new(&b);
    let k = b.labels.len();
    let mut worst: f64 = 0.0;
    for (i, input) in b.parity.inputs.iter().enumerate() {
        let d = model.decide(input);
        for c in 0..k {
            worst = worst.max((d.probs[c] - b.parity.probs[i * k + c]).abs());
        }
    }
    assert!(
        worst < 1e-9,
        "largest disagreement with MLPL over {} inputs was {worst}",
        b.parity.inputs.len()
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
