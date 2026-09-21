//! `?say=` parameters: each one becomes a turn when the page loads, so a demo
//! can be shared as a link that opens straight onto a traced reply. Pure, so it
//! is tested natively.

/// Decode one `application/x-www-form-urlencoded` value: `+` is a space and
/// `%XX` is a byte. Malformed escapes are kept literally rather than dropped.
fn decode(v: &str) -> String {
    let bytes = v.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..=i + 2]).ok();
                match hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    Some(b) => {
                        out.push(b);
                        i += 2;
                    }
                    None => out.push(b'%'),
                }
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Every non-empty `say` value in a `location.search` string, in order.
#[must_use]
pub fn says(search: &str) -> Vec<String> {
    search
        .trim_start_matches('?')
        .split('&')
        .filter_map(|pair| pair.strip_prefix("say="))
        .map(decode)
        .filter(|s| !s.trim().is_empty())
        .collect()
}

/// The training snapshot a `snap=N` parameter asks for, if it names one that
/// exists; otherwise `None` and the page uses the bundle's default.
#[must_use]
pub fn snapshot(search: &str, count: usize) -> Option<usize> {
    search
        .trim_start_matches('?')
        .split('&')
        .filter_map(|pair| pair.strip_prefix("snap="))
        .find_map(|v| v.parse::<usize>().ok())
        .filter(|&s| s < count)
}

#[cfg(test)]
mod tests {
    use super::{says, snapshot};

    #[test]
    fn a_snap_parameter_selects_an_existing_snapshot_only() {
        assert_eq!(snapshot("?say=hi&snap=1", 5), Some(1));
        assert_eq!(snapshot("?snap=9", 5), None);
        assert_eq!(snapshot("?snap=x", 5), None);
        assert_eq!(snapshot("", 5), None);
    }

    #[test]
    fn each_say_parameter_is_one_turn_in_order() {
        assert_eq!(
            says("?say=my+mom+never+listens&x=1&say=yeah"),
            ["my mom never listens", "yeah"]
        );
    }

    #[test]
    fn percent_escapes_decode_and_malformed_ones_survive() {
        assert_eq!(says("?say=don%27t%20go"), ["don't go"]);
        assert_eq!(says("?say=100%"), ["100%"]);
        assert_eq!(says("?say=%zz"), ["%zz"]);
    }

    #[test]
    fn empty_and_missing_values_are_ignored() {
        assert!(says("").is_empty());
        assert!(says("?say=&say=+").is_empty());
    }
}
