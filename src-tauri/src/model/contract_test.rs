//! Cross-language contract tests: the fixture in `tests/fixtures/contract.json` is shared
//! with the frontend (`src/lib/ipc/types.test.ts`). Deserializing and re-serializing every
//! section must reproduce the fixture exactly, which pins the serde representation.
//! Event names are checked in `crate::app::events` to keep this layer free of upward imports.

use super::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

const FIXTURE: &str = include_str!("../../../tests/fixtures/contract.json");

pub(crate) fn section(name: &str) -> Value {
    let root: Value = serde_json::from_str(FIXTURE).expect("fixture is valid JSON");
    root.get(name)
        .unwrap_or_else(|| panic!("fixture has section {name}"))
        .clone()
}

fn round_trip<T: Serialize + DeserializeOwned>(name: &str) -> T {
    let value = section(name);
    let typed: T = serde_json::from_value(value.clone())
        .unwrap_or_else(|e| panic!("section {name} deserializes: {e}"));
    let back = serde_json::to_value(&typed).expect("serializes");
    assert_eq!(back, value, "section {name} must round-trip unchanged");
    typed
}

#[test]
fn item_dto_round_trips() {
    let dto: ItemDto = round_trip("itemDto");
    assert_eq!(dto.id, 42);
    assert_eq!(
        dto.warnings,
        vec![Warning::HasStyle, Warning::EdgeWhitespace, Warning::HasTab]
    );
}

#[test]
fn enums_round_trip() {
    let warnings: Vec<Warning> = round_trip("warnings");
    assert_eq!(warnings.len(), 7);
    let modes: Vec<NewlineMode> = round_trip("newlineModes");
    assert_eq!(
        modes,
        vec![NewlineMode::Keep, NewlineMode::Remove, NewlineMode::Space]
    );
    let transfer: Vec<TransferMode> = round_trip("transferModes");
    assert_eq!(
        transfer,
        vec![
            TransferMode::Options,
            TransferMode::Plain,
            TransferMode::Raw
        ]
    );
    let kinds: Vec<ErrorKind> = round_trip("errorKinds");
    assert_eq!(kinds.len(), 7);
    let caps: Vec<CaptureCapability> = round_trip("captureCapabilities");
    assert_eq!(caps.len(), 4);
    let statuses: Vec<CaptureStatus> = round_trip("captureStatuses");
    assert_eq!(statuses.last(), Some(&CaptureStatus::ReadFailed));
}

#[test]
fn structs_round_trip() {
    let options: TransferOptions = round_trip("transferOptions");
    assert!(options.has_text_transform());
    let settings: Settings = round_trip("settings");
    assert_eq!(settings.language, Some(Language::En));
    let outcome: TransferOutcome = round_trip("transferOutcome");
    assert!(outcome.skipped_transforms);
    let preview: TransferPreview = round_trip("transferPreview");
    assert_eq!(preview.text, "a b");
    assert!(!preview.skipped_transforms);
    let err: AppError = round_trip("appError");
    assert_eq!(err.kind, ErrorKind::HotkeyUnavailable);
    let info: PlatformInfo = round_trip("platformInfo");
    assert_eq!(info.capture, CaptureCapability::Full);
}

#[test]
fn default_settings_match_fixture() {
    let expected = section("defaultSettings");
    let actual = serde_json::to_value(Settings::default()).expect("serializes");
    assert_eq!(actual, expected);
    assert!(!TransferOptions::default().has_text_transform());
}

#[test]
fn capture_status_converts_from_capability() {
    assert_eq!(
        CaptureStatus::from(CaptureCapability::Denied),
        CaptureStatus::Denied
    );
}

#[test]
fn app_error_never_carries_text() {
    let err = AppError::from(ErrorKind::WriteFailed);
    let json = serde_json::to_string(&err).unwrap();
    assert_eq!(json, r#"{"kind":"writeFailed"}"#);
}
