//! Build the (state, question, choices, answer) tuples demo 01 supplies to the
//! PR05 scorer, and featurize them. The choices are **cards**: text describing
//! a candidate, not a column of a fixed head.
//!
//! Two questions share one scorer and one encoder:
//!
//! - *which rule should answer this?* — the cards are the DOCTOR script's
//!   rules, written out as the keyword, its decomposition pattern and its
//!   reassemblies. The answer is the rule the oracle fired, **unmerged**: a
//!   rule too rare for model 4 to carry a column for still has a card here.
//! - *what is true of this input?* — the cards are the three Nouls and "none
//!   of these". Only the generated sentences carry all three labels, so only
//!   they supply these tuples.
//!
//! A set of rules is **held out entirely**: their cards are never offered
//! during training and every row answered by one is removed from training. At
//! evaluation their cards are offered like any other. That is the measurement
//! the lesson exists for, and a fixed head cannot even be asked the question.
//!
//! usage: `cargo run -p eliza-sim --bin cards -- CORPUS FRAME_NOULS SCRIPT OUT`

use std::collections::{BTreeSet, HashMap};

use eliza_sim::read_corpus;
use serde_json::json;
use tdm_model::{KeywordScript, hash, tokens, words};

const WIDTH: usize = 24;
/// Cards are longer than inputs: a pattern plus its reassemblies.
const CARD_WIDTH: usize = 48;
const MAX_VOCAB: usize = 2400;
/// How many cards are in the small field: the answer and four others.
const SMALL: usize = 5;
const QUESTION_TOKEN: &str = "qmark";

/// The rules whose cards are hidden during training. Chosen before any
/// training run: two common rules, two middling ones, and every rule model 4
/// had to merge away for want of examples.
const HELD_OUT: [&str; 12] = [
    "i#6",
    "you#1",
    "alike#0",
    "dreamed#0",
    "remember#0",
    "remember#1",
    "why#0",
    "why#1",
    "am#0",
    "am#1",
    "was#0",
    "can#1",
];

const QUESTIONS: [&str; 2] = [
    "which rule should answer this",
    "what is true of this input",
];

const NOUL_CARDS: [(&str, &str); 4] = [
    ("noul:question", "the visitor is asking a question"),
    ("noul:negative", "the visitor sounds negative or unhappy"),
    ("noul:positive", "the visitor sounds positive or grateful"),
    ("noul:none", "none of these is true of this input"),
];

fn prep(text: &str) -> String {
    text.replace('?', &format!(" {QUESTION_TOKEN} "))
}

fn norm(s: &str) -> String {
    words(s).join(" ")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A rule as text: its keyword, its decomposition pattern and its
/// reassemblies, with the script's own markup removed.
fn card_text(key: &str, pattern: &str, replies: &[String]) -> String {
    let clean = |s: &str| {
        s.split_whitespace()
            .filter(|w| *w != "*" && !w.starts_with('@') && !w.starts_with('('))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let body: Vec<String> = replies
        .iter()
        .filter(|r| !r.starts_with("goto "))
        .take(3)
        .map(|r| clean(r))
        .collect();
    format!("{key} {} {}", clean(pattern), body.join(" "))
}

struct Card {
    name: String,
    text: String,
}

fn rule_cards(script: &KeywordScript) -> Vec<Card> {
    script
        .keys
        .iter()
        .filter(|k| k.goto.is_none())
        .flat_map(|k| {
            k.decomps
                .iter()
                .enumerate()
                .filter(|(_, d)| !d.memory)
                .map(move |(i, d)| Card {
                    name: format!("{}#{i}", k.key),
                    text: card_text(&k.key, &d.pattern, &d.replies),
                })
        })
        .collect()
}

/// Ids and known-masks for one text, padded to `width`.
fn row(text: &str, width: usize, index: &HashMap<&str, usize>) -> (Vec<usize>, Vec<u8>) {
    let toks = tokens(&prep(text), width);
    (0..width)
        .map(|i| {
            let r = toks.get(i).and_then(|t| index.get(t.as_str())).copied();
            (r.unwrap_or(0), u8::from(r.is_some()))
        })
        .unzip()
}

fn matrices(texts: &[&str], width: usize, index: &HashMap<&str, usize>) -> (Vec<usize>, Vec<u8>) {
    let (mut ids, mut mask) = (Vec::new(), Vec::new());
    for t in texts {
        let (i, m) = row(t, width, index);
        ids.extend(i);
        mask.extend(m);
    }
    (ids, mask)
}

/// Every word the script names, so a card's own keyword is never an unknown.
fn script_words(script: &KeywordScript) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for k in &script.keys {
        out.insert(k.key.clone());
        for d in &k.decomps {
            out.extend(
                d.pattern
                    .split_whitespace()
                    .filter(|w| *w != "*" && !w.starts_with('@'))
                    .map(str::to_owned),
            );
        }
    }
    out.extend(script.synonyms.values().flatten().cloned());
    out
}

/// One tuple: a state, which question was asked, and the card that answers.
struct Tuple {
    text: String,
    question: usize,
    answer: usize,
    val: bool,
    held_out: bool,
}

/// The vocabulary covers inputs, cards and questions: a card is read by the
/// same encoder as a state, so its words have to be in it, and so do the
/// script's own, before frequency fills the rest.
fn vocabulary(train: &[&Tuple], cards: &[Card], script: &KeywordScript) -> Vec<String> {
    let mut count: HashMap<String, usize> = HashMap::new();
    for t in train {
        for tok in tokens(&prep(&t.text), WIDTH) {
            *count.entry(tok).or_default() += 1;
        }
    }
    let mut named: BTreeSet<String> = script_words(script);
    for c in cards {
        named.extend(tokens(&prep(&c.text), CARD_WIDTH));
    }
    for q in QUESTIONS {
        named.extend(tokens(q, WIDTH));
    }
    let named: Vec<String> = named.into_iter().collect();
    let mut rest: Vec<(String, usize)> = count
        .into_iter()
        .filter(|(t, n)| *n >= 2 && !named.contains(t))
        .collect();
    rest.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rest.truncate(MAX_VOCAB.saturating_sub(named.len()));
    named
        .into_iter()
        .chain(rest.into_iter().map(|(t, _)| t))
        .collect()
}

/// Every tuple the corpus supplies: one per row for the rule question, and one
/// more for the Noul question wherever all three Nouls are labelled.
fn tuples(
    corpus_path: &str,
    tagged: &HashMap<String, [bool; 3]>,
    index_of: &HashMap<&str, usize>,
    rule_count: usize,
) -> Vec<Tuple> {
    let mut tuples: Vec<Tuple> = Vec::new();
    for c in &read_corpus(corpus_path) {
        let Some(&answer) = index_of.get(c.rule.as_str()) else {
            continue;
        };
        tuples.push(Tuple {
            text: c.text.clone(),
            question: 0,
            answer,
            val: c.val,
            held_out: HELD_OUT.contains(&c.rule.as_str()),
        });
        // The second question, only where all three Nouls are labelled.
        if c.source == "frames" {
            if let Some(n) = tagged.get(&norm(&c.text)) {
                let which = n.iter().position(|b| *b).unwrap_or(3);
                tuples.push(Tuple {
                    text: c.text.clone(),
                    question: 1,
                    answer: rule_count + which,
                    val: c.val,
                    held_out: false,
                });
            }
        }
    }
    tuples
}

/// The small field for one row: the answer plus `SMALL - 1` other cards from
/// the same block, drawn by a stable pseudo-random walk so the field is the
/// same on every run.
fn small_field(t: &Tuple, i: usize, cards: &[Card], rule_count: usize) -> Vec<u8> {
    let block: Vec<usize> = (0..cards.len())
        .filter(|&c| (c < rule_count) == (t.question == 0))
        .collect();
    let mut chosen = vec![t.answer];
    let mut h = hash(&format!("{i}:{}", t.text));
    while chosen.len() < SMALL.min(block.len()) {
        h = h.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let pick = block[usize::try_from(h >> 33).unwrap_or(0) % block.len()];
        if !chosen.contains(&pick) {
            chosen.push(pick);
        }
    }
    (0..cards.len())
        .map(|c| u8::from(chosen.contains(&c)))
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [corpus_path, frame_nouls, script_path, out] = &args[..] else {
        panic!("usage: cards CORPUS FRAME_NOULS SCRIPT OUT");
    };
    let script: KeywordScript = serde_json::from_str(&read(script_path)).expect("parse script");
    let mut cards = rule_cards(&script);
    let rule_count = cards.len();
    cards.extend(NOUL_CARDS.map(|(name, text)| Card {
        name: name.to_owned(),
        text: text.to_owned(),
    }));
    let index_of: HashMap<&str, usize> = cards
        .iter()
        .enumerate()
        .map(|(i, c)| (c.name.as_str(), i))
        .collect();

    let tagged: HashMap<String, [bool; 3]> = read(frame_nouls)
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f.len() == 4).then(|| (norm(f[0]), [f[1] == "1", f[2] == "1", f[3] == "1"]))
        })
        .collect();

    let tuples = tuples(corpus_path, &tagged, &index_of, rule_count);

    let train: Vec<&Tuple> = tuples.iter().filter(|t| !t.val && !t.held_out).collect();
    let vocab = vocabulary(&train, &cards, &script);
    let index: HashMap<&str, usize> = vocab
        .iter()
        .enumerate()
        .map(|(i, t)| (t.as_str(), i + 1))
        .collect();

    // Candidate masks: a row is offered its question's cards. During training
    // the held-out rules are not among them.
    //
    // A second, small field is also emitted: the answer plus SMALL - 1 other
    // cards, drawn deterministically from the question's block. A 51-way field
    // asks whether an unseen card can beat every trained one at once; a
    // reranker is asked something narrower, and both numbers are worth having.
    let offered = |question: usize, training: bool| -> Vec<u8> {
        (0..cards.len())
            .map(|i| {
                let right_block = if question == 0 {
                    i < rule_count
                } else {
                    i >= rule_count
                };
                let hidden = training && HELD_OUT.contains(&cards[i].name.as_str());
                u8::from(right_block && !hidden)
            })
            .collect()
    };

    let small = |t: &Tuple, i: usize| small_field(t, i, &cards, rule_count);

    let emit = |rows: Vec<&Tuple>, training: bool| {
        let texts: Vec<&str> = rows.iter().map(|t| t.text.as_str()).collect();
        let (ids, mask) = matrices(&texts, WIDTH, &index);
        json!({
            "n": rows.len(),
            "ids": ids,
            "mask": mask,
            "question": rows.iter().map(|t| t.question).collect::<Vec<_>>(),
            "y": rows.iter().map(|t| t.answer).collect::<Vec<_>>(),
            "cand": rows.iter().flat_map(|t| offered(t.question, training)).collect::<Vec<_>>(),
            "cand_small": rows.iter().enumerate().flat_map(|(i, t)| small(t, i)).collect::<Vec<_>>(),
            "texts": texts,
        })
    };

    let val: Vec<&Tuple> = tuples.iter().filter(|t| t.val && !t.held_out).collect();
    let unseen: Vec<&Tuple> = tuples.iter().filter(|t| t.held_out).collect();
    let card_texts: Vec<&str> = cards.iter().map(|c| c.text.as_str()).collect();
    let (cids, cmask) = matrices(&card_texts, CARD_WIDTH, &index);
    let (qids, qmask) = matrices(&QUESTIONS, WIDTH, &index);
    let data = json!({
        "vocab": vocab,
        "width": WIDTH,
        "card_width": CARD_WIDTH,
        "rule_count": rule_count,
        "held_out": HELD_OUT,
        "cards": {
            "names": cards.iter().map(|c| c.name.clone()).collect::<Vec<_>>(),
            "texts": card_texts,
            "ids": cids,
            "mask": cmask,
        },
        "questions": {"texts": QUESTIONS, "ids": qids, "mask": qmask},
        "train": emit(train.clone(), true),
        "val": emit(val.clone(), false),
        "unseen": emit(unseen.clone(), false),
    });
    std::fs::write(out, data.to_string()).expect("write tuples");
    eprintln!(
        "{} cards ({rule_count} rules + {} Nouls), vocab {}: {} train, {} val, {} answered by a held-out rule",
        cards.len(),
        NOUL_CARDS.len(),
        vocab.len(),
        train.len(),
        val.len(),
        unseen.len()
    );
}
