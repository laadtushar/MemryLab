use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::domain::models::common::SourcePlatform;
use crate::domain::models::document::Document;
use crate::error::AppError;

use super::{SourceAdapter, SourceAdapterMeta};
use super::parse_utils;

pub struct SpotifyAdapter;

impl SourceAdapter for SpotifyAdapter {
    fn metadata(&self) -> SourceAdapterMeta {
        SourceAdapterMeta {
            id: "spotify".into(),
            display_name: "Spotify".into(),
            icon: "music-2".into(),
            takeout_url: Some("https://www.spotify.com/account/privacy/".into()),
            instructions: "Request your data from Spotify (Account > Privacy > Download your data). Upload the ZIP.".into(),
            accepted_extensions: vec!["zip".into(), "json".into()],
            handles_zip: true,
            platform: SourcePlatform::Spotify,
        }
    }

    fn detect(&self, file_listing: &[&str]) -> f32 {
        let has_streaming = file_listing.iter().any(|f| f.contains("StreamingHistory"));
        if has_streaming { 0.9 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "spotify"
    }

    fn parse(&self, path: &Path) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        let json_files: Vec<_> = if path.is_dir() {
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .map(|e| e.path().to_path_buf())
                .collect()
        } else {
            vec![path.to_path_buf()]
        };

        for json_path in json_files {
            let file_name = json_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            if !file_name.contains("StreamingHistory") && !file_name.contains("Playlist") {
                continue;
            }

            let content = match std::fs::read_to_string(&json_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(arr) = value.as_array() {
                for item in arr {
                    let artist = item
                        .get("artistName")
                        .or_else(|| item.get("master_metadata_album_artist_name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let track = item
                        .get("trackName")
                        .or_else(|| item.get("master_metadata_track_name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    if artist.is_empty() && track.is_empty() {
                        continue;
                    }

                    let text = if artist.is_empty() {
                        track.to_string()
                    } else if track.is_empty() {
                        artist.to_string()
                    } else {
                        format!("{} - {}", artist, track)
                    };

                    let timestamp = item
                        .get("endTime")
                        .or_else(|| item.get("ts"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| {
                            chrono::DateTime::parse_from_rfc3339(s)
                                .ok()
                                .map(|dt| dt.with_timezone(&Utc))
                                .or_else(|| {
                                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
                                        .ok()
                                        .map(|dt| dt.and_utc())
                                })
                        })
                        ;

                    let ms_played = item
                        .get("msPlayed")
                        .or_else(|| item.get("ms_played"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    let mut meta = serde_json::Map::new();
                    meta.insert("artist".into(), serde_json::Value::String(artist.into()));
                    meta.insert("track".into(), serde_json::Value::String(track.into()));
                    meta.insert("ms_played".into(), serde_json::Value::Number(ms_played.into()));

                    documents.push(parse_utils::build_document(
                        text,
                        SourcePlatform::Spotify,
                        timestamp,
                        vec![],
                        serde_json::Value::Object(meta),
                    ));
                }
            }
        }

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_spotify() {
        let adapter = SpotifyAdapter;
        let files = vec!["StreamingHistory0.json", "Userdata.json"];
        assert!(adapter.detect(&files) >= 0.9);
    }

    #[test]
    fn test_detect_no_match() {
        let adapter = SpotifyAdapter;
        assert!(adapter.detect(&["random.json"]) < 0.1);
    }

    #[test]
    fn test_metadata() {
        let adapter = SpotifyAdapter;
        let meta = adapter.metadata();
        assert_eq!(meta.id, "spotify");
        assert!(meta.takeout_url.is_some());
    }
}
