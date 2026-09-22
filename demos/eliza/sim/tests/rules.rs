//! Model 4: the Choice is over the script's own rules, and the oracle that
//! labelled the corpus is the judge. Two things are pinned here -- that the
//! model still agrees with the oracle about as often as it did when it was
//! measured, and that a session reads the way the 1966 program did.

use eliza_sim::{class_of, parse_corpus, rule_classes};
use tdm_model::{
    Bundle, Conversation, KeywordEngine, KeywordScript, Model, Move, Script, ScriptReply,
};

const BUNDLE: &str = include_str!("../../../../fixtures/bundles/demo01-model4.json");
const SCRIPT: &str = include_str!("../../../../fixtures/bundles/demo01-script.json");
const DOCTOR: &str = include_str!("../../../../fixtures/bundles/demo01-doctor.json");
const CORPUS: &str = include_str!("../../oracle/corpus.tsv");
const PROBES: &str = include_str!("../../probe-labels.tsv");

fn parts() -> (Bundle, KeywordScript) {
    (
        Bundle::parse(BUNDLE).expect("model 4"),
        serde_json::from_str(DOCTOR).expect("keyword script"),
    )
}

/// How often the model picks the rule the oracle picked, on the held-out split
/// and on the 96 frozen probes.
fn agreement() -> (f64, f64) {
    let (b, script) = parts();
    let model = Model::at(&b, b.default_snapshot);
    let corpus = parse_corpus(CORPUS);
    let (_, merged) = rule_classes(corpus.iter().filter(|c| !c.val).map(|c| c.rule.as_str()));
    let agree = |rows: Vec<(String, String)>| {
        let right = rows
            .iter()
            .filter(|(text, truth)| &b.labels[model.decide(text).selected] == truth)
            .count();
        #[expect(clippy::cast_precision_loss, reason = "hundreds of rows")]
        let share = right as f64 / rows.len().max(1) as f64;
        share
    };
    let val = agree(
        corpus
            .iter()
            .filter(|c| c.val)
            .map(|c| (c.text.clone(), class_of(&merged, &c.rule).to_owned()))
            .collect(),
    );
    let probes = agree(
        PROBES
            .lines()
            .filter_map(|l| l.split('\t').nth(1))
            .map(|t| {
                let rule = KeywordEngine::new(&script).respond(t).rule;
                (t.to_owned(), class_of(&merged, &rule).to_owned())
            })
            .collect(),
    );
    (val, probes)
}

#[test]
fn it_still_agrees_with_the_oracle_about_as_often_as_when_it_was_measured() {
    let (val, probes) = agreement();
    assert!(
        val >= 0.84,
        "validation agreement {val:.3} (measured 0.849, majority class 0.301)"
    );
    assert!(
        probes >= 0.92,
        "probe agreement {probes:.3} (measured 0.938, majority class 0.219)"
    );
}

#[test]
fn every_reply_is_the_scripts_own_frame_filled_with_the_visitors_words() {
    let (b, doctor) = parts();
    let s: Script = serde_json::from_str(SCRIPT).expect("script");
    let mut c = Conversation::with_keywords(&b, &s, &doctor, b.default_snapshot);
    let session = [
        "hello",
        "my mom never listens to me",
        "I don't know",
        "i lost my job last week",
        "are you a computer?",
        "i am unhappy",
        "because we did",
        "nobody cares about me",
        "skill issue",
        "what do you mean",
    ];
    let out: Vec<(String, Move)> = session
        .iter()
        .map(|line| {
            let x = c.say(line);
            assert_eq!(
                ScriptReply::render(&x.reply.frame, &x.reply.slots),
                x.reply.text,
                "the reply to {line:?} was not built from a frame and slots"
            );
            (x.reply.text, x.reply.by)
        })
        .collect();
    assert_eq!(out[0].0, "How do you do. Please state your problem.");
    assert_eq!(out[2].0, "Don't you really know?");
    assert_eq!(
        out[5].0,
        "Is it because you are unhappy that you came to me?"
    );
    assert_eq!(out[6].0, "Is that the real reason?", "not a question");
    assert_eq!(out[7].0, "Really, nobody?");
    for i in [4, 9] {
        assert!(
            matches!(out[i].1, Move::Deflect { .. }),
            "{:?} was answered, not turned back: {:?}",
            session[i],
            out[i].0
        );
    }
    assert!(
        matches!(out[8].1, Move::Memory { .. }),
        "an input with nothing in it brings back a memory: {:?}",
        out[8]
    );
}

#[test]
fn a_rule_that_does_not_fit_the_words_still_answers_without_inventing_any() {
    let (_, doctor) = parts();
    // "my#2" reassembles "your (2)", which needs words this input has not got.
    let a = KeywordEngine::new(&doctor).respond_with("my#2", "yes");
    assert!(!a.matched, "the pattern does not fit");
    assert!(!a.text.is_empty());
    assert_eq!(a.text, tdm_model::render_frame(&a.frame, &a.slots));
}
