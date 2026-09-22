//! The question-conditioned scorer: `score_i = f(h_state, h_question,
//! h_card_i)`, ported from `lib/scorer.mlpl`.
//!
//! The difference from [`crate::Model`] is the whole point of this module. A
//! `Model` has a head with one column per label, so it can only choose among
//! the labels it was trained on. A `Scorer` reads each candidate as **text**
//! through the same encoder that reads the input, so a candidate written after
//! training ended is scored like any other, and adding one costs no
//! parameters. What it cannot do is know anything about a candidate that its
//! words do not say.

use serde::Deserialize;

use crate::bundle::{BundleError, Provenance};
use crate::features::featurize_vocab;

const SCHEMA: &str = "sw-ml-study.scorer-bundle";
const VERSIONS: [u32; 1] = [1];

/// The candidates a bundle was exported with. A consumer is free to score
/// others: these are what the parity block and the demo use.
#[derive(Clone, Debug, Deserialize)]
pub struct Cards {
    pub names: Vec<String>,
    pub texts: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScorerSnapshots {
    pub steps: Vec<usize>,
    pub seconds: Vec<f64>,
    #[serde(default)]
    pub budgets: Option<Vec<String>>,
    pub metric_names: Vec<String>,
    pub metrics: Vec<Vec<f64>>,
    /// Per snapshot, flattened: the shared embedding and the three projections.
    pub embedding: Vec<Vec<f64>>,
    pub state: Vec<Vec<f64>>,
    pub question: Vec<Vec<f64>>,
    pub card: Vec<Vec<f64>>,
    /// The learned scale a cosine is multiplied by before the softmax.
    pub scale: Vec<f64>,
}

/// MLPL's own probabilities for a handful of rows, so the port can be held to
/// them.
#[derive(Clone, Debug, Deserialize)]
pub struct ScorerParity {
    pub inputs: Vec<String>,
    pub questions: Vec<usize>,
    /// Row-major `[inputs, cards]`, over every card, with the cards a row was
    /// not offered at zero.
    pub probs: Vec<f64>,
    pub cards: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScorerBundle {
    pub schema: String,
    pub version: u32,
    pub provenance: Provenance,
    pub demo: String,
    pub title: String,
    pub dim: usize,
    pub width: usize,
    pub card_width: usize,
    #[serde(default)]
    pub question_token: Option<String>,
    pub vocab: Vec<String>,
    pub questions: Vec<String>,
    pub cards: Cards,
    /// Candidates deliberately kept out of training, named so a reader can tell
    /// which numbers are the generality measurement.
    #[serde(default)]
    pub held_out: Vec<String>,
    /// Cards below this index answer the first question; the rest answer the
    /// second.
    pub rule_count: usize,
    pub snapshots: ScorerSnapshots,
    pub default_snapshot: usize,
    pub parity: ScorerParity,
    #[serde(skip)]
    vocab_index: std::collections::HashMap<String, usize>,
}

/// One snapshot's four arrays.
#[derive(Clone, Copy, Debug)]
pub struct ScorerWeights<'a> {
    pub embedding: &'a [f64],
    pub state: &'a [f64],
    pub question: &'a [f64],
    pub card: &'a [f64],
    pub scale: f64,
}

impl ScorerBundle {
    /// # Errors
    /// If the JSON is malformed, the schema or version is not one this crate
    /// reads, or the arrays do not have the shapes the fields declare.
    pub fn parse(source: &str) -> Result<Self, BundleError> {
        let mut b: Self =
            serde_json::from_str(source).map_err(|e| BundleError::Malformed(e.to_string()))?;
        if b.schema != SCHEMA {
            return Err(BundleError::UnsupportedSchema);
        }
        if !VERSIONS.contains(&b.version) {
            return Err(BundleError::UnsupportedVersion(b.version));
        }
        b.check_shapes()?;
        b.vocab_index = b
            .vocab
            .iter()
            .enumerate()
            .map(|(i, t)| (t.clone(), i + 1))
            .collect();
        Ok(b)
    }

    fn check_shapes(&self) -> Result<(), BundleError> {
        let (d, rows) = (self.dim, self.vocab.len() + 1);
        let s = &self.snapshots;
        let n = s.steps.len();
        if n == 0 || s.seconds.len() != n || s.metrics.len() != n {
            return Err(BundleError::Shape("snapshots"));
        }
        if s.embedding.len() != n
            || s.state.len() != n
            || s.question.len() != n
            || s.scale.len() != n
        {
            return Err(BundleError::Shape("snapshots"));
        }
        if s.embedding.iter().any(|e| e.len() != rows * d)
            || s.state
                .iter()
                .chain(&s.question)
                .chain(&s.card)
                .any(|w| w.len() != d * d)
        {
            return Err(BundleError::Shape("weights"));
        }
        if self.cards.names.len() != self.cards.texts.len() || self.cards.names.is_empty() {
            return Err(BundleError::Shape("cards"));
        }
        if self.default_snapshot >= n {
            return Err(BundleError::Shape("default snapshot"));
        }
        if self.parity.questions.len() != self.parity.inputs.len()
            || self.parity.probs.len() != self.parity.inputs.len() * self.parity.cards
        {
            return Err(BundleError::Shape("parity"));
        }
        Ok(())
    }

    #[must_use]
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.steps.len()
    }

    /// The display label of a snapshot: its named budget, else its seconds.
    #[must_use]
    pub fn snapshot_label(&self, i: usize) -> String {
        if let Some(b) = self.snapshots.budgets.as_ref().and_then(|b| b.get(i)) {
            return b.clone();
        }
        format!(
            "{} s",
            self.snapshots.seconds.get(i).copied().unwrap_or(0.0)
        )
    }

    /// A named metric at one snapshot.
    #[must_use]
    pub fn metric(&self, name: &str, snapshot: usize) -> Option<f64> {
        let j = self.snapshots.metric_names.iter().position(|m| m == name)?;
        self.snapshots.metrics.get(snapshot)?.get(j).copied()
    }

    /// The cards that answer a question: the first block answers question 0,
    /// the rest answer the others. Demo data, not part of the scorer.
    #[must_use]
    pub fn offered(&self, question: usize) -> Vec<usize> {
        (0..self.cards.names.len())
            .filter(|&i| (i < self.rule_count) == (question == 0))
            .collect()
    }

    #[must_use]
    pub fn weights(&self, snapshot: usize) -> ScorerWeights<'_> {
        let s = &self.snapshots;
        ScorerWeights {
            embedding: &s.embedding[snapshot],
            state: &s.state[snapshot],
            question: &s.question[snapshot],
            card: &s.card[snapshot],
            scale: s.scale.get(snapshot).copied().unwrap_or(1.0),
        }
    }
}

/// The scores of one call, and what they select.
#[derive(Clone, Debug)]
pub struct Ranking {
    /// Raw scores, one per card given, in the order they were given.
    pub scores: Vec<f64>,
    /// The same, softmaxed over exactly the cards given.
    pub probs: Vec<f64>,
    pub selected: usize,
    pub confidence: f64,
    pub margin: f64,
}

pub struct Scorer<'a> {
    bundle: &'a ScorerBundle,
    weights: ScorerWeights<'a>,
}

impl<'a> Scorer<'a> {
    #[must_use]
    pub fn at(bundle: &'a ScorerBundle, snapshot: usize) -> Self {
        Self {
            bundle,
            weights: bundle.weights(snapshot),
        }
    }

    #[must_use]
    pub fn new(bundle: &'a ScorerBundle) -> Self {
        Self::at(bundle, bundle.default_snapshot)
    }

    /// Mean-pooled embedding rows of a text's known features.
    #[expect(
        clippy::many_single_char_names,
        reason = "b, f, d, v, n are the bundle, features, dimension, vector and count, as in MLPL"
    )]
    fn pooled(&self, text: &str, width: usize) -> Vec<f64> {
        let b = self.bundle;
        let prepped = b
            .question_token
            .as_ref()
            .map_or_else(|| text.to_owned(), |q| text.replace('?', &format!(" {q} ")));
        let f = featurize_vocab(&prepped, width, |t| b.vocab_index.get(t).copied());
        let d = b.dim;
        let mut v = vec![0.0; d];
        for &slot in &f.slots {
            for (j, x) in v.iter_mut().enumerate() {
                *x += self.weights.embedding[slot * d + j];
            }
        }
        if !f.slots.is_empty() {
            #[expect(clippy::cast_precision_loss, reason = "at most the width")]
            let n = f.slots.len() as f64;
            for x in &mut v {
                *x /= n;
            }
        }
        v
    }

    /// The query vector: the state and its question through their own
    /// projections, bounded by tanh, exactly as in MLPL.
    #[must_use]
    pub fn query(&self, state: &str, question: usize) -> Vec<f64> {
        let b = self.bundle;
        let hs = self.pooled(state, b.width);
        let hq = self.pooled(
            b.questions.get(question).map_or("", String::as_str),
            b.width,
        );
        let d = b.dim;
        (0..d)
            .map(|j| {
                let s: f64 = (0..d).map(|i| hs[i] * self.weights.state[i * d + j]).sum();
                let q: f64 = (0..d)
                    .map(|i| hq[i] * self.weights.question[i * d + j])
                    .sum();
                (s + q).tanh()
            })
            .collect()
    }

    /// One candidate encoded: its text pooled, projected, and scaled to unit
    /// length. Nothing here knows whether the card existed during training --
    /// and the normalization is what keeps that true, since without it a card
    /// that trained as an answer carries a magnitude a new card cannot match.
    #[must_use]
    pub fn encode_card(&self, text: &str) -> Vec<f64> {
        let d = self.bundle.dim;
        let h = self.pooled(text, self.bundle.card_width);
        let mut v: Vec<f64> = (0..d)
            .map(|j| (0..d).map(|i| h[i] * self.weights.card[i * d + j]).sum())
            .collect();
        let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for x in &mut v {
                *x /= norm;
            }
        }
        v
    }

    /// Score candidates supplied as text, in the order given: the query
    /// against each unit card, times the learned scale.
    #[must_use]
    pub fn scores(&self, state: &str, question: usize, cards: &[&str]) -> Vec<f64> {
        let u = self.query(state, question);
        let dot = |c: &&str| -> f64 {
            self.encode_card(c)
                .iter()
                .zip(&u)
                .map(|(a, b)| a * b)
                .sum::<f64>()
        };
        cards.iter().map(|c| self.weights.scale * dot(c)).collect()
    }

    /// Score candidates and read off the selection: this is the call a
    /// reranker makes.
    #[must_use]
    pub fn rank(&self, state: &str, question: usize, cards: &[&str]) -> Ranking {
        let scores = self.scores(state, question, cards);
        let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let exp: Vec<f64> = scores.iter().map(|s| (s - max).exp()).collect();
        let total: f64 = exp.iter().sum();
        let probs: Vec<f64> = exp.iter().map(|e| e / total).collect();
        let mut order: Vec<usize> = (0..probs.len()).collect();
        order.sort_by(|&a, &b| probs[b].total_cmp(&probs[a]).then(a.cmp(&b)));
        let selected = order.first().copied().unwrap_or(0);
        let second = order.get(1).map_or(0.0, |&i| probs[i]);
        Ranking {
            confidence: probs.get(selected).copied().unwrap_or(0.0),
            margin: if probs.len() < 2 {
                1.0
            } else {
                probs[selected] - second
            },
            selected,
            scores,
            probs,
        }
    }
}
