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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn with_temp_home<F: FnOnce()>(f: F) {
        let dir = tempfile::tempdir().unwrap();
        let orig = env::var("HOME").unwrap_or_default();
        env::set_var("HOME", dir.path());
        f();
        env::set_var("HOME", orig);
    }

    #[test]
    fn save_and_load_round_trip() {
        with_temp_home(|| {
            let session = Session {
                username: "pg".into(),
                cookie: "user=abc123xyz".into(),
            };
            session.save();
            let loaded = Session::load().expect("should load");
            assert_eq!(loaded.username, "pg");
            assert_eq!(loaded.cookie, "user=abc123xyz");
        });
    }

    #[test]
    fn load_returns_none_when_no_file() {
        with_temp_home(|| {
            assert!(Session::load().is_none());
        });
    }

    #[test]
    fn delete_removes_session() {
        with_temp_home(|| {
            let session = Session { username: "x".into(), cookie: "user=y".into() };
            session.save();
            assert!(Session::load().is_some());
            Session::delete();
            assert!(Session::load().is_none());
        });
    }
}
