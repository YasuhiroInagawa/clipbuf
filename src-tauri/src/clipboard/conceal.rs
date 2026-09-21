//! "Do not record this" markers set by password managers and similar apps (requirement 1.5).
//!
//! Each is an extra clipboard format whose presence, not content, carries the meaning.
//! clipbuf only detects them; it never sets them.

/// Known conceal markers: macOS (nspasteboard.org convention), Windows (registered format
/// honoured by Cloud Clipboard / history), KDE (Klipper hint).
pub const CONCEALED_FORMATS: [&str; 3] = [
    "org.nspasteboard.ConcealedType",
    "ExcludeClipboardContentFromMonitorProcessing",
    "x-kde-passwordManagerHint",
];

/// True when any conceal marker is among `formats`.
pub fn has_concealed_format<'a>(formats: impl IntoIterator<Item = &'a str>) -> bool {
    formats.into_iter().any(|f| CONCEALED_FORMATS.contains(&f))
}
