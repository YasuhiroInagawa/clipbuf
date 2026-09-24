//! Shared types used across all layers (items, warnings, options, settings, errors).
//!
//! Everything here is serialized to the frontend as camelCase JSON. The exact wire
//! representation is pinned by `tests/fixtures/contract.json`, which is shared with the
//! TypeScript types in `src/lib/ipc/types.ts`.

mod error;
mod item;
mod options;
mod platform;
mod settings;
mod warning;

pub use error::{AppError, ErrorKind};
pub use item::{ClipItem, ClipboardSnapshot, ItemDto, ItemId, TransferOutcome, TransferPreview};
pub use options::{NewlineMode, TransferMode, TransferOptions};
pub use platform::{CaptureCapability, CaptureStatus, PlatformInfo};
pub use settings::{
    CAPACITY_RANGE, Language, POLL_INTERVAL_MS_RANGE, SETTINGS_VERSION, Settings, TAB_WIDTH_RANGE,
};
pub use warning::Warning;

#[cfg(test)]
pub(crate) mod contract_test;
