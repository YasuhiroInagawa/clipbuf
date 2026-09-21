//! Classification of whitespace and invisible characters.
//!
//! Mirrored by `src/lib/preview/charset.ts`; both are checked against
//! `tests/fixtures/charset.json`. Keep the two in sync.

/// Display class of a non-printing character. Plain text characters have no class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    Space,
    FullwidthSpace,
    Tab,
    Newline,
    Nbsp,
    ZeroWidth,
    Bidi,
    Control,
}

impl CharClass {
    /// Name used in the shared fixture and by the frontend.
    pub fn fixture_name(self) -> &'static str {
        match self {
            Self::Space => "space",
            Self::FullwidthSpace => "fullwidthSpace",
            Self::Tab => "tab",
            Self::Newline => "newline",
            Self::Nbsp => "nbsp",
            Self::ZeroWidth => "zeroWidth",
            Self::Bidi => "bidi",
            Self::Control => "control",
        }
    }

    pub fn from_fixture_name(name: &str) -> Option<Self> {
        [
            Self::Space,
            Self::FullwidthSpace,
            Self::Tab,
            Self::Newline,
            Self::Nbsp,
            Self::ZeroWidth,
            Self::Bidi,
            Self::Control,
        ]
        .into_iter()
        .find(|c| c.fixture_name() == name)
    }
}

/// Bidirectional control characters that the `EncodingNotice` warning reacts to (5.7).
/// This is a subset of `CharClass::Bidi`: the marks U+200E/U+200F are displayed but not warned.
pub fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// C0 (except tab / LF / CR), DEL, C1, or the replacement character (5.5).
pub fn is_control_or_binary(c: char) -> bool {
    matches!(c, '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' | '\u{7f}'..='\u{9f}' | '\u{fffd}')
}

/// Classify one character. `None` means ordinary visible text.
pub fn classify(c: char) -> Option<CharClass> {
    Some(match c {
        ' ' => CharClass::Space,
        '\u{3000}' => CharClass::FullwidthSpace,
        '\t' => CharClass::Tab,
        '\n' | '\r' => CharClass::Newline,
        '\u{a0}' | '\u{202f}' => CharClass::Nbsp,
        '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{feff}' => CharClass::ZeroWidth,
        '\u{200e}' | '\u{200f}' => CharClass::Bidi,
        c if is_bidi_control(c) => CharClass::Bidi,
        c if is_control_or_binary(c) => CharClass::Control,
        _ => return None,
    })
}
