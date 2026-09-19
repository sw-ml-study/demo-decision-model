//! One turn: the state that went in, the decisions made over it, the policy
//! rule that consumed them, and the output that resulted.

use serde::Deserialize;

use crate::decision::Decision;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Turn {
    pub(crate) index: u32,
    pub(crate) state: State,
    pub(crate) decisions: Vec<Decision>,
    pub(crate) policy: Policy,
    pub(crate) output: Output,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    /// The unstructured input this turn decided over.
    pub(crate) text: String,
    /// What the encoder actually saw, for display beside the decision.
    #[serde(default)]
    pub(crate) features: Vec<String>,
    /// Structured state the program keeps outside the model.
    #[serde(default)]
    pub(crate) memory: Vec<Memory>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    pub(crate) id: String,
    pub(crate) text: String,
    #[serde(default)]
    pub(crate) tag: String,
    pub(crate) age_turns: u32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    /// The rule as written in the application. Thresholds belong to the program,
    /// not the model, so the trace shows them.
    pub(crate) rule: String,
    /// The branch the rule selected.
    pub(crate) branch: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub(crate) table: String,
    pub(crate) index: usize,
    pub(crate) text: String,
    /// Values spliced into the entry's `{}` placeholders, in order.
    #[serde(default)]
    pub(crate) slots: Vec<String>,
}

impl Turn {
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }
    #[must_use]
    pub fn decisions(&self) -> &[Decision] {
        &self.decisions
    }
    #[must_use]
    pub const fn policy(&self) -> &Policy {
        &self.policy
    }
    #[must_use]
    pub const fn output(&self) -> &Output {
        &self.output
    }
}

impl State {
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
    #[must_use]
    pub fn features(&self) -> &[String] {
        &self.features
    }
    #[must_use]
    pub fn memory(&self) -> &[Memory] {
        &self.memory
    }
}

impl Memory {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }
    #[must_use]
    pub const fn age_turns(&self) -> u32 {
        self.age_turns
    }
}

impl Policy {
    #[must_use]
    pub fn rule(&self) -> &str {
        &self.rule
    }
    #[must_use]
    pub fn branch(&self) -> &str {
        &self.branch
    }
}

impl Output {
    #[must_use]
    pub fn table(&self) -> &str {
        &self.table
    }
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
    #[must_use]
    pub fn slots(&self) -> &[String] {
        &self.slots
    }
}
