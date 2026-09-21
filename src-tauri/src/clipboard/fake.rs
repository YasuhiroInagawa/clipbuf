//! Test double for `ClipboardPort`. Lets tests inject clipboard changes and inspect writes.

use std::sync::Mutex;
use std::sync::mpsc::Sender;

use super::{ClipError, ClipboardEvent, ClipboardPort, WritePayload};
use crate::model::{CaptureCapability, ClipboardSnapshot};

/// An owned copy of a `WritePayload`, recorded by `FakeClipboard::write`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenPayload {
    pub text: String,
    pub html: Option<String>,
    pub rtf: Option<String>,
}

#[derive(Debug)]
pub struct FakeClipboard {
    capability: CaptureCapability,
    current: Mutex<ClipboardSnapshot>,
    sender: Mutex<Option<Sender<ClipboardEvent>>>,
    written: Mutex<Vec<WrittenPayload>>,
    fail_reads: Mutex<bool>,
    fail_writes: Mutex<bool>,
}

impl FakeClipboard {
    pub fn new(capability: CaptureCapability) -> Self {
        Self {
            capability,
            current: Mutex::new(ClipboardSnapshot::default()),
            sender: Mutex::new(None),
            written: Mutex::new(Vec::new()),
            fail_reads: Mutex::new(false),
            fail_writes: Mutex::new(false),
        }
    }

    /// Simulate another application changing the clipboard.
    pub fn push_change(&self, snapshot: ClipboardSnapshot) {
        *self.current.lock().unwrap() = snapshot;
        self.notify();
    }

    /// Everything written through `write`, oldest first.
    pub fn written(&self) -> Vec<WrittenPayload> {
        self.written.lock().unwrap().clone()
    }

    pub fn fail_reads(&self, fail: bool) {
        *self.fail_reads.lock().unwrap() = fail;
    }

    pub fn fail_writes(&self, fail: bool) {
        *self.fail_writes.lock().unwrap() = fail;
    }

    fn notify(&self) {
        if let Some(sender) = self.sender.lock().unwrap().as_ref() {
            // A disconnected receiver only means the capture service is gone; ignore.
            let _ = sender.send(ClipboardEvent::Changed);
        }
    }
}

impl ClipboardPort for FakeClipboard {
    fn start_watch(&self, sender: Sender<ClipboardEvent>) -> Result<(), ClipError> {
        let mut slot = self.sender.lock().unwrap();
        if slot.is_some() {
            return Err(ClipError::WatchAlreadyStarted);
        }
        *slot = Some(sender);
        Ok(())
    }

    fn read(&self) -> Result<ClipboardSnapshot, ClipError> {
        if *self.fail_reads.lock().unwrap() {
            return Err(ClipError::Read);
        }
        Ok(self.current.lock().unwrap().clone())
    }

    fn write(&self, payload: WritePayload<'_>) -> Result<(), ClipError> {
        if *self.fail_writes.lock().unwrap() {
            return Err(ClipError::Write);
        }
        let written = WrittenPayload {
            text: payload.text.to_string(),
            html: payload.html.map(str::to_string),
            rtf: payload.rtf.map(str::to_string),
        };
        // Like a real clipboard: the write becomes the current content (with our marker)
        // and watchers are notified.
        *self.current.lock().unwrap() = ClipboardSnapshot {
            text: Some(written.text.clone()),
            html: written.html.clone(),
            rtf: written.rtf.clone(),
            concealed: false,
            own_marker: true,
        };
        self.written.lock().unwrap().push(written);
        self.notify();
        Ok(())
    }

    fn capability(&self) -> CaptureCapability {
        self.capability
    }
}
