use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A time range for queries and analysis windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Supported source platforms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourcePlatform {
    Obsidian,
    Markdown,
    DayOne,
    WhatsApp,
    Telegram,
    Twitter,
    Instagram,
    Facebook,
    Reddit,
    LinkedIn,
    GoogleTakeout,
    AppleNotes,
    Notion,
    PlainText,
    Custom,
    // Added for 30-platform support
    Discord,
    Snapchat,
    TikTok,
    YouTube,
    Pinterest,
    Spotify,
    Apple,
    Amazon,
    Netflix,
    Slack,
    Signal,
    Evernote,
    Microsoft,
    Mastodon,
    Threads,
    Bluesky,
    Substack,
    Medium,
    Tumblr,
}

impl std::fmt::Display for SourcePlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{:?}", self).to_lowercase());
        write!(f, "{}", s)
    }
}

/// Time granularity for analysis windows (micro/meso/macro)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeGranularity {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

impl TimeGranularity {
    /// Number of days for each granularity level.
    pub fn window_days(&self) -> i64 {
        match self {
            TimeGranularity::Daily => 1,
            TimeGranularity::Weekly => 7,
            TimeGranularity::Monthly => 30,
            TimeGranularity::Quarterly => 90,
            TimeGranularity::Yearly => 365,
        }
    }

    /// Parse from a string, defaulting to Monthly for unknown values.
    pub fn from_str_opt(s: Option<&str>) -> Self {
        match s {
            Some("daily") => TimeGranularity::Daily,
            Some("weekly") => TimeGranularity::Weekly,
            Some("monthly") => TimeGranularity::Monthly,
            Some("quarterly") => TimeGranularity::Quarterly,
            Some("yearly") => TimeGranularity::Yearly,
            _ => TimeGranularity::Monthly,
        }
    }
}

/// A custom time boundary (life event marker).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBoundary {
    pub id: String,
    pub name: String,
    pub date: String,
    pub end_date: Option<String>,
    pub color: Option<String>,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub offset: usize,
    pub limit: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}
