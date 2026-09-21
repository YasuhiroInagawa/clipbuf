//! Pure text transformations applied at transfer time. No OS or Tauri dependencies.
//!
//! `apply` implements requirements 7.7–7.12. `keep_style` is not a text transform and is
//! handled by the transfer command, not here.

use crate::model::{NewlineMode, TransferOptions};

/// Apply the enabled text transformations to `text` and return the new string.
///
/// Order (7.12): newline handling → tabs to spaces → fullwidth space to space → trim.
/// The input is never modified; when no text transform is enabled the result equals the input.
pub fn apply(text: &str, options: &TransferOptions, tab_width: u8) -> String {
    if !options.has_text_transform() {
        return text.to_string();
    }
    let mut out = match options.newline {
        NewlineMode::Keep => text.to_string(),
        NewlineMode::Remove => replace_newlines(text, ""),
        NewlineMode::Space => replace_newlines(text, " "),
    };
    if options.tabs_to_spaces {
        out = out.replace('\t', &" ".repeat(usize::from(tab_width)));
    }
    if options.fullwidth_to_space {
        out = out.replace('\u{3000}', " ");
    }
    if options.trim {
        out = out.trim().to_string();
    }
    out
}

/// Replace every newline (CRLF counted once, then lone LF, then lone CR) with `with`.
fn replace_newlines(text: &str, with: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push_str(with);
            }
            '\n' => out.push_str(with),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests;
