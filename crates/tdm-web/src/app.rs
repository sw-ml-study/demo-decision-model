//! Page state: the bundle, the conversation so far, which turn is selected, and
//! whether the trace is showing.

use std::rc::Rc;

use tdm_model::{Bundle, Turn, respond};
use yew::prelude::*;

use crate::chat::Chat;
use crate::footer::Footer;
use crate::trace::Trace;

/// The trained model and its demo data, embedded at build time so the page is
/// one static download with nothing to fetch.
const BUNDLE: &str = include_str!("../../../fixtures/bundles/demo01.json");

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

/// A turn plus how long the decision took in this browser.
#[derive(Clone)]
pub struct Timed {
    pub turn: Turn,
    pub micros: f64,
}

/// Turns requested by `?say=` parameters, run once at load.
fn initial_turns(bundle: &Bundle) -> Vec<Timed> {
    let search = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default();
    crate::query::says(&search)
        .iter()
        .enumerate()
        .map(|(i, text)| timed(bundle, text, i))
        .collect()
}

fn timed(bundle: &Bundle, text: &str, index: usize) -> Timed {
    let start = now_ms();
    let turn = respond(bundle, text, index);
    Timed {
        turn,
        micros: (now_ms() - start) * 1000.0,
    }
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0.0, |p| p.now())
}

#[derive(Properties, PartialEq)]
struct HeaderProps {
    title: String,
    tracing: bool,
    on_toggle: Callback<MouseEvent>,
}

#[function_component(Header)]
fn header(props: &HeaderProps) -> Html {
    html! {
        <header>
            <div>
                <h1>{ &props.title }</h1>
                <p class="sub">{ "a typed decision model chooses every reply; it never writes one" }</p>
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
    let turns = {
        let bundle = bundle.clone();
        use_state(move || initial_turns(&bundle))
    };
    let selected = {
        let n = turns.len();
        use_state(move || n.checked_sub(1))
    };
    let tracing = use_state(|| true);

    let on_say = {
        let (bundle, turns, selected) = (bundle.clone(), turns.clone(), selected.clone());
        Callback::from(move |text: String| {
            let mut next = (*turns).clone();
            next.push(timed(&bundle, &text, turns.len()));
            selected.set(Some(next.len() - 1));
            turns.set(next);
        })
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
            <Header title={bundle.title.clone()} tracing={*tracing} on_toggle={on_toggle} />
            <main class={classes!(tracing.then_some("split"))}>
                <Chat bundle={Shared(bundle.clone())} turns={(*turns).clone()} selected={*selected}
                      on_say={on_say} on_select={on_select} />
                if *tracing {
                    <Trace bundle={Shared(bundle.clone())} shown={shown} />
                }
            </main>
            <Footer bundle={Shared(bundle.clone())} />
        </div>
    }
}
