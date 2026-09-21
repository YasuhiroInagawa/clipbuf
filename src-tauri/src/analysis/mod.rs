//! Pure warning analysis over captured text. No OS or Tauri dependencies.
//!
//! One warning, one function. `analyze` combines them in `Warning` declaration order.

mod charset;
mod control;
mod edge;
mod encoding_notice;
mod newline;
mod platform_dependent;

pub use charset::{CharClass, classify};
pub use control::has_control_or_binary;
pub use edge::has_edge_whitespace;
pub use encoding_notice::has_encoding_notice;
pub use newline::has_mixed_newlines;
pub use platform_dependent::has_platform_dependent;

use crate::model::Warning;

/// Contains a tab character (5.3).
pub fn has_tab(text: &str) -> bool {
    text.contains('\t')
}

/// All warnings that apply to `text`, in `Warning` declaration order, without duplicates.
pub fn analyze(text: &str, has_style: bool) -> Vec<Warning> {
    let checks: [(Warning, bool); 7] = [
        (Warning::HasStyle, has_style),
        (Warning::EdgeWhitespace, has_edge_whitespace(text)),
        (Warning::HasTab, has_tab(text)),
        (Warning::PlatformDependent, has_platform_dependent(text)),
        (Warning::ControlOrBinary, has_control_or_binary(text)),
        (Warning::MixedNewlines, has_mixed_newlines(text)),
        (Warning::EncodingNotice, has_encoding_notice(text)),
    ];
    checks
        .into_iter()
        .filter_map(|(w, hit)| hit.then_some(w))
        .collect()
}

#[cfg(test)]
mod tests;
