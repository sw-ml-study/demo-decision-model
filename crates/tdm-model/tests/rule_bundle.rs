//! A bundle whose labels are a keyword script's rules. The port must reproduce
//! MLPL's probabilities at every snapshot, the bundle carries no keyword list
//! of its own (the script is its yardstick), and every label must name a rule
//! the script can answer with.

use tdm_model::{Bundle, KeywordEngine, KeywordScript, Model};

const BUNDLE: &str = include_str!("../../../fixtures/bundles/demo01-model4.json");
const SCRIPT: &str = include_str!("../../../fixtures/bundles/demo01-doctor.json");

fn bundle() -> Bundle {
    Bundle::parse(BUNDLE).expect("the committed rule bundle must parse")
}

fn script() -> KeywordScript {
    serde_json::from_str(SCRIPT).expect("the committed keyword script must parse")
}

#[test]
#[expect(
    clippy::needless_range_loop,
    reason = "snap indexes the model, the Choice parity and the Noul parity together"
)]
fn every_choice_and_noul_probability_matches_mlpl_at_every_snapshot() {
    let b = bundle();
    let (k, m) = (b.labels.len(), b.noul_names().len());
    let nouls = b.parity.nouls.as_ref().expect("parity carries Nouls");
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
fn every_label_is_a_rule_the_script_can_answer_with() {
    let (b, s) = (bundle(), script());
    assert!(b.keywords.is_empty() && b.match_order.is_empty());
    let rules = KeywordEngine::new(&s).rule_ids();
    for label in &b.labels {
        assert!(rules.contains(label), "{label} is not a rule of the script");
        assert!(s.pattern(label).is_some());
    }
    assert!(b.labels.contains(&b.fallback));
}
