use crate::api::{fetch_ask_ids, fetch_best_ids, fetch_new_ids, fetch_show_ids, fetch_top_ids, Item, User};
use crate::session::Session;
use arboard::Clipboard;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

const HN_API_BASE: &str = "https://hacker-news.firebaseio.com/v0";

#[derive(Debug, Clone, PartialEq)]
pub enum Feed {
    Top,
    New,
    Best,
    Ask,
    Show,
    Bookmarks,
}

impl Feed {
    pub fn label(&self) -> &str {
        match self {
            Feed::Top => "Top",
            Feed::New => "New",
            Feed::Best => "Best",
            Feed::Ask => "Ask HN",
            Feed::Show => "Show HN",
            Feed::Bookmarks => "Bookmarks",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pane {
    Stories,
    Comments,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Story,
    User,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoginField {
    Username,
    Password,
}

#[derive(Debug, Clone)]
pub struct LoginState {
    pub username: String,
    pub password: String,
    pub field: LoginField,
    pub error: String,
}

impl LoginState {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            field: LoginField::Username,
            error: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComposeState {
    pub text: String,
    pub parent_id: u64,
    pub story_id: u64,
    pub hmac: String,
    pub parent_by: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Command,
    Login,
    Compose,
    CommentDetail,
}

#[derive(Clone)]
pub struct CommentNode {
    pub item: Item,
    pub depth: usize,
    pub collapsed: bool,
    pub children: Vec<CommentNode>,
}

impl CommentNode {
    pub fn new(item: Item, depth: usize) -> Self {
        Self { item, depth, collapsed: false, children: vec![] }
    }

    pub fn flatten(&self) -> Vec<(&CommentNode, usize)> {
        let mut out = vec![(self, self.depth)];
        if !self.collapsed {
            for child in &self.children {
                out.extend(child.flatten());
            }
        }
        out
    }
}

pub struct App {
    pub feed: Feed,
    pub stories: Vec<Item>,
    pub story_ids: Vec<u64>,
    pub story_cursor: usize,
    pub story_scroll: usize,

    pub comments: Vec<CommentNode>,
    pub comment_cursor: usize,
    pub comment_scroll: usize,

    pub active_pane: Pane,
    pub mode: Mode,
    pub command_input: String,
    pub status_message: String,

    pub view_mode: ViewMode,
    pub user_profile: Option<User>,

    pub session: Option<Session>,
    pub login_state: LoginState,
    pub compose_state: Option<ComposeState>,

    pub comment_detail: Option<(String, String)>, // (author, plain text)
    pub comment_detail_scroll: usize,

    pub bookmark_ids: HashSet<u64>,
    pub bookmarks_path: PathBuf,

    pub loading: bool,
    pub client: reqwest::Client,
    pub api_base: String,
    pub item_cache: HashMap<u64, Item>,
    pub comment_pos_cache: HashMap<u64, (usize, usize)>, // story_id -> (cursor, scroll)
}

impl App {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            feed: Feed::Top,
            stories: vec![],
            story_ids: vec![],
            story_cursor: 0,
            story_scroll: 0,
            comments: vec![],
            comment_cursor: 0,
            comment_scroll: 0,
            active_pane: Pane::Stories,
            mode: Mode::Normal,
            command_input: String::new(),
            status_message: String::from("Loading..."),
            view_mode: ViewMode::Story,
            user_profile: None,
            session: Session::load(),
            login_state: LoginState::new(),
            compose_state: None,
            comment_detail: None,
            comment_detail_scroll: 0,
            bookmark_ids: {
                let p = crate::bookmarks::default_path();
                crate::bookmarks::load(&p).iter().map(|b| b.id).collect()
            },
            bookmarks_path: crate::bookmarks::default_path(),
            loading: false,
            client,
            api_base: HN_API_BASE.to_string(),
            item_cache: HashMap::new(),
            comment_pos_cache: HashMap::new(),
        }
    }

    pub fn selected_story(&self) -> Option<&Item> {
        self.stories.get(self.story_cursor)
    }

    pub fn flat_comments(&self) -> Vec<(&CommentNode, usize)> {
        self.comments.iter().flat_map(|c| c.flatten()).collect()
    }

    pub fn scroll_story_down(&mut self, visible: usize) {
        if self.story_cursor + 1 < self.stories.len() {
            self.story_cursor += 1;
            if self.story_cursor >= self.story_scroll + visible {
                self.story_scroll += 1;
            }
        }
    }

    pub fn scroll_story_up(&mut self) {
        if self.story_cursor > 0 {
            self.story_cursor -= 1;
            if self.story_cursor < self.story_scroll {
                self.story_scroll = self.story_cursor;
            }
        }
    }

    pub fn scroll_comment_down(&mut self, visible: usize) {
        let total = self.flat_comments().len();
        if self.comment_cursor + 1 < total {
            self.comment_cursor += 1;
            if self.comment_cursor >= self.comment_scroll + visible {
                self.comment_scroll += 1;
            }
        }
    }

    pub fn scroll_comment_up(&mut self) {
        if self.comment_cursor > 0 {
            self.comment_cursor -= 1;
            if self.comment_cursor < self.comment_scroll {
                self.comment_scroll = self.comment_cursor;
            }
        }
    }

    pub fn toggle_current_comment(&mut self) {
        let id = {
            let flat = self.flat_comments();
            flat.get(self.comment_cursor).map(|(node, _)| node.item.id)
        };
        if let Some(id) = id {
            toggle_in_tree(&mut self.comments, id);
        }
    }

    pub fn username_at_cursor(&self) -> Option<String> {
        match self.active_pane {
            Pane::Stories => self.selected_story().and_then(|s| s.by.clone()),
            Pane::Comments => {
                let flat = self.flat_comments();
                flat.get(self.comment_cursor)
                    .and_then(|(node, _)| node.item.by.clone())
            }
        }
    }

    // ── Auth ──────────────────────────────────────────────────────────────

    pub fn start_login(&mut self) {
        self.login_state = LoginState::new();
        self.mode = Mode::Login;
    }

    pub async fn submit_login(&mut self) {
        let username = self.login_state.username.trim().to_string();
        let password = self.login_state.password.clone();
        if username.is_empty() || password.is_empty() {
            self.login_state.error = "Username and password required".into();
            return;
        }
        self.login_state.error = "Logging in...".into();
        match crate::api::login(&username, &password).await {
            Ok(cookie) => {
                let session = Session { username: username.clone(), cookie };
                session.save();
                self.session = Some(session);
                self.mode = Mode::Normal;
                self.status_message = format!("Logged in as {username} | v vote | c comment");
            }
            Err(e) => {
                self.login_state.error = e.to_string();
            }
        }
    }

    pub fn logout(&mut self) {
        Session::delete();
        self.session = None;
        self.status_message = "Logged out.".into();
    }

    // ── Vote ──────────────────────────────────────────────────────────────

    pub async fn vote_current(&mut self) {
        let session = match &self.session {
            Some(s) => s.clone(),
            None => {
                self.status_message = "Not logged in — /login first".into();
                return;
            }
        };

        let (item_id, story_id) = match self.active_pane {
            Pane::Stories => {
                match self.selected_story() {
                    Some(s) => (s.id, s.id),
                    None => return,
                }
            }
            Pane::Comments => {
                let story_id = match self.selected_story() {
                    Some(s) => s.id,
                    None => return,
                };
                let flat = self.flat_comments();
                match flat.get(self.comment_cursor) {
                    Some((node, _)) => (node.item.id, story_id),
                    None => return,
                }
            }
        };

        self.status_message = "Fetching vote token...".into();
        self.loading = true;
        let client = self.client.clone();
        match crate::api::fetch_vote_auth(&client, &session.cookie, item_id, story_id).await {
            Ok(auth) => {
                match crate::api::vote_item(&client, &session.cookie, item_id, &auth, story_id).await {
                    Ok(_) => self.status_message = "Voted!".into(),
                    Err(e) => self.status_message = format!("Vote failed: {e}"),
                }
            }
            Err(e) => self.status_message = format!("Vote: {e}"),
        }
        self.loading = false;
    }

    // ── Compose ───────────────────────────────────────────────────────────

    pub async fn start_compose(&mut self) {
        let session = match &self.session {
            Some(s) => s.clone(),
            None => {
                self.status_message = "Not logged in — /login first".into();
                return;
            }
        };

        let (parent_id, story_id, parent_by) = match self.active_pane {
            Pane::Stories => {
                match self.selected_story() {
                    Some(s) => (s.id, s.id, s.display_by().to_string()),
                    None => return,
                }
            }
            Pane::Comments => {
                let story_id = match self.selected_story() {
                    Some(s) => s.id,
                    None => return,
                };
                let flat = self.flat_comments();
                match flat.get(self.comment_cursor) {
                    Some((node, _)) => (node.item.id, story_id, node.item.display_by().to_string()),
                    None => return,
                }
            }
        };

        self.status_message = "Fetching reply token...".into();
        self.loading = true;
        let client = self.client.clone();
        match crate::api::fetch_reply_hmac(&client, &session.cookie, parent_id).await {
            Ok(hmac) => {
                self.compose_state = Some(ComposeState {
                    text: String::new(),
                    parent_id,
                    story_id,
                    hmac,
                    parent_by,
                });
                self.mode = Mode::Compose;
                self.status_message = "Ctrl+S submit | Esc cancel".into();
            }
            Err(e) => self.status_message = format!("Compose: {e}"),
        }
        self.loading = false;
    }

    pub async fn submit_comment(&mut self) {
        let session = match &self.session {
            Some(s) => s.clone(),
            None => return,
        };
        let state = match self.compose_state.take() {
            Some(s) => s,
            None => return,
        };
        if state.text.trim().is_empty() {
            self.compose_state = Some(state);
            self.status_message = "Comment is empty.".into();
            return;
        }
        self.mode = Mode::Normal;
        self.status_message = "Posting comment...".into();
        self.loading = true;
        let client = self.client.clone();
        match crate::api::post_comment(
            &client,
            &session.cookie,
            state.parent_id,
            state.story_id,
            &state.hmac,
            &state.text,
        )
        .await
        {
            Ok(_) => {
                self.status_message = "Comment posted! Reloading...".into();
                self.load_comments().await;
            }
            Err(e) => self.status_message = format!("Post failed: {e}"),
        }
        self.loading = false;
    }

    // ── Feed / comments ───────────────────────────────────────────────────

    pub async fn load_feed(&mut self) {
        if self.feed == Feed::Bookmarks {
            self.stories = crate::bookmarks::load(&self.bookmarks_path);
            self.story_cursor = 0;
            self.story_scroll = 0;
            self.comments = vec![];
            self.comment_cursor = 0;
            self.comment_scroll = 0;
            self.active_pane = Pane::Stories;
            self.status_message = if self.stories.is_empty() {
                "No bookmarks yet — press b to save a story.".into()
            } else {
                format!("{} bookmarks | b to toggle | j/k navigate", self.stories.len())
            };
            return;
        }
        self.loading = true;
        self.status_message = format!("Loading {} stories...", self.feed.label());
        let client = self.client.clone();
        let base = self.api_base.clone();
        let ids = match self.feed {
            Feed::Top  => fetch_top_ids(&client, &base).await,
            Feed::New  => fetch_new_ids(&client, &base).await,
            Feed::Best => fetch_best_ids(&client, &base).await,
            Feed::Ask  => fetch_ask_ids(&client, &base).await,
            Feed::Show => fetch_show_ids(&client, &base).await,
            Feed::Bookmarks => unreachable!(),
        };
        match ids {
            Ok(mut ids) => {
                ids.truncate(60);
                self.story_ids = ids.clone();
                self.status_message = format!("Fetching {} items...", ids.len());
                let items = crate::api::fetch_items(&client, &ids, &base).await;
                for item in &items {
                    self.item_cache.insert(item.id, item.clone());
                }
                self.stories = ids
                    .iter()
                    .filter_map(|id| self.item_cache.get(id).cloned())
                    .collect();
                self.story_cursor = 0;
                self.story_scroll = 0;
                self.comments = vec![];
                self.comment_cursor = 0;
                self.comment_scroll = 0;
                self.active_pane = Pane::Stories;
                self.status_message = format!(
                    "{} stories | j/k navigate | Enter comments | Tab pane | v vote | c reply | / cmd",
                    self.stories.len()
                );
            }
            Err(e) => self.status_message = format!("Error: {e}"),
        }
        self.loading = false;
    }

    pub async fn load_comments(&mut self) {
        if let Some(story) = self.selected_story().cloned() {
            let story_id = story.id;
            let kids = story.kids.clone().unwrap_or_default();
            if kids.is_empty() {
                self.status_message = "No comments.".into();
                self.comments = vec![];
                return;
            }
            self.status_message = "Loading comments...".into();
            let client = self.client.clone();
            let base = self.api_base.clone();
            let nodes = load_comment_tree(&client, &kids, 0, &base).await;
            self.comments = nodes;
            let (cursor, scroll) = self.comment_pos_cache.get(&story_id).copied().unwrap_or((0, 0));
            self.comment_cursor = cursor.min(self.flat_comments().len().saturating_sub(1));
            self.comment_scroll = scroll;
            self.status_message = format!(
                "{} top-level threads | j/k | Space collapse | u profile | v vote | c reply | Tab back",
                self.comments.len()
            );
        }
    }

    pub fn toggle_bookmark(&mut self) {
        if let Some(story) = self.selected_story().cloned() {
            let mut items = crate::bookmarks::load(&self.bookmarks_path);
            if self.bookmark_ids.contains(&story.id) {
                items.retain(|b| b.id != story.id);
                self.bookmark_ids.remove(&story.id);
                crate::bookmarks::save(&self.bookmarks_path, &items);
                self.status_message = format!("Removed bookmark: {}", story.display_title());
            } else {
                let title = story.display_title().to_string();
                self.bookmark_ids.insert(story.id);
                items.push(story);
                crate::bookmarks::save(&self.bookmarks_path, &items);
                self.status_message = format!("★ Bookmarked: {title}");
            }
        }
    }

    pub fn open_comment_detail(&mut self) {
        let data = {
            let flat = self.flat_comments();
            flat.get(self.comment_cursor).map(|(node, _)| {
                (node.item.display_by().to_string(), node.item.text_plain())
            })
        };
        match data {
            None => {}
            Some((_, ref text)) if text.trim().is_empty() => {
                self.status_message = "No text for this comment.".into();
            }
            Some((author, text)) => {
                self.comment_detail = Some((author, text));
                self.comment_detail_scroll = 0;
                self.mode = Mode::CommentDetail;
            }
        }
    }

    pub fn close_comment_detail(&mut self) {
        self.comment_detail = None;
        self.comment_detail_scroll = 0;
        self.mode = Mode::Normal;
    }

    pub fn save_comment_pos(&mut self) {
        if let Some(story) = self.selected_story() {
            self.comment_pos_cache.insert(story.id, (self.comment_cursor, self.comment_scroll));
        }
    }

    pub fn copy_url_to_clipboard(&mut self) {
        if let Some(story) = self.selected_story() {
            let url = story.url.clone()
                .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", story.id));
            match Clipboard::new() {
                Ok(mut cb) => {
                    if cb.set_text(&url).is_ok() {
                        self.status_message = format!("Copied: {url}");
                    } else {
                        self.status_message = "Failed to copy to clipboard".into();
                    }
                }
                Err(_) => self.status_message = "Clipboard unavailable".into(),
            }
        }
    }

    pub async fn load_user(&mut self, username: String) {
        self.status_message = format!("Loading profile for {username}...");
        self.loading = true;
        let client = self.client.clone();
        let base = self.api_base.clone();
        match crate::api::fetch_user(&client, &username, &base).await {
            Ok(user) => {
                self.status_message = format!("{} | {} karma | Esc back", user.id, user.karma);
                self.user_profile = Some(user);
                self.view_mode = ViewMode::User;
            }
            Err(e) => self.status_message = format!("Failed to load user: {e}"),
        }
        self.loading = false;
    }

    pub fn close_user_profile(&mut self) {
        self.view_mode = ViewMode::Story;
        self.user_profile = None;
        self.status_message = "j/k navigate | Enter open | Tab switch pane | / command".into();
    }

    pub fn open_story_in_browser(&self) {
        if let Some(story) = self.selected_story() {
            let url = story
                .url
                .clone()
                .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", story.id));
            let _ = open::that(url);
        }
    }

    pub fn open_hn_page_in_browser(&self) {
        if let Some(story) = self.selected_story() {
            let url = format!("https://news.ycombinator.com/item?id={}", story.id);
            let _ = open::that(url);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{make_item, User};
    use crate::session::Session;

    fn make_node(id: u64, children: Vec<CommentNode>) -> CommentNode {
        CommentNode { item: make_item(id), depth: 0, collapsed: false, children }
    }

    fn make_node_depth(id: u64, depth: usize, children: Vec<CommentNode>) -> CommentNode {
        CommentNode { item: make_item(id), depth, collapsed: false, children }
    }

    // ── CommentNode::flatten ──────────────────────────────────────────────

    #[test]
    fn flatten_leaf_returns_self() {
        let node = make_node(1, vec![]);
        let flat = node.flatten();
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].0.item.id, 1);
    }

    #[test]
    fn flatten_includes_children() {
        let child = make_node(2, vec![]);
        let parent = make_node(1, vec![child]);
        let flat = parent.flatten();
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].0.item.id, 1);
        assert_eq!(flat[1].0.item.id, 2);
    }

    #[test]
    fn flatten_collapsed_hides_children() {
        let child = make_node(2, vec![]);
        let mut parent = make_node(1, vec![child]);
        parent.collapsed = true;
        let flat = parent.flatten();
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].0.item.id, 1);
    }

    #[test]
    fn flatten_nested_depth_order() {
        let grandchild = make_node_depth(3, 2, vec![]);
        let child = make_node_depth(2, 1, vec![grandchild]);
        let parent = make_node_depth(1, 0, vec![child]);
        let flat = parent.flatten();
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].0.item.id, 1);
        assert_eq!(flat[1].0.item.id, 2);
        assert_eq!(flat[2].0.item.id, 3);
    }

    #[test]
    fn flatten_collapsed_mid_tree_hides_subtree() {
        let grandchild = make_node_depth(3, 2, vec![]);
        let mut child = make_node_depth(2, 1, vec![grandchild]);
        child.collapsed = true;
        let parent = make_node_depth(1, 0, vec![child]);
        let flat = parent.flatten();
        // parent + collapsed child visible, grandchild hidden
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[1].0.item.id, 2);
    }

    // ── toggle_in_tree ────────────────────────────────────────────────────

    #[test]
    fn toggle_root_node() {
        let child = make_node(2, vec![]);
        let mut nodes = vec![make_node(1, vec![child])];
        assert!(!nodes[0].collapsed);
        toggle_in_tree(&mut nodes, 1);
        assert!(nodes[0].collapsed);
        toggle_in_tree(&mut nodes, 1);
        assert!(!nodes[0].collapsed);
    }

    #[test]
    fn toggle_nested_node() {
        let child = make_node(2, vec![]);
        let mut nodes = vec![make_node(1, vec![child])];
        assert!(!nodes[0].children[0].collapsed);
        toggle_in_tree(&mut nodes, 2);
        assert!(nodes[0].children[0].collapsed);
    }

    #[test]
    fn toggle_nonexistent_id_is_noop() {
        let mut nodes = vec![make_node(1, vec![])];
        toggle_in_tree(&mut nodes, 99);
        assert!(!nodes[0].collapsed);
    }

    // ── App scroll ────────────────────────────────────────────────────────

    fn make_app_with_stories(n: usize) -> App {
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        app.stories = (1..=n as u64).map(make_item).collect();
        app
    }

    fn make_app_with_temp_bookmarks(n: usize) -> (App, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let mut app = make_app_with_stories(n);
        app.bookmarks_path = dir.path().join("bookmarks.json");
        app.bookmark_ids = HashSet::new();
        (app, dir)
    }

    fn make_app_with_comments(n: usize) -> App {
        let mut app = make_app_with_stories(1);
        app.comments = (100..100 + n as u64)
            .map(|id| CommentNode::new(make_item(id), 0))
            .collect();
        app
    }

    #[test]
    fn story_scroll_down_increments_cursor() {
        let mut app = make_app_with_stories(5);
        app.scroll_story_down(10);
        assert_eq!(app.story_cursor, 1);
        assert_eq!(app.story_scroll, 0);
    }

    #[test]
    fn story_scroll_advances_when_cursor_hits_visible_boundary() {
        let mut app = make_app_with_stories(10);
        for _ in 0..5 {
            app.scroll_story_down(5);
        }
        assert_eq!(app.story_cursor, 5);
        assert_eq!(app.story_scroll, 1);
    }

    #[test]
    fn story_scroll_up_decrements_cursor() {
        let mut app = make_app_with_stories(5);
        app.story_cursor = 3;
        app.story_scroll = 2;
        app.scroll_story_up();
        assert_eq!(app.story_cursor, 2);
        assert_eq!(app.story_scroll, 2);
    }

    #[test]
    fn story_scroll_up_adjusts_scroll_when_cursor_above_window() {
        let mut app = make_app_with_stories(5);
        app.story_cursor = 2;
        app.story_scroll = 3;
        app.scroll_story_up();
        assert_eq!(app.story_cursor, 1);
        assert_eq!(app.story_scroll, 1);
    }

    #[test]
    fn story_scroll_down_stops_at_end() {
        let mut app = make_app_with_stories(3);
        app.story_cursor = 2;
        app.scroll_story_down(10);
        assert_eq!(app.story_cursor, 2);
    }

    #[test]
    fn story_scroll_up_stops_at_zero() {
        let mut app = make_app_with_stories(3);
        app.story_cursor = 0;
        app.scroll_story_up();
        assert_eq!(app.story_cursor, 0);
        assert_eq!(app.story_scroll, 0);
    }

    #[test]
    fn selected_story_returns_correct_item() {
        let mut app = make_app_with_stories(5);
        app.story_cursor = 2;
        assert_eq!(app.selected_story().unwrap().id, 3);
    }

    #[test]
    fn selected_story_none_when_empty() {
        let client = reqwest::Client::new();
        let app = App::new(client);
        assert!(app.selected_story().is_none());
    }

    // ── Comment scroll memory ─────────────────────────────────────────────

    #[test]
    fn save_comment_pos_stores_cursor_and_scroll() {
        let mut app = make_app_with_stories(3);
        app.story_cursor = 1;
        app.comment_cursor = 5;
        app.comment_scroll = 3;
        app.save_comment_pos();
        let story_id = app.stories[1].id;
        assert_eq!(app.comment_pos_cache.get(&story_id).copied(), Some((5, 3)));
    }

    #[test]
    fn save_comment_pos_noop_when_no_stories() {
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        app.save_comment_pos();
        assert!(app.comment_pos_cache.is_empty());
    }

    #[test]
    fn save_comment_pos_overwrites_previous_entry() {
        let mut app = make_app_with_stories(2);
        app.story_cursor = 0;
        app.comment_cursor = 2;
        app.comment_scroll = 1;
        app.save_comment_pos();
        app.comment_cursor = 7;
        app.comment_scroll = 4;
        app.save_comment_pos();
        let story_id = app.stories[0].id;
        assert_eq!(app.comment_pos_cache.get(&story_id).copied(), Some((7, 4)));
    }

    // ── copy_url_to_clipboard ─────────────────────────────────────────────

    #[test]
    fn copy_url_no_op_when_no_stories() {
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        app.copy_url_to_clipboard();
        assert!(!app.status_message.starts_with("Copied:"));
    }

    #[test]
    fn copy_url_uses_story_url() {
        let mut app = make_app_with_stories(1);
        // make_item sets url = "https://example.com/1"
        app.copy_url_to_clipboard();
        let msg = &app.status_message;
        assert!(
            msg.starts_with("Copied: https://example.com/1")
                || msg == "Failed to copy to clipboard"
                || msg == "Clipboard unavailable",
            "unexpected: {msg}"
        );
    }

    #[test]
    fn copy_url_falls_back_to_hn_url_when_no_story_url() {
        let mut app = make_app_with_stories(1);
        app.stories[0].url = None;
        app.copy_url_to_clipboard();
        let msg = &app.status_message;
        assert!(
            msg.starts_with("Copied: https://news.ycombinator.com/item?id=")
                || msg == "Failed to copy to clipboard"
                || msg == "Clipboard unavailable",
            "unexpected: {msg}"
        );
    }

    // ── Feed::label ───────────────────────────────────────────────────────

    #[test]
    fn feed_label_returns_correct_strings() {
        assert_eq!(Feed::Top.label(), "Top");
        assert_eq!(Feed::New.label(), "New");
        assert_eq!(Feed::Best.label(), "Best");
        assert_eq!(Feed::Ask.label(), "Ask HN");
        assert_eq!(Feed::Show.label(), "Show HN");
        assert_eq!(Feed::Bookmarks.label(), "Bookmarks");
    }

    // ── flat_comments ─────────────────────────────────────────────────────

    #[test]
    fn flat_comments_empty_when_no_comments() {
        let app = make_app_with_stories(1);
        assert!(app.flat_comments().is_empty());
    }

    #[test]
    fn flat_comments_includes_nested_nodes() {
        let mut app = make_app_with_stories(1);
        let child = CommentNode::new(make_item(101), 1);
        app.comments = vec![
            CommentNode { item: make_item(100), depth: 0, collapsed: false, children: vec![child] },
            CommentNode::new(make_item(102), 0),
        ];
        // root 100, child 101, root 102
        assert_eq!(app.flat_comments().len(), 3);
        assert_eq!(app.flat_comments()[0].0.item.id, 100);
        assert_eq!(app.flat_comments()[1].0.item.id, 101);
        assert_eq!(app.flat_comments()[2].0.item.id, 102);
    }

    // ── Comment scroll ────────────────────────────────────────────────────

    #[test]
    fn scroll_comment_down_increments_cursor() {
        let mut app = make_app_with_comments(5);
        app.scroll_comment_down(10);
        assert_eq!(app.comment_cursor, 1);
        assert_eq!(app.comment_scroll, 0);
    }

    #[test]
    fn scroll_comment_down_advances_scroll_at_boundary() {
        let mut app = make_app_with_comments(10);
        for _ in 0..5 {
            app.scroll_comment_down(5);
        }
        assert_eq!(app.comment_cursor, 5);
        assert_eq!(app.comment_scroll, 1);
    }

    #[test]
    fn scroll_comment_down_stops_at_end() {
        let mut app = make_app_with_comments(3);
        app.comment_cursor = 2;
        app.scroll_comment_down(10);
        assert_eq!(app.comment_cursor, 2);
    }

    #[test]
    fn scroll_comment_up_decrements_cursor() {
        let mut app = make_app_with_comments(5);
        app.comment_cursor = 3;
        app.comment_scroll = 2;
        app.scroll_comment_up();
        assert_eq!(app.comment_cursor, 2);
        assert_eq!(app.comment_scroll, 2);
    }

    #[test]
    fn scroll_comment_up_adjusts_scroll_when_cursor_above_window() {
        let mut app = make_app_with_comments(5);
        app.comment_cursor = 2;
        app.comment_scroll = 3;
        app.scroll_comment_up();
        assert_eq!(app.comment_cursor, 1);
        assert_eq!(app.comment_scroll, 1);
    }

    #[test]
    fn scroll_comment_up_stops_at_zero() {
        let mut app = make_app_with_comments(3);
        app.scroll_comment_up();
        assert_eq!(app.comment_cursor, 0);
        assert_eq!(app.comment_scroll, 0);
    }

    // ── toggle_current_comment ────────────────────────────────────────────

    #[test]
    fn toggle_current_comment_collapses_and_expands() {
        let child = CommentNode::new(make_item(101), 1);
        let mut app = make_app_with_stories(1);
        app.comments = vec![CommentNode {
            item: make_item(100),
            depth: 0,
            collapsed: false,
            children: vec![child],
        }];
        app.comment_cursor = 0;
        app.toggle_current_comment();
        assert!(app.comments[0].collapsed);
        app.toggle_current_comment();
        assert!(!app.comments[0].collapsed);
    }

    #[test]
    fn toggle_current_comment_noop_when_no_comments() {
        let mut app = make_app_with_stories(1);
        app.toggle_current_comment(); // should not panic
    }

    // ── username_at_cursor ────────────────────────────────────────────────

    #[test]
    fn username_at_cursor_stories_pane() {
        let app = make_app_with_stories(3);
        // make_item(1) sets by = "user1"
        assert_eq!(app.username_at_cursor(), Some("user1".to_string()));
    }

    #[test]
    fn username_at_cursor_comments_pane() {
        let mut app = make_app_with_stories(1);
        app.active_pane = Pane::Comments;
        app.comments = vec![CommentNode::new(make_item(100), 0)];
        app.comment_cursor = 0;
        assert_eq!(app.username_at_cursor(), Some("user100".to_string()));
    }

    #[test]
    fn username_at_cursor_none_when_empty() {
        let client = reqwest::Client::new();
        let app = App::new(client);
        assert!(app.username_at_cursor().is_none());
    }

    // ── start_login / logout ──────────────────────────────────────────────

    #[test]
    fn start_login_sets_mode_and_clears_state() {
        let mut app = make_app_with_stories(1);
        app.login_state.username = "old".into();
        app.login_state.password = "pw".into();
        app.start_login();
        assert_eq!(app.mode, Mode::Login);
        assert!(app.login_state.username.is_empty());
        assert!(app.login_state.password.is_empty());
    }

    #[test]
    fn logout_clears_session_and_sets_status() {
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        app.session = Some(Session { username: "pg".into(), cookie: "user=abc".into() });
        app.logout();
        assert!(app.session.is_none());
        assert!(app.status_message.contains("Logged out"), "got: {}", app.status_message);
    }

    // ── toggle_bookmark ───────────────────────────────────────────────────

    #[test]
    fn toggle_bookmark_adds_then_removes() {
        let (mut app, _dir) = make_app_with_temp_bookmarks(1);
        app.toggle_bookmark();
        assert!(app.bookmark_ids.contains(&1));
        assert!(app.status_message.contains("Bookmarked"), "got: {}", app.status_message);
        app.toggle_bookmark();
        assert!(!app.bookmark_ids.contains(&1));
        assert!(app.status_message.contains("Removed bookmark"), "got: {}", app.status_message);
    }

    #[test]
    fn toggle_bookmark_persists_to_disk() {
        let (mut app, dir) = make_app_with_temp_bookmarks(1);
        let path = dir.path().join("bookmarks.json");
        app.toggle_bookmark();
        let saved = crate::bookmarks::load(&path);
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].id, 1);
        app.toggle_bookmark();
        let saved = crate::bookmarks::load(&path);
        assert!(saved.is_empty());
    }

    #[test]
    fn toggle_bookmark_noop_when_no_stories() {
        let (mut app, _dir) = make_app_with_temp_bookmarks(0);
        app.toggle_bookmark(); // should not panic
        assert!(app.bookmark_ids.is_empty());
    }

    // ── Feed::Bookmarks in load_feed ──────────────────────────────────────

    #[tokio::test]
    async fn load_feed_bookmarks_reads_from_disk() {
        let (mut app, dir) = make_app_with_temp_bookmarks(0);
        crate::bookmarks::save(&dir.path().join("bookmarks.json"), &[make_item(1), make_item(2)]);
        app.feed = Feed::Bookmarks;
        app.load_feed().await;
        assert_eq!(app.stories.len(), 2);
        assert_eq!(app.stories[0].id, 1);
    }

    #[tokio::test]
    async fn load_feed_bookmarks_empty_shows_help_message() {
        let (mut app, _dir) = make_app_with_temp_bookmarks(0);
        app.feed = Feed::Bookmarks;
        app.load_feed().await;
        assert!(app.stories.is_empty());
        assert!(app.status_message.contains("No bookmarks"), "got: {}", app.status_message);
    }

    // ── open/close_comment_detail ─────────────────────────────────────────

    #[test]
    fn open_comment_detail_sets_mode_and_captures_text() {
        let mut app = make_app_with_stories(1);
        let mut item = make_item(100);
        item.text = Some("<p>Hello <b>world</b></p>".into());
        app.comments = vec![CommentNode::new(item, 0)];
        app.comment_cursor = 0;
        app.open_comment_detail();
        assert_eq!(app.mode, Mode::CommentDetail);
        let (author, text) = app.comment_detail.as_ref().unwrap();
        assert_eq!(author, "user100");
        assert!(text.contains("Hello") && text.contains("world"), "html not stripped: {text}");
    }

    #[test]
    fn open_comment_detail_noop_when_text_empty() {
        let mut app = make_app_with_stories(1);
        // make_item has text: None → text_plain() returns ""
        app.comments = vec![CommentNode::new(make_item(100), 0)];
        app.comment_cursor = 0;
        app.open_comment_detail();
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.comment_detail.is_none());
        assert!(app.status_message.contains("No text"), "got: {}", app.status_message);
    }

    #[test]
    fn open_comment_detail_noop_when_no_comments() {
        let mut app = make_app_with_stories(1);
        app.open_comment_detail(); // should not panic
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn close_comment_detail_resets_all_state() {
        let mut app = make_app_with_stories(1);
        app.mode = Mode::CommentDetail;
        app.comment_detail = Some(("alice".into(), "text".into()));
        app.comment_detail_scroll = 7;
        app.close_comment_detail();
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.comment_detail.is_none());
        assert_eq!(app.comment_detail_scroll, 0);
    }

    // ── close_user_profile ────────────────────────────────────────────────

    #[test]
    fn close_user_profile_resets_view_mode_and_clears_profile() {
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        app.view_mode = ViewMode::User;
        app.user_profile = Some(User { id: "pg".into(), karma: 1, created: 0, about: None, submitted: None });
        app.close_user_profile();
        assert_eq!(app.view_mode, ViewMode::Story);
        assert!(app.user_profile.is_none());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::api::make_item;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_app(base: &str) -> App {
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        app.api_base = base.to_string();
        app
    }

    #[tokio::test]
    async fn load_feed_populates_stories() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/topstories.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([1, 2, 3])))
            .mount(&server)
            .await;
        for id in 1u64..=3 {
            Mock::given(method("GET"))
                .and(path(format!("/item/{id}.json")))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "id": id, "type": "story",
                    "title": format!("Story {id}"),
                    "by": format!("user{id}"),
                    "score": 100, "descendants": 5,
                })))
                .mount(&server)
                .await;
        }

        let mut app = test_app(&server.uri());
        app.load_feed().await;

        assert_eq!(app.stories.len(), 3);
        assert_eq!(app.stories[0].display_title(), "Story 1");
        assert_eq!(app.stories[0].score(), 100);
        assert!(app.status_message.contains("stories"), "got: {}", app.status_message);
    }

    #[tokio::test]
    async fn load_feed_api_error_sets_error_status() {
        let server = MockServer::start().await;
        // No routes: wiremock returns 404 → JSON parse fails → Err propagates
        let mut app = test_app(&server.uri());
        app.load_feed().await;

        assert!(app.status_message.starts_with("Error:"), "got: {}", app.status_message);
        assert!(app.stories.is_empty());
    }

    #[tokio::test]
    async fn load_feed_truncates_to_60_stories() {
        let server = MockServer::start().await;
        let ids: Vec<u64> = (1..=70).collect();
        Mock::given(method("GET"))
            .and(path("/topstories.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&ids))
            .mount(&server)
            .await;
        for id in 1u64..=70 {
            Mock::given(method("GET"))
                .and(path(format!("/item/{id}.json")))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "id": id, "type": "story", "title": format!("Story {id}"),
                    "by": "x", "score": 1,
                })))
                .mount(&server)
                .await;
        }

        let mut app = test_app(&server.uri());
        app.load_feed().await;

        assert_eq!(app.stories.len(), 60, "should truncate to 60, got {}", app.stories.len());
    }

    #[tokio::test]
    async fn load_comments_builds_tree() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/item/10.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 10, "type": "comment",
                "text": "<p>Hello world</p>",
                "by": "alice",
            })))
            .mount(&server)
            .await;

        let mut app = test_app(&server.uri());
        let mut story = make_item(1);
        story.kids = Some(vec![10]);
        app.stories = vec![story];

        app.load_comments().await;

        assert_eq!(app.comments.len(), 1);
        assert_eq!(app.comments[0].item.id, 10);
        assert_eq!(app.comments[0].item.display_by(), "alice");
    }

    #[tokio::test]
    async fn load_user_populates_profile() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/user/pg.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "pg", "karma": 155000, "created": 1000000,
                "about": "<p>Lisp hacker</p>", "submitted": [1, 2, 3]
            })))
            .mount(&server)
            .await;

        let mut app = test_app(&server.uri());
        app.load_user("pg".to_string()).await;

        let user = app.user_profile.as_ref().expect("user should be loaded");
        assert_eq!(user.id, "pg");
        assert_eq!(user.karma, 155000);
        assert_eq!(user.submission_count(), 3);
        assert_eq!(app.view_mode, ViewMode::User);
    }

    #[tokio::test]
    async fn load_user_error_sets_error_status() {
        let server = MockServer::start().await;
        // No routes → 404 → JSON parse fails
        let mut app = test_app(&server.uri());
        app.load_user("nobody".to_string()).await;

        assert!(app.user_profile.is_none());
        assert!(
            app.status_message.contains("Failed to load user"),
            "got: {}",
            app.status_message
        );
    }

    #[tokio::test]
    async fn load_comments_skips_deleted() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/item/10.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 10, "deleted": true,
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/item/11.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 11, "type": "comment", "text": "alive", "by": "bob",
            })))
            .mount(&server)
            .await;

        let mut app = test_app(&server.uri());
        let mut story = make_item(1);
        story.kids = Some(vec![10, 11]);
        app.stories = vec![story];

        app.load_comments().await;

        assert_eq!(app.comments.len(), 1, "deleted comment should be filtered");
        assert_eq!(app.comments[0].item.id, 11);
    }
}

fn toggle_in_tree(nodes: &mut Vec<CommentNode>, id: u64) {
    for node in nodes.iter_mut() {
        if node.item.id == id {
            node.collapsed = !node.collapsed;
            return;
        }
        toggle_in_tree(&mut node.children, id);
    }
}

async fn load_comment_tree<'a>(
    client: &'a reqwest::Client,
    ids: &'a [u64],
    depth: usize,
    base: &'a str,
) -> Vec<CommentNode> {
    let items = crate::api::fetch_items(client, ids, base).await;
    let mut nodes = vec![];
    for item in items {
        if item.is_deleted_or_dead() {
            continue;
        }
        let kids = item.kids.clone().unwrap_or_default();
        let mut node = CommentNode::new(item, depth);
        if !kids.is_empty() && depth < 6 {
            node.children = Box::pin(load_comment_tree(client, &kids, depth + 1, base)).await;
        }
        nodes.push(node);
    }
    nodes
}
