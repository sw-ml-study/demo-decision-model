//! The exported bundle: weights, featurizer settings, and demo data.

use serde::Deserialize;

const SCHEMA: &str = "sw-ml-study.decision-bundle";
const VERSION: u32 = 2;

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
    pub slots: usize,
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

/// One training run, snapshotted. Row `i` of every array is snapshot `i`.
#[derive(Clone, Debug, Deserialize)]
pub struct Snapshots {
    /// Adam steps taken when each snapshot was taken.
    pub steps: Vec<usize>,
    /// The training time each snapshot stands for, on the machine that trained it.
    pub seconds: Vec<f64>,
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
}

/// Inputs with the numbers MLPL produced for them, so a port can prove itself.
#[derive(Clone, Debug, Deserialize)]
pub struct Parity {
    pub inputs: Vec<String>,
    /// Per snapshot, row-major `[inputs, labels]` probabilities.
    pub probs: Vec<Vec<f64>>,
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
        let b: Self =
            serde_json::from_str(source).map_err(|e| BundleError::Malformed(e.to_string()))?;
        if b.schema != SCHEMA {
            return Err(BundleError::UnsupportedSchema);
        }
        if b.version != VERSION {
            return Err(BundleError::UnsupportedVersion(b.version));
        }
        b.check_shapes()?;
        Ok(b)
    }

    fn check_shapes(&self) -> Result<(), BundleError> {
        let k = self.labels.len();
        if k == 0 {
            return Err(BundleError::Shape("labels"));
        }
        self.check_snapshots(k)?;
        if self.responses.len() != k || self.keywords.len() != k {
            return Err(BundleError::Shape("per-label data"));
        }
        if self.match_order.iter().any(|&i| i >= k) || !self.labels.contains(&self.fallback) {
            return Err(BundleError::Shape("matcher"));
        }
        let n = self.parity.inputs.len();
        let per_snapshot = self.parity.probs.iter().all(|p| p.len() == n * k);
        if self.parity.probs.len() != self.snapshot_count()
            || !per_snapshot
            || self.parity.matcher.len() != n
        {
            return Err(BundleError::Shape("parity"));
        }
        Ok(())
    }

    fn check_snapshots(&self, k: usize) -> Result<(), BundleError> {
        let s = &self.snapshots;
        let c = s.steps.len();
        let rows = [
            s.seconds.len(),
            s.metrics.len(),
            s.embedding.len(),
            s.head.len(),
            s.bias.len(),
        ];
        if c == 0 || rows.iter().any(|&r| r != c) || self.default_snapshot >= c {
            return Err(BundleError::Shape("snapshot count"));
        }
        let shapes_ok = s.embedding.iter().all(|e| e.len() == self.slots * self.dim)
            && s.head.iter().all(|h| h.len() == self.dim * k)
            && s.bias.iter().all(|b| b.len() == k)
            && s.metrics.iter().all(|m| m.len() == s.metric_names.len());
        if !shapes_ok {
            return Err(BundleError::Shape("snapshot weights"));
        }
        Ok(())
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
