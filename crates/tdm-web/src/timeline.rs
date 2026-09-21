//! The training timeline: one run from random weights, snapshotted. Picking a
//! snapshot re-decides the whole conversation with the model as it was then,
//! beside the numbers MLPL measured for that snapshot.

use tdm_model::Bundle;
use yew::prelude::*;

use crate::app::Shared;

const ACC: &str = "val accuracy";
const PROBE: &str = "probe confidence";
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
    let s = b.snapshots.seconds[i];
    if b.snapshots.steps[i] == 0 {
        "0 s · random".to_owned()
    } else {
        format!("{s} s")
    }
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

fn chart(b: &Bundle, snapshot: usize) -> Html {
    let n = b.snapshot_count();
    let sx = format!("{:.1}", x(snapshot, n));
    html! {
        <svg class="chart" viewBox={format!("0 0 {W} {H}")} role="img"
             aria-label="validation accuracy and probe confidence across the training snapshots">
            <line class="axis" x1={PAD.to_string()} y1={y(0.0).to_string()} x2={(W - PAD).to_string()} y2={y(0.0).to_string()} />
            <line class="axis faint" x1={PAD.to_string()} y1={y(1.0).to_string()} x2={(W - PAD).to_string()} y2={y(1.0).to_string()} />
            <line class="cursor" x1={sx.clone()} y1={y(1.0).to_string()} x2={sx} y2={y(0.0).to_string()} />
            <polyline class="acc" points={line(b, ACC)} />
            <polyline class="conf" points={line(b, PROBE)} />
        </svg>
    }
}

/// A sentence read off the numbers, not written in advance: where validation
/// accuracy stopped changing, and what confidence did after that.
fn reading(b: &Bundle) -> String {
    let n = b.snapshot_count();
    let acc = |i| b.metric(i, ACC).unwrap_or(0.0);
    let conf = |i| b.metric(i, PROBE).unwrap_or(0.0);
    let settled = (1..n)
        .find(|&i| (i..n).all(|j| (acc(j) - acc(i)).abs() < 1e-9))
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
        acc(settled),
        label(b, n - 1),
        conf(settled),
        conf(n - 1)
    )
}

#[function_component(Timeline)]
pub fn timeline(props: &TimelineProps) -> Html {
    let b = &props.bundle;
    let s = props.snapshot;
    let num = |name: &str| {
        b.metric(s, name)
            .map_or_else(|| "—".to_owned(), |v| format!("{v:.3}"))
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
                    <p class="legend"><span class="k acc"></span>{ "validation accuracy" }
                        <span class="k conf"></span>{ "confidence on probe inputs" }</p>
                </div>
                <div class="tl-nums mono">
                    <p>{ format!("val accuracy   {}", num("val accuracy")) }</p>
                    <p>{ format!("wild accuracy  {}", num("wild accuracy")) }</p>
                    <p>{ format!("val confidence {}", num("val confidence")) }</p>
                    <p>{ format!("probe conf.    {}", num("probe confidence")) }</p>
                    <p>{ format!("train loss     {}", num("train loss")) }</p>
                </div>
            </div>
            <p class="reading">{ reading(b) }</p>
        </section>
    }
}
