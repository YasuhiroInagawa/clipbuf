/// Leading or trailing whitespace or newline (5.2).
///
/// Uses `char::is_whitespace`, which covers U+0020, U+3000, NBSP, tab, LF and CR.
pub fn has_edge_whitespace(text: &str) -> bool {
    let first = text.chars().next().is_some_and(char::is_whitespace);
    let last = text.chars().next_back().is_some_and(char::is_whitespace);
    first || last
}
