use crate::api::Item;
use std::path::{Path, PathBuf};

pub fn default_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".hnr").join("bookmarks.json")
}

pub fn load(path: &Path) -> Vec<Item> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save(path: &Path, items: &[Item]) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string_pretty(items) {
        let _ = std::fs::write(path, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::make_item;

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bookmarks.json");
        let items = vec![make_item(1), make_item(2)];
        save(&path, &items);
        let loaded = load(&path);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, 1);
        assert_eq!(loaded[1].id, 2);
    }

    #[test]
    fn load_returns_empty_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bookmarks.json");
        let items = load(&path);
        assert!(items.is_empty());
    }

    #[test]
    fn save_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bookmarks.json");
        save(&path, &[make_item(1)]);
        save(&path, &[make_item(2), make_item(3)]);
        let loaded = load(&path);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, 2);
    }

    #[test]
    fn load_returns_empty_on_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bookmarks.json");
        std::fs::write(&path, b"not valid json").unwrap();
        assert!(load(&path).is_empty());
    }
}
