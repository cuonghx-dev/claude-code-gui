use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Identifies a long-running async request returned to the frontend so it can
/// correlate progress events emitted on `<topic>:{request_id}`.
#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(transparent)]
pub struct RequestId(#[ts(type = "string")] pub Uuid);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Identifies a PTY terminal session managed by `crates/pty::PtyManager`.
#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(transparent)]
pub struct SessionId(#[ts(type = "string")] pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
