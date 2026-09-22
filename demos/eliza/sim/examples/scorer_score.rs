//! What the card scorer buys, and what it costs. Scores the PR05 scorer and
//! model 4's fixed head on the same held-out split, then asks the question a
//! fixed head cannot be asked at all: how well is a candidate chosen when its
//! card was never offered during training?
//!
//! Three fields are reported, because "accuracy" alone would hide the result:
//!
//! - **full**: every card of the question, 51 of them.
//! - **five, warm**: the answer and four cards that did train.
//! - **five, cold**: the answer and four cards that were also held out, so
//!   nothing in the field has a training advantage over anything else.
//!
//! usage: `cargo run -p eliza-sim --example scorer_score`

use std::collections::BTreeSet;

use eliza_sim::{class_of, read_corpus, rule_classes};
use tdm_model::{Bundle, Model, Scorer, ScorerBundle, hash};

const SCORER: &str = "fixtures/bundles/demo01-scorer.json";
const MODEL4: &str = "fixtures/bundles/demo01-model4.json";
const CORPUS: &str = "demos/eliza/oracle/corpus.tsv";

/// The rule question.
const RULES: usize = 0;
const FIELD: usize = 5;

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[expect(clippy::cast_precision_loss, reason = "counts are small")]
fn pct(a: usize, b: usize) -> f64 {
    100.0 * a as f64 / b.max(1) as f64
}

/// A deterministic field of `FIELD` cards: the answer, then others drawn from
/// `pool` by a stable walk keyed on `draw`.
fn field(answer: usize, pool: &[usize], draw: &str) -> Vec<usize> {
    let mut chosen = vec![answer];
    let mut h = hash(draw);
    while chosen.len() < FIELD.min(pool.len().max(1)) {
        h = h.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let pick = pool[usize::try_from(h >> 33).unwrap_or(0) % pool.len()];
        if !chosen.contains(&pick) {
            chosen.push(pick);
        }
        if pool.iter().all(|p| chosen.contains(p)) {
            break;
        }
    }
    chosen
}

fn picks(s: &Scorer, b: &ScorerBundle, text: &str, cards: &[usize]) -> usize {
    let texts: Vec<&str> = cards.iter().map(|&c| b.cards.texts[c].as_str()).collect();
    cards[s.rank(text, RULES, &texts).selected]
}

struct Tally {
    n: usize,
    full: usize,
    warm: usize,
    cold: usize,
}

fn main() {
    let sb = ScorerBundle::parse(&read(SCORER)).expect("scorer bundle");
    let m4 = Bundle::parse(&read(MODEL4)).expect("model 4");
    let s = Scorer::new(&sb);
    let model = Model::at(&m4, m4.default_snapshot);
    let corpus = read_corpus(CORPUS);
    let (_, merged) = rule_classes(corpus.iter().filter(|c| !c.val).map(|c| c.rule.as_str()));

    let card_of = |rule: &str| sb.cards.names.iter().position(|n| n == rule);
    let held: BTreeSet<usize> = sb.held_out.iter().filter_map(|r| card_of(r)).collect();
    let rule_cards: Vec<usize> = (0..sb.rule_count).collect();
    let warm_pool: Vec<usize> = rule_cards
        .iter()
        .copied()
        .filter(|c| !held.contains(c))
        .collect();
    let cold_pool: Vec<usize> = held.iter().copied().collect();

    let mut seen = Tally {
        n: 0,
        full: 0,
        warm: 0,
        cold: 0,
    };
    let mut unseen = Tally {
        n: 0,
        full: 0,
        warm: 0,
        cold: 0,
    };
    // Model 4 answers only the rows whose rule it has a column for; the
    // comparison is over exactly those.
    let (mut m4_n, mut m4_right, mut sc_right_same) = (0, 0, 0);

    for (i, c) in corpus.iter().enumerate() {
        let Some(answer) = card_of(&c.rule) else {
            continue;
        };
        let is_held = held.contains(&answer);
        // A held-out rule's rows never trained, so all of them are evaluation
        // rows; for the rest, only the validation split is.
        if !is_held && !c.val {
            continue;
        }
        let draw = format!("{i}:{}", c.text);
        let t = if is_held { &mut unseen } else { &mut seen };
        t.n += 1;
        t.full += usize::from(picks(&s, &sb, &c.text, &rule_cards) == answer);
        let mut warm = field(answer, &warm_pool, &draw);
        warm.sort_unstable();
        t.warm += usize::from(picks(&s, &sb, &c.text, &warm) == answer);
        let mut cold = field(answer, &cold_pool, &draw);
        cold.sort_unstable();
        t.cold += usize::from(picks(&s, &sb, &c.text, &cold) == answer);

        if !is_held && c.val {
            let class = class_of(&merged, &c.rule);
            if m4.labels.iter().any(|l| l == class) {
                m4_n += 1;
                m4_right += usize::from(m4.labels[model.decide(&c.text).selected] == class);
                // The scorer is credited only when it picks the very rule,
                // which is the harder target: model 4's classes are merged.
                sc_right_same += usize::from(picks(&s, &sb, &c.text, &rule_cards) == answer);
            }
        }
    }

    report(&sb, &seen, &unseen, (m4_n, m4_right, sc_right_same), &m4);
}

/// Everything the run has to say, as markdown.
fn report(
    sb: &ScorerBundle,
    seen: &Tally,
    unseen: &Tally,
    against: (usize, usize, usize),
    m4: &Bundle,
) {
    let (m4_n, m4_right, sc_right_same) = against;
    let row = |name: &str, t: &Tally| {
        println!(
            "| {name} | {} | {:.1}% | {:.1}% | {:.1}% |",
            t.n,
            pct(t.full, t.n),
            pct(t.warm, t.n),
            pct(t.cold, t.n)
        );
    };
    println!(
        "# The card scorer, snapshot {} ({})\n",
        sb.default_snapshot,
        sb.snapshot_label(sb.default_snapshot)
    );
    println!("Cards held out of training: {}\n", sb.held_out.join(", "));
    println!("| Rows | n | full field (51) | five, warm | five, cold |");
    println!("|---|---:|---:|---:|---:|");
    row("cards that trained (validation split)", seen);
    row("cards held out of training", unseen);
    println!("\nChance in a five-card field is 20.0%.\n");

    println!("## Generality against the fixed head\n");
    println!(
        "- on the {m4_n} validation rows model 4 has a column for: model 4 {:.1}%, scorer {:.1}%",
        pct(m4_right, m4_n),
        pct(sc_right_same, m4_n)
    );
    println!(
        "- on the {} rows answered by a held-out rule: model 4 has no column at all, so the question cannot be put to it; the scorer picks the right card {:.1}% of the time in the full field",
        unseen.n,
        pct(unseen.full, unseen.n)
    );
    let w = sb.weights(sb.default_snapshot);
    let scorer_params = w.embedding.len() + w.state.len() + w.question.len() + w.card.len() + 1;
    let m4_params = m4.snapshots.embedding[0].len()
        + m4.snapshots.head[0].len()
        + m4.snapshots.bias[0].len()
        + m4.nouls
            .as_ref()
            .map_or(0, |n| n.head[0].len() + n.bias[0].len());
    println!(
        "- parameters: scorer {scorer_params}, model 4 {m4_params}. A new candidate costs the scorer nothing and model 4 a new column ({} weights) and a retrain.",
        m4.dim + 1
    );
    println!(
        "- the scorer answers {} questions with one set of weights; its cards for the second question are Nouls, not rules.",
        sb.questions.len()
    );
}
