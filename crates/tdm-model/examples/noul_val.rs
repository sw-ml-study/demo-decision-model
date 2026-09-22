//! Per-snapshot validation accuracy of each Noul head, from a bundle whose
//! parity set begins with the validation sentences, against their labels
//! (`q n p` per line, in the same order).
//!
//! usage: cargo run -p tdm-model --example `noul_val` -- BUNDLE LABELS

#[expect(
    clippy::needless_range_loop,
    reason = "s indexes the snapshots of several parallel arrays"
)]
fn main() {
    let mut a = std::env::args().skip(1);
    let b = tdm_model::Bundle::parse(
        &std::fs::read_to_string(a.next().expect("BUNDLE")).expect("read"),
    )
    .expect("parse");
    let labels: Vec<Vec<f64>> = std::fs::read_to_string(a.next().expect("LABELS"))
        .expect("read labels")
        .lines()
        .map(|l| {
            l.split_whitespace()
                .map(|x| x.parse().expect("0 or 1"))
                .collect()
        })
        .collect();
    let m = b.noul_names().len();
    let nouls = b.parity.nouls.as_ref().expect("noul parity");
    for s in 0..b.snapshot_count() {
        let accs: Vec<f64> = (0..m)
            .map(|j| {
                let hits = labels
                    .iter()
                    .enumerate()
                    .filter(|(i, l)| (nouls[s][i * m + j] >= 0.5) == (l[j] >= 0.5))
                    .count();
                #[expect(clippy::cast_precision_loss, reason = "counts are small")]
                let acc = hits as f64 / labels.len() as f64;
                acc
            })
            .collect();
        let choice = b.metric(s, "val accuracy").unwrap_or(0.0);
        #[expect(clippy::cast_precision_loss, reason = "three heads")]
        let noul_mean = accs.iter().sum::<f64>() / m as f64;
        println!(
            "{:<9} choice val {choice:.3}  noul val {:?}  mean of all heads {:.3}",
            b.snapshot_label(s),
            accs.iter()
                .map(|x| (x * 1000.0).round() / 1000.0)
                .collect::<Vec<_>>(),
            (choice + noul_mean * f64::from(u32::try_from(m).unwrap_or(1)))
                / (1.0 + f64::from(u32::try_from(m).unwrap_or(1)))
        );
    }
}
