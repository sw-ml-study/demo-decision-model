//! Page state: the bundle, the conversation so far, which turn is selected, and
//! whether the trace is showing.

use std::rc::Rc;

use tdm_model::{
    Bundle, Conversation, KeywordEngine, KeywordScript, Remembered, Script, ScriptReply, Turn,
    respond_at,
};
use yew::prelude::*;

use crate::chat::Chat;
use crate::footer::Footer;
use crate::timeline::Timeline;
use crate::trace::Trace;

/// The trained model and its demo data, embedded at build time so the page is
/// one static download with nothing to fetch.
const BUNDLE: &str = include_str!("../../../fixtures/bundles/demo01-model4.json");

/// The bundle, shared by pointer. Props compare by identity: the bundle never
/// changes after load, and comparing 33,000 weights on every render would be
/// waste.
#[derive(Clone)]
pub struct Shared(pub Rc<Bundle>);

impl PartialEq for Shared {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::ops::Deref for Shared {
    type Target = Bundle;
    fn deref(&self) -> &Bundle {
        &self.0
    }
}

/// The conversation script: the deterministic half of the demo, as data.
const SCRIPT: &str = include_str!("../../../fixtures/bundles/demo01-script.json");

/// The keyword script whose rules the model chooses among, and which answers
/// the rule it chose.
pub const KEYWORDS: &str = include_str!("../../../fixtures/bundles/demo01-doctor.json");

/// The keyword script, parsed.
#[must_use]
pub fn keywords() -> KeywordScript {
    serde_json::from_str(KEYWORDS)
        .expect("the embedded keyword script is validated by the demo tests")
}

/// A turn: the model's decision, how long it took in this browser, and what the
/// program did with it -- the reply it built, and what it remembered.
#[derive(Clone)]
pub struct Timed {
    pub turn: Turn,
    pub micros: f64,
    pub reply: ScriptReply,
    pub keywords: Vec<String>,
    /// The rule the keyword script alone would have used: the yardstick.
    pub oracle: String,
    /// Memory as it stood when this input arrived.
    pub memory: Vec<Remembered>,
}

fn search() -> String {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default()
}

/// Inputs requested by `?say=` parameters, replayed at load.
fn initial_inputs() -> Vec<String> {
    crate::query::says(&search())
}

/// Every turn of the conversation, decided by the model at one snapshot. The
/// conversation is stored as inputs, so choosing another point on the training
/// timeline re-decides all of it and the difference is visible turn by turn.
fn decide_all(bundle: &Bundle, snapshot: usize, inputs: &[String]) -> Vec<Timed> {
    let script: Script =
        serde_json::from_str(SCRIPT).expect("the embedded script is validated by the demo tests");
    let doctor = keywords();
    let mut conversation = Conversation::with_keywords(bundle, &script, &doctor, snapshot);
    inputs
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let memory = conversation.memory().to_vec();
            let micros = time_decision(bundle, snapshot, text, i);
            let x = conversation.say(text);
            let oracle = KeywordEngine::new(&doctor).respond(text).rule;
            Timed {
                oracle,
                turn: x.turn,
                micros,
                reply: x.reply,
                keywords: x.keywords,
                memory,
            }
        })
        .collect()
}

/// Browsers coarsen `performance.now()` (to 100 us without cross-origin
/// isolation), which is slower than one decision. So each decision is timed over
/// this many repetitions and the mean is reported.
const TIMING_RUNS: u32 = 100;

/// Mean microseconds for the model's decision on one input.
fn time_decision(bundle: &Bundle, snapshot: usize, text: &str, index: usize) -> f64 {
    let start = now_ms();
    for _ in 0..TIMING_RUNS {
        let _ = respond_at(bundle, snapshot, text, index);
    }
    (now_ms() - start) * 1000.0 / f64::from(TIMING_RUNS)
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0.0, |p| p.now())
}

#[derive(Properties, PartialEq)]
struct HeaderProps {
    demo: String,
    tracing: bool,
    on_toggle: Callback<MouseEvent>,
}

#[function_component(Header)]
fn header(props: &HeaderProps) -> Html {
    html! {
        <header>
            <div>
                <h1>{ "Typed Decision Model" }</h1>
                <p class="sub">{ format!("{} · the model decides; the program answers in your own words; nothing is generated", props.demo) }</p>
            </div>
            <button class={classes!("toggle", props.tracing.then_some("on"))} onclick={props.on_toggle.clone()}>
                { if props.tracing { "trace: on" } else { "trace: off" } }
            </button>
        </header>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let bundle: Rc<Bundle> = use_memo((), |()| {
        Bundle::parse(BUNDLE).expect("the embedded bundle is validated by the crate tests")
    });
    let inputs = use_state(initial_inputs);
    let snapshot = {
        let (count, default) = (bundle.snapshot_count(), bundle.default_snapshot);
        use_state(move || crate::query::snapshot(&search(), count).unwrap_or(default))
    };
    let selected = {
        let n = inputs.len();
        use_state(move || n.checked_sub(1))
    };
    let tracing = use_state(|| true);
    let turns = {
        let bundle = bundle.clone();
        use_memo(((*inputs).clone(), *snapshot), move |(inputs, snap)| {
            decide_all(&bundle, *snap, inputs)
        })
    };

    let on_say = {
        let (inputs, selected) = (inputs.clone(), selected.clone());
        Callback::from(move |text: String| {
            let mut next = (*inputs).clone();
            next.push(text);
            selected.set(Some(next.len() - 1));
            inputs.set(next);
        })
    };
    let on_pick = {
        let snapshot = snapshot.clone();
        Callback::from(move |s: usize| snapshot.set(s))
    };
    let on_select = {
        let selected = selected.clone();
        Callback::from(move |i: usize| selected.set(Some(i)))
    };
    let on_toggle = {
        let tracing = tracing.clone();
        Callback::from(move |_: MouseEvent| tracing.set(!*tracing))
    };
    let shown = selected.and_then(|i| turns.get(i).cloned());

    html! {
        <div class="page">
            <Header demo={format!("{}: {}", bundle.demo, bundle.title)} tracing={*tracing} on_toggle={on_toggle} />
            <Timeline bundle={Shared(bundle.clone())} snapshot={*snapshot} on_pick={on_pick} />
            <main class={classes!(tracing.then_some("split"))}>
                <Chat bundle={Shared(bundle.clone())} turns={(*turns).clone()} selected={*selected}
                      on_say={on_say} on_select={on_select} />
                if *tracing {
                    <Trace bundle={Shared(bundle.clone())} shown={shown} snapshot={*snapshot} />
                }
            </main>
            <Footer bundle={Shared(bundle.clone())} />
        </div>
    }
}
