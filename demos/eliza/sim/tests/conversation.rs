//! Demo 01's conversation mechanics, pinned on a scripted session: the ELIZA
//! behaviours the user asked for must keep happening.

use tdm_model::{Bundle, Conversation, Move, Script, ScriptReply};

const BUNDLE: &str = include_str!("../../../../fixtures/bundles/demo01-model2.json");
const SCRIPT: &str = include_str!("../../../../fixtures/bundles/demo01-script.json");
const SESSION: &[&str] = &[
    "I don't have any problems",
    "well my mom never listens to me",
    "I don't know",
    "my boss yelled at me today",
    "I want a better job",
    "skill issue",
    "my mom called again last night",
    "everyone hates me",
    "you are just a computer",
    "I am so tired",
    "asdf",
    "my job is killing me",
    "thank you",
];

fn session() -> Vec<(String, String, Move)> {
    let b = Bundle::parse(BUNDLE).expect("bundle");
    let s: Script = serde_json::from_str(SCRIPT).expect("script");
    let mut c = Conversation::new(&b, &s, b.default_snapshot);
    SESSION
        .iter()
        .map(|line| {
            let x = c.say(line);
            assert_eq!(
                ScriptReply::render(&x.reply.frame, &x.reply.slots),
                x.reply.text,
                "reply to {line:?} was not built from the script"
            );
            ((*line).to_owned(), x.reply.text, x.reply.by)
        })
        .collect()
}

#[test]
fn it_answers_in_the_visitors_own_words_the_way_eliza_did() {
    let t = session();
    assert_eq!(t[0].1, "Don't you really have any problems?");
    assert_eq!(t[1].1, "Tell me more about your mom.");
    assert_eq!(t[4].1, "What would it mean to you if you got a better job?");
    assert_eq!(t[7].1, "Really, everyone?");
    assert_eq!(t[9].1, "How long have you been so tired?");
}

#[test]
fn it_says_earlier_you_said_when_a_word_comes_back_and_when_nothing_fits() {
    let t = session();
    let recalls = t
        .iter()
        .filter(|x| matches!(x.2, Move::Recall { .. }))
        .count();
    let memories = t
        .iter()
        .filter(|x| matches!(x.2, Move::Memory { .. }))
        .count();
    assert!(recalls >= 1, "the repeat rule fired {recalls} times");
    assert!(memories >= 2, "the memory rule fired {memories} times");
    assert!(
        t[11].1.contains("you want a better job"),
        "\"job\" brings back the earlier job statement: {}",
        t[11].1
    );
    assert!(t.iter().any(|x| x.1.starts_with("Earlier you said")));
}

#[test]
fn it_never_says_the_same_thing_twice_in_a_row() {
    let t = session();
    for pair in t.windows(2) {
        assert_ne!(pair[0].1, pair[1].1, "repeated reply after {:?}", pair[1].0);
    }
}
