//! A keyword script engine of the 1966 kind, run from data: keywords with
//! ranks, synonym groups, pre- and post-substitution, wildcard decomposition,
//! cycling reassembly, `goto` between keywords, and memory. Given an input it
//! reports which rule fired and what it said, so it can serve as a labelling
//! oracle; given a rule chosen by a model, it produces that rule's reply.
//!
//! Numbering: in a reassembly, `(n)` is the n-th *part* of the decomposition,
//! where parts are the wildcards (`*`) and the synonym matches (`@group`), in
//! order; literal words are not parts. Parts are post-substituted (my -> your)
//! before they are spliced in.

use std::collections::{HashMap, VecDeque};

use serde::Deserialize;

use crate::features::words;

#[derive(Clone, Debug, Deserialize)]
pub struct KeywordScript {
    pub initial: String,
    #[serde(rename = "final")]
    pub farewell: String,
    #[serde(default)]
    pub quit: Vec<String>,
    #[serde(default)]
    pub pre: HashMap<String, String>,
    #[serde(default)]
    pub post: HashMap<String, String>,
    #[serde(default)]
    pub synonyms: HashMap<String, Vec<String>>,
    pub keys: Vec<Key>,
}

impl KeywordScript {
    /// The decomposition pattern of a `key#i` rule, for display.
    #[must_use]
    pub fn pattern(&self, rule: &str) -> Option<&str> {
        let (key, i) = rule.split_once('#')?;
        let k = self.keys.iter().find(|k| k.key == key)?;
        Some(k.decomps.get(i.parse::<usize>().ok()?)?.pattern.as_str())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Key {
    pub key: String,
    #[serde(default)]
    pub rank: i32,
    #[serde(default)]
    pub goto: Option<String>,
    #[serde(default)]
    pub decomps: Vec<Decomp>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Decomp {
    pub pattern: String,
    pub replies: Vec<String>,
    /// A memory decomposition stores its reassembly instead of replying.
    #[serde(default)]
    pub memory: bool,
}

/// What the engine said, which rule said it, and the parts it was built from.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Answer {
    /// `key#i` for decomposition `i` of `key`, or `memory`.
    pub rule: String,
    pub text: String,
    /// The reassembly as written in the script, with each `(n)` as `{n}`.
    pub frame: String,
    /// What each `{n}` was filled with: the input's own words, post-substituted.
    pub slots: Vec<(String, String)>,
    /// Whether the rule's own decomposition matched the input. False when a
    /// rule chosen from outside did not fit and a fallback answered instead.
    pub matched: bool,
}

impl Answer {
    fn fixed(rule: &str, text: String) -> Self {
        Self {
            rule: rule.to_owned(),
            frame: text.clone(),
            text,
            slots: Vec::new(),
            matched: false,
        }
    }
}

/// The fallback keyword, used when nothing in the input matches.
const NONE_KEY: &str = "xnone";

pub struct KeywordEngine<'a> {
    script: &'a KeywordScript,
    counters: HashMap<(usize, usize), usize>,
    memory: VecDeque<String>,
}

impl<'a> KeywordEngine<'a> {
    #[must_use]
    pub fn new(script: &'a KeywordScript) -> Self {
        Self {
            script,
            counters: HashMap::new(),
            memory: VecDeque::new(),
        }
    }

    /// Every rule that can produce a reply: `key#i` for each non-memory
    /// decomposition of each keyword that has its own decompositions, plus
    /// `memory`. This is the label set a model chooses from.
    #[must_use]
    pub fn rule_ids(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .script
            .keys
            .iter()
            .filter(|k| k.goto.is_none())
            .flat_map(|k| {
                k.decomps
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| !d.memory)
                    .map(move |(i, _)| format!("{}#{i}", k.key))
            })
            .collect();
        out.push("memory".to_owned());
        out
    }

    /// The input as the script sees it: cleaned words, pre-substituted.
    #[must_use]
    pub fn prepare(&self, input: &str) -> Vec<String> {
        words(input)
            .into_iter()
            .flat_map(|w| {
                self.script.pre.get(&w).map_or_else(
                    || vec![w.clone()],
                    |s| s.split(' ').map(str::to_owned).collect(),
                )
            })
            .collect()
    }

    /// Respond the way the script alone would: highest-ranked keyword first,
    /// then memory, then the fallback keyword.
    pub fn respond(&mut self, input: &str) -> Answer {
        let ws = self.prepare(input);
        for ki in self.ranked_keys(&ws) {
            if let Some(a) = self.try_key(ki, &ws, 0) {
                return a;
            }
        }
        if let Some(text) = self.memory.pop_front() {
            return Answer {
                matched: true,
                ..Answer::fixed("memory", text)
            };
        }
        self.fallback(&ws)
    }

    /// Respond with a rule someone else chose. If its decomposition matches,
    /// its reassembly is used as the script would; if not, the rule's first
    /// reassembly that needs no captured part is used; failing both, the
    /// keyword's other decompositions, then the fallback keyword.
    pub fn respond_with(&mut self, rule: &str, input: &str) -> Answer {
        let ws = self.prepare(input);
        if rule == "memory" {
            if let Some(text) = self.memory.pop_front() {
                return Answer {
                    matched: true,
                    ..Answer::fixed(rule, text)
                };
            }
            return self.fallback(&ws);
        }
        let Some((ki, di)) = self.parse_rule(rule) else {
            return self.fallback(&ws);
        };
        if let Some(parts) = self.decompose(&self.script.keys[ki].decomps[di].pattern, &ws) {
            if let Some(a) = self.reassemble(ki, di, &parts, &ws, 0) {
                return a;
            }
        }
        let fixed = self.script.keys[ki].decomps[di]
            .replies
            .iter()
            .find(|r| !r.contains('(') && !r.starts_with("goto "))
            .cloned();
        if let Some(text) = fixed {
            return Answer::fixed(rule, capitalize(&text));
        }
        self.try_key(ki, &ws, 0)
            .unwrap_or_else(|| self.fallback(&ws))
    }

    fn parse_rule(&self, rule: &str) -> Option<(usize, usize)> {
        let (key, i) = rule.split_once('#')?;
        let ki = self.script.keys.iter().position(|k| k.key == key)?;
        let di: usize = i.parse().ok()?;
        (di < self.script.keys[ki].decomps.len()).then_some((ki, di))
    }

    /// Keywords present in the input, highest rank first, ties by position.
    fn ranked_keys(&self, ws: &[String]) -> Vec<usize> {
        let mut found: Vec<(i32, usize, usize)> = self
            .script
            .keys
            .iter()
            .enumerate()
            .filter(|(_, k)| k.key != NONE_KEY)
            .filter_map(|(i, k)| {
                ws.iter()
                    .position(|w| *w == k.key)
                    .map(|pos| (k.rank, pos, i))
            })
            .collect();
        found.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        found.into_iter().map(|(_, _, i)| i).collect()
    }

    fn key_index(&self, name: &str) -> Option<usize> {
        self.script.keys.iter().position(|k| k.key == name)
    }

    /// Try every decomposition of a keyword, following keyword-level `goto`.
    fn try_key(&mut self, ki: usize, ws: &[String], depth: usize) -> Option<Answer> {
        if depth > 8 {
            return None;
        }
        if let Some(target) = self.script.keys[ki].goto.clone() {
            let ti = self.key_index(&target)?;
            return self.try_key(ti, ws, depth + 1);
        }
        for di in 0..self.script.keys[ki].decomps.len() {
            let pattern = self.script.keys[ki].decomps[di].pattern.clone();
            let Some(parts) = self.decompose(&pattern, ws) else {
                continue;
            };
            if self.script.keys[ki].decomps[di].memory {
                let frame = self.next_reply(ki, di);
                let (frame, slots) = self.fill(&frame, &parts);
                self.memory.push_back(render(&frame, &slots));
                continue;
            }
            if let Some(a) = self.reassemble(ki, di, &parts, ws, depth) {
                return Some(a);
            }
        }
        None
    }

    fn reassemble(
        &mut self,
        ki: usize,
        di: usize,
        parts: &[Vec<String>],
        ws: &[String],
        depth: usize,
    ) -> Option<Answer> {
        let frame = self.next_reply(ki, di);
        if let Some(target) = frame.strip_prefix("goto ") {
            let ti = self.key_index(target.trim())?;
            return self.try_key(ti, ws, depth + 1);
        }
        let rule = format!("{}#{di}", self.script.keys[ki].key);
        let (frame, slots) = self.fill(&frame, parts);
        Some(Answer {
            rule,
            text: render(&frame, &slots),
            frame,
            slots,
            matched: true,
        })
    }

    fn fallback(&mut self, ws: &[String]) -> Answer {
        let ki = self
            .key_index(NONE_KEY)
            .expect("a script has a fallback keyword");
        self.try_key(ki, ws, 0)
            .unwrap_or_else(|| Answer::fixed(&format!("{NONE_KEY}#0"), String::new()))
    }

    fn next_reply(&mut self, ki: usize, di: usize) -> String {
        let replies = &self.script.keys[ki].decomps[di].replies;
        let c = self.counters.entry((ki, di)).or_insert(0);
        let r = replies[*c % replies.len()].clone();
        *c += 1;
        r
    }

    /// A reassembly as a frame with `{n}` slots, and the post-substituted
    /// parts that fill them.
    fn fill(&self, frame: &str, parts: &[Vec<String>]) -> (String, Vec<(String, String)>) {
        let mut out = frame.to_owned();
        let mut slots = Vec::new();
        for (i, part) in parts.iter().enumerate().rev() {
            let (paren, brace) = (format!("({})", i + 1), format!("{{{}}}", i + 1));
            if !out.contains(&paren) {
                continue;
            }
            out = out.replace(&paren, &brace);
            let text = part
                .iter()
                .map(|w| {
                    self.script
                        .post
                        .get(w)
                        .cloned()
                        .unwrap_or_else(|| w.clone())
                })
                .collect::<Vec<_>>()
                .join(" ");
            slots.insert(0, ((i + 1).to_string(), text));
        }
        (out, slots)
    }

    /// Match a pattern; the parts are the wildcard runs and synonym matches.
    fn decompose(&self, pattern: &str, ws: &[String]) -> Option<Vec<Vec<String>>> {
        let tokens: Vec<&str> = pattern.split_whitespace().collect();
        self.matches(&tokens, ws)
    }

    fn matches(&self, pattern: &[&str], ws: &[String]) -> Option<Vec<Vec<String>>> {
        let Some((first, rest)) = pattern.split_first() else {
            return ws.is_empty().then(Vec::new);
        };
        if *first == "*" {
            return (0..=ws.len()).find_map(|k| {
                self.matches(rest, &ws[k..]).map(|mut parts| {
                    parts.insert(0, ws[..k].to_vec());
                    parts
                })
            });
        }
        let (w, tail) = ws.split_first()?;
        if let Some(group) = first.strip_prefix('@') {
            let members = self.script.synonyms.get(group)?;
            if !members.contains(w) {
                return None;
            }
            return self.matches(rest, tail).map(|mut parts| {
                parts.insert(0, vec![w.clone()]);
                parts
            });
        }
        (w == first).then(|| self.matches(rest, tail)).flatten()
    }
}

/// Fill a frame's `{n}` slots and capitalize: the only way an answer's text is
/// made, so the text can always be rebuilt from its frame and slots.
#[must_use]
pub fn render(frame: &str, slots: &[(String, String)]) -> String {
    let mut out = frame.to_owned();
    for (name, value) in slots {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    // An empty capture leaves the gap it filled: "who else in your family ?".
    let tidy: String = out.split_whitespace().collect::<Vec<_>>().join(" ");
    let tidy = tidy
        .replace(" ?", "?")
        .replace(" .", ".")
        .replace(" ,", ",");
    capitalize(&tidy)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |f| {
        f.to_uppercase().collect::<String>() + c.as_str()
    })
}
