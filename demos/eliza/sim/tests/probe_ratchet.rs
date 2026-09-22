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

const BUNDLE3: &str = include_str!("../../../../fixtures/bundles/demo01-model3.json");
const PROBE_NOULS: &str = include_str!("../../probe-nouls.tsv");

#[test]
fn model_3_keeps_the_fixes_and_escalates_less() {
    let b = Bundle::parse(BUNDLE3).expect("model 3");
    let (mut wrong_computer, mut wrong_greet, mut right, mut escalated, mut total) =
        (0, 0, 0, 0, 0);
    for (i, row) in PROBES.lines().filter(|r| !r.trim().is_empty()).enumerate() {
        let (truth, text) = row.split_once('\t').expect("LABEL<TAB>text");
        let turn = respond(&b, text, i);
        let shown = b.labels[turn.reply.label].as_str();
        total += 1;
        if turn.reply.outcome == Outcome::Escalate {
            escalated += 1;
            continue;
        }
        right += u32::from(shown == truth);
        wrong_computer += u32::from(shown == "COMPUTER" && truth != "COMPUTER");
        wrong_greet += u32::from(shown == "GREET" && truth != "GREET");
    }
    assert!(
        wrong_computer <= 1,
        "visible wrong COMPUTER replies: {wrong_computer}"
    );
    assert_eq!(wrong_greet, 0, "visible wrong GREET replies");
    assert!(
        escalated * 100 <= total * 20,
        "escalated {escalated} of {total}; model 2 escalated a third"
    );
    assert!(
        right * 100 >= (total - escalated) * 75,
        "right on {right} of {} non-escalated",
        total - escalated
    );
}

#[test]
fn model_3_catches_the_questions_it_is_asked() {
    // The question Noul is what the deflection rule relies on: it must catch
    // nearly every question in the frozen probes and rarely flag a statement.
    let b = Bundle::parse(BUNDLE3).expect("model 3");
    let q = b
        .noul_names()
        .iter()
        .position(|n| n == "question")
        .expect("a question Noul");
    let (mut caught, mut asked, mut false_alarms, mut statements) = (0, 0, 0, 0);
    for row in PROBE_NOULS.lines().filter(|r| !r.trim().is_empty()) {
        let cols: Vec<&str> = row.split('\t').collect();
        let (is_q, text) = (cols[0] == "1", cols[3]);
        let flagged = tdm_model::Model::new(&b).decide(text).nouls[q] >= 0.5;
        if is_q {
            asked += 1;
            caught += u32::from(flagged);
        } else {
            statements += 1;
            false_alarms += u32::from(flagged);
        }
    }
    assert!(
        caught * 100 >= asked * 90,
        "caught {caught} of {asked} questions"
    );
    assert!(
        false_alarms * 100 <= statements * 3,
        "{false_alarms} of {statements} statements flagged"
    );
}
