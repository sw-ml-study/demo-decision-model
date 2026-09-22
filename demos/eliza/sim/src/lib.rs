//! What the demo 01 tools share: reading the oracle corpus, and the rule that
//! turns the oracle's rules into model 4's classes.

use std::collections::{BTreeMap, HashMap};

/// Fewest training examples a rule needs to be a class of its own.
pub const MIN_RULE: usize = 8;

/// The rule every unmatched or merged-away input falls back to.
pub const CATCH_ALL: &str = "xnone#0";

/// One labelled input from `corpus.tsv`.
pub struct CorpusRow {
    pub source: String,
    pub val: bool,
    pub rule: String,
    pub text: String,
    pub reply: String,
}

/// Read `source<TAB>split<TAB>rule<TAB>text<TAB>reply` rows.
///
/// # Panics
/// If the file cannot be read.
#[must_use]
pub fn read_corpus(path: &str) -> Vec<CorpusRow> {
    parse_corpus(&std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}")))
}

/// The same rows, from text already in hand.
#[must_use]
pub fn parse_corpus(text: &str) -> Vec<CorpusRow> {
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f.len() == 5).then(|| CorpusRow {
                source: f[0].to_owned(),
                val: f[1] == "val",
                rule: f[2].to_owned(),
                text: f[3].to_owned(),
                reply: f[4].to_owned(),
            })
        })
        .collect()
}

/// The classes, and every rule the oracle used mapped to its class. A rule
/// with at least [`MIN_RULE`] training examples is its own class; a rarer one
/// is merged into its keyword's most common class, else into [`CATCH_ALL`].
/// A rule seen only outside training maps the same way through [`class_of`].
#[must_use]
pub fn rule_classes<'a>(
    train: impl Iterator<Item = &'a str>,
) -> (Vec<String>, HashMap<String, String>) {
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for r in train {
        *count.entry(r).or_default() += 1;
    }
    let kept: Vec<String> = count
        .iter()
        .filter(|&(_, &n)| n >= MIN_RULE)
        .map(|(r, _)| (*r).to_owned())
        .collect();
    let map = count
        .keys()
        .map(|rule| {
            let key = rule.split('#').next().unwrap_or_default();
            let target = if kept.iter().any(|k| k == rule) {
                (*rule).to_owned()
            } else {
                kept.iter()
                    .filter(|r| r.split('#').next() == Some(key))
                    .max_by_key(|r| (count[r.as_str()], std::cmp::Reverse((*r).clone())))
                    .map_or_else(|| CATCH_ALL.to_owned(), Clone::clone)
            };
            ((*rule).to_owned(), target)
        })
        .collect();
    (kept, map)
}

/// The class of a rule under a map from [`rule_classes`].
#[must_use]
pub fn class_of<'a, S: std::hash::BuildHasher>(
    map: &'a HashMap<String, String, S>,
    rule: &str,
) -> &'a str {
    map.get(rule).map_or(CATCH_ALL, String::as_str)
}
