use super::*;
use crate::model::{ErrorKind, Language, NewlineMode, SETTINGS_VERSION, Settings, TransferOptions};
use serde_json::json;

fn temp_path(name: &str) -> std::path::PathBuf {
    let dir = tempfile::tempdir().expect("tempdir");
    // Keep the directory alive for the test by leaking it; tests are short-lived.
    let path = dir.path().join(name);
    std::mem::forget(dir);
    path
}

// ---- validate ----------------------------------------------------------------------------

#[test]
fn default_settings_are_valid() {
    assert_eq!(validate(&Settings::default()), Ok(()));
}

#[test]
fn validate_rejects_values_outside_their_ranges() {
    let base = Settings::default();
    let cases = [
        Settings {
            capacity: 0,
            ..base.clone()
        },
        Settings {
            capacity: 201,
            ..base.clone()
        },
        Settings {
            tab_width: 0,
            ..base.clone()
        },
        Settings {
            tab_width: 17,
            ..base.clone()
        },
        Settings {
            poll_interval_ms: 49,
            ..base.clone()
        },
        Settings {
            poll_interval_ms: 2001,
            ..base.clone()
        },
        Settings {
            hotkey: String::new(),
            ..base.clone()
        },
        Settings {
            hotkey: "   ".into(),
            ..base.clone()
        },
    ];
    for s in cases {
        assert_eq!(validate(&s), Err(ErrorKind::InvalidSettings), "{s:?}");
    }
    let edges = Settings {
        capacity: 200,
        tab_width: 16,
        poll_interval_ms: 2000,
        ..base
    };
    assert_eq!(validate(&edges), Ok(()));
}

// ---- sanitize ----------------------------------------------------------------------------

#[test]
fn sanitize_fills_missing_keys_with_defaults() {
    let s = sanitize(json!({ "capacity": 50 }));
    assert_eq!(
        s,
        Settings {
            capacity: 50,
            ..Settings::default()
        }
    );
}

#[test]
fn sanitize_replaces_out_of_range_or_wrong_type_values_with_defaults() {
    let s = sanitize(json!({
        "version": 1,
        "capacity": 9999,
        "hotkey": "",
        "tabWidth": "four",
        "pollIntervalMs": 10,
        "autostart": "yes",
        "language": "klingon",
        "transfer": { "keepStyle": true, "newline": "bogus" }
    }));
    let d = Settings::default();
    assert_eq!(s.capacity, d.capacity);
    assert_eq!(s.hotkey, d.hotkey);
    assert_eq!(s.tab_width, d.tab_width);
    assert_eq!(s.poll_interval_ms, d.poll_interval_ms);
    assert_eq!(s.autostart, d.autostart);
    assert_eq!(s.language, None);
    // A partially valid nested object keeps its valid fields.
    assert!(s.transfer.keep_style);
    assert_eq!(s.transfer.newline, NewlineMode::Keep);
}

#[test]
fn sanitize_ignores_unknown_keys_and_non_objects() {
    let s = sanitize(json!({ "capacity": 7, "someFutureKey": [1, 2, 3] }));
    assert_eq!(s.capacity, 7);
    assert_eq!(sanitize(json!("not an object")), Settings::default());
    assert_eq!(sanitize(json!(null)), Settings::default());
}

#[test]
fn sanitize_always_yields_the_current_schema_version() {
    assert_eq!(sanitize(json!({ "version": 0 })).version, SETTINGS_VERSION);
    assert_eq!(sanitize(json!({})).version, SETTINGS_VERSION);
}

// ---- diff --------------------------------------------------------------------------------

#[test]
fn diff_flags_only_the_fields_that_changed() {
    let a = Settings::default();
    assert_eq!(diff(&a, &a), SettingsDiff::default());

    let b = Settings {
        capacity: 5,
        hotkey: "Ctrl+Shift+V".into(),
        ..a.clone()
    };
    let d = diff(&a, &b);
    assert!(d.capacity && d.hotkey);
    assert!(!(d.autostart || d.language || d.transfer || d.poll_interval || d.tab_width));
    assert!(d.any());

    let c = Settings {
        autostart: true,
        language: Some(Language::Ja),
        poll_interval_ms: 300,
        tab_width: 2,
        transfer: TransferOptions {
            trim: true,
            ..a.transfer
        },
        ..a.clone()
    };
    let d = diff(&a, &c);
    assert!(d.autostart && d.language && d.poll_interval && d.tab_width && d.transfer);
    assert!(!(d.capacity || d.hotkey));
}

// ---- store -------------------------------------------------------------------------------

#[test]
fn open_creates_the_file_with_defaults_when_missing() {
    let path = temp_path("settings.json");
    let store = SettingsStore::open(path.clone()).expect("open");
    assert_eq!(store.get(), Settings::default());
    let on_disk: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(on_disk["version"], SETTINGS_VERSION);
    assert_eq!(on_disk["capacity"], 20);
    assert_eq!(on_disk["transfer"]["keepStyle"], false);
}

#[test]
fn update_validates_persists_and_returns_the_diff() {
    let path = temp_path("settings.json");
    let store = SettingsStore::open(path.clone()).expect("open");

    let next = Settings {
        capacity: 30,
        transfer: TransferOptions {
            keep_style: true,
            newline: NewlineMode::Space,
            trim: true,
            tabs_to_spaces: false,
            fullwidth_to_space: true,
        },
        ..Settings::default()
    };
    let d = store.update(next.clone()).expect("update");
    assert!(d.capacity && d.transfer);
    assert!(!d.hotkey);
    assert_eq!(store.get(), next);

    // A fresh store on the same path sees the persisted settings, toggles included (7.3, 9.2).
    let reopened = SettingsStore::open(path).expect("reopen");
    assert_eq!(reopened.get(), next);
}

#[test]
fn invalid_update_is_rejected_and_nothing_is_written() {
    let path = temp_path("settings.json");
    let store = SettingsStore::open(path.clone()).expect("open");
    let before = std::fs::read_to_string(&path).unwrap();

    let bad = Settings {
        capacity: 0,
        ..Settings::default()
    };
    assert_eq!(
        store.update(bad).unwrap_err().kind,
        ErrorKind::InvalidSettings
    );
    assert_eq!(store.get(), Settings::default());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn open_recovers_from_corrupt_or_partial_files() {
    let path = temp_path("settings.json");
    std::fs::write(&path, "{ this is not json").unwrap();
    let store = SettingsStore::open(path.clone()).expect("open corrupt");
    assert_eq!(store.get(), Settings::default());

    std::fs::write(&path, r#"{"capacity": 3, "hotkey": "Alt+V"}"#).unwrap();
    let store = SettingsStore::open(path).expect("open partial");
    assert_eq!(store.get().capacity, 3);
    assert_eq!(store.get().hotkey, "Alt+V");
    assert_eq!(store.get().tab_width, 4);
}

#[test]
fn open_fails_with_settings_io_when_the_directory_cannot_be_created() {
    // A file where a directory is expected makes create_dir_all fail.
    let blocker = temp_path("blocker");
    std::fs::write(&blocker, "x").unwrap();
    let err = SettingsStore::open(blocker.join("sub").join("settings.json")).unwrap_err();
    assert_eq!(err.kind, ErrorKind::SettingsIo);
}
