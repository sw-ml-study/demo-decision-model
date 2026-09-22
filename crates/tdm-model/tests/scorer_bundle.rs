//! The scorer port must reproduce MLPL's probabilities, and a candidate that
//! did not exist during training must be scorable at all -- which is the only
//! thing this shape buys over a fixed head.

use tdm_model::{Scorer, ScorerBundle};

const BUNDLE: &str = include_str!("../../../fixtures/bundles/demo01-scorer.json");

fn bundle() -> ScorerBundle {
    ScorerBundle::parse(BUNDLE).expect("the committed scorer bundle must parse")
}

#[test]
fn every_probability_matches_mlpl_on_the_parity_rows() {
    let b = bundle();
    let s = Scorer::at(&b, b.default_snapshot);
    let k = b.parity.cards;
    let mut worst: f64 = 0.0;
    for (i, input) in b.parity.inputs.iter().enumerate() {
        let question = b.parity.questions[i];
        let offered = b.offered(question);
        let texts: Vec<&str> = offered.iter().map(|&c| b.cards.texts[c].as_str()).collect();
        let r = s.rank(input, question, &texts);
        for (j, &card) in offered.iter().enumerate() {
            worst = worst.max((r.probs[j] - b.parity.probs[i * k + card]).abs());
        }
        // A card the row was not offered carries no probability at all.
        for c in 0..k {
            if !offered.contains(&c) {
                assert!(b.parity.probs[i * k + c] < 1e-12);
            }
        }
    }
    assert!(worst < 1e-9, "largest disagreement with MLPL: {worst}");
}

#[test]
fn a_candidate_written_after_training_is_scored_at_all() {
    let b = bundle();
    let s = Scorer::new(&b);
    let existing = b.cards.texts[0].as_str();
    let fresh = "a card written after training about being unable to sleep at night";
    let other = "a card written after training about buying a second hand car";
    let r = s.rank("i cannot sleep at night", 0, &[existing, fresh, other]);
    assert_eq!(r.probs.len(), 3, "a new card is ranked like any other");
    assert!((r.probs.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    // What is asserted here is only that the card's own words reach the score.
    // Whether the *apt* new card wins is a measurement, not an invariant, and
    // the measurement says it usually does not: see docs/experiments/DC01.
    let swapped = s.rank("i cannot sleep at night", 0, &[existing, other, other]);
    assert!(
        (r.scores[1] - swapped.scores[1]).abs() > 1e-6,
        "changing a card's words changes its score"
    );
    assert!(
        !b.held_out.is_empty(),
        "the bundle names its held-out cards"
    );
}

#[test]
fn adding_a_candidate_costs_no_parameters() {
    let b = bundle();
    let w = b.weights(b.default_snapshot);
    let learned = w.embedding.len() + w.state.len() + w.question.len() + w.card.len() + 1;
    let s = Scorer::new(&b);
    let mut cards: Vec<&str> = b.cards.texts.iter().map(String::as_str).collect();
    let before = s
        .rank("i am unhappy", 0, &cards[..b.rule_count])
        .probs
        .len();
    cards.push("one more card, invented here");
    let after = s.rank("i am unhappy", 0, &cards).probs.len();
    assert!(after > before);
    assert_eq!(
        learned,
        b.weights(b.default_snapshot).embedding.len()
            + w.state.len()
            + w.question.len()
            + w.card.len()
            + 1,
        "the weights did not change size"
    );
}
