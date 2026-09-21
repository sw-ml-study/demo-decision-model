//! Browser demo: a chat with a trained typed decision model, with a trace view
//! that shows every decision behind every reply. The model runs entirely in the
//! page; no server, no interpreter.

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod chat;
#[cfg(target_arch = "wasm32")]
mod footer;
#[cfg_attr(
    all(not(target_arch = "wasm32"), not(test)),
    expect(
        dead_code,
        reason = "only the browser build reads the page URL; native builds only test the parser"
    )
)]
mod query;
#[cfg(target_arch = "wasm32")]
mod timeline;
#[cfg(target_arch = "wasm32")]
mod trace;

#[cfg(target_arch = "wasm32")]
fn main() {
    yew::Renderer::<app::App>::new().render();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
