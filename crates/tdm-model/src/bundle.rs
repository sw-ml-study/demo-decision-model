//! The exported bundle: weights, featurizer settings, and demo data.

use serde::Deserialize;

const SCHEMA: &str = "sw-ml-study.decision-bundle";
const VERSION: u32 = 1;

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
    pub labels: Vec<String>,
    pub fallback: String,
    pub threshold: f64,
    pub slots: usize,
    pub width: usize,
    pub dim: usize,
    pub embedding: Vec<f64>,
    pub head: Vec<f64>,
    pub bias: Vec<f64>,
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

/// Inputs with the numbers MLPL produced for them, so a port can prove itself.
#[derive(Clone, Debug, Deserialize)]
pub struct Parity {
    pub inputs: Vec<String>,
    /// Row-major `[inputs, labels]` probabilities.
    pub probs: Vec<f64>,
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
        if self.embedding.len() != self.slots * self.dim {
            return Err(BundleError::Shape("embedding"));
        }
        if self.head.len() != self.dim * k {
            return Err(BundleError::Shape("head"));
        }
        if self.bias.len() != k {
            return Err(BundleError::Shape("bias"));
        }
        if self.responses.len() != k || self.keywords.len() != k {
            return Err(BundleError::Shape("per-label data"));
        }
        if self.match_order.iter().any(|&i| i >= k) || !self.labels.contains(&self.fallback) {
            return Err(BundleError::Shape("matcher"));
        }
        let n = self.parity.inputs.len();
        if self.parity.probs.len() != n * k || self.parity.matcher.len() != n {
            return Err(BundleError::Shape("parity"));
        }
        Ok(())
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
        self.embedding.len() + self.head.len() + self.bias.len()
    }
}
