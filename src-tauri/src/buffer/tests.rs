use super::*;
use crate::model::Warning;

fn push_text(buf: &mut Buffer, text: &str) -> PushResult {
    buf.push(text.to_string(), None, None, Vec::new())
}

fn texts(buf: &Buffer) -> Vec<&str> {
    buf.items().map(|i| i.text.as_str()).collect()
}

#[test]
fn items_are_returned_newest_first() {
    let mut buf = Buffer::new(5);
    push_text(&mut buf, "a");
    push_text(&mut buf, "b");
    push_text(&mut buf, "c");
    assert_eq!(texts(&buf), vec!["c", "b", "a"]);
    assert_eq!(buf.len(), 3);
}

#[test]
fn push_stores_style_data_and_warnings() {
    let mut buf = Buffer::new(5);
    let id = match buf.push(
        "t".into(),
        Some("<b>t</b>".into()),
        Some("{\\rtf1 t}".into()),
        vec![Warning::HasStyle],
    ) {
        PushResult::Pushed(id) => id,
        PushResult::Duplicate => panic!("expected Pushed"),
    };
    let item = buf.get(id).expect("stored");
    assert_eq!(item.html.as_deref(), Some("<b>t</b>"));
    assert_eq!(item.rtf.as_deref(), Some("{\\rtf1 t}"));
    assert!(item.has_style());
    assert_eq!(item.warnings, vec![Warning::HasStyle]);
}

#[test]
fn oldest_item_is_dropped_when_capacity_is_exceeded() {
    let mut buf = Buffer::new(2);
    push_text(&mut buf, "a");
    push_text(&mut buf, "b");
    push_text(&mut buf, "c");
    assert_eq!(texts(&buf), vec!["c", "b"]);
    assert_eq!(buf.len(), 2);
}

#[test]
fn same_text_as_the_newest_item_is_a_duplicate() {
    let mut buf = Buffer::new(5);
    assert!(matches!(push_text(&mut buf, "a"), PushResult::Pushed(_)));
    assert!(matches!(push_text(&mut buf, "a"), PushResult::Duplicate));
    assert_eq!(buf.len(), 1);
    // style data is not part of the comparison (1.6)
    assert!(matches!(
        buf.push(
            "a".into(),
            Some("<b>a</b>".into()),
            None,
            vec![Warning::HasStyle]
        ),
        PushResult::Duplicate
    ));
    // same text as an older, non-newest item is not a duplicate
    push_text(&mut buf, "b");
    assert!(matches!(push_text(&mut buf, "a"), PushResult::Pushed(_)));
    assert_eq!(texts(&buf), vec!["a", "b", "a"]);
}

#[test]
fn ids_are_monotonic_and_never_reused() {
    let mut buf = Buffer::new(2);
    let PushResult::Pushed(a) = push_text(&mut buf, "a") else {
        panic!()
    };
    let PushResult::Pushed(b) = push_text(&mut buf, "b") else {
        panic!()
    };
    let PushResult::Pushed(c) = push_text(&mut buf, "c") else {
        panic!()
    }; // evicts a
    assert!(a < b && b < c);
    assert!(buf.remove(c));
    buf.clear();
    let PushResult::Pushed(d) = push_text(&mut buf, "d") else {
        panic!()
    };
    assert!(
        d > c,
        "ids keep increasing after eviction, removal and clear"
    );
    assert!(buf.get(a).is_none());
}

#[test]
fn remove_deletes_only_that_item_and_keeps_order() {
    let mut buf = Buffer::new(5);
    push_text(&mut buf, "a");
    let PushResult::Pushed(b) = push_text(&mut buf, "b") else {
        panic!()
    };
    push_text(&mut buf, "c");
    assert!(buf.remove(b));
    assert_eq!(texts(&buf), vec!["c", "a"]);
    assert!(!buf.remove(b), "removing an unknown id is a no-op");
    assert!(!buf.remove(9999));
}

#[test]
fn clear_removes_everything() {
    let mut buf = Buffer::new(5);
    push_text(&mut buf, "a");
    push_text(&mut buf, "b");
    buf.clear();
    assert_eq!(buf.len(), 0);
    assert!(buf.items().next().is_none());
}

#[test]
fn shrinking_capacity_drops_the_oldest_items() {
    let mut buf = Buffer::new(5);
    for t in ["a", "b", "c", "d", "e"] {
        push_text(&mut buf, t);
    }
    buf.set_capacity(2);
    assert_eq!(texts(&buf), vec!["e", "d"]);
    assert_eq!(buf.capacity(), 2);
    // growing keeps items and allows more
    buf.set_capacity(3);
    push_text(&mut buf, "f");
    assert_eq!(texts(&buf), vec!["f", "e", "d"]);
}

#[test]
fn capacity_is_at_least_one() {
    let mut buf = Buffer::new(0);
    assert_eq!(buf.capacity(), 1);
    push_text(&mut buf, "a");
    push_text(&mut buf, "b");
    assert_eq!(texts(&buf), vec!["b"]);
    buf.set_capacity(0);
    assert_eq!(buf.capacity(), 1);
}

#[test]
fn captured_at_is_set_at_push_time() {
    let before = std::time::SystemTime::now();
    let mut buf = Buffer::new(1);
    let PushResult::Pushed(id) = push_text(&mut buf, "a") else {
        panic!()
    };
    let at = buf.get(id).unwrap().captured_at;
    assert!(at >= before && at <= std::time::SystemTime::now());
}
