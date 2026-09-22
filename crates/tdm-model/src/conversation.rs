//! The deterministic half of a conversation: memory the program keeps, keyword
//! tracking, recall, and pattern decomposition with reassembly, all driven by a
//! data script. The model chooses a label; this module decides how to answer it
//! from the visitor's own words.
//!
//! Every reply is a frame from the script with slots filled by the visitor's
//! own words, reflected (my -> your). Nothing is generated: the frame is a fixed
//! literal and each slot is text the program already had, and
//! [`Reply::render`] rebuilds the text from exactly those parts.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::bundle::Bundle;
use crate::features::words;
use crate::responder::{Outcome, Turn, respond_at};

/// A conversation script: stop words, pronoun reflections, recall and memory
/// frames, keyword rules that apply to any label, and rules per label.
#[derive(Clone, Debug, Deserialize)]
pub struct Script {
    pub stopwords: Vec<String>,
    pub reflections: HashMap<String, String>,
    pub recall: Frames,
    pub memory: Memory,
    pub reflect: Frames,
    pub generic: Vec<Rule>,
    pub rules: HashMap<String, Vec<Rule>>,
    /// How Noul answers steer the reply, when the model has Noul heads.
    #[serde(default)]
    pub nouls: Option<NoulRules>,
}

/// Replies steered by Noul heads: deflect first, then sentiment.
#[derive(Clone, Debug, Deserialize)]
pub struct NoulRules {
    #[serde(default)]
    pub deflect: Option<NoulRule>,
    #[serde(default)]
    pub sentiment: Vec<NoulRule>,
}

/// A reply family that fires when the named Noul is at least `threshold`, or,
/// for a deflection, when the Choice acted on one of `labels`.
#[derive(Clone, Debug, Deserialize)]
pub struct NoulRule {
    pub noul: String,
    pub threshold: f64,
    #[serde(default)]
    pub labels: Vec<String>,
    pub frames: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Frames {
    /// Recall: how many turns back a memory must be. Reflect: fewest words.
    #[serde(default, alias = "min_words")]
    pub min_gap: usize,
    pub frames: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Memory {
    /// A statement containing this word is remembered for later.
    pub trigger: String,
    pub min_words: usize,
    pub frames: Vec<String>,
    /// Discourse fillers dropped from the start of a remembered line before
    /// it is quoted back ("well my mom ..." comes back as "your mom ...").
    #[serde(default)]
    pub lead_fillers: Vec<String>,
}

/// A decomposition rule: either a wildcard `pattern` over words, or a list of
/// words to `find`; and the reassemblies to cycle through when it matches.
#[derive(Clone, Debug, Deserialize)]
pub struct Rule {
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub find: Option<Vec<String>>,
    pub replies: Vec<String>,
}

/// Which mechanism produced a reply, for the trace.
#[derive(Clone, Debug, PartialEq)]
pub enum Move {
    /// The visitor reused a word from an earlier turn, which is recalled.
    Recall { turn: usize, keyword: String },
    /// Nothing else fit, so a remembered statement is brought back.
    Memory { turn: usize },
    /// A rule for the label the model chose matched.
    LabelRule { label: String, rule: String },
    /// A keyword rule that applies to any label matched.
    KeywordRule { rule: String },
    /// Nothing matched; the visitor's own words are reflected back.
    Reflect,
    /// The label's canned reply, unchanged.
    Canned,
    /// A question is deflected rather than answered, as the 1966 program did.
    Deflect { noul: String, p: f64 },
    /// Nothing fit the label, so the input is reflected with its sentiment.
    Sentiment { noul: String, p: f64 },
}

/// A reply as the parts it was built from.
#[derive(Clone, Debug)]
pub struct Reply {
    pub frame: String,
    pub slots: Vec<(String, String)>,
    pub text: String,
    pub by: Move,
}

impl Reply {
    /// Rebuild the text from the frame and slots. A reply whose text does not
    /// equal this was not built from the script, and a test checks every one.
    #[must_use]
    pub fn render(frame: &str, slots: &[(String, String)]) -> String {
        let mut out = frame.to_owned();
        for (name, value) in slots {
            out = out.replace(&format!("{{{name}}}"), value);
        }
        capitalize(&out)
    }
}

/// One remembered statement.
#[derive(Clone, Debug)]
pub struct Remembered {
    pub turn: usize,
    pub text: String,
    pub keywords: Vec<String>,
    pub mine: bool,
}

/// One full turn: the model's decision, and what the program did with it.
#[derive(Clone, Debug)]
pub struct Exchange {
    pub turn: Turn,
    pub reply: Reply,
    /// The keywords this input contributed to memory.
    pub keywords: Vec<String>,
}

/// A conversation in progress, at one snapshot of the model.
pub struct Conversation<'a> {
    bundle: &'a Bundle,
    script: &'a Script,
    snapshot: usize,
    memory: Vec<Remembered>,
    /// Turns already brought back by the repeat rule, and by the memory rule;
    /// kept apart so one mechanism does not starve the other.
    recalled: HashSet<usize>,
    /// Memory turn -> the turn at which the memory rule brought it back.
    remembered: HashMap<usize, usize>,
    counters: HashMap<String, usize>,
}

impl<'a> Conversation<'a> {
    #[must_use]
    pub fn new(bundle: &'a Bundle, script: &'a Script, snapshot: usize) -> Self {
        Self {
            bundle,
            script,
            snapshot,
            memory: Vec::new(),
            recalled: HashSet::new(),
            remembered: HashMap::new(),
            counters: HashMap::new(),
        }
    }

    /// What the program remembers so far.
    #[must_use]
    pub fn memory(&self) -> &[Remembered] {
        &self.memory
    }

    /// Take one turn: ask the model, then answer in the order Weizenbaum's 1966 program did --
    /// a reused word first, then rules for the chosen label, then keyword rules,
    /// then memory, then reflection, then the canned reply.
    pub fn say(&mut self, text: &str) -> Exchange {
        let index = self.memory.len();
        let turn = respond_at(self.bundle, self.snapshot, text, index);
        let ws = words(text);
        let keywords = self.keywords(&ws);
        let reply = self.choose(&turn, &ws, &keywords, index);
        let mine =
            ws.contains(&self.script.memory.trigger) && ws.len() >= self.script.memory.min_words;
        self.memory.push(Remembered {
            turn: index,
            text: text.to_owned(),
            keywords: keywords.clone(),
            mine,
        });
        Exchange {
            turn,
            reply,
            keywords,
        }
    }

    fn keywords(&self, ws: &[String]) -> Vec<String> {
        let stop: HashSet<&str> = self.script.stopwords.iter().map(String::as_str).collect();
        let mut seen = HashSet::new();
        ws.iter()
            .filter(|w| w.len() >= 3 && !stop.contains(w.as_str()) && seen.insert(w.as_str()))
            .cloned()
            .collect()
    }

    fn choose(&mut self, turn: &Turn, ws: &[String], keywords: &[String], index: usize) -> Reply {
        let acted = turn.reply.outcome == Outcome::Act;
        let label = self.bundle.labels[turn.reply.label].clone();
        if let Some(r) = self.deflect(turn, acted, &label) {
            return r;
        }
        if let Some(r) = self.recall(keywords, index) {
            return r;
        }
        if acted {
            if let Some(r) = self.apply_rules(&label, ws) {
                return r;
            }
        }
        if let Some(r) = self.apply_generic(ws) {
            return r;
        }
        if !acted {
            // The model's top guess was not confident enough to act on alone,
            // but if the input literally contains a word that guess's find rule
            // looks for, belief and evidence agree, and the program trusts them.
            let top = self.bundle.labels[turn.decision.selected].clone();
            if let Some(r) = self.apply_find_rules(&top, ws) {
                return r;
            }
            if ws.len() >= self.script.reflect.min_gap {
                if let Some(r) = self.sentiment(turn, ws) {
                    return r;
                }
                let frame = self.next("reflect", &self.script.reflect.frames.clone());
                return Self::build(
                    frame,
                    vec![("all".to_owned(), self.reflect(ws))],
                    Move::Reflect,
                );
            }
            if let Some(r) = self.memory_rule(index) {
                return r;
            }
        }
        Reply {
            frame: turn.reply.text.clone(),
            slots: Vec::new(),
            text: turn.reply.text.clone(),
            by: Move::Canned,
        }
    }

    /// The probability the model gave a named Noul, if it has that head.
    fn noul(&self, turn: &Turn, name: &str) -> Option<f64> {
        let i = self.bundle.noul_names().iter().position(|n| n == name)?;
        turn.decision.nouls.get(i).copied()
    }

    /// Questions are deflected, not answered: when the question Noul fires, or
    /// the Choice acted on a question label.
    fn deflect(&mut self, turn: &Turn, acted: bool, label: &str) -> Option<Reply> {
        let rule = self.script.nouls.as_ref()?.deflect.clone()?;
        let p = self.noul(turn, &rule.noul).unwrap_or(0.0);
        let by_label = acted && rule.labels.iter().any(|l| l == label);
        if p < rule.threshold && !by_label {
            return None;
        }
        // Never answer a question by echoing it: skip a frame that says what
        // the visitor just said.
        let mut frame = self.next("deflect", &rule.frames);
        if words(&frame) == words(&turn.input) {
            frame = self.next("deflect", &rule.frames);
        }
        Some(Self::build(
            frame,
            Vec::new(),
            Move::Deflect { noul: rule.noul, p },
        ))
    }

    /// Reflect the input through the first sentiment Noul that fires.
    fn sentiment(&mut self, turn: &Turn, ws: &[String]) -> Option<Reply> {
        let rules = self.script.nouls.as_ref()?.sentiment.clone();
        for rule in rules {
            let p = self.noul(turn, &rule.noul).unwrap_or(0.0);
            if p >= rule.threshold {
                let frame = self.next(&format!("sentiment:{}", rule.noul), &rule.frames);
                let by = Move::Sentiment { noul: rule.noul, p };
                return Some(Self::build(
                    frame,
                    vec![("all".to_owned(), self.reflect(ws))],
                    by,
                ));
            }
        }
        None
    }

    /// The visitor's repeat rule: a content word they used at least `min_gap`
    /// turns ago, in a statement not yet recalled, brings that statement back.
    fn recall(&mut self, keywords: &[String], index: usize) -> Option<Reply> {
        let gap = self.script.recall.min_gap.max(1);
        let hit = self.memory.iter().find(|m| {
            m.turn + gap <= index
                && !self.recalled.contains(&m.turn)
                && self
                    .remembered
                    .get(&m.turn)
                    .is_none_or(|&at| at + 3 <= index)
                && keywords.iter().any(|k| m.keywords.contains(k))
        })?;
        let (earlier, text) = (hit.turn, hit.text.clone());
        let keyword = keywords.iter().find(|k| hit.keywords.contains(k))?.clone();
        self.recalled.insert(earlier);
        let frame = self.next("recall", &self.script.recall.frames.clone());
        Some(Self::build(
            frame,
            vec![("all".to_owned(), self.reflect(&self.statement(&text)))],
            Move::Recall {
                turn: earlier,
                keyword,
            },
        ))
    }

    /// Weizenbaum's MEMORY rule: when nothing else fits, bring back the oldest
    /// remembered "my ..." statement not yet recalled.
    fn memory_rule(&mut self, index: usize) -> Option<Reply> {
        let hit = self
            .memory
            .iter()
            .find(|m| m.mine && m.turn < index && !self.remembered.contains_key(&m.turn))?;
        let (earlier, text) = (hit.turn, hit.text.clone());
        self.remembered.insert(earlier, index);
        let frame = self.next("memory", &self.script.memory.frames.clone());
        Some(Self::build(
            frame,
            vec![("all".to_owned(), self.reflect(&self.statement(&text)))],
            Move::Memory { turn: earlier },
        ))
    }

    fn apply_rules(&mut self, label: &str, ws: &[String]) -> Option<Reply> {
        let rules = self.script.rules.get(label)?.clone();
        for (i, rule) in rules.iter().enumerate() {
            if let Some(slots) = self.matches(rule, ws) {
                let key = format!("{label}#{i}");
                let frame = self.next(&key, &rule.replies);
                let name = rule.pattern.clone().unwrap_or_else(|| "find".to_owned());
                return Some(Self::build(
                    frame,
                    slots,
                    Move::LabelRule {
                        label: label.to_owned(),
                        rule: name,
                    },
                ));
            }
        }
        None
    }

    fn apply_find_rules(&mut self, label: &str, ws: &[String]) -> Option<Reply> {
        let rules = self.script.rules.get(label)?.clone();
        for (i, rule) in rules.iter().enumerate().filter(|(_, r)| r.find.is_some()) {
            if let Some(slots) = self.matches(rule, ws) {
                let frame = self.next(&format!("{label}#{i}"), &rule.replies);
                let by = Move::LabelRule {
                    label: label.to_owned(),
                    rule: "find, agreeing with the model's top guess".to_owned(),
                };
                return Some(Self::build(frame, slots, by));
            }
        }
        None
    }

    fn apply_generic(&mut self, ws: &[String]) -> Option<Reply> {
        let rules = self.script.generic.clone();
        for (i, rule) in rules.iter().enumerate() {
            if let Some(slots) = self.matches(rule, ws) {
                let frame = self.next(&format!("generic#{i}"), &rule.replies);
                let name = rule.pattern.clone().unwrap_or_default();
                return Some(Self::build(frame, slots, Move::KeywordRule { rule: name }));
            }
        }
        None
    }

    /// The slots a rule binds, if it matches.
    fn matches(&self, rule: &Rule, ws: &[String]) -> Option<Vec<(String, String)>> {
        if let Some(find) = &rule.find {
            let w = ws.iter().find(|w| find.contains(w))?;
            return Some(vec![("w".to_owned(), w.clone())]);
        }
        let pattern: Vec<&str> = rule.pattern.as_deref()?.split_whitespace().collect();
        let caps = decompose(&pattern, ws)?;
        let slots = caps
            .iter()
            .enumerate()
            .map(|(i, c)| ((i + 1).to_string(), self.reflect(c)))
            .collect::<Vec<_>>();
        // A wildcard that must carry meaning into the reply may not be empty.
        let used_empty = rule.replies.iter().any(|r| {
            slots
                .iter()
                .any(|(n, v)| v.is_empty() && r.contains(&format!("{{{n}}}")))
        });
        (!used_empty).then_some(slots)
    }

    /// The part of a remembered line worth quoting back: the whole line, less
    /// any leading discourse fillers the script names, so "well my mom never
    /// listens" comes back as "your mom never listens" and "i lost my job" as
    /// "you lost your job" rather than being cut at "my".
    fn statement(&self, text: &str) -> Vec<String> {
        let ws = words(text);
        let skip = ws
            .iter()
            .take_while(|w| self.script.memory.lead_fillers.contains(w))
            .count();
        ws[skip..].to_vec()
    }

    /// Reflect a run of words: my -> your, i -> you, and a final "you" becomes
    /// "me" rather than "I", since at the end of a phrase it is an object.
    fn reflect(&self, ws: &[String]) -> String {
        let n = ws.len();
        ws.iter()
            .enumerate()
            .map(|(i, w)| {
                if w == "you" && i + 1 == n {
                    "me".to_owned()
                } else {
                    self.script
                        .reflections
                        .get(w)
                        .cloned()
                        .unwrap_or_else(|| w.clone())
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Cycle through a rule's reassemblies, as the 1966 program did, so a repeated input
    /// does not get a repeated answer.
    fn next(&mut self, key: &str, frames: &[String]) -> String {
        let c = self.counters.entry(key.to_owned()).or_insert(0);
        let frame = frames[*c % frames.len()].clone();
        *c += 1;
        frame
    }

    fn build(frame: String, slots: Vec<(String, String)>, by: Move) -> Reply {
        let text = Reply::render(&frame, &slots);
        Reply {
            frame,
            slots,
            text,
            by,
        }
    }
}

/// Match a wildcard pattern (`*` is any run of words, including none) against
/// words, returning what each `*` captured.
fn decompose(pattern: &[&str], ws: &[String]) -> Option<Vec<Vec<String>>> {
    match pattern.split_first() {
        None => ws.is_empty().then(Vec::new),
        Some((&"*", rest)) => (0..=ws.len()).find_map(|k| {
            decompose(rest, &ws[k..]).map(|mut caps| {
                caps.insert(0, ws[..k].to_vec());
                caps
            })
        }),
        Some((lit, rest)) => {
            let (first, tail) = ws.split_first()?;
            (first == lit).then(|| decompose(rest, tail)).flatten()
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |f| {
        f.to_uppercase().collect::<String>() + c.as_str()
    })
}
