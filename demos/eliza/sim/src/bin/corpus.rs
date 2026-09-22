//! Build demo 01's oracle input corpus from dialogue in public-domain novels:
//! every “quoted” span, split into sentences of 3 to 18 words, deduplicated,
//! capped per book so no single voice dominates, and never equal to one of the
//! frozen probes, which stay the test.
//!
//! usage: `cargo run -p eliza-sim --bin corpus -- OUT PER_BOOK PROBES BOOK...`

use std::collections::HashSet;

fn body(text: &str) -> &str {
    let start = text
        .find("*** START OF")
        .and_then(|i| text[i..].find('\n').map(|j| i + j))
        .unwrap_or(0);
    let end = text.find("*** END OF").unwrap_or(text.len());
    &text[start..end.max(start)]
}

/// Quoted spans, with hard line breaks inside a paragraph joined.
fn quotes(text: &str) -> Vec<String> {
    let flat = text.replace("\r\n", "\n").replace('\n', " ");
    let mut out = Vec::new();
    let mut rest = flat.as_str();
    while let Some(open) = rest.find('“') {
        let after = &rest[open + '“'.len_utf8()..];
        let Some(close) = after.find('”') else {
            break;
        };
        out.push(after[..close].to_owned());
        rest = &after[close + '”'.len_utf8()..];
    }
    out
}

/// Sentences of 3 to 18 words. A sentence that ended in a question mark keeps
/// it: the author's punctuation is a free, natural label for "is this a
/// question?".
fn sentences(span: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, c) in span.char_indices() {
        if matches!(c, '.' | '!' | '?' | ';') {
            let body = span[start..i]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let body = body.trim_matches(|c: char| !c.is_alphanumeric()).to_owned();
            if (3..=18).contains(&body.split_whitespace().count()) {
                out.push(if c == '?' { format!("{body}?") } else { body });
            }
            start = i + c.len_utf8();
        }
    }
    out
}

fn norm(s: &str) -> String {
    tdm_model::words(s).join(" ")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (out, per_book, probes) = (
        &args[0],
        args[1].parse::<usize>().expect("PER_BOOK"),
        &args[2],
    );
    let excluded: HashSet<String> = std::fs::read_to_string(probes)
        .expect("probes")
        .lines()
        .filter_map(|l| l.split('\t').next_back())
        .map(norm)
        .collect();
    let mut seen = HashSet::new();
    let mut lines = Vec::new();
    for path in &args[3..] {
        let text = std::fs::read_to_string(path).expect("book");
        let book = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_owned();
        let kept = quotes(body(&text))
            .iter()
            .flat_map(|q| sentences(q))
            .filter(|s| {
                let n = norm(s);
                !n.is_empty() && !excluded.contains(&n) && seen.insert(n)
            })
            .take(per_book)
            .map(|s| format!("gutenberg-{book}\t{s}"))
            .collect::<Vec<_>>();
        eprintln!("{book}: {} sentences", kept.len());
        lines.extend(kept);
    }
    std::fs::write(out, lines.join("\n") + "\n").expect("write");
    eprintln!("wrote {} sentences to {out}", lines.len());
}
