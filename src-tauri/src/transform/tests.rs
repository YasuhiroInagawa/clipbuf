use super::*;
use crate::model::{NewlineMode, TransferOptions};

fn opts() -> TransferOptions {
    TransferOptions::default()
}

#[test]
fn identity_when_no_text_transform_is_enabled() {
    let input = " \ta\r\nb\u{3000}c\n ";
    let keep_style_only = TransferOptions {
        keep_style: true,
        ..opts()
    };
    assert!(!keep_style_only.has_text_transform());
    assert_eq!(apply(input, &opts(), 4), input);
    assert_eq!(apply(input, &keep_style_only, 4), input);
}

#[test]
fn newline_remove_drops_every_newline_convention() {
    let o = TransferOptions {
        newline: NewlineMode::Remove,
        ..opts()
    };
    assert_eq!(apply("a\r\nb\nc\rd\r\n", &o, 4), "abcd");
    assert_eq!(apply("no newline", &o, 4), "no newline");
}

#[test]
fn newline_space_replaces_each_newline_with_one_space() {
    let o = TransferOptions {
        newline: NewlineMode::Space,
        ..opts()
    };
    assert_eq!(apply("a\r\nb\nc\rd", &o, 4), "a b c d");
    // CRLF counts as one newline, not two spaces (7.8)
    assert_eq!(apply("a\r\n\r\nb", &o, 4), "a  b");
}

#[test]
fn tabs_become_the_configured_number_of_spaces() {
    let o = TransferOptions {
        tabs_to_spaces: true,
        ..opts()
    };
    assert_eq!(apply("a\tb\t\tc", &o, 4), "a    b        c");
    assert_eq!(apply("a\tb", &o, 2), "a  b");
    assert_eq!(apply("a\tb", &o, 1), "a b");
}

#[test]
fn fullwidth_spaces_become_single_halfwidth_spaces() {
    let o = TransferOptions {
        fullwidth_to_space: true,
        ..opts()
    };
    assert_eq!(apply("a\u{3000}b\u{3000}\u{3000}c", &o, 4), "a b  c");
}

#[test]
fn trim_removes_leading_and_trailing_whitespace_and_newlines() {
    let o = TransferOptions {
        trim: true,
        ..opts()
    };
    assert_eq!(apply(" \t\u{3000}\u{a0}abc\r\n\n ", &o, 4), "abc");
    // interior whitespace is untouched
    assert_eq!(apply(" a  b ", &o, 4), "a  b");
    assert_eq!(apply("   ", &o, 4), "");
}

#[test]
fn transforms_apply_in_the_documented_order() {
    // newline → tabs → fullwidth → trim (7.12)
    let all = TransferOptions {
        keep_style: false,
        newline: NewlineMode::Space,
        trim: true,
        tabs_to_spaces: true,
        fullwidth_to_space: true,
    };
    // Every transform maps whitespace to whitespace or nothing, so the order is not
    // observable from the output; it is fixed by design (7.12) and verified here only as the
    // combined result of all options on one input.
    assert_eq!(apply("\ta\r\nb\u{3000}", &all, 4), "a b");

    // With Remove instead of Space, the newline vanishes and words join.
    let remove = TransferOptions {
        newline: NewlineMode::Remove,
        ..all
    };
    assert_eq!(apply("\ta\r\nb\u{3000}", &remove, 4), "ab");

    // Newline → Space happens before trim, so a lone trailing newline is trimmed away.
    let space_trim = TransferOptions {
        newline: NewlineMode::Space,
        trim: true,
        ..opts()
    };
    assert_eq!(apply("x\n", &space_trim, 4), "x");
}

#[test]
fn returns_a_new_string_and_leaves_the_input_untouched() {
    let input = String::from("a\tb\n");
    let o = TransferOptions {
        tabs_to_spaces: true,
        newline: NewlineMode::Remove,
        ..opts()
    };
    let out = apply(&input, &o, 2);
    assert_eq!(out, "a  b");
    assert_eq!(input, "a\tb\n");
}
