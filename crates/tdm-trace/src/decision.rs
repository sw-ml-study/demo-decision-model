//! A typed decision and the calibration that produced its distribution.
//!
//! The shape is uniform across all three kinds so that one validator, one
//! schema, and one renderer serve them all; `p` and `expectation` are the only
//! kind-specific fields.

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub(crate) kind: String,
    /// The question asked. For a noul this is the proposition.
    pub(crate) question: String,
    pub(crate) labels: Vec<String>,
    pub(crate) probs: Vec<f64>,
    pub(crate) selected: usize,
    pub(crate) confidence: f64,
    pub(crate) margin: f64,
    pub(crate) calibration: Calibration,
    /// Noul only: probability that the proposition holds.
    #[serde(default)]
    pub(crate) p: Option<f64>,
    /// Scale only: the expected level index over ordered labels.
    #[serde(default)]
    pub(crate) expectation: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Calibration {
    /// `"none"` for a raw model output, or the name of the map applied.
    pub(crate) method: String,
    #[serde(default)]
    pub(crate) t: Option<f64>,
}

impl Decision {
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }
    #[must_use]
    pub fn question(&self) -> &str {
        &self.question
    }
    #[must_use]
    pub fn labels(&self) -> &[String] {
        &self.labels
    }
    #[must_use]
    pub fn probs(&self) -> &[f64] {
        &self.probs
    }
    #[must_use]
    pub const fn selected(&self) -> usize {
        self.selected
    }
    #[must_use]
    pub fn selected_label(&self) -> &str {
        &self.labels[self.selected]
    }
    /// Probability of the selected label.
    #[must_use]
    pub const fn confidence(&self) -> f64 {
        self.confidence
    }
    /// Gap between the two most probable labels. Reported beside confidence
    /// because the two are different doubts: `.48` with a `.21` margin is an
    /// uncertain decision, `.48` with `.01` is a coin flip between two
    /// candidates, and a policy may reasonably treat those differently.
    #[must_use]
    pub const fn margin(&self) -> f64 {
        self.margin
    }
    #[must_use]
    pub const fn calibration(&self) -> &Calibration {
        &self.calibration
    }
    /// Noul only: the probability that the proposition holds.
    #[must_use]
    pub const fn p(&self) -> Option<f64> {
        self.p
    }
    /// Scale only: the expected level index over the ordered labels.
    #[must_use]
    pub const fn expectation(&self) -> Option<f64> {
        self.expectation
    }
    /// Probability of a label by name, or `None` when the decision did not offer
    /// it. Policy reads a distribution this way; an index may be renumbered by a
    /// later revision, but a name survives.
    #[must_use]
    pub fn prob_of(&self, label: &str) -> Option<f64> {
        self.labels
            .iter()
            .position(|l| l == label)
            .map(|i| self.probs[i])
    }
}

impl Calibration {
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }
    /// The fitted scalar, when the method is a temperature.
    #[must_use]
    pub const fn temperature(&self) -> Option<f64> {
        self.t
    }
    #[must_use]
    pub fn is_calibrated(&self) -> bool {
        self.method != "none"
    }
}
