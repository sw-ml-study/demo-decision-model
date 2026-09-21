//! The training timeline: one run from random weights, snapshotted. Picking a
//! snapshot re-decides the whole conversation with the model as it was then,
//! beside the numbers MLPL measured for that snapshot.

use tdm_model::Bundle;
use yew::prelude::*;

use crate::app::Shared;

const ACC: &str = "val accuracy";
const PROBE: &str = "probe confidence";
const TEST: &str = "probe accuracy";
const WRONG: &str = "probe confidence when wrong";
const RIGHT: &str = "probe confidence when right";
const W: f64 = 360.0;
const H: f64 = 110.0;
const PAD: f64 = 14.0;

#[derive(Properties, PartialEq)]
pub struct TimelineProps {
    pub bundle: Shared,
    pub snapshot: usize,
    pub on_pick: Callback<usize>,
}

fn label(b: &Bundle, i: usize) -> String {
    b.snapshot_label(i)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "snapshot counts are single digits"
)]
fn x(i: usize, n: usize) -> f64 {
    PAD + (W - 2.0 * PAD) * i as f64 / (n.max(2) - 1) as f64
}

fn y(v: f64) -> f64 {
    H - PAD - (H - 2.0 * PAD) * v.clamp(0.0, 1.0)
}

fn line(b: &Bundle, metric: &str) -> String {
    let n = b.snapshot_count();
    (0..n)
        .map(|i| {
            format!(
                "{:.1},{:.1}",
                x(i, n),
                y(b.metric(i, metric).unwrap_or(0.0))
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The series to draw: validation accuracy, then test accuracy when the bundle
/// measured it, then a confidence series -- on wrong answers if measured.
fn series(b: &Bundle) -> Vec<(&'static str, &'static str, &'static str)> {
    let has = |m: &str| b.snapshots.metric_names.iter().any(|n| n == m);
    let mut out = vec![(ACC, "acc", "validation accuracy")];
    if has(TEST) {
        out.push((TEST, "test", "accuracy on the 96 hand-labelled probes"));
    }
    if has(WRONG) {
        out.push((WRONG, "conf", "confidence when wrong"));
    } else {
        out.push((PROBE, "conf", "confidence on probe inputs"));
    }
    out
}

fn chart(b: &Bundle, snapshot: usize) -> Html {
    let n = b.snapshot_count();
    let sx = format!("{:.1}", x(snapshot, n));
    html! {
        <svg class="chart" viewBox={format!("0 0 {W} {H}")} role="img"
             aria-label="accuracy and confidence across the training snapshots">
            <line class="axis" x1={PAD.to_string()} y1={y(0.0).to_string()} x2={(W - PAD).to_string()} y2={y(0.0).to_string()} />
            <line class="axis faint" x1={PAD.to_string()} y1={y(1.0).to_string()} x2={(W - PAD).to_string()} y2={y(1.0).to_string()} />
            <line class="cursor" x1={sx.clone()} y1={y(1.0).to_string()} x2={sx} y2={y(0.0).to_string()} />
            { for series(b).into_iter().map(|(m, class, _)| html! { <polyline class={class} points={line(b, m)} /> }) }
        </svg>
    }
}

/// A sentence read off the numbers, not written in advance: where validation
/// accuracy stopped changing, and what confidence did after that.
fn reading(b: &Bundle) -> String {
    let n = b.snapshot_count();
    let get = |i, m| b.metric(i, m).unwrap_or(0.0);
    if b.metric(0, TEST).is_some() {
        let d = b.default_snapshot;
        return format!(
            "The page uses the {} snapshot, chosen by accuracy on held-out frames ({:.2}), never by the probes. On the 96 hand-labelled probes it is right {:.0}% of the time, and it knows when it is not: {:.2} confident when wrong against {:.2} when right, which is what the escalation threshold relies on.",
            label(b, d),
            get(d, ACC),
            100.0 * get(d, TEST),
            get(d, WRONG),
            get(d, RIGHT)
        );
    }
    let settled = (1..n)
        .find(|&i| (i..n).all(|j| (get(j, ACC) - get(i, ACC)).abs() < 1e-9))
        .unwrap_or(n - 1);
    if settled + 1 >= n {
        return format!(
            "Validation accuracy is still changing at {}.",
            label(b, n - 1)
        );
    }
    format!(
        "Validation accuracy stops changing at {} ({:.2}). From there to {}, it stays put while confidence on the probe inputs — which the model was never trained on — rises from {:.2} to {:.2}.",
        label(b, settled),
        get(settled, ACC),
        label(b, n - 1),
        get(settled, PROBE),
        get(n - 1, PROBE)
    )
}

#[function_component(Timeline)]
pub fn timeline(props: &TimelineProps) -> Html {
    let b = &props.bundle;
    let s = props.snapshot;
    let num = |name: &str| {
        b.metric(s, name).map_or_else(
            || "—".to_owned(),
            |v| {
                if v.fract() == 0.0 && v.abs() >= 1.0 {
                    format!("{v:.0}")
                } else {
                    format!("{v:.3}")
                }
            },
        )
    };
    html! {
        <section class="timeline">
            <div class="tl-head">
                <h2>{ "Training" }</h2>
                <p class="meta">{ "One run from random weights, snapshotted. Pick a point: every reply below is re-decided by the model as it was then." }</p>
            </div>
            <div class="tl-body">
                <div class="tl-pick">
                    { for (0..b.snapshot_count()).map(|i| {
                        let pick = props.on_pick.clone();
                        html! {
                            <button class={classes!("snap", (i == s).then_some("on"))}
                                    onclick={Callback::from(move |_: MouseEvent| pick.emit(i))}>
                                <span class="t">{ label(b, i) }</span>
                                <span class="st">{ format!("{} steps", b.snapshots.steps[i]) }</span>
                            </button>
                        }
                    }) }
                </div>
                <div class="tl-chart">
                    { chart(b, s) }
                    <p class="legend">
                        { for series(b).into_iter().map(|(_, class, name)| html! {
                            <span class="item"><span class={classes!("k", class)}></span>{ name }</span>
                        }) }
                    </p>
                </div>
                <div class="tl-nums mono">
                    { for b.snapshots.metric_names.iter().map(|m| html! {
                        <p>{ format!("{m:<28} {}", num(m)) }</p>
                    }) }
                </div>
            </div>
            <p class="reading">{ reading(b) }</p>
        </section>
    }
}
