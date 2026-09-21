//! A version 3 bundle: an exact vocabulary instead of hashing, a trained escape
//! hatch, and a three-way policy (act, none applies, escalate). The port must
//! still reproduce MLPL's numbers at every snapshot.

use tdm_model::{Bundle, Model, Outcome, respond_at};

const BUNDLE: &str = include_str!("../../../fixtures/bundles/demo01-model2.json");

fn bundle() -> Bundle {
    Bundle::parse(BUNDLE).expect("the committed version 3 bundle must parse")
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
            "snapshot {snap}: largest disagreement {worst}"
        );
    }
}

#[test]
fn an_unknown_word_contributes_nothing_instead_of_borrowing_a_meaning() {
    let b = bundle();
    let d = Model::new(&b).decide("zzyzx qwv");
    assert!(
        d.features.slots.is_empty(),
        "no row is read for words the vocabulary lacks"
    );
    assert!(d.features.known.iter().all(|k| !k));
    assert_eq!(
        d.features.tokens.len(),
        3,
        "they are still shown, marked unknown"
    );
}

#[test]
fn with_no_evidence_the_program_escalates_rather_than_guessing() {
    let b = bundle();
    let t = respond_at(&b, b.default_snapshot, "zzyzx qwv", 0);
    assert_eq!(t.reply.outcome, Outcome::Escalate);
    assert_eq!(Some(t.reply.label), b.label_index(&b.fallback));
}

#[test]
fn every_reply_is_quoted_from_the_table_it_names_under_every_outcome() {
    let b = bundle();
    for snap in 0..b.snapshot_count() {
        for (i, input) in b.parity.inputs.iter().enumerate() {
            let t = respond_at(&b, snap, input, i);
            assert_eq!(b.replies(t.reply.label)[t.reply.index], t.reply.text);
        }
    }
}

#[test]
fn the_default_snapshot_is_the_best_on_validation_not_on_the_probes() {
    let b = bundle();
    let val = |s| b.metric(s, "val accuracy").expect("val accuracy");
    let best = (0..b.snapshot_count())
        .max_by(|&x, &y| val(x).total_cmp(&val(y)))
        .expect("snapshots");
    assert_eq!(
        b.default_snapshot, best,
        "chosen by held-out frames; the probes are the test"
    );
}
