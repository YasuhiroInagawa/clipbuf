use serde::{Deserialize, Serialize};

/// Paste-hazard warnings attached to a captured item (requirements 5.1–5.7).
///
/// Declaration order is the display order; `analysis::analyze` returns them in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Warning {
    /// HTML / RTF data accompanies the text (5.1).
    HasStyle,
    /// Leading or trailing whitespace or newline (5.2).
    EdgeWhitespace,
    /// Contains a tab character (5.3).
    HasTab,
    /// Contains CP932 vendor-extension characters (5.4).
    PlatformDependent,
    /// Contains control characters or replacement characters (5.5).
    ControlOrBinary,
    /// Mixes CRLF / LF / CR (5.6).
    MixedNewlines,
    /// BOM, bidi controls, or mixed NFC/NFD (5.7).
    EncodingNotice,
}
