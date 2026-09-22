//! Diagnostic: for one input, each known feature's push on one Noul's logit
//! (its embedding row dotted with that Noul's head column, over the feature
//! count), plus the bias. Unknown features push nothing.
//!
//! usage: cargo run -p tdm-model --example `noul_blame` -- BUNDLE NOUL "text"

use tdm_model::{Bundle, Model};

#[expect(
    clippy::many_single_char_names,
    reason = "a, j, w, m, d, n follow the notation of model.rs"
)]
fn main() {
    let mut a = std::env::args().skip(1);
    let bundle = Bundle::parse(&std::fs::read_to_string(a.next().expect("BUNDLE")).expect("read"))
        .expect("parse");
    let noul = a.next().expect("NOUL");
    let text = a.next().expect("text");
    let j = bundle
        .noul_names()
        .iter()
        .position(|n| *n == noul)
        .expect("no such Noul");
    let w = bundle.weights(bundle.default_snapshot);
    let (head, bias) = (
        w.noul_head.expect("Noul heads"),
        w.noul_bias.expect("Noul bias"),
    );
    let (dim, m) = (bundle.dim, bundle.noul_names().len());
    let d = Model::new(&bundle).decide(&text);
    #[expect(clippy::cast_precision_loss, reason = "feature counts are tiny")]
    let n = d.features.slots.len().max(1) as f64;
    println!(
        "{text:?}: {noul} = {:.3}   (bias {:+.2})",
        d.nouls[j], bias[j]
    );
    let mut known = d.features.slots.iter();
    for (tok, k) in d.features.tokens.iter().zip(&d.features.known) {
        if *k {
            let row = *known.next().expect("row");
            let push: f64 = (0..dim)
                .map(|i| w.embedding[row * dim + i] * head[i * m + j])
                .sum::<f64>()
                / n;
            println!("  {tok:<16} {push:+.2}");
        } else {
            println!("  {tok:<16} unknown, contributes nothing");
        }
    }
}
