//! The conversation engine, exercised with a small synthetic script so the
//! crate's tests stay free of any demo's vocabulary. The model underneath is a
//! real bundle; only the script is invented here.

use tdm_model::{Bundle, Conversation, Move, Script, ScriptReply};

const BUNDLE: &str = include_str!("../../../fixtures/bundles/demo01-model2.json");

fn script() -> Script {
    serde_json::from_str(
        r#"{
        "stopwords": ["i", "my", "the", "a", "to", "is", "want"],
        "reflections": {"i": "you", "my": "your", "me": "you"},
        "recall": {"min_gap": 2, "frames": ["Earlier: {all}."]},
        "memory": {"trigger": "my", "min_words": 3, "frames": ["Back to {all}."]},
        "reflect": {"min_words": 3, "frames": ["Say more about {all}."]},
        "generic": [{"pattern": "* i want *", "replies": ["Why {2}?", "Truly {2}?"]}],
        "rules": {}
    }"#,
    )
    .expect("synthetic script")
}

#[test]
fn a_wildcard_rule_reflects_what_it_captured_and_cycles_its_replies() {
    let (b, s) = (Bundle::parse(BUNDLE).expect("bundle"), script());
    let mut c = Conversation::new(&b, &s, b.default_snapshot);
    let first = c.say("i want my lantern back");
    assert_eq!(first.reply.text, "Why your lantern back?");
    assert_eq!(
        first.reply.by,
        Move::KeywordRule {
            rule: "* i want *".to_owned()
        }
    );
    let second = c.say("i want a kettle");
    assert_eq!(
        second.reply.text, "Truly a kettle?",
        "reassemblies cycle, as in 1966"
    );
}

#[test]
fn a_reused_content_word_recalls_the_turn_that_first_used_it() {
    let (b, s) = (Bundle::parse(BUNDLE).expect("bundle"), script());
    let mut c = Conversation::new(&b, &s, b.default_snapshot);
    c.say("my lantern broke yesterday");
    // A turn a keyword rule answers, so the memory rule does not use turn 0 up
    // first: a statement it has just brought back is held for three turns.
    c.say("i want a kettle");
    let back = c.say("the lantern again");
    assert_eq!(
        back.reply.by,
        Move::Recall {
            turn: 0,
            keyword: "lantern".to_owned()
        }
    );
    assert_eq!(back.reply.text, "Earlier: your lantern broke yesterday.");
    let again = c.say("still the lantern");
    assert!(
        !matches!(again.reply.by, Move::Recall { turn: 0, .. }),
        "each statement is recalled once"
    );
}

#[test]
fn every_reply_is_exactly_its_frame_with_its_slots_filled() {
    let (b, s) = (Bundle::parse(BUNDLE).expect("bundle"), script());
    let mut c = Conversation::new(&b, &s, b.default_snapshot);
    for line in b.parity.inputs.iter().take(120) {
        let x = c.say(line);
        assert_eq!(
            ScriptReply::render(&x.reply.frame, &x.reply.slots),
            x.reply.text,
            "on {line:?}"
        );
    }
}
