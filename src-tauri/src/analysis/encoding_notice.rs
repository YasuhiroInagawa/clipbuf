use unicode_normalization::UnicodeNormalization;

use super::charset::is_bidi_control;

/// Leading BOM, bidirectional control characters, or mixed NFC/NFD (5.7).
///
/// "Mixed" means the text is neither fully composed nor fully decomposed: both
/// normalizations change it.
pub fn has_encoding_notice(text: &str) -> bool {
    if text.starts_with('\u{feff}') {
        return true;
    }
    if text.chars().any(is_bidi_control) {
        return true;
    }
    if text.is_ascii() {
        return false;
    }
    let nfc_differs = !text.nfc().eq(text.chars());
    let nfd_differs = !text.nfd().eq(text.chars());
    nfc_differs && nfd_differs
}
