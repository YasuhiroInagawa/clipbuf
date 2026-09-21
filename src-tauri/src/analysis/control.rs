use super::charset::is_control_or_binary;

/// Contains control characters other than tab / LF / CR, DEL, C1 controls, or U+FFFD (5.5).
///
/// Invalid UTF-8 cannot reach a `&str`, so U+FFFD is the trace of decoding failures upstream.
pub fn has_control_or_binary(text: &str) -> bool {
    text.chars().any(is_control_or_binary)
}
