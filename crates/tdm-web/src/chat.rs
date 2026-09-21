//! The conversation: what the visitor said, what came back, and a box to type in.

use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::app::{Shared, Timed};

#[derive(Properties, PartialEq)]
pub struct ChatProps {
    pub bundle: Shared,
    pub turns: Vec<Timed>,
    pub selected: Option<usize>,
    pub on_say: Callback<String>,
    pub on_select: Callback<usize>,
}

impl PartialEq for Timed {
    fn eq(&self, other: &Self) -> bool {
        self.turn.input == other.turn.input && self.turn.reply.text == other.turn.reply.text
    }
}

fn log(props: &ChatProps) -> Html {
    html! {
        <div class="log">
            if props.turns.is_empty() {
                <p class="hint">{ "Say something, or try one of the examples below." }</p>
            }
            { for props.turns.iter().enumerate().map(|(i, t)| {
                let pick = props.on_select.clone();
                let chosen = props.selected == Some(i);
                html! {
                    <div class={classes!("exchange", chosen.then_some("chosen"))}
                         onclick={Callback::from(move |_: MouseEvent| pick.emit(i))}>
                        <p class="you"><span>{ "YOU" }</span>{ &t.turn.input }</p>
                        <p class="them"><span>{ &props.bundle.title }</span>{ &t.turn.reply.text }</p>
                    </div>
                }
            }) }
        </div>
    }
}

fn examples(props: &ChatProps) -> Html {
    html! {
        <div class="examples">
            { for props.bundle.examples.iter().map(|ex| {
                let (say, text) = (props.on_say.clone(), ex.clone());
                html! { <button onclick={Callback::from(move |_: MouseEvent| say.emit(text.clone()))}>{ ex }</button> }
            }) }
        </div>
    }
}

#[function_component(Chat)]
pub fn chat(props: &ChatProps) -> Html {
    let input = use_node_ref();
    let say = {
        let (input, on_say) = (input.clone(), props.on_say.clone());
        move || {
            if let Some(el) = input.cast::<HtmlInputElement>() {
                let text = el.value();
                if !text.trim().is_empty() {
                    on_say.emit(text);
                    el.set_value("");
                }
            }
        }
    };
    let on_key = {
        let say = say.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                say();
            }
        })
    };
    let on_send = Callback::from(move |_: MouseEvent| say());

    html! {
        <section class="chat">
            { log(props) }
            { examples(props) }
            <div class="say">
                <input ref={input} type="text" placeholder="type and press Enter" onkeydown={on_key} />
                <button onclick={on_send}>{ "Send" }</button>
            </div>
        </section>
    }
}
