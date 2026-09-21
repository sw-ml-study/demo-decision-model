//! Demo 01 regression ratchet on the two complaints that prompted model 2: the
//! demo asked about machines, and greeted, when the visitor had done neither.
//! Scored on the 96 frozen, hand-labelled probes, on what a visitor actually
//! sees: the reply after the policy, with escalations counted separately. The
//! bounds are the values model 2 reached; they may tighten, never loosen.

use tdm_model::{Bundle, Outcome, respond};

const BUNDLE: &str = include_str!("../../../../fixtures/bundles/demo01-model2.json");
const PROBES: &str = include_str!("../../probe-labels.tsv");

struct Tally {
    shown_wrong_computer: u32,
    shown_wrong_greet: u32,
    correct_replies: u32,
    escalated: u32,
    total: u32,
}

fn tally() -> Tally {
    let b = Bundle::parse(BUNDLE).expect("bundle");
    let mut t = Tally {
        shown_wrong_computer: 0,
        shown_wrong_greet: 0,
        correct_replies: 0,
        escalated: 0,
        total: 0,
    };
    for (i, row) in PROBES.lines().filter(|r| !r.trim().is_empty()).enumerate() {
        let (truth, text) = row.split_once('\t').expect("LABEL<TAB>text");
        let turn = respond(&b, text, i);
        let shown = b.labels[turn.reply.label].as_str();
        t.total += 1;
        if turn.reply.outcome == Outcome::Escalate {
            t.escalated += 1;
            continue;
        }
        t.correct_replies += u32::from(shown == truth);
        t.shown_wrong_computer += u32::from(shown == "COMPUTER" && truth != "COMPUTER");
        t.shown_wrong_greet += u32::from(shown == "GREET" && truth != "GREET");
    }
    t
}

#[test]
fn it_no_longer_asks_about_machines_or_greets_unprompted() {
    let t = tally();
    assert!(
        t.shown_wrong_computer <= 1,
        "visible wrong COMPUTER replies: {} (the v1 model gave 12)",
        t.shown_wrong_computer
    );
    assert_eq!(
        t.shown_wrong_greet, 0,
        "visible wrong GREET replies (the v1 model gave 7)"
    );
}

#[test]
fn replies_it_does_give_are_mostly_right_and_escalation_is_bounded() {
    let t = tally();
    let acted = t.total - t.escalated;
    assert!(
        t.correct_replies * 100 >= acted * 75,
        "right on {} of {} non-escalated replies",
        t.correct_replies,
        acted
    );
    assert!(
        t.escalated * 100 <= t.total * 35,
        "escalated {} of {}",
        t.escalated,
        t.total
    );
}
