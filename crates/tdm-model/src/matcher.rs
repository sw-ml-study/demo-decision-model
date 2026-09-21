//! The keyword-matcher yardstick, ported from the demo's MLPL: per-label
//! whole-word keyword lists, tried in priority order, first hit wins, else the
//! fallback label. Generic: the lists and the order are bundle data.

use crate::bundle::Bundle;
use crate::features::words;

pub struct KeywordMatcher<'a> {
    bundle: &'a Bundle,
    fallback: usize,
}

impl<'a> KeywordMatcher<'a> {
    #[must_use]
    pub fn new(bundle: &'a Bundle) -> Self {
        let fallback = bundle.label_index(&bundle.fallback).unwrap_or(0);
        Self { bundle, fallback }
    }

    /// The label the keyword list picks for an input.
    #[must_use]
    pub fn classify(&self, text: &str) -> usize {
        let ws = words(text);
        for &label in &self.bundle.match_order {
            let hit = self.bundle.keywords[label]
                .split(' ')
                .filter(|k| !k.is_empty())
                .any(|k| ws.iter().any(|w| w == k));
            if hit {
                return label;
            }
        }
        self.fallback
    }
}
