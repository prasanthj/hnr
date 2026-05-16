use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Session {
    pub username: String,
    pub cookie: String,
}

impl Session {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join(".hnr").join("session")
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&path, format!("{}\n{}", self.username, self.cookie));
    }

    pub fn load() -> Option<Self> {
        let content = std::fs::read_to_string(Self::path()).ok()?;
        let mut lines = content.lines();
        let username = lines.next()?.to_string();
        let cookie = lines.next()?.to_string();
        if username.is_empty() || cookie.is_empty() {
            return None;
        }
        Some(Self { username, cookie })
    }

    pub fn delete() {
        let _ = std::fs::remove_file(Self::path());
    }
}
