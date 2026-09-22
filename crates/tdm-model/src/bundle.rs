//! The exported bundle: weights, featurizer settings, and demo data.

use std::collections::HashMap;

use serde::Deserialize;

const SCHEMA: &str = "sw-ml-study.decision-bundle";
/// Versions this crate reads: 2 hashes features into slots, 3 uses an exact
/// vocabulary and adds the escalation policy.
const VERSIONS: [u32; 2] = [2, 3];

#[derive(Clone, Debug, Deserialize)]
pub struct Provenance {
    pub producer: String,
    pub producer_revision: String,
    pub generated_at: String,
    pub source_description: String,
}

/// Everything the browser needs to run one demo. Field names match the JSON
/// the MLPL exporter writes.
#[derive(Clone, Debug, Deserialize)]
pub struct Bundle {
    pub schema: String,
    pub version: u32,
    pub provenance: Provenance,
    pub demo: String,
    pub title: String,
    pub question: String,
    /// What the demo says before the visitor says anything. Canned, like
    /// every other line it can speak.
    #[serde(default)]
    pub opening: Option<String>,
    pub labels: Vec<String>,
    pub fallback: String,
    pub threshold: f64,
    /// Version 2: features are hashed into this many slots.
    #[serde(default)]
    pub slots: Option<usize>,
    /// Version 3: the exact training vocabulary. Token `i` owns embedding row
    /// `i + 1`; row 0 is never read, because an unknown token contributes
    /// nothing rather than borrowing a trained token's meaning.
    #[serde(default)]
    pub vocab: Option<Vec<String>>,
    #[serde(skip)]
    pub(crate) vocab_index: HashMap<String, usize>,
    /// When the policy acts, declares none of the offered options apply, or
    /// escalates to a larger decider. Absent in version 2, whose policy is the
    /// single `threshold`.
    #[serde(default)]
    pub escalation: Option<Escalation>,
    /// When set, every `?` in an input becomes this word before featurizing,
    /// so the model can see a question mark the cleaner would otherwise drop.
    #[serde(default)]
    pub question_token: Option<String>,
    /// Independent yes-or-no heads over the same pooled state as the Choice.
    #[serde(default)]
    pub nouls: Option<Nouls>,
    pub width: usize,
    pub dim: usize,
    /// The training timeline: one run, snapshotted at fixed step counts.
    pub snapshots: Snapshots,
    /// The snapshot a visitor sees first.
    pub default_snapshot: usize,
    /// Per label, the canned replies joined by `|`, in label order.
    pub responses: Vec<String>,
    /// Per label, the matcher's keywords joined by spaces, in label order.
    pub keywords: Vec<String>,
    /// Label indices in keyword-priority order.
    pub match_order: Vec<usize>,
    /// Suggested inputs for a first-time visitor.
    #[serde(default)]
    pub examples: Vec<String>,
    pub parity: Parity,
}

/// Noul heads: one sigmoid per named proposition, per snapshot.
#[derive(Clone, Debug, Deserialize)]
pub struct Nouls {
    pub names: Vec<String>,
    /// Per snapshot, row-major `[dim, names]` weights.
    pub head: Vec<Vec<f64>>,
    pub bias: Vec<Vec<f64>>,
}

/// The thresholds under which the program will not act on the model's choice.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Escalation {
    pub min_confidence: f64,
    pub min_margin: f64,
    /// Fewer known features than this is not enough evidence to act on.
    pub min_known: usize,
}

/// One training run, snapshotted. Row `i` of every array is snapshot `i`.
#[derive(Clone, Debug, Deserialize)]
pub struct Snapshots {
    /// Adam steps taken when each snapshot was taken.
    pub steps: Vec<usize>,
    /// The training time each snapshot stands for, on the machine that trained it.
    pub seconds: Vec<f64>,
    /// Human labels for the training budgets, when the bundle names them.
    #[serde(default)]
    pub budgets: Option<Vec<String>>,
    /// Names of the columns of `metrics`, in order.
    pub metric_names: Vec<String>,
    /// Per snapshot, the numbers MLPL measured for it.
    pub metrics: Vec<Vec<f64>>,
    pub embedding: Vec<Vec<f64>>,
    pub head: Vec<Vec<f64>>,
    pub bias: Vec<Vec<f64>>,
}

/// The weights of one snapshot, borrowed from the bundle.
#[derive(Clone, Copy, Debug)]
pub struct Weights<'a> {
    pub embedding: &'a [f64],
    pub head: &'a [f64],
    pub bias: &'a [f64],
    /// Noul head and bias for this snapshot, when the bundle has Noul heads.
    pub noul_head: Option<&'a [f64]>,
    pub noul_bias: Option<&'a [f64]>,
}

/// Inputs with the numbers MLPL produced for them, so a port can prove itself.
#[derive(Clone, Debug, Deserialize)]
pub struct Parity {
    pub inputs: Vec<String>,
    /// Per snapshot, row-major `[inputs, labels]` probabilities.
    pub probs: Vec<Vec<f64>>,
    /// Per snapshot, row-major `[inputs, nouls]` probabilities.
    #[serde(default)]
    pub nouls: Option<Vec<Vec<f64>>>,
    /// The keyword matcher's label index for each input.
    pub matcher: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BundleError {
    Malformed(String),
    UnsupportedSchema,
    UnsupportedVersion(u32),
    Shape(&'static str),
}

impl Bundle {
    /// Parses a bundle and checks every array against the shapes it declares.
    ///
    /// # Errors
    ///
    /// Returns the first inconsistency found.
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
            .flatten()
            .enumerate()
            .map(|(i, t)| (t.clone(), i + 1))
            .collect();
        Ok(b)
    }

    fn check_shapes(&self) -> Result<(), BundleError> {
        let k = self.labels.len();
        if k == 0 {
            return Err(BundleError::Shape("labels"));
        }
        self.check_snapshots(k)?;
        // A bundle whose labels are another program's rules has no keyword
        // yardstick of its own: no keywords, and no match order.
        let no_matcher = self.keywords.is_empty() && self.match_order.is_empty();
        if self.responses.len() != k || (self.keywords.len() != k && !no_matcher) {
            return Err(BundleError::Shape("per-label data"));
        }
        if self.match_order.iter().any(|&i| i >= k) || !self.labels.contains(&self.fallback) {
            return Err(BundleError::Shape("matcher"));
        }
        let n = self.parity.inputs.len();
        let per_snapshot = self.parity.probs.iter().all(|p| p.len() == n * k);
        if self.parity.probs.len() != self.snapshot_count()
            || !per_snapshot
            || (self.parity.matcher.len() != n && !no_matcher)
        {
            return Err(BundleError::Shape("parity"));
        }
        Ok(())
    }

    fn check_snapshots(&self, k: usize) -> Result<(), BundleError> {
        let s = &self.snapshots;
        let c = s.steps.len();
        let counts = [
            s.seconds.len(),
            s.metrics.len(),
            s.embedding.len(),
            s.head.len(),
            s.bias.len(),
        ];
        if c == 0 || counts.iter().any(|&r| r != c) || self.default_snapshot >= c {
            return Err(BundleError::Shape("snapshot count"));
        }
        let rows = self.embedding_rows();
        let shapes_ok = s.embedding.iter().all(|e| e.len() == rows * self.dim)
            && s.head.iter().all(|h| h.len() == self.dim * k)
            && s.bias.iter().all(|b| b.len() == k)
            && s.metrics.iter().all(|m| m.len() == s.metric_names.len());
        if !shapes_ok {
            return Err(BundleError::Shape("snapshot weights"));
        }
        if let Some(n) = &self.nouls {
            let m = n.names.len();
            let ok = n.head.len() == c
                && n.bias.len() == c
                && n.head.iter().all(|h| h.len() == self.dim * m)
                && n.bias.iter().all(|b| b.len() == m);
            if !ok {
                return Err(BundleError::Shape("noul heads"));
            }
        }
        Ok(())
    }

    /// Rows in the embedding table: hash slots, or the vocabulary plus the
    /// unused unknown row.
    #[must_use]
    pub fn embedding_rows(&self) -> usize {
        self.vocab
            .as_ref()
            .map_or(self.slots.unwrap_or(0), |v| v.len() + 1)
    }

    /// The embedding row of a known token, or `None` if the vocabulary does not
    /// contain it. Always `None` for a hashed bundle.
    #[must_use]
    pub fn vocab_row(&self, token: &str) -> Option<usize> {
        self.vocab_index.get(token).copied()
    }

    /// The display label of a snapshot: its named budget, else its seconds.
    #[must_use]
    pub fn snapshot_label(&self, i: usize) -> String {
        if let Some(b) = self.snapshots.budgets.as_ref().and_then(|b| b.get(i)) {
            return b.clone();
        }
        if self.snapshots.steps[i] == 0 {
            "0 s".to_owned()
        } else {
            format!("{} s", self.snapshots.seconds[i])
        }
    }

    /// The names of the Noul heads, in output order; empty without heads.
    #[must_use]
    pub fn noul_names(&self) -> &[String] {
        self.nouls.as_ref().map_or(&[], |n| n.names.as_slice())
    }

    /// How many snapshots the timeline has.
    #[must_use]
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.steps.len()
    }

    /// The weights of one snapshot.
    #[must_use]
    pub fn weights(&self, snapshot: usize) -> Weights<'_> {
        Weights {
            embedding: &self.snapshots.embedding[snapshot],
            head: &self.snapshots.head[snapshot],
            bias: &self.snapshots.bias[snapshot],
            noul_head: self.nouls.as_ref().map(|n| n.head[snapshot].as_slice()),
            noul_bias: self.nouls.as_ref().map(|n| n.bias[snapshot].as_slice()),
        }
    }

    /// One measured number for one snapshot, by metric name.
    #[must_use]
    pub fn metric(&self, snapshot: usize, name: &str) -> Option<f64> {
        let col = self.snapshots.metric_names.iter().position(|m| m == name)?;
        self.snapshots.metrics[snapshot].get(col).copied()
    }

    /// Index of a label by name.
    #[must_use]
    pub fn label_index(&self, name: &str) -> Option<usize> {
        self.labels.iter().position(|l| l == name)
    }

    /// The canned replies for a label.
    #[must_use]
    pub fn replies(&self, label: usize) -> Vec<&str> {
        self.responses[label].split('|').collect()
    }

    /// Trainable parameters.
    #[must_use]
    pub fn param_count(&self) -> usize {
        let w = self.weights(0);
        w.embedding.len() + w.head.len() + w.bias.len()
    }
}
