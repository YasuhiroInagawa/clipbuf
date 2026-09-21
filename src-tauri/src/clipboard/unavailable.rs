//! Fallback port used when no clipboard can be opened (no display server, init failure).
//! Every operation fails with `ClipError::Unavailable`; watching is a silent no-op.

use std::sync::mpsc::Sender;

use super::{ClipError, ClipboardEvent, ClipboardPort, WritePayload};
use crate::model::{CaptureCapability, ClipboardSnapshot};

pub struct UnavailableClipboard;

impl ClipboardPort for UnavailableClipboard {
    fn start_watch(&self, _sender: Sender<ClipboardEvent>) -> Result<(), ClipError> {
        Ok(())
    }

    fn read(&self) -> Result<ClipboardSnapshot, ClipError> {
        Err(ClipError::Unavailable)
    }

    fn write(&self, _payload: WritePayload<'_>) -> Result<(), ClipError> {
        Err(ClipError::Unavailable)
    }

    fn capability(&self) -> CaptureCapability {
        CaptureCapability::Unavailable
    }
}
