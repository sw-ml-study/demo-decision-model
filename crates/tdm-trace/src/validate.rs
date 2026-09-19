//! The validation rules, in the order they are applied.
//!
//! A trace is an artifact other tools consume, so parsing has to mean more than
//! "the JSON was shaped right": a distribution that disagrees with its own
//! confidence, or an output that was not in its own table, must not get through.

use std::collections::HashSet;

use crate::decision::Decision;
use crate::error::ValidationError;
use crate::limits::{
    MAX_FEATURES, MAX_LABELS, MAX_MEMORIES, MAX_TEXT_BYTES, PLACEHOLDER, TOLERANCE,
};
use crate::trace::{Provenance, Table};
use crate::turn::{Output, State};

pub(crate) fn validate_provenance(p: &Provenance) -> Result<(), ValidationError> {
    bounded_text(&p.producer, "producer")?;
    bounded_text(&p.producer_revision, "producer revision")?;
    bounded_text(&p.generated_at, "generated at")?;
    bounded_text(&p.source_description, "source description")
}

pub(crate) fn validate_state(state: &State) -> Result<(), ValidationError> {
    bounded_text(&state.text, "state text")?;
    if state.features.len() > MAX_FEATURES {
        return Err(ValidationError::Budget("features"));
    }
    for feature in &state.features {
        bounded_text(feature, "feature")?;
    }
    if state.memory.len() > MAX_MEMORIES {
        return Err(ValidationError::Budget("memories"));
    }
    let mut seen = HashSet::new();
    for memory in &state.memory {
        bounded_text(&memory.id, "memory id")?;
        bounded_text(&memory.text, "memory text")?;
        if !seen.insert(memory.id.as_str()) {
            return Err(ValidationError::DuplicateMemoryId(memory.id.clone()));
        }
    }
    Ok(())
}

pub(crate) fn validate_decision(d: &Decision, turn: u32) -> Result<(), ValidationError> {
    if !crate::limits::KINDS.contains(&d.kind.as_str()) {
        return Err(ValidationError::UnknownKind(d.kind.clone()));
    }
    bounded_text(&d.question, "question")?;
    bounded_text(&d.calibration.method, "calibration method")?;
    validate_distribution(d, turn)?;
    validate_derived(d, turn)?;
    validate_kind_fields(d)
}

/// One probability per label, none negative, summing to one.
fn validate_distribution(d: &Decision, turn: u32) -> Result<(), ValidationError> {
    let n = d.labels.len();
    if n == 0 || n > MAX_LABELS || n != d.probs.len() {
        return Err(ValidationError::LabelCount(turn));
    }
    for label in &d.labels {
        bounded_text(label, "label")?;
    }
    let mut sum = 0.0;
    for p in &d.probs {
        if !p.is_finite() || *p < 0.0 {
            return Err(ValidationError::NotADistribution(turn));
        }
        sum += *p;
    }
    if (sum - 1.0).abs() > TOLERANCE {
        return Err(ValidationError::NotADistribution(turn));
    }
    Ok(())
}

/// The scalars a policy reads must agree with the distribution they come from,
/// so a trace cannot disagree with itself.
fn validate_derived(d: &Decision, turn: u32) -> Result<(), ValidationError> {
    if d.selected >= d.labels.len() {
        return Err(ValidationError::SelectedOutOfRange(turn));
    }
    if (d.confidence - d.probs[d.selected]).abs() > TOLERANCE {
        return Err(ValidationError::DerivedMismatch("confidence"));
    }
    if (d.margin - margin_of(&d.probs)).abs() > TOLERANCE {
        return Err(ValidationError::DerivedMismatch("margin"));
    }
    Ok(())
}

fn validate_kind_fields(d: &Decision) -> Result<(), ValidationError> {
    match d.kind.as_str() {
        "noul" => validate_noul(d),
        "scale" => validate_scale(d),
        _ => validate_choice(d),
    }
}

fn validate_noul(d: &Decision) -> Result<(), ValidationError> {
    if d.expectation.is_some() {
        return Err(ValidationError::FieldForWrongKind("expectation on a noul"));
    }
    if d.labels != ["false", "true"] {
        return Err(ValidationError::FieldForWrongKind("noul labels"));
    }
    let Some(p) = d.p else {
        return Err(ValidationError::FieldForWrongKind("noul without p"));
    };
    if (p - d.probs[1]).abs() > TOLERANCE {
        return Err(ValidationError::DerivedMismatch("noul p"));
    }
    Ok(())
}

fn validate_scale(d: &Decision) -> Result<(), ValidationError> {
    if d.p.is_some() {
        return Err(ValidationError::FieldForWrongKind("p on a scale"));
    }
    let Some(e) = d.expectation else {
        return Err(ValidationError::FieldForWrongKind(
            "scale without expectation",
        ));
    };
    if (e - expectation_of(&d.probs)).abs() > TOLERANCE {
        return Err(ValidationError::DerivedMismatch("scale expectation"));
    }
    Ok(())
}

fn validate_choice(d: &Decision) -> Result<(), ValidationError> {
    if d.p.is_some() {
        return Err(ValidationError::FieldForWrongKind("p on a choice"));
    }
    if d.expectation.is_some() {
        return Err(ValidationError::FieldForWrongKind(
            "expectation on a choice",
        ));
    }
    Ok(())
}

/// The bounding invariant: a turn's text is exactly one of the candidates the
/// program offered the model. Either it was chosen from a composed candidate
/// set, in which case it must be byte-identical to that decision's selected
/// label; or it was framed from a table entry, in which case it must equal that
/// entry with the turn's recorded slots filled in order.
///
/// In both cases the program built the options and the model only ranked them.
pub(crate) fn validate_output(
    output: &Output,
    decisions: &[Decision],
    table: Option<&Table>,
    turn: u32,
) -> Result<(), ValidationError> {
    match output {
        Output::Choice { decision, text } => validate_chosen(*decision, text, decisions, turn),
        Output::Table {
            index, text, slots, ..
        } => validate_framed(*index, text, slots, table, output, turn),
    }
}

/// The output was one of the composed candidates a decision offered.
fn validate_chosen(
    decision: usize,
    text: &str,
    decisions: &[Decision],
    turn: u32,
) -> Result<(), ValidationError> {
    let Some(d) = decisions.get(decision) else {
        return Err(ValidationError::UnknownDecision(turn));
    };
    if d.labels[d.selected].as_bytes() != text.as_bytes() {
        return Err(ValidationError::TextNotOffered(turn));
    }
    Ok(())
}

/// The output was a table frame with deterministically retrieved text spliced in.
fn validate_framed(
    index: usize,
    text: &str,
    slots: &[String],
    table: Option<&Table>,
    output: &Output,
    turn: u32,
) -> Result<(), ValidationError> {
    let Some(table) = table else {
        let name = output.table().unwrap_or_default().to_owned();
        return Err(ValidationError::UnknownTable(name));
    };
    let Some(entry) = table.entries.get(index) else {
        return Err(ValidationError::IndexOutOfTable(turn));
    };
    for slot in slots {
        bounded_text(slot, "output slot")?;
    }
    if entry.matches(PLACEHOLDER).count() != slots.len() {
        return Err(ValidationError::SlotCount(turn));
    }
    if render(entry, slots).as_bytes() != text.as_bytes() {
        return Err(ValidationError::TextNotOffered(turn));
    }
    Ok(())
}

/// Fills each `{}` in `entry` with the next slot, in order. The caller has
/// already checked that the counts agree.
fn render(entry: &str, slots: &[String]) -> String {
    let mut out = String::with_capacity(entry.len());
    let mut slots = slots.iter();
    let mut rest = entry;
    while let Some(at) = rest.find(PLACEHOLDER) {
        out.push_str(&rest[..at]);
        if let Some(slot) = slots.next() {
            out.push_str(slot);
        }
        rest = &rest[at + PLACEHOLDER.len()..];
    }
    out.push_str(rest);
    out
}

/// Gap between the two largest probabilities; 1 for a single-label set.
fn margin_of(probs: &[f64]) -> f64 {
    if probs.len() < 2 {
        return 1.0;
    }
    let mut first = f64::NEG_INFINITY;
    let mut second = f64::NEG_INFINITY;
    for p in probs {
        if *p > first {
            second = first;
            first = *p;
        } else if *p > second {
            second = *p;
        }
    }
    first - second
}

/// Expected level index over ordered labels.
fn expectation_of(probs: &[f64]) -> f64 {
    let mut total = 0.0;
    for (i, p) in probs.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "index is bounded by MAX_LABELS")]
        let weight = i as f64;
        total += weight * p;
    }
    total
}

pub(crate) fn bounded_text(text: &str, what: &'static str) -> Result<(), ValidationError> {
    if text.is_empty() {
        return Err(ValidationError::EmptyText(what));
    }
    if text.len() > MAX_TEXT_BYTES {
        return Err(ValidationError::TextTooLong(what));
    }
    Ok(())
}
