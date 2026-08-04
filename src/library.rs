use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Library {
    pub artists: BTreeMap<String, BTreeMap<String, Vec<Track>>>,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub path: PathBuf,
}

pub struct LibraryItem {
    pub artist: String,
    pub album: String,
    pub title: String,
    pub path: PathBuf,
}

impl Library {
    pub fn build(items: Vec<LibraryItem>) -> Self {
        let mut artists = BTreeMap::new();
        for item in items {
            artists
                .entry(item.artist)
                .or_insert_with(BTreeMap::new)
                .entry(item.album)
                .or_insert_with(Vec::new)
                .push(Track {
                    title: item.title,
                    path: item.path,
                });
        }
        Self { artists }
    }
}

pub fn scan_library(path: &str) -> Vec<LibraryItem> {
    let mut items = Vec::new();
    if path.is_empty() {
        return items;
    }
    let root = Path::new(path);
    if root.exists() && root.is_dir() {
        visit_dirs(root, &mut items);
    }
    items
}

fn visit_dirs(dir: &Path, items: &mut Vec<LibraryItem>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, items);
            } else if path.is_file() && is_music_file(&path) {
                let title = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let album = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown Album".to_string());

                let artist = path
                    .parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown Artist".to_string());

                items.push(LibraryItem {
                    artist,
                    album,
                    title,
                    path,
                });
            }
        }
    }
}

fn is_music_file(path: &PathBuf) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(ext.as_str(), "flac" | "mp3" | "ogg" | "wav")
}
