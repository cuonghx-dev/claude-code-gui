pub mod agents;
pub mod claude_cli;
pub mod claude_dir;
pub mod commands;
pub mod files;
pub mod frontmatter;
pub mod git;
pub mod hooks;
pub mod io;
pub mod marketplace;
pub mod mcp;
pub mod mcp_probe;
pub mod models;
pub mod relationships;
pub mod output_styles;
pub mod plans;
pub mod plugins;
pub mod projects;
pub mod sessions;
pub mod settings;
pub mod setup;
pub mod skills;
pub mod types;

pub use types::{AppError, ErrorCode, RequestId, SessionId};

pub type Result<T, E = AppError> = std::result::Result<T, E>;
