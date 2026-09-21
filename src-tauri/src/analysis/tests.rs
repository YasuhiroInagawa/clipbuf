use super::*;
use crate::model::Warning;
use serde_json::Value;

const CHARSET: &str = include_str!("../../../tests/fixtures/charset.json");

fn parse_cp(s: &str) -> char {
    let hex = s.strip_prefix("U+").expect("U+XXXX");
    char::from_u32(u32::from_str_radix(hex, 16).unwrap()).unwrap()
}

#[test]
fn charset_classification_matches_shared_fixture() {
    let root: Value = serde_json::from_str(CHARSET).unwrap();
    for (name, cps) in root["classes"].as_object().unwrap() {
        let expected =
            CharClass::from_fixture_name(name).unwrap_or_else(|| panic!("unknown class {name}"));
        for cp in cps.as_array().unwrap() {
            let c = parse_cp(cp.as_str().unwrap());
            assert_eq!(classify(c), Some(expected), "{cp} should be {name}");
        }
    }
    for cp in root["text"].as_array().unwrap() {
        let c = parse_cp(cp.as_str().unwrap());
        assert_eq!(classify(c), None, "{cp} should be plain text");
    }
}

#[test]
fn edge_whitespace() {
    for (text, expected) in [
        ("abc", false),
        (" abc", true),
        ("abc ", true),
        ("abc\n", true),
        ("\u{3000}abc", true),
        ("abc\u{a0}", true),
        ("\tabc", true),
        ("a b", false),
        ("", false),
    ] {
        assert_eq!(has_edge_whitespace(text), expected, "{text:?}");
    }
}

#[test]
fn tab() {
    assert!(has_tab("a\tb"));
    assert!(!has_tab("a b"));
    assert!(!has_tab(""));
}

#[test]
fn platform_dependent() {
    for (text, expected) in [
        ("\u{2460}", true),   // ① NEC row 13
        ("\u{3231}", true),   // ㈱ NEC row 13
        ("\u{9ad9}", true),   // 髙 IBM extension
        ("\u{fa11}", true),   // 﨑 IBM extension
        ("\u{2252}", false),  // ≒ exists in JIS X 0208 row 2 (also NEC row 13) → standard wins
        ("\u{00e9}", false),  // é not encodable in Shift_JIS
        ("\u{1f600}", false), // 😀 not encodable
        ("\u{6f22}", false),  // 漢 JIS X 0208
        ("abc", false),
        ("", false),
    ] {
        assert_eq!(has_platform_dependent(text), expected, "{text:?}");
    }
}

#[test]
fn control_or_binary() {
    for (text, expected) in [
        ("a\tb\nc\r\n", false),
        ("a\u{0}b", true),
        ("a\u{1b}[0m", true),
        ("a\u{7f}", true),
        ("a\u{85}b", true),
        ("a\u{fffd}b", true),
        ("plain", false),
    ] {
        assert_eq!(has_control_or_binary(text), expected, "{text:?}");
    }
}

#[test]
fn mixed_newlines() {
    for (text, expected) in [
        ("a\r\nb\r\n", false),
        ("a\nb\n", false),
        ("a\rb\r", false),
        ("a\r\nb\n", true),
        ("a\nb\r", true),
        ("a\r\nb\rc", true),
        ("no newline", false),
        ("", false),
    ] {
        assert_eq!(has_mixed_newlines(text), expected, "{text:?}");
    }
}

#[test]
fn encoding_notice() {
    for (text, expected) in [
        ("\u{feff}abc", true),       // BOM
        ("abc\u{feff}", false),      // ZWNBSP not at start is not a BOM
        ("a\u{202e}b", true),        // RLO
        ("a\u{2066}b", true),        // LRI
        ("\u{e9}", false),           // pure NFC
        ("e\u{301}", false),         // pure NFD
        ("\u{e9} e\u{301}", true),   // NFC + NFD mixed
        ("\u{30ac}", false),         // ガ NFC
        ("\u{30ab}\u{3099}", false), // カ + combining dakuten: pure NFD
        ("\u{30ac}\u{30ab}\u{3099}", true),
        ("plain ascii", false),
        ("", false),
    ] {
        assert_eq!(has_encoding_notice(text), expected, "{text:?}");
    }
}

#[test]
fn analyze_returns_declaration_order_without_duplicates() {
    let text = "\u{feff}x\t\u{2460}\u{0}\r\ny\n "; // BOM, tab, platform, control, mixed newlines, trailing space
    let warnings = analyze(text, true);
    assert_eq!(
        warnings,
        vec![
            Warning::HasStyle,
            Warning::EdgeWhitespace,
            Warning::HasTab,
            Warning::PlatformDependent,
            Warning::ControlOrBinary,
            Warning::MixedNewlines,
            Warning::EncodingNotice,
        ]
    );
    assert!(analyze("clean", false).is_empty());
    assert_eq!(analyze("clean", true), vec![Warning::HasStyle]);
}
