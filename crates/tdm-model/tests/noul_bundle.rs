//! A bundle with Noul heads: several typed questions answered from one pooled
//! state. The port must reproduce MLPL's Choice and Noul probabilities at every
//! snapshot, and see a question mark the cleaner would otherwise drop.

use tdm_model::{Bundle, Model};

const BUNDLE: &str = include_str!("../../../fixtures/bundles/demo01-model3.json");

fn bundle() -> Bundle {
    Bundle::parse(BUNDLE).expect("the committed Noul bundle must parse")
}

#[test]
#[expect(
    clippy::needless_range_loop,
    reason = "snap indexes the model, the Choice parity and the Noul parity together"
)]
fn every_choice_and_noul_probability_matches_mlpl_at_every_snapshot() {
    let b = bundle();
    let (k, m) = (b.labels.len(), b.noul_names().len());
    let nouls = b
        .parity
        .nouls
        .as_ref()
        .expect("parity carries Noul probabilities");
    for snap in 0..b.snapshot_count() {
        let model = Model::at(&b, snap);
        let (mut worst_c, mut worst_n): (f64, f64) = (0.0, 0.0);
        for (i, input) in b.parity.inputs.iter().enumerate() {
            let d = model.decide(input);
            for c in 0..k {
                worst_c = worst_c.max((d.probs[c] - b.parity.probs[snap][i * k + c]).abs());
            }
            for j in 0..m {
                worst_n = worst_n.max((d.nouls[j] - nouls[snap][i * m + j]).abs());
            }
        }
        assert!(
            worst_c < 1e-9,
            "snapshot {snap}: Choice disagreement {worst_c}"
        );
        assert!(
            worst_n < 1e-9,
            "snapshot {snap}: Noul disagreement {worst_n}"
        );
    }
}

#[test]
fn the_nouls_come_from_the_same_pass_and_see_the_question_mark() {
    let b = bundle();
    assert_eq!(b.noul_names(), ["question", "negative", "positive"]);
    let d = Model::new(&b).decide("are you ok?");
    assert_eq!(d.nouls.len(), 3, "three answers from one forward pass");
    assert!(
        d.features.tokens.iter().any(|t| t == "qmark"),
        "the ? is a feature"
    );
    assert!(d.nouls.iter().all(|p| (0.0..=1.0).contains(p)));
}
