pub mod claude_cli;
pub mod claude_dir;
pub mod frontmatter;
pub mod models;
pub mod types;

pub use types::{AppError, ErrorCode, RequestId, SessionId};

pub type Result<T, E = AppError> = std::result::Result<T, E>;
