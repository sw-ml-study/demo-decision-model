//! The forward pass: mean of the addressed embedding rows, one linear head,
//! softmax. The same arithmetic as `u:cm_infer` in `lib/choice_model.mlpl`.

use crate::bundle::{Bundle, Weights};
use crate::features::{Features, featurize, featurize_vocab};

/// One snapshot of a trained model, ready to decide.
pub struct Model<'a> {
    bundle: &'a Bundle,
    weights: Weights<'a>,
}

/// One Choice decision, with what the model saw.
#[derive(Clone, Debug)]
pub struct Decision {
    pub probs: Vec<f64>,
    pub selected: usize,
    pub confidence: f64,
    pub margin: f64,
    pub features: Features,
    /// One probability per Noul head, in the bundle's `nouls.names` order.
    pub nouls: Vec<f64>,
}

impl<'a> Model<'a> {
    /// The model as it was at one snapshot of the training timeline.
    #[must_use]
    pub fn at(bundle: &'a Bundle, snapshot: usize) -> Self {
        Self {
            bundle,
            weights: bundle.weights(snapshot),
        }
    }

    /// The model at the bundle's default snapshot.
    #[must_use]
    pub fn new(bundle: &'a Bundle) -> Self {
        Self::at(bundle, bundle.default_snapshot)
    }

    /// The shared state: the mean of the embedding rows the known features
    /// address, or zeros when there are none.
    #[must_use]
    pub fn pooled(&self, features: &Features) -> Vec<f64> {
        let d = self.bundle.dim;
        let mut pooled = vec![0.0; d];
        if !features.slots.is_empty() {
            for &slot in &features.slots {
                for (j, p) in pooled.iter_mut().enumerate() {
                    *p += self.weights.embedding[slot * d + j];
                }
            }
            #[expect(
                clippy::cast_precision_loss,
                reason = "feature count is at most the width"
            )]
            let n = features.slots.len() as f64;
            for p in &mut pooled {
                *p /= n;
            }
        }
        pooled
    }

    /// Logits for one input. An input with no features pools to zero, so its
    /// logits are the bias alone, exactly as in MLPL.
    #[must_use]
    pub fn logits(&self, features: &Features) -> Vec<f64> {
        linear(&self.pooled(features), self.weights.head, self.weights.bias)
    }

    /// Noul probabilities from the same pooled state: one pass, several typed
    /// questions. Empty when the bundle has no Noul heads.
    #[must_use]
    pub fn nouls(&self, features: &Features) -> Vec<f64> {
        match (self.weights.noul_head, self.weights.noul_bias) {
            (Some(h), Some(b)) => linear(&self.pooled(features), h, b)
                .into_iter()
                .map(|z| 1.0 / (1.0 + (-z).exp()))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Featurize, run the forward pass, and read off the decision.
    #[must_use]
    pub fn decide(&self, text: &str) -> Decision {
        let b = self.bundle;
        let prepped = b
            .question_token
            .as_ref()
            .map_or_else(|| text.to_owned(), |q| text.replace('?', &format!(" {q} ")));
        let text = prepped.as_str();
        let features = if b.vocab.is_some() {
            featurize_vocab(text, b.width, |t| b.vocab_row(t))
        } else {
            featurize(text, b.slots.unwrap_or(1), b.width)
        };
        let probs = softmax(&self.logits(&features));
        let nouls = self.nouls(&features);
        let mut order: Vec<usize> = (0..probs.len()).collect();
        order.sort_by(|&a, &b| probs[b].total_cmp(&probs[a]).then(a.cmp(&b)));
        let selected = order[0];
        let second = order.get(1).map_or(0.0, |&i| probs[i]);
        let margin = if probs.len() < 2 {
            1.0
        } else {
            probs[selected] - second
        };
        Decision {
            confidence: probs[selected],
            selected,
            margin,
            probs,
            features,
            nouls,
        }
    }
}

/// `x W + b` for one row, with `W` row-major `[x.len(), b.len()]`.
fn linear(x: &[f64], w: &[f64], b: &[f64]) -> Vec<f64> {
    let k = b.len();
    (0..k)
        .map(|c| {
            b[c] + x
                .iter()
                .enumerate()
                .map(|(j, xj)| xj * w[j * k + c])
                .sum::<f64>()
        })
        .collect()
}

fn softmax(logits: &[f64]) -> Vec<f64> {
    let max = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = logits.iter().map(|l| (l - max).exp()).collect();
    let total: f64 = exps.iter().sum();
    exps.iter().map(|e| e / total).collect()
}
