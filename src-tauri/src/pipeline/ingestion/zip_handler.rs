use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Extract a ZIP file to a temporary directory.
/// Returns (temp_dir_path, flat list of relative file paths inside).
pub fn extract_zip(zip_path: &Path) -> Result<(PathBuf, Vec<String>), AppError> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Other(format!("Failed to open ZIP: {}", e)))?;

    let temp_dir = std::env::temp_dir().join(format!(
        "mempalace_import_{}",
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("tmp")
    ));
    fs::create_dir_all(&temp_dir)?;

    let mut file_listing = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::Other(format!("ZIP entry error: {}", e)))?;

        let name = entry.name().to_string();

        // Skip directories and hidden/system files
        if entry.is_dir() || name.starts_with("__MACOSX") || name.contains(".DS_Store") {
            continue;
        }

        // Skip non-text files (media, binaries) to save disk space
        let lower = name.to_lowercase();
        let is_text = lower.ends_with(".json")
            || lower.ends_with(".csv")
            || lower.ends_with(".txt")
            || lower.ends_with(".html")
            || lower.ends_with(".htm")
            || lower.ends_with(".md")
            || lower.ends_with(".xml")
            || lower.ends_with(".enex")
            || lower.ends_with(".js")
            || lower.ends_with(".mbox")
            || lower.ends_with(".car")
            || lower.ends_with(".signal")
            || lower.ends_with(".ics")
            || lower.ends_with(".vcf");

        if !is_text {
            // Still add to listing for detection, but don't extract
            file_listing.push(name);
            continue;
        }

        let out_path = temp_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut out_file = fs::File::create(&out_path)?;
        io::copy(&mut entry, &mut out_file)?;
        file_listing.push(name);
    }

    Ok((temp_dir, file_listing))
}

/// List files inside a ZIP without extracting (fast peek for detection).
pub fn list_zip_contents(zip_path: &Path) -> Result<Vec<String>, AppError> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Other(format!("Failed to open ZIP: {}", e)))?;

    let mut listing = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index_raw(i) {
            let name = entry.name().to_string();
            if !entry.is_dir() && !name.starts_with("__MACOSX") {
                listing.push(name);
            }
        }
    }

    Ok(listing)
}

/// List files in a directory recursively.
pub fn list_dir_contents(dir_path: &Path) -> Result<Vec<String>, AppError> {
    let mut listing = Vec::new();
    let base = dir_path.to_path_buf();

    for entry in walkdir::WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Ok(relative) = entry.path().strip_prefix(&base) {
            listing.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }

    Ok(listing)
}

/// Clean up a temporary extraction directory.
pub fn cleanup_temp_dir(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_dir_contents() {
        // Create a small temp structure
        let dir = std::env::temp_dir().join("mp_test_list_dir");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("file1.txt"), "hello").unwrap();
        fs::write(dir.join("sub/file2.json"), "{}").unwrap();

        let listing = list_dir_contents(&dir).unwrap();
        assert!(listing.len() >= 2);
        assert!(listing.iter().any(|f| f.contains("file1.txt")));
        assert!(listing.iter().any(|f| f.contains("file2.json")));

        let _ = fs::remove_dir_all(&dir);
    }
}
