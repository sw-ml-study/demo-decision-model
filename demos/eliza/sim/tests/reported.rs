//! Every failure a person reported on the live demo, kept as a test so it
//! cannot come back. The list lives in demos/eliza/reported.tsv.

use tdm_model::{Bundle, Conversation, KeywordScript, Move, Script};

const BUNDLE: &str = include_str!("../../../../fixtures/bundles/demo01-model3.json");
const BUNDLE4: &str = include_str!("../../../../fixtures/bundles/demo01-model4.json");
const SCRIPT: &str = include_str!("../../../../fixtures/bundles/demo01-script.json");
const DOCTOR: &str = include_str!("../../../../fixtures/bundles/demo01-doctor.json");
const REPORTED: &str = include_str!("../../reported.tsv");

#[test]
fn no_reported_failure_comes_back() {
    check_reports(BUNDLE, None);
}

#[test]
fn no_reported_failure_comes_back_on_the_model_the_page_ships() {
    check_reports(BUNDLE4, Some(DOCTOR));
}

fn check_reports(bundle: &str, doctor: Option<&str>) {
    let b = Bundle::parse(bundle).expect("bundle");
    let s: Script = serde_json::from_str(SCRIPT).expect("script");
    let doctor: Option<KeywordScript> =
        doctor.map(|d| serde_json::from_str(d).expect("keyword script"));
    for row in REPORTED
        .lines()
        .filter(|r| !r.trim().is_empty() && !r.starts_with('#'))
    {
        let (want, input) = row.split_once('\t').expect("expectation<TAB>input");
        // Each report is checked as the opening line of a fresh conversation,
        // so no earlier turn can mask it.
        let mut c = doctor.as_ref().map_or_else(
            || Conversation::new(&b, &s, b.default_snapshot),
            |d| Conversation::with_keywords(&b, &s, d, b.default_snapshot),
        );
        let x = c.say(input);
        let deflected = matches!(x.reply.by, Move::Deflect { .. });
        match want {
            "not-deflected" => assert!(
                !deflected,
                "{input:?} was deflected as a question: {:?}",
                x.reply.text
            ),
            "deflected" => assert!(deflected, "{input:?} was not deflected: {:?}", x.reply.text),
            other => panic!("unknown expectation {other:?}"),
        }
    }
}

#[test]
fn a_question_without_a_mark_is_still_turned_back_when_the_model_is_sure() {
    let b = Bundle::parse(BUNDLE).expect("model 3");
    let s: Script = serde_json::from_str(SCRIPT).expect("script");
    for q in ["what should i do", "why do you ask"] {
        let mut c = Conversation::new(&b, &s, b.default_snapshot);
        assert!(
            matches!(c.say(q).reply.by, Move::Deflect { .. }),
            "{q:?} was not deflected"
        );
    }
}
