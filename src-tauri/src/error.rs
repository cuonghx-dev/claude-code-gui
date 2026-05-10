//! Error type for Tauri command boundaries.
//!
//! Tauri commands return `Result<T, AppError>`. `AppError` is defined in
//! `core::types::error` so frontend can consume it via ts-rs. This module
//! exists for `From` impls that are specific to the app crate (e.g.,
//! `tauri::Error`).

use app_core::AppError;

#[allow(dead_code)] // Phase 0; populated as commands land.
pub type Result<T> = std::result::Result<T, AppError>;
