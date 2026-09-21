//! One turn of a demo: the model decides, ordinary code applies the policy, and
//! a reply is quoted from the bundle's table. Nothing here composes a string.

use crate::bundle::Bundle;
use crate::matcher::KeywordMatcher;
use crate::model::{Decision, Model};

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

/// Run one turn. The threshold belongs to the bundle's policy, not the model:
/// the model reports belief and this function decides whether belief is enough.
#[must_use]
pub fn respond(bundle: &Bundle, text: &str, turn: usize) -> Turn {
    let decision = Model::new(bundle).decide(text);
    let acted = decision.confidence >= bundle.threshold;
    let label = if acted {
        decision.selected
    } else {
        bundle
            .label_index(&bundle.fallback)
            .unwrap_or(decision.selected)
    };
    let replies = bundle.replies(label);
    let index = turn % replies.len();
    let reply = Reply {
        label,
        index,
        text: replies[index].to_owned(),
        acted,
    };
    let matcher = KeywordMatcher::new(bundle).classify(text);
    Turn {
        input: text.to_owned(),
        decision,
        reply,
        matcher,
    }
}
