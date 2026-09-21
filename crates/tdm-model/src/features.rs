//! The featurizer, ported exactly from `lib/text.mlpl`: normalize, split into
//! words, then hash words and adjacent bigrams into a fixed number of slots.
//!
//! Exactness matters more than elegance here. A single character handled
//! differently changes a hash, which changes which embedding row is read, which
//! changes the decision. The parity test is what keeps this honest.

/// The rolling-hash modulus, as in MLPL.
const MODULUS: u64 = 2_147_483_647;

/// Lowercase ASCII letters, keep letters, digits and spaces, delete apostrophes,
/// and turn every other character into a space. Matches `u:text_clean`, which
/// works byte by byte, so non-ASCII bytes also become spaces.
fn clean(s: &str) -> String {
    s.bytes()
        .filter_map(|b| match b {
            b'A'..=b'Z' => Some(char::from(b + 32)),
            b'a'..=b'z' | b'0'..=b'9' | b' ' => Some(char::from(b)),
            b'\'' => None,
            _ => Some(' '),
        })
        .collect()
}

/// The non-empty words of a cleaned string.
#[must_use]
pub fn words(s: &str) -> Vec<String> {
    clean(s)
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Base-31 rolling checksum of a string's bytes modulo 2147483647.
#[must_use]
pub fn hash(s: &str) -> u64 {
    s.bytes().fold(7, |h, b| (h * 31 + u64::from(b)) % MODULUS)
}

/// What the model sees for one input.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Features {
    /// Embedding rows of the features the model has evidence about; these are
    /// the rows the mean pool reads.
    pub slots: Vec<usize>,
    /// Every feature, for display: words first, then `a_b` bigrams.
    pub tokens: Vec<String>,
    /// Per token, whether it contributes. Always true under hashing, which is
    /// exactly its flaw: an unseen token still reads some row.
    pub known: Vec<bool>,
}

/// The features' text, before any lookup: words first, then adjacent bigrams,
/// up to `width` in all.
#[must_use]
pub fn tokens(text: &str, width: usize) -> Vec<String> {
    let ws = words(text);
    let mut out: Vec<String> = ws.iter().take(width).cloned().collect();
    for pair in ws.windows(2) {
        if out.len() >= width {
            break;
        }
        out.push(format!("{}_{}", pair[0], pair[1]));
    }
    out
}

/// Features against an exact vocabulary: unknown tokens are kept for display
/// and contribute nothing.
#[must_use]
pub fn featurize_vocab(text: &str, width: usize, row: impl Fn(&str) -> Option<usize>) -> Features {
    let toks = tokens(text, width);
    let rows: Vec<Option<usize>> = toks.iter().map(|t| row(t)).collect();
    Features {
        slots: rows.iter().flatten().copied().collect(),
        known: rows.iter().map(Option::is_some).collect(),
        tokens: toks,
    }
}

/// Words first, then adjacent bigrams, up to `width` features in all.
#[must_use]
pub fn featurize(text: &str, slots: usize, width: usize) -> Features {
    let tokens = tokens(text, width);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a hash modulo a slot count below u32::MAX fits usize on every target"
    )]
    let ids = tokens
        .iter()
        .map(|t| (hash(t) % slots as u64) as usize)
        .collect();
    let known = vec![true; tokens.len()];
    Features {
        slots: ids,
        tokens,
        known,
    }
}
