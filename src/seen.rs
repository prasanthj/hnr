use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn default_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".hnr").join("seen.json")
}

pub fn load(path: &Path) -> HashSet<u64> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let ids: Vec<u64> = serde_json::from_str(&content).unwrap_or_default();
    ids.into_iter().collect()
}

pub fn save(path: &Path, ids: &HashSet<u64>) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let mut sorted: Vec<u64> = ids.iter().copied().collect();
    sorted.sort_unstable();
    if let Ok(json) = serde_json::to_string_pretty(&sorted) {
        let _ = std::fs::write(path, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("seen.json");
        let ids: HashSet<u64> = [1, 2, 3].into_iter().collect();
        save(&path, &ids);
        assert_eq!(load(&path), ids);
    }

    #[test]
    fn load_returns_empty_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load(&dir.path().join("seen.json")).is_empty());
    }

    #[test]
    fn load_returns_empty_on_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("seen.json");
        std::fs::write(&path, b"not valid json").unwrap();
        assert!(load(&path).is_empty());
    }

    #[test]
    fn save_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("seen.json");
        let ids: HashSet<u64> = [10, 20].into_iter().collect();
        save(&path, &ids);
        save(&path, &ids);
        assert_eq!(load(&path), ids);
    }
}
