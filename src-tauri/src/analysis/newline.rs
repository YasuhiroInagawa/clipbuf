/// Counts of each newline convention in `text`.
fn newline_kinds(text: &str) -> (bool, bool, bool) {
    let (mut crlf, mut lf, mut cr) = (false, false, false);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' if chars.peek() == Some(&'\n') => {
                chars.next();
                crlf = true;
            }
            '\r' => cr = true,
            '\n' => lf = true,
            _ => {}
        }
    }
    (crlf, lf, cr)
}

/// Two or more of CRLF, lone LF and lone CR are present (5.6).
pub fn has_mixed_newlines(text: &str) -> bool {
    let (crlf, lf, cr) = newline_kinds(text);
    [crlf, lf, cr].into_iter().filter(|&b| b).count() >= 2
}
