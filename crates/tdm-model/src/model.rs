//! The forward pass: mean of the addressed embedding rows, one linear head,
//! softmax. The same arithmetic as `u:cm_infer` in `lib/choice_model.mlpl`.

use crate::bundle::Bundle;
use crate::features::{Features, featurize};

/// A trained model, ready to decide.
pub struct Model<'a> {
    bundle: &'a Bundle,
}

/// One Choice decision, with what the model saw.
#[derive(Clone, Debug)]
pub struct Decision {
    pub probs: Vec<f64>,
    pub selected: usize,
    pub confidence: f64,
    pub margin: f64,
    pub features: Features,
}

impl<'a> Model<'a> {
    #[must_use]
    pub const fn new(bundle: &'a Bundle) -> Self {
        Self { bundle }
    }

    /// Logits for one input. An input with no features pools to zero, so its
    /// logits are the bias alone, exactly as in MLPL.
    #[must_use]
    pub fn logits(&self, features: &Features) -> Vec<f64> {
        let b = self.bundle;
        let (d, k) = (b.dim, b.labels.len());
        let mut pooled = vec![0.0; d];
        if !features.slots.is_empty() {
            for &slot in &features.slots {
                for (j, p) in pooled.iter_mut().enumerate() {
                    *p += b.embedding[slot * d + j];
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
        (0..k)
            .map(|c| b.bias[c] + (0..d).map(|j| pooled[j] * b.head[j * k + c]).sum::<f64>())
            .collect()
    }

    /// Featurize, run the forward pass, and read off the decision.
    #[must_use]
    pub fn decide(&self, text: &str) -> Decision {
        let features = featurize(text, self.bundle.slots, self.bundle.width);
        let probs = softmax(&self.logits(&features));
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
        }
    }
}

fn softmax(logits: &[f64]) -> Vec<f64> {
    let max = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = logits.iter().map(|l| (l - max).exp()).collect();
    let total: f64 = exps.iter().sum();
    exps.iter().map(|e| e / total).collect()
}
