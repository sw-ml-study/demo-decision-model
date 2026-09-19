//! Validation contract for the typed decision trace format.
//!
//! These tests are deliberately heavy on rejection cases. A trace is an
//! artifact other tools consume, so "it parsed" has to mean "it cannot be
//! internally inconsistent" — a distribution that disagrees with its own
//! confidence, or an output that was not in its own table, must not get through.

use serde_json::{Value, json};
use tdm_trace::{KINDS, Trace, ValidationError};

const EXAMPLE: &str = include_str!("../../../fixtures/traces/example-turn-v1.json");

/// A minimal trace that validates. Tests mutate one thing and expect rejection.
fn base() -> Value {
    json!({
        "schema": "sw-ml-study.decision-trace",
        "version": 1,
        "provenance": {
            "producer": "test",
            "producer_revision": "0",
            "generated_at": "2026-09-18",
            "source_description": "synthetic"
        },
        "demo": "test",
        "tables": [{ "name": "t", "entries": ["plain", "framed {}"] }],
        "turns": [{
            "index": 0,
            "state": { "text": "input" },
            "decisions": [{
                "kind": "choice",
                "question": "q",
                "labels": ["A", "B", "C"],
                "probs": [0.6, 0.3, 0.1],
                "selected": 0,
                "confidence": 0.6,
                "margin": 0.3,
                "calibration": { "method": "none" }
            }],
            "policy": { "rule": "r", "branch": "b" },
            "output": { "source": "table", "table": "t", "index": 0, "text": "plain" }
        }]
    })
}

fn parse(v: &Value) -> Result<Trace, ValidationError> {
    Trace::parse(&v.to_string())
}

fn decision(v: &mut Value) -> &mut Value {
    &mut v["turns"][0]["decisions"][0]
}

fn expect_err(v: &Value, expected: &ValidationError) {
    assert_eq!(parse(v).err().as_ref(), Some(expected));
}

#[test]
fn the_committed_example_validates_and_reads_back() {
    let trace = Trace::parse(EXAMPLE).expect("the committed example must validate");
    assert_eq!(trace.turns().len(), 1);

    let turn = &trace.turns()[0];
    assert_eq!(turn.index(), 12);
    assert_eq!(turn.state().memory().len(), 4);
    assert_eq!(turn.decisions().len(), 6);

    // All three kinds appear, which is the point of keeping this example around.
    let kinds: Vec<&str> = turn
        .decisions()
        .iter()
        .map(tdm_trace::Decision::kind)
        .collect();
    assert!(kinds.contains(&"choice"));
    assert!(kinds.contains(&"noul"));
    assert!(kinds.contains(&"scale"));

    let action = &turn.decisions()[0];
    assert_eq!(action.selected_label(), "RECALL");
    assert!((action.confidence() - 0.61).abs() < 1e-9);
    assert!(action.calibration().is_calibrated());

    // Policy reads by name, never by an index a later revision might renumber.
    let recall = turn.decisions().last().expect("a recall decision");
    assert!((recall.prob_of("NONE").expect("NONE is always offered") - 0.05).abs() < 1e-9);
    assert_eq!(recall.prob_of("M9"), None);

    // The no-generation invariant, on the case that actually needs slots.
    assert_eq!(turn.output().table(), Some("recall"));
    assert_eq!(turn.output().slots(), ["work has been stressful"]);
    assert_eq!(
        turn.output().text(),
        "Earlier you said that work has been stressful. Tell me more about that."
    );
}

#[test]
fn there_are_exactly_three_kinds_and_generate_is_not_one() {
    assert_eq!(KINDS, ["choice", "noul", "scale"]);
    let mut v = base();
    decision(&mut v)["kind"] = json!("generate");
    expect_err(&v, &ValidationError::UnknownKind("generate".to_owned()));
}

#[test]
fn schema_and_version_are_pinned() {
    let mut v = base();
    v["schema"] = json!("sw-ml-study.something-else");
    expect_err(&v, &ValidationError::UnsupportedSchema);

    let mut v = base();
    v["version"] = json!(2);
    expect_err(&v, &ValidationError::UnsupportedVersion(2));
}

#[test]
fn malformed_json_and_unknown_fields_are_rejected() {
    assert_eq!(Trace::parse("{").err(), Some(ValidationError::Malformed));
    let mut v = base();
    v["surprise"] = json!(1);
    expect_err(&v, &ValidationError::Malformed);
}

#[test]
fn provenance_must_be_present_and_non_empty() {
    let mut v = base();
    v["provenance"]["producer"] = json!("");
    expect_err(&v, &ValidationError::EmptyText("producer"));
}

#[test]
fn a_distribution_must_be_one_probability_per_label_summing_to_one() {
    let mut v = base();
    decision(&mut v)["probs"] = json!([0.6, 0.3]);
    expect_err(&v, &ValidationError::LabelCount(0));

    let mut v = base();
    decision(&mut v)["probs"] = json!([0.6, 0.3, 0.2]);
    expect_err(&v, &ValidationError::NotADistribution(0));

    let mut v = base();
    decision(&mut v)["probs"] = json!([1.5, -0.3, -0.2]);
    expect_err(&v, &ValidationError::NotADistribution(0));

    let mut v = base();
    decision(&mut v)["labels"] = json!([]);
    decision(&mut v)["probs"] = json!([]);
    expect_err(&v, &ValidationError::LabelCount(0));
}

#[test]
fn derived_scalars_must_agree_with_the_distribution_they_come_from() {
    let mut v = base();
    decision(&mut v)["selected"] = json!(9);
    expect_err(&v, &ValidationError::SelectedOutOfRange(0));

    // Selecting B is legal, but then confidence must be B's probability.
    let mut v = base();
    decision(&mut v)["selected"] = json!(1);
    expect_err(&v, &ValidationError::DerivedMismatch("confidence"));

    let mut v = base();
    decision(&mut v)["confidence"] = json!(0.9);
    expect_err(&v, &ValidationError::DerivedMismatch("confidence"));

    let mut v = base();
    decision(&mut v)["margin"] = json!(0.5);
    expect_err(&v, &ValidationError::DerivedMismatch("margin"));
}

#[test]
fn a_single_label_decision_has_margin_one() {
    let mut v = base();
    decision(&mut v)["labels"] = json!(["ONLY"]);
    decision(&mut v)["probs"] = json!([1.0]);
    decision(&mut v)["selected"] = json!(0);
    decision(&mut v)["confidence"] = json!(1.0);
    decision(&mut v)["margin"] = json!(1.0);
    parse(&v).expect("a one-candidate set is a legitimate decision");
}

#[test]
fn kind_specific_fields_belong_to_their_own_kind() {
    let noul = |p: f64| {
        json!({
            "kind": "noul", "question": "q?", "labels": ["false", "true"],
            "probs": [0.3, 0.7], "selected": 1, "confidence": 0.7, "margin": 0.4,
            "calibration": { "method": "none" }, "p": p
        })
    };

    let mut v = base();
    v["turns"][0]["decisions"][0] = noul(0.7);
    parse(&v).expect("a well-formed noul");

    let mut v = base();
    v["turns"][0]["decisions"][0] = noul(0.5);
    expect_err(&v, &ValidationError::DerivedMismatch("noul p"));

    let mut v = base();
    let mut d = noul(0.7);
    d.as_object_mut().expect("object").remove("p");
    v["turns"][0]["decisions"][0] = d;
    expect_err(&v, &ValidationError::FieldForWrongKind("noul without p"));

    let mut v = base();
    let mut d = noul(0.7);
    d["expectation"] = json!(0.7);
    v["turns"][0]["decisions"][0] = d;
    expect_err(
        &v,
        &ValidationError::FieldForWrongKind("expectation on a noul"),
    );

    let mut v = base();
    let mut d = noul(0.7);
    d["labels"] = json!(["no", "yes"]);
    v["turns"][0]["decisions"][0] = d;
    expect_err(&v, &ValidationError::FieldForWrongKind("noul labels"));

    let mut v = base();
    decision(&mut v)["p"] = json!(0.6);
    expect_err(&v, &ValidationError::FieldForWrongKind("p on a choice"));

    let mut v = base();
    decision(&mut v)["expectation"] = json!(0.5);
    expect_err(
        &v,
        &ValidationError::FieldForWrongKind("expectation on a choice"),
    );
}

#[test]
fn a_scale_carries_an_expectation_that_matches_its_levels() {
    let scale = |expectation: Value| {
        let mut d = json!({
            "kind": "scale", "question": "intensity",
            "labels": ["LOW", "MEDIUM", "HIGH"],
            "probs": [0.08, 0.57, 0.35], "selected": 1,
            "confidence": 0.57, "margin": 0.22,
            "calibration": { "method": "none" }
        });
        if !expectation.is_null() {
            d["expectation"] = expectation;
        }
        d
    };

    let mut v = base();
    v["turns"][0]["decisions"][0] = scale(json!(1.27));
    parse(&v).expect("0*.08 + 1*.57 + 2*.35 = 1.27");

    let mut v = base();
    v["turns"][0]["decisions"][0] = scale(json!(1.0));
    expect_err(&v, &ValidationError::DerivedMismatch("scale expectation"));

    let mut v = base();
    v["turns"][0]["decisions"][0] = scale(Value::Null);
    expect_err(
        &v,
        &ValidationError::FieldForWrongKind("scale without expectation"),
    );
}

#[test]
fn output_text_must_be_rebuildable_from_the_table_it_cites() {
    // The headline invariant: a string that is not in the table is rejected,
    // however plausible it looks.
    let mut v = base();
    v["turns"][0]["output"]["text"] = json!("something the model made up");
    expect_err(&v, &ValidationError::TextNotOffered(0));

    let mut v = base();
    v["turns"][0]["output"]["table"] = json!("nonexistent");
    expect_err(&v, &ValidationError::UnknownTable("nonexistent".to_owned()));

    let mut v = base();
    v["turns"][0]["output"]["index"] = json!(99);
    expect_err(&v, &ValidationError::IndexOutOfTable(0));
}

#[test]
fn slots_fill_placeholders_in_order_and_must_be_counted_exactly() {
    let mut v = base();
    v["turns"][0]["output"] = json!({
        "source": "table", "table": "t", "index": 1, "text": "framed VALUE", "slots": ["VALUE"]
    });
    parse(&v).expect("one placeholder, one slot, rendered exactly");

    // Right slot, wrong rendering.
    let mut v = base();
    v["turns"][0]["output"] = json!({
        "source": "table", "table": "t", "index": 1, "text": "framed something else", "slots": ["VALUE"]
    });
    expect_err(&v, &ValidationError::TextNotOffered(0));

    // A frame with a placeholder and no slot supplied.
    let mut v = base();
    v["turns"][0]["output"] =
        json!({ "source": "table", "table": "t", "index": 1, "text": "framed {}" });
    expect_err(&v, &ValidationError::SlotCount(0));

    // A frame with no placeholder cannot absorb a slot.
    let mut v = base();
    v["turns"][0]["output"] = json!({
        "source": "table", "table": "t", "index": 0, "text": "plain", "slots": ["VALUE"]
    });
    expect_err(&v, &ValidationError::SlotCount(0));
}

#[test]
fn turns_are_ordered_and_memory_ids_are_unique() {
    let mut v = base();
    let turn = v["turns"][0].clone();
    v["turns"] = json!([turn.clone(), turn]);
    expect_err(&v, &ValidationError::TurnOrder(0));

    let mut v = base();
    v["turns"][0]["state"]["memory"] = json!([
        { "id": "M1", "text": "a", "age_turns": 1 },
        { "id": "M1", "text": "b", "age_turns": 2 }
    ]);
    expect_err(&v, &ValidationError::DuplicateMemoryId("M1".to_owned()));
}

#[test]
fn duplicate_table_names_are_rejected() {
    let mut v = base();
    v["tables"] = json!([
        { "name": "t", "entries": ["a"] },
        { "name": "t", "entries": ["b"] }
    ]);
    expect_err(&v, &ValidationError::DuplicateTable("t".to_owned()));
}

#[test]
fn a_choice_set_may_exceed_what_any_fixed_head_would_have() {
    // Choices are inputs, not an output layer, so a large set is legal up to the
    // point where the two-stage path takes over.
    let n = 256;
    let labels: Vec<String> = (0..n).map(|i| format!("C{i}")).collect();
    let mut probs = vec![0.0; n];
    probs[0] = 1.0;

    let mut v = base();
    decision(&mut v)["labels"] = json!(labels);
    decision(&mut v)["probs"] = json!(probs);
    decision(&mut v)["selected"] = json!(0);
    decision(&mut v)["confidence"] = json!(1.0);
    decision(&mut v)["margin"] = json!(1.0);
    parse(&v).expect("256 candidates is the documented ceiling for one softmax");

    let labels: Vec<String> = (0..=n).map(|i| format!("C{i}")).collect();
    let mut probs = vec![0.0; n + 1];
    probs[0] = 1.0;
    decision(&mut v)["labels"] = json!(labels);
    decision(&mut v)["probs"] = json!(probs);
    expect_err(&v, &ValidationError::LabelCount(0));
}

/// The composition pattern where ordinary code builds the candidate list as
/// fully-composed strings and the model only ranks them. The output is then
/// byte-identical to the candidate that won, and nothing was templated at all.
fn composed_candidates() -> Value {
    let mut v = base();
    v["turns"][0]["decisions"][0] = json!({
        "kind": "choice",
        "question": "what should the reply be?",
        "labels": [
            "Tell me more.",
            "Why do you say that?",
            "Earlier you said you bought a new car. How does that connect?",
            "Earlier you said work has been stressful. Tell me more about that."
        ],
        "probs": [0.19, 0.12, 0.21, 0.48],
        "selected": 3,
        "confidence": 0.48,
        "margin": 0.27,
        "calibration": { "method": "none" }
    });
    v["turns"][0]["output"] = json!({
        "source": "choice",
        "decision": 0,
        "text": "Earlier you said work has been stressful. Tell me more about that."
    });
    v
}

#[test]
fn an_output_may_be_a_composed_candidate_the_model_ranked() {
    let v = composed_candidates();
    let trace = parse(&v).expect("a composed candidate set is a legitimate output");
    let turn = &trace.turns()[0];
    assert_eq!(
        turn.output().text(),
        "Earlier you said work has been stressful. Tell me more about that."
    );
    // Nothing was templated, so there is no table and no slot to account for.
    assert_eq!(turn.output().table(), None);
    assert_eq!(turn.output().index(), None);
    assert!(turn.output().slots().is_empty());
    assert_eq!(turn.output().decision(), Some(0));
}

#[test]
fn a_composed_output_must_be_the_candidate_that_actually_won() {
    // A candidate that was offered but lost is still not what happened.
    let mut v = composed_candidates();
    v["turns"][0]["output"]["text"] = json!("Tell me more.");
    expect_err(&v, &ValidationError::TextNotOffered(0));

    // A plausible sentence that was never offered at all.
    let mut v = composed_candidates();
    v["turns"][0]["output"]["text"] = json!("Earlier you said your mother worries. Go on.");
    expect_err(&v, &ValidationError::TextNotOffered(0));

    // A decision index this turn does not have.
    let mut v = composed_candidates();
    v["turns"][0]["output"]["decision"] = json!(7);
    expect_err(&v, &ValidationError::UnknownDecision(0));
}

#[test]
fn an_output_must_declare_which_pattern_composed_it() {
    // No source discriminator at all.
    let mut v = base();
    v["turns"][0]["output"] = json!({ "table": "t", "index": 0, "text": "plain" });
    expect_err(&v, &ValidationError::Malformed);

    // Table fields on a chosen output, or the reverse, are not a mix-and-match.
    let mut v = composed_candidates();
    v["turns"][0]["output"]["table"] = json!("t");
    expect_err(&v, &ValidationError::Malformed);
}
