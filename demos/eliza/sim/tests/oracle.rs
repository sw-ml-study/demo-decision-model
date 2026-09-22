//! The labelling oracle: Weizenbaum's DOCTOR script on the keyword engine. It
//! must behave the way the 1966 program is documented to, because everything
//! trained from it inherits its mistakes.

use tdm_model::{KeywordEngine, KeywordScript};

const DOCTOR: &str = include_str!("../../../../fixtures/bundles/demo01-doctor.json");

fn script() -> KeywordScript {
    serde_json::from_str(DOCTOR).expect("the DOCTOR script parses")
}

fn first(input: &str) -> (String, String) {
    let s = script();
    let a = KeywordEngine::new(&s).respond(input);
    (a.rule, a.text)
}

#[test]
fn the_classic_exchanges() {
    assert_eq!(
        first("Men are all alike."),
        ("alike#0".into(), "In what way?".into())
    );
    assert_eq!(
        first("They're always bugging us about something or other.").0,
        "always#0"
    );
    assert_eq!(
        first("Well, my boyfriend made me come here."),
        ("my#2".into(), "Your boyfriend made you come here?".into())
    );
    assert_eq!(
        first("He says I'm depressed much of the time."),
        (
            "i#1".into(),
            "I am sorry to hear that you are depressed.".into()
        )
    );
    assert_eq!(
        first("I need some help, that much seems certain.").1,
        "What would it mean to you if you got some help that much seems certain?"
    );
    assert_eq!(
        first("Perhaps I could learn to get along with my mother."),
        ("my#1".into(), "Tell me more about your family.".into())
    );
}

#[test]
fn ranks_decide_between_keywords() {
    assert_eq!(
        first("my computer hates me").0,
        "computer#0",
        "computer (50) outranks my (2)"
    );
    assert_eq!(
        first("I dreamt about my mother").0,
        "dreamed#0",
        "dreamed (4) outranks my (2)"
    );
    assert_eq!(
        first("I don't have any problems"),
        ("i#8".into(), "Don't you really have any problems?".into())
    );
    assert_eq!(
        first("because we did"),
        ("because#0".into(), "Is that the real reason?".into())
    );
}

#[test]
fn questions_are_turned_back_and_nothing_falls_to_the_fallback() {
    assert_eq!(first("what should i do?").1, "Why do you ask?");
    assert_eq!(
        first("how are you").0,
        "what#0",
        "how is pre-substituted to what"
    );
    assert_eq!(first("are you a computer?").0, "computer#0");
    assert_eq!(first("the weather is nice").0, "xnone#0");
}

#[test]
fn memory_stores_a_my_statement_and_returns_it_when_nothing_matches() {
    let s = script();
    let mut e = KeywordEngine::new(&s);
    e.respond("my job is killing me");
    let later = e.respond("the weather is nice");
    assert_eq!(later.rule, "memory");
    assert_eq!(
        later.text,
        "Lets discuss further why your job is killing you."
    );
}

#[test]
fn a_chosen_rule_that_does_not_match_still_answers_without_inventing_parts() {
    let s = script();
    let a = KeywordEngine::new(&s).respond_with("my#1", "mom keeps calling");
    assert_eq!(
        a.text, "Tell me more about your family.",
        "the first reassembly that needs no captured part"
    );
}
