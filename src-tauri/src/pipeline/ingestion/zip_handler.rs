use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Known binary extensions that should NOT be extracted (media, executables, etc.)
const BINARY_EXTENSIONS: &[&str] = &[
    // Images
    "jpg", "jpeg", "png", "gif", "bmp", "ico", "svg", "webp", "tiff", "tif", "heic", "heif", "raw", "cr2", "nef", "avif",
    // Video
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "3gp", "mpg", "mpeg",
    // Audio
    "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a", "opus", "aiff",
    // Archives (nested zips)
    "zip", "tar", "gz", "bz2", "7z", "rar", "xz", "zst",
    // Executables / binaries
    "exe", "dll", "so", "dylib", "bin", "dat", "o", "class", "pyc", "pyd",
    // Fonts
    "ttf", "otf", "woff", "woff2", "eot",
    // Databases
    "sqlite", "db", "db-journal", "db-wal",
    // Legacy Office (no extractor yet)
    "doc", "xls", "ppt", "odt", "ods",
];

/// Extract a ZIP file to a temporary directory.
/// Extracts ALL files except known binary formats. Logs everything skipped.
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
    let mut extracted = 0usize;
    let mut skipped_binary = 0usize;
    let mut skipped_system = 0usize;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::Other(format!("ZIP entry error: {}", e)))?;

        let name = entry.name().to_string();

        // Skip directories
        if entry.is_dir() {
            continue;
        }

        // Skip system/hidden files
        if name.starts_with("__MACOSX") || name.contains(".DS_Store") || name.contains("Thumbs.db") {
            skipped_system += 1;
            log::debug!("ZIP: skipping system file: {}", name);
            continue;
        }

        // Always add to listing for adapter detection
        file_listing.push(name.clone());

        // Check if it's a known binary format
        let lower = name.to_lowercase();
        let ext = lower.rsplit('.').next().unwrap_or("");
        if BINARY_EXTENSIONS.contains(&ext) {
            skipped_binary += 1;
            log::debug!("ZIP: skipping binary file: {} (ext={})", name, ext);
            continue;
        }

        // Extract everything else
        let out_path = temp_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut out_file = fs::File::create(&out_path)?;
        io::copy(&mut entry, &mut out_file)?;
        extracted += 1;
    }

    log::info!(
        "ZIP: extracted {} files, skipped {} binary, {} system ({} total entries)",
        extracted, skipped_binary, skipped_system,
        extracted + skipped_binary + skipped_system
    );

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

    #[test]
    fn test_binary_extensions_skip() {
        assert!(BINARY_EXTENSIONS.contains(&"jpg"));
        assert!(BINARY_EXTENSIONS.contains(&"mp4"));
        assert!(!BINARY_EXTENSIONS.contains(&"json"));
        assert!(!BINARY_EXTENSIONS.contains(&"csv"));
    }
}
