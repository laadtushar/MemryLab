pub mod obsidian;
pub mod markdown;
pub mod dayone;
pub mod generic;
pub mod parse_utils;
pub mod registry;

// Platform adapters
pub mod reddit;
pub mod linkedin;
pub mod facebook;
pub mod instagram;
pub mod google_takeout;
pub mod twitter;
pub mod whatsapp;
pub mod telegram;
pub mod discord;
pub mod snapchat;
pub mod tiktok;
pub mod youtube;
pub mod pinterest;
pub mod spotify;
pub mod apple;
pub mod amazon;
pub mod netflix;
pub mod slack;
pub mod signal;
pub mod evernote;
pub mod microsoft;
pub mod mastodon;
pub mod threads;
pub mod bluesky;
pub mod substack;
pub mod medium;
pub mod tumblr;
pub mod notion;

use serde::{Deserialize, Serialize};

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

/// Metadata a source adapter publishes for registry and frontend display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceAdapterMeta {
    pub id: String,
    pub display_name: String,
    pub icon: String,
    pub takeout_url: Option<String>,
    pub instructions: String,
    pub accepted_extensions: Vec<String>,
    pub handles_zip: bool,
    pub platform: SourcePlatform,
}

/// Trait every source adapter implements.
pub trait SourceAdapter: Send + Sync {
    /// Rich metadata for self-registration and frontend display.
    fn metadata(&self) -> SourceAdapterMeta;

    /// Given a list of relative file paths inside a ZIP or directory,
    /// return a confidence score (0.0–1.0) that this adapter can handle it.
    fn detect(&self, file_listing: &[&str]) -> f32;

    /// Parse the source at the given path into Documents.
    /// Path may be a file, directory, or extracted ZIP root.
    fn parse(&self, path: &std::path::Path) -> Result<Vec<Document>, AppError>;

    /// Human-readable name (derived from metadata for backward compat).
    fn name(&self) -> &str;
}
