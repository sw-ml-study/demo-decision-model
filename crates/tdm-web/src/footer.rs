//! The page footer: what this is, where it came from, and exactly which build
//! is running. Build facts are stamped at compile time by `scripts/build-site`;
//! a local `trunk serve` shows them as unknown rather than inventing them.

use yew::prelude::*;

use crate::app::Shared;

/// sw-MLPL's own hosted editor: it interprets one self-contained program, with
/// no file system to resolve an include against.
const EDITOR: &str = "https://sw-ml-study.github.io/sw-mlpl/";

const REPO: &str = "https://github.com/sw-ml-study/demo-decision-model";
const HOST: Option<&str> = option_env!("TDM_BUILD_HOST");
const SHA: Option<&str> = option_env!("TDM_BUILD_SHA");
const TIME: Option<&str> = option_env!("TDM_BUILD_TIME");

#[derive(Properties, PartialEq)]
pub struct FooterProps {
    pub bundle: Shared,
}

fn build_line() -> Html {
    let sha = SHA.unwrap_or("unknown");
    let commit = SHA
        .filter(|s| !s.ends_with("-dirty"))
        .map(|s| format!("{REPO}/commit/{s}"));
    html! {
        <p class="build">
            { "build " }
            if let Some(url) = commit {
                <a href={url}><code>{ sha }</code></a>
            } else {
                <code>{ sha }</code>
            }
            { format!(" · {} · host {}", TIME.unwrap_or("unknown time"), HOST.unwrap_or("unknown")) }
        </p>
    }
}

#[function_component(Footer)]
pub fn footer(props: &FooterProps) -> Html {
    let b = &props.bundle;
    html! {
        <footer>
            <p>
                { format!("Runs entirely in your browser: {} parameters, trained from random initialization in sw-MLPL. ",
                    b.param_count()) }
                { "The model picks a kind of reply; a script, like Weizenbaum's, builds it from a fixed frame and your own words, and the program remembers what you said. When the model cannot tell, the program escalates instead of guessing. Its confidences are not yet formally calibrated." }
            </p>
            <p>
                { "Train one yourself — demo 01's first model, nine classes, not the one above: " }
                <a href="demo01-train.mlpl" download={"demo01-train.mlpl"}>{ "download this program" }</a>
                { ", open " }
                <a href={EDITOR}>{ "sw-MLPL's Live Editor" }</a>
                { ", press Load, then Run. It is one self-contained file: the interpreter runs it in your browser, from random weights, in about half a minute." }
            </p>
            <p>
                { "New here? " }
                <a href="typed-decisions.html">{ "Typed decisions: the hello world" }</a>
                { " — one state, three typed heads and a calibration measurement, in a literate document you can run ("}
                <a href="typed-decisions.mlpl" download={"typed-decisions.mlpl"}>{ "the program" }</a>
                { ")." }
            </p>
            <p>
                <a href="literate.html">{ "How the MLPL code works — the literate document" }</a>
                { " · " }
                <a href={REPO}>{ "source repository" }</a>
            </p>
            <p>
                { "Copyright (c) 2026 Michael A Wright · " }
                <a href={format!("{REPO}/blob/main/LICENSE")}>{ "MIT License" }</a>
            </p>
            { build_line() }
        </footer>
    }
}
