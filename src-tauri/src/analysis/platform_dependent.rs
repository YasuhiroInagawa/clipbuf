//! "機種依存文字" detection: CP932 vendor extensions.
//!
//! A character is platform dependent when its Shift_JIS encoding lands in one of the
//! Windows-31J extension areas: NEC row 13 (0x8740–0x879E), NEC-selected IBM extensions
//! (0xED40–0xEEFC) or IBM extensions (0xFA40–0xFC4B). Characters that Shift_JIS cannot
//! encode at all (é, emoji) are deliberately not flagged: they are not "vendor dependent",
//! and flagging them would drown Latin text in false positives (see research.md).

use encoding_rs::SHIFT_JIS;

fn in_extension_area(bytes: &[u8]) -> bool {
    let [lead, trail] = bytes else { return false };
    let code = u16::from_be_bytes([*lead, *trail]);
    matches!(code, 0x8740..=0x879E | 0xED40..=0xEEFC | 0xFA40..=0xFC4B)
}

/// Contains at least one CP932 vendor-extension character (5.4).
pub fn has_platform_dependent(text: &str) -> bool {
    let mut buf = [0u8; 4];
    text.chars().filter(|c| !c.is_ascii()).any(|c| {
        let (bytes, _, had_errors) = SHIFT_JIS.encode(c.encode_utf8(&mut buf));
        !had_errors && in_extension_area(&bytes)
    })
}
