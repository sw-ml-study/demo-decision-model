//! One turn of a demo: the model decides, ordinary code applies the policy, and
//! a reply is quoted from the bundle's table. Nothing here composes a string.

use crate::bundle::Bundle;
use crate::matcher::KeywordMatcher;
use crate::model::{Decision, Model};

/// What the program decided to do with the model's choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Confident enough, on enough evidence: reply for the chosen label.
    Act,
    /// The model chose the escape hatch: none of the offered options applies.
    NoneApplies,
    /// Not enough evidence or confidence to act: hand the decision to a larger
    /// decider over the same options. Where none is available (in a browser),
    /// reply from the fallback table and say so.
    Escalate,
}

/// The reply the program chose, and why.
#[derive(Clone, Debug)]
pub struct Reply {
    /// The label whose table the reply was quoted from.
    pub label: usize,
    /// The entry within that table.
    pub index: usize,
    /// The entry, verbatim.
    pub text: String,
    /// Whether the policy acted on the model's choice, or abstained to fallback.
    pub acted: bool,
    pub outcome: Outcome,
}

/// Everything one turn produced, for the chat view and the trace view.
#[derive(Clone, Debug)]
pub struct Turn {
    pub input: String,
    pub decision: Decision,
    pub reply: Reply,
    /// What the 1966-style keyword list would have picked.
    pub matcher: usize,
}

/// Run one turn at the bundle's default snapshot.
#[must_use]
pub fn respond(bundle: &Bundle, text: &str, turn: usize) -> Turn {
    respond_at(bundle, bundle.default_snapshot, text, turn)
}

/// Run one turn with the model as it was at one snapshot of training. The
/// threshold belongs to the bundle's policy, not the model: the model reports
/// belief and this function decides whether belief is enough.
#[must_use]
pub fn respond_at(bundle: &Bundle, snapshot: usize, text: &str, turn: usize) -> Turn {
    let decision = Model::at(bundle, snapshot).decide(text);
    let fallback = bundle
        .label_index(&bundle.fallback)
        .unwrap_or(decision.selected);
    let outcome = match bundle.escalation {
        Some(_) if decision.selected == fallback => Outcome::NoneApplies,
        Some(e)
            if decision.features.slots.len() < e.min_known
                || decision.confidence < e.min_confidence
                || decision.margin < e.min_margin =>
        {
            Outcome::Escalate
        }
        Some(_) => Outcome::Act,
        None if decision.confidence >= bundle.threshold => Outcome::Act,
        None => Outcome::Escalate,
    };
    let acted = outcome == Outcome::Act;
    let label = if acted { decision.selected } else { fallback };
    let replies = bundle.replies(label);
    let index = turn % replies.len();
    let reply = Reply {
        label,
        index,
        text: replies[index].to_owned(),
        acted,
        outcome,
    };
    let matcher = KeywordMatcher::new(bundle).classify(text);
    Turn {
        input: text.to_owned(),
        decision,
        reply,
        matcher,
    }
}
