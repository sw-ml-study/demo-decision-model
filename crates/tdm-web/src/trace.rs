//! The trace: for one turn, the state that went in, the choices that were
//! offered, the distribution that came out, the policy that consumed it, and
//! the table entry that resulted.

use tdm_model::{Bundle, Move, Outcome};
use yew::prelude::*;

use crate::app::{Shared, Timed};

#[derive(Properties, PartialEq)]
pub struct TraceProps {
    pub bundle: Shared,
    pub shown: Option<Timed>,
    pub snapshot: usize,
}

fn bars(bundle: &Bundle, t: &Timed) -> Html {
    let d = &t.turn.decision;
    let mut order: Vec<usize> = (0..d.probs.len()).collect();
    order.sort_by(|&a, &b| d.probs[b].total_cmp(&d.probs[a]));
    html! {
        <div class="bars">
            { for order.iter().map(|&i| {
                let p = d.probs[i];
                let style = format!("width: {:.1}%", p * 100.0);
                html! {
                    <div class={classes!("bar", (i == d.selected).then_some("top"))}>
                        <span class="label">{ &bundle.labels[i] }</span>
                        <span class="track"><span class="fill" style={style}></span></span>
                        <span class="num">{ format!("{p:.3}") }</span>
                    </div>
                }
            }) }
        </div>
    }
}

fn policy(bundle: &Bundle, t: &Timed) -> Html {
    let (d, r) = (&t.turn.decision, &t.turn.reply);
    let chosen = &bundle.labels[d.selected];
    let text = match (r.outcome, bundle.escalation) {
        (Outcome::Act, Some(e)) => format!(
            "confidence {:.3} ≥ {:.2}, margin {:.3} ≥ {:.2}, {} known features  →  act on {chosen}",
            d.confidence,
            e.min_confidence,
            d.margin,
            e.min_margin,
            d.features.slots.len()
        ),
        (Outcome::Act, None) => format!(
            "confidence {:.3} ≥ {:.2}  →  act on {chosen}",
            d.confidence, bundle.threshold
        ),
        (Outcome::NoneApplies, _) => format!(
            "the model chose {chosen}: none of the offered replies fits  →  reply from {chosen}"
        ),
        (Outcome::Escalate, Some(e)) => format!(
            "not enough to act on (needs confidence ≥ {:.2}, margin ≥ {:.2}, {} known feature{})  →  ESCALATE: a larger decider would choose among the same {} options; none runs in the browser, so reply from {}",
            e.min_confidence,
            e.min_margin,
            e.min_known,
            if e.min_known == 1 { "" } else { "s" },
            bundle.labels.len(),
            bundle.labels[r.label]
        ),
        (Outcome::Escalate, None) => format!(
            "confidence {:.3} < {:.2}  →  abstain, fall back to {}",
            d.confidence, bundle.threshold, bundle.labels[r.label]
        ),
    };
    html! { <p class={classes!("mono", (r.outcome == Outcome::Escalate).then_some("escalate"))}>{ text }</p> }
}

fn describe(by: &Move) -> String {
    match by {
        Move::Recall { turn, keyword } => format!(
            "repeat rule: \"{keyword}\" was used before, in turn {}, which is brought back",
            turn + 1
        ),
        Move::Memory { turn } => format!(
            "memory rule: nothing else fit, so a remembered statement from turn {} comes back",
            turn + 1
        ),
        Move::LabelRule { label, rule } => format!("{label} rule: {rule}"),
        Move::KeywordRule { rule } => format!("keyword rule for any label: {rule}"),
        Move::Reflect => {
            "reflection: nothing matched, so the visitor's own words come back".to_owned()
        }
        Move::Canned => "the chosen label's canned reply".to_owned(),
    }
}

fn built(t: &Timed) -> Html {
    let r = &t.reply;
    let slots = r
        .slots
        .iter()
        .map(|(n, v)| format!("{{{n}}} = \"{v}\""))
        .collect::<Vec<_>>()
        .join(",  ");
    html! {
        <>
            <p class="mono">{ describe(&r.by) }</p>
            <p class="mono">{ format!("frame: \"{}\"", r.frame) }</p>
            if !slots.is_empty() {
                <p class="mono">{ format!("slots: {slots}  — the visitor's own words, pronouns reflected") }</p>
            }
            <p class="meta">{ "The reply is exactly the frame with the slots filled. Nothing is generated." }</p>
        </>
    }
}

fn memory(t: &Timed) -> Html {
    let recalled = match t.reply.by {
        Move::Recall { turn, .. } | Move::Memory { turn } => Some(turn),
        _ => None,
    };
    if t.memory.is_empty() {
        return html! { <p class="meta">{ "Nothing remembered yet." }</p> };
    }
    html! {
        <div class="memory">
            { for t.memory.iter().map(|m| html! {
                <p class={classes!("mono", (Some(m.turn) == recalled).then_some("recalled"))}>
                    { format!("{:>2}. {}{}", m.turn + 1, m.text, if m.mine { "   [my …]" } else { "" }) }
                    <span class="kw">{ format!("  {}", m.keywords.join(" ")) }</span>
                </p>
            }) }
            <p class="meta">{ format!("this input's keywords: {}", if t.keywords.is_empty() { "none".to_owned() } else { t.keywords.join(", ") }) }</p>
        </div>
    }
}

fn features(b: &Bundle, t: &Timed) -> Html {
    let f = &t.turn.decision.features;
    let known = f.known.iter().filter(|k| **k).count();
    let meta = if b.vocab.is_some() {
        format!(
            "{known} of {} features are in the model's vocabulary; unknown ones contribute nothing",
            f.tokens.len()
        )
    } else {
        format!(
            "{} features of {}, hashed into {} slots",
            f.slots.len(),
            b.width,
            b.slots.unwrap_or(0)
        )
    };
    html! {
        <>
            <p class="meta">{ meta }</p>
            <div class="chips">
                { for f.tokens.iter().zip(&f.known).map(|(tok, k)| html! {
                    <span class={classes!("chip", (!*k).then_some("unknown"))}
                          title={if *k { "known: read by the model" } else { "not in the vocabulary: contributes nothing" }}>{ tok }</span>
                }) }
            </div>
        </>
    }
}

#[function_component(Trace)]
pub fn trace(props: &TraceProps) -> Html {
    let b = &props.bundle;
    let Some(t) = &props.shown else {
        return html! { <section class="trace"><p class="hint">{ "Each reply you get is traced here: select one to inspect it." }</p></section> };
    };
    let d = &t.turn.decision;
    let agree = t.turn.matcher == d.selected;
    html! {
        <section class="trace">
            <h2>{ "State in" }</h2>
            <p class="quote">{ &t.turn.input }</p>
            { features(b, t) }

            <h2>{ "Choices in" }</h2>
            <p class="meta">{ format!("choice: {} — {} offered", b.question, b.labels.len()) }</p>

            <h2>{ "Decision out" }</h2>
            <p class="meta">{ format!("model as of {} of training ({} steps)",
                b.snapshot_label(props.snapshot),
                b.snapshots.steps[props.snapshot]) }</p>
            { bars(b, t) }
            <p class="mono">{ format!("confidence {:.3}   margin {:.3}   decided in {:.1} µs (mean of 100 runs)", d.confidence, d.margin, t.micros) }</p>

            <h2>{ "Policy" }<span class="aside">{ " ordinary code, not the model" }</span></h2>
            { policy(b, t) }

            <h2>{ "Output" }</h2>
            { built(t) }

            <h2>{ "Memory" }<span class="aside">{ " kept by the program, not the model" }</span></h2>
            { memory(t) }

            <h2>{ "Yardstick" }</h2>
            <p class={classes!("mono", if agree { "agree" } else { "disagree" })}>
                { format!("1966-style keyword list picks {} — {}", b.labels[t.turn.matcher],
                    if agree { "agrees with the model" } else { "disagrees with the model" }) }
            </p>
        </section>
    }
}
