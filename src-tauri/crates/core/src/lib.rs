pub mod agents;
pub mod claude_cli;
pub mod claude_dir;
pub mod commands;
pub mod frontmatter;
pub mod mcp;
pub mod models;
pub mod output_styles;
pub mod plans;
pub mod plugins;
pub mod projects;
pub mod sessions;
pub mod settings;
pub mod skills;
pub mod types;

pub use types::{AppError, ErrorCode, RequestId, SessionId};

pub type Result<T, E = AppError> = std::result::Result<T, E>;
