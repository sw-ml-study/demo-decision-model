//! The trace: for one turn, the state that went in, the choices that were
//! offered, the distribution that came out, the policy that consumed it, and
//! the table entry that resulted.

use tdm_model::Bundle;
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
    let text = if r.acted {
        format!(
            "confidence {:.3} ≥ {:.2}  →  act on {}",
            d.confidence, bundle.threshold, bundle.labels[r.label]
        )
    } else {
        format!(
            "confidence {:.3} < {:.2}  →  abstain, fall back to {}",
            d.confidence, bundle.threshold, bundle.labels[r.label]
        )
    };
    html! { <p class="mono">{ text }</p> }
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
            <p class="meta">{ format!("{} features of {}, hashed into {} slots", d.features.slots.len(), b.width, b.slots) }</p>
            <div class="chips">
                { for d.features.tokens.iter().zip(&d.features.slots).map(|(tok, slot)| html! {
                    <span class="chip" title={format!("slot {slot}")}>{ tok }</span>
                }) }
            </div>

            <h2>{ "Choices in" }</h2>
            <p class="meta">{ format!("choice: {} — {} offered", b.question, b.labels.len()) }</p>

            <h2>{ "Decision out" }</h2>
            <p class="meta">{ format!("model as of {} of training ({} steps)",
                if b.snapshots.steps[props.snapshot] == 0 { "0 s".to_owned() } else { format!("{} s", b.snapshots.seconds[props.snapshot]) },
                b.snapshots.steps[props.snapshot]) }</p>
            { bars(b, t) }
            <p class="mono">{ format!("confidence {:.3}   margin {:.3}   decided in {:.1} µs (mean of 100 runs)", d.confidence, d.margin, t.micros) }</p>

            <h2>{ "Policy" }<span class="aside">{ " ordinary code, not the model" }</span></h2>
            { policy(b, t) }

            <h2>{ "Output" }</h2>
            <p class="mono">{ format!("table {}, entry {} — quoted verbatim, not composed", b.labels[t.turn.reply.label], t.turn.reply.index) }</p>

            <h2>{ "Yardstick" }</h2>
            <p class={classes!("mono", if agree { "agree" } else { "disagree" })}>
                { format!("1966-style keyword list picks {} — {}", b.labels[t.turn.matcher],
                    if agree { "agrees with the model" } else { "disagrees with the model" }) }
            </p>
        </section>
    }
}
