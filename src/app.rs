use crate::api::{fetch_ask_ids, fetch_best_ids, fetch_job_ids, fetch_new_ids, fetch_show_ids, fetch_top_ids, Item, User};
use crate::session::Session;
use arboard::Clipboard;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::shimmer::Progress;

const HN_API_BASE: &str = "https://hacker-news.firebaseio.com/v0";

#[derive(Debug, Clone, PartialEq)]
pub enum Feed {
    Top,
    New,
    Best,
    Ask,
    Show,
    Jobs,
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
            Feed::Jobs => "Jobs",
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
    Search,
    Reader,
}

pub struct ReaderContent {
    pub title: String,
    pub blocks: Vec<crate::reader::Block>,
}

#[derive(Clone)]
pub struct CommentNode {
    pub item: Item,
    pub text: String,
    pub depth: usize,
    pub collapsed: bool,
    pub children: Vec<CommentNode>,
}

impl CommentNode {
    pub fn new(item: Item, depth: usize) -> Self {
        let text = item.text_plain();
        Self { item, text, depth, collapsed: false, children: vec![] }
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
    pub comments_story_id: Option<u64>,
    pub comments_loading: bool,
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

    pub expanded_comment: Option<u64>,

    pub bookmark_ids: HashSet<u64>,
    pub bookmarks_path: PathBuf,

    pub seen_ids: HashSet<u64>,
    pub seen_path: PathBuf,

    pub search_input: String,
    pub search_query: Option<String>,
    pub search_base: String,

    pub reader_content: Option<ReaderContent>,
    pub reader_scroll: usize,
    pub reader_section_offsets: Vec<usize>,
    pub reader_total_lines: usize,
    pub pending_reader: Arc<Mutex<Option<Result<(String, Vec<crate::reader::Block>, Vec<usize>, usize), String>>>>,

    pub story_dwell_start: Option<Instant>,
    pub prefetching_story_id: Option<u64>,
    pub pending_comments: Arc<Mutex<Option<(u64, Vec<CommentNode>)>>>,
    pub progress: Option<Progress>,

    pub loading: bool,
    pub client: reqwest::Client,
    pub api_base: String,
    pub item_cache: HashMap<u64, Item>,
    pub comment_pos_cache: HashMap<u64, (usize, usize)>,
    pub update_available: Arc<Mutex<Option<String>>>,
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
            comments_story_id: None,
            comments_loading: false,
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
            expanded_comment: None,
            bookmark_ids: {
                let p = crate::bookmarks::default_path();
                crate::bookmarks::load(&p).iter().map(|b| b.id).collect()
            },
            bookmarks_path: crate::bookmarks::default_path(),
            seen_ids: {
                let p = crate::seen::default_path();
                crate::seen::load(&p)
            },
            seen_path: crate::seen::default_path(),
            search_input: String::new(),
            search_query: None,
            search_base: "https://hn.algolia.com/api/v1".to_string(),
            reader_content: None,
            reader_scroll: 0,
            reader_section_offsets: vec![],
            reader_total_lines: 0,
            pending_reader: Arc::new(Mutex::new(None)),
            story_dwell_start: Some(Instant::now()),
            prefetching_story_id: None,
            pending_comments: Arc::new(Mutex::new(None)),
            progress: None,
            loading: false,
            client,
            api_base: HN_API_BASE.to_string(),
            item_cache: HashMap::new(),
            comment_pos_cache: HashMap::new(),
            update_available: Arc::new(Mutex::new(None)),
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
            self.story_dwell_start = Some(Instant::now());
            self.prefetching_story_id = None;
            self.progress = None;
        }
    }

    pub fn scroll_story_up(&mut self) {
        if self.story_cursor > 0 {
            self.story_cursor -= 1;
            if self.story_cursor < self.story_scroll {
                self.story_scroll = self.story_cursor;
            }
            self.story_dwell_start = Some(Instant::now());
            self.prefetching_story_id = None;
            self.progress = None;
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
            self.comments_story_id = None;
            self.comments_loading = false;
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
        self.progress = Some(Progress::new("Refreshing"));
        self.status_message = format!("Loading {} stories...", self.feed.label());
        let client = self.client.clone();
        let base = self.api_base.clone();
        let ids = match self.feed {
            Feed::Top  => fetch_top_ids(&client, &base).await,
            Feed::New  => fetch_new_ids(&client, &base).await,
            Feed::Best => fetch_best_ids(&client, &base).await,
            Feed::Ask  => fetch_ask_ids(&client, &base).await,
            Feed::Show => fetch_show_ids(&client, &base).await,
            Feed::Jobs => fetch_job_ids(&client, &base).await,
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
                self.comments_story_id = None;
                self.comments_loading = false;
                self.comment_cursor = 0;
                self.comment_scroll = 0;
                self.active_pane = Pane::Stories;
                self.status_message = format!(
                    "{} stories · j/k navigate · Enter comments · Tab pane · v vote · c reply · / cmd",
                    self.stories.len()
                );
            }
            Err(e) => self.status_message = format!("Error: {e}"),
        }
        self.loading = false;
        self.progress = None;
    }

    pub fn mark_unseen(&mut self) {
        if let Some(story) = self.selected_story() {
            let id = story.id;
            let title = story.display_title().to_string();
            if self.seen_ids.remove(&id) {
                crate::seen::save(&self.seen_path, &self.seen_ids);
                self.status_message = format!("Marked unread: {title}");
            }
        }
    }

    pub async fn load_comments(&mut self) {
        if let Some(story) = self.selected_story().cloned() {
            let story_id = story.id;
            if self.seen_ids.insert(story_id) {
                crate::seen::save(&self.seen_path, &self.seen_ids);
            }
            self.expanded_comment = None;
            // Use prefetched comments if available
            if self.comments_story_id == Some(story_id) && !self.comments.is_empty() {
                let (cursor, scroll) = self.comment_pos_cache.get(&story_id).copied().unwrap_or((0, 0));
                self.comment_cursor = cursor.min(self.flat_comments().len().saturating_sub(1));
                self.comment_scroll = scroll;
                self.status_message = format!(
                    "{} top-level threads · j/k nav · Space collapse · u unread · v vote · c reply · Tab back",
                    self.comments.len()
                );
                return;
            }
            self.comments = vec![];
            self.comments_story_id = Some(story_id);
            self.comments_loading = true;
            self.status_message = "Loading comments...".into();
            let client = self.client.clone();
            let base = self.api_base.clone();
            // Algolia results have no kids — hydrate from HN to get real ids
            let kids = match story.kids.clone() {
                Some(k) if !k.is_empty() => k,
                _ => match crate::api::fetch_item(&client, story_id, &base).await {
                    Ok(item) => item.kids.unwrap_or_default(),
                    Err(_) => { self.status_message = "Failed to load comments.".into(); self.comments_loading = false; return; }
                },
            };
            if kids.is_empty() {
                self.status_message = "No comments.".into();
                self.comments_loading = false;
                return;
            }
            let nodes = load_comment_tree(&client, &kids, 0, &base).await;
            self.comments = nodes;
            self.comments_loading = false;
            let (cursor, scroll) = self.comment_pos_cache.get(&story_id).copied().unwrap_or((0, 0));
            self.comment_cursor = cursor.min(self.flat_comments().len().saturating_sub(1));
            self.comment_scroll = scroll;
            self.status_message = format!(
                "{} top-level threads · j/k nav · Space collapse · u unread · v vote · c reply · Tab back",
                self.comments.len()
            );
        }
    }

    pub fn load_reader(&mut self) {
        let url = match self.selected_story().and_then(|s| s.url.clone()) {
            Some(u) => u,
            None => { self.status_message = "No URL for this story.".into(); return; }
        };
        self.progress = Some(Progress::new("Fetching article"));
        let client = self.client.clone();
        let pending = self.pending_reader.clone();
        tokio::spawn(async move {
            let result = crate::api::fetch_readable(&client, &url).await
                .map(|(title, html)| {
                    let (blocks, offsets) = crate::reader::parse_html(&html);
                    let total = offsets.last().copied().unwrap_or(0) + 40;
                    (title, blocks, offsets, total)
                })
                .map_err(|e| e.to_string());
            if let Ok(mut lock) = pending.lock() {
                *lock = Some(result);
            }
        });
    }

    pub fn apply_pending_reader(&mut self) {
        let ready = {
            let mut lock = match self.pending_reader.lock() {
                Ok(l) => l,
                Err(_) => return,
            };
            lock.take()
        };
        if let Some(result) = ready {
            match result {
                Ok((title, blocks, section_offsets, total_lines)) => {
                    self.reader_section_offsets = section_offsets;
                    self.reader_total_lines = total_lines;
                    self.reader_content = Some(ReaderContent { title, blocks });
                    self.reader_scroll = 0;
                    self.mode = Mode::Reader;
                    self.progress = None;
                    self.story_dwell_start = Some(Instant::now() - std::time::Duration::from_secs(3));
                    self.prefetching_story_id = None;
                }
                Err(e) => {
                    self.status_message = format!("Reader: {e}");
                    self.progress = None;
                }
            }
        }
    }

    pub fn close_reader(&mut self) {
        self.reader_content = None;
        self.mode = Mode::Normal;
    }

    pub fn maybe_start_prefetch(&mut self) {
        let story = match self.selected_story() {
            Some(s) if s.kids.as_ref().map_or(0, |k| k.len()) > 0
                || s.descendants.unwrap_or(0) > 0 => s.clone(),
            _ => return,
        };
        let story_id = story.id;
        if self.prefetching_story_id == Some(story_id) {
            return;
        }
        if self.comments_story_id == Some(story_id) && !self.comments.is_empty() {
            return;
        }
        let elapsed = match self.story_dwell_start {
            Some(t) => t.elapsed().as_secs(),
            None => return,
        };
        if elapsed < 3 {
            return;
        }
        self.prefetching_story_id = Some(story_id);
        self.progress = Some(Progress::new("Fetching comments"));
        let client = self.client.clone();
        let base = self.api_base.clone();
        let pending = self.pending_comments.clone();
        let kids = story.kids.clone();
        tokio::spawn(async move {
            // Algolia results have no kids — hydrate from HN to get real ids
            let kids = match kids {
                Some(k) if !k.is_empty() => k,
                _ => match crate::api::fetch_item(&client, story_id, &base).await {
                    Ok(item) => item.kids.unwrap_or_default(),
                    Err(_) => return,
                },
            };
            let nodes = load_comment_tree(&client, &kids, 0, &base).await;
            if let Ok(mut lock) = pending.lock() {
                *lock = Some((story_id, nodes));
            }
        });
    }

    pub fn apply_pending_comments(&mut self) {
        let ready = {
            let mut lock = match self.pending_comments.lock() {
                Ok(l) => l,
                Err(_) => return,
            };
            lock.take()
        };
        if let Some((story_id, nodes)) = ready {
            let current_id = self.selected_story().map(|s| s.id);
            if current_id == Some(story_id) && self.comments_story_id != Some(story_id) {
                self.comments_story_id = Some(story_id);
                self.comments = nodes;
                self.comments_loading = false;
                self.progress = None;
                if self.mode == Mode::Normal && self.active_pane == Pane::Stories {
                    let (cursor, scroll) = self.comment_pos_cache.get(&story_id).copied().unwrap_or((0, 0));
                    self.comment_cursor = cursor.min(self.flat_comments().len().saturating_sub(1));
                    self.comment_scroll = scroll;
                    self.active_pane = Pane::Comments;
                    self.status_message = format!(
                        "{} top-level threads · j/k nav · Space collapse · u unread · v vote · c reply · Tab back",
                        self.comments.len()
                    );
                }
            }
        }
    }

    pub fn tick_progress(&mut self) {
        let story_id = self.selected_story().map(|s| s.id);
        let has_comments = self.selected_story().map_or(false, |s| {
            s.kids.as_ref().map_or(0, |k| k.len()) > 0
                || s.descendants.unwrap_or(0) > 0
        });
        let comments_ready = self.comments_story_id == story_id && !self.comments.is_empty();

        if !has_comments || comments_ready {
            self.progress = None;
        } else if self.progress.is_none() {
            self.progress = Some(Progress::new("Fetching comments"));
        }
    }

    pub fn start_search(&mut self) {
        self.search_input = String::new();
        self.mode = Mode::Search;
    }

    pub async fn run_search(&mut self) {
        let query = self.search_input.trim().to_string();
        if query.is_empty() {
            self.mode = Mode::Normal;
            self.status_message = "Search cancelled.".into();
            return;
        }
        self.mode = Mode::Normal;
        self.loading = true;
        self.progress = Some(Progress::new("Searching"));
        self.status_message = format!("Searching for '{query}'...");
        let client = self.client.clone();
        let base = self.search_base.clone();
        match crate::api::search_stories(&client, &query, &base).await {
            Ok(items) => {
                self.search_query = Some(query.clone());
                self.stories = items;
                self.story_cursor = 0;
                self.story_scroll = 0;
                self.comments = vec![];
                self.comment_cursor = 0;
                self.comment_scroll = 0;
                self.active_pane = Pane::Stories;
                self.status_message = format!(
                    "{} results for '{}' | j/k navigate | 1-6 back to feeds",
                    self.stories.len(),
                    query
                );
            }
            Err(e) => {
                self.search_query = None;
                self.status_message = format!("Search error: {e}");
            }
        }
        self.loading = false;
        self.progress = None;
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

    pub fn toggle_comment_expand(&mut self) {
        let id = {
            let flat = self.flat_comments();
            flat.get(self.comment_cursor).map(|(node, _)| node.item.id)
        };
        if let Some(id) = id {
            if self.expanded_comment == Some(id) {
                self.expanded_comment = None;
            } else {
                self.expanded_comment = Some(id);
            }
        }
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

    pub fn open_story_in_browser(&mut self) {
        if let Some(story) = self.selected_story() {
            let url = story
                .url
                .clone()
                .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", story.id));
            let _ = open::that(url);
        }
        self.story_dwell_start = Some(Instant::now() - std::time::Duration::from_secs(3));
        self.prefetching_story_id = None;
        self.progress = None;
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
        CommentNode { item: make_item(id), text: String::new(), depth: 0, collapsed: false, children }
    }

    fn make_node_depth(id: u64, depth: usize, children: Vec<CommentNode>) -> CommentNode {
        CommentNode { item: make_item(id), text: String::new(), depth, collapsed: false, children }
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
        assert_eq!(Feed::Jobs.label(), "Jobs");
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
            CommentNode { item: make_item(100), text: String::new(), depth: 0, collapsed: false, children: vec![child] },
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
            text: String::new(),
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

    // ── toggle_comment_expand ─────────────────────────────────────────────

    #[test]
    fn toggle_comment_expand_sets_and_clears_expanded() {
        let mut app = make_app_with_stories(1);
        let mut item = make_item(100);
        item.text = Some("<p>Hello world</p>".into());
        app.comments = vec![CommentNode::new(item, 0)];
        app.comment_cursor = 0;
        app.toggle_comment_expand();
        assert_eq!(app.expanded_comment, Some(100));
        app.toggle_comment_expand();
        assert!(app.expanded_comment.is_none());
    }

    #[test]
    fn toggle_comment_expand_switches_to_different_comment() {
        let mut app = make_app_with_stories(1);
        app.comments = vec![
            CommentNode::new(make_item(100), 0),
            CommentNode::new(make_item(101), 0),
        ];
        app.comment_cursor = 0;
        app.toggle_comment_expand();
        assert_eq!(app.expanded_comment, Some(100));
        app.comment_cursor = 1;
        app.toggle_comment_expand();
        assert_eq!(app.expanded_comment, Some(101));
    }

    #[test]
    fn toggle_comment_expand_noop_when_no_comments() {
        let mut app = make_app_with_stories(1);
        app.toggle_comment_expand(); // should not panic
        assert!(app.expanded_comment.is_none());
    }

    // ── seen tracking ─────────────────────────────────────────────────────

    fn make_app_with_temp_seen(n: usize) -> (App, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let mut app = make_app_with_stories(n);
        app.seen_path = dir.path().join("seen.json");
        app.seen_ids = HashSet::new();
        (app, dir)
    }

    #[test]
    fn mark_unseen_removes_from_seen_ids_and_saves() {
        let (mut app, dir) = make_app_with_temp_seen(1);
        app.seen_ids.insert(1);
        crate::seen::save(&app.seen_path, &app.seen_ids);
        app.mark_unseen();
        assert!(!app.seen_ids.contains(&1));
        assert!(app.status_message.contains("unread"), "got: {}", app.status_message);
        let on_disk = crate::seen::load(&dir.path().join("seen.json"));
        assert!(!on_disk.contains(&1));
    }

    #[test]
    fn mark_unseen_noop_when_story_not_seen() {
        let (mut app, _dir) = make_app_with_temp_seen(1);
        let prev = app.status_message.clone();
        app.mark_unseen();
        assert!(app.seen_ids.is_empty());
        assert_eq!(app.status_message, prev, "status should not change");
    }

    #[test]
    fn mark_unseen_noop_when_no_stories() {
        let (mut app, _dir) = make_app_with_temp_seen(0);
        app.mark_unseen(); // should not panic
    }

    // ── start_search / run_search ─────────────────────────────────────────

    #[test]
    fn start_search_sets_mode_and_clears_input() {
        let mut app = make_app_with_stories(1);
        app.search_input = "old query".into();
        app.start_search();
        assert_eq!(app.mode, Mode::Search);
        assert!(app.search_input.is_empty());
    }

    #[tokio::test]
    async fn run_search_empty_input_cancels() {
        let mut app = make_app_with_stories(1);
        app.search_input = "  ".into();
        app.run_search().await;
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.status_message.contains("cancelled"), "got: {}", app.status_message);
        assert!(app.search_query.is_none());
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
    async fn load_comments_marks_story_as_seen() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/item/10.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 10, "type": "comment", "text": "hi", "by": "alice",
            })))
            .mount(&server)
            .await;

        let dir = tempfile::tempdir().unwrap();
        let mut app = test_app(&server.uri());
        app.seen_path = dir.path().join("seen.json");
        app.seen_ids = HashSet::new();
        let mut story = make_item(1);
        story.kids = Some(vec![10]);
        app.stories = vec![story];

        app.load_comments().await;

        assert!(app.seen_ids.contains(&1), "story should be seen after loading comments");
        let on_disk = crate::seen::load(&dir.path().join("seen.json"));
        assert!(on_disk.contains(&1), "seen state should persist to disk");
    }

    fn test_search_app(search_base: &str) -> App {
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        app.search_base = search_base.to_string();
        app
    }

    #[tokio::test]
    async fn run_search_populates_stories_on_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "hits": [
                    {"objectID": "1", "title": "Rust is great", "url": "https://example.com/1",
                     "author": "alice", "points": 200, "num_comments": 42, "created_at_i": 0},
                    {"objectID": "2", "title": "Go is fast", "url": "https://example.com/2",
                     "author": "bob", "points": 150, "num_comments": 20, "created_at_i": 0},
                ]
            })))
            .mount(&server)
            .await;

        let mut app = test_search_app(&server.uri());
        app.search_input = "rust".into();
        app.run_search().await;

        assert_eq!(app.stories.len(), 2, "expected 2 results");
        assert_eq!(app.stories[0].display_title(), "Rust is great");
        assert_eq!(app.stories[1].display_title(), "Go is fast");
        assert_eq!(app.search_query.as_deref(), Some("rust"));
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.status_message.contains("rust"), "got: {}", app.status_message);
    }

    #[tokio::test]
    async fn run_search_error_clears_search_query() {
        let server = MockServer::start().await;
        // no routes → 404 → JSON parse fails
        let mut app = test_search_app(&server.uri());
        app.search_input = "rust".into();
        app.search_query = Some("old".into());
        app.run_search().await;

        assert!(app.search_query.is_none());
        assert!(app.status_message.contains("Search error"), "got: {}", app.status_message);
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

    // ── Algolia / search result comment loading ───────────────────────────

    fn algolia_node(id: u64) -> CommentNode {
        CommentNode { item: make_item(id), text: String::new(), depth: 0, collapsed: false, children: vec![] }
    }

    #[test]
    fn tick_progress_shows_for_algolia_story_with_descendants() {
        // Algolia items have kids=None but descendants>0 — progress should show
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        let mut story = make_item(1);
        story.kids = None;
        story.descendants = Some(42);
        app.stories = vec![story];

        app.tick_progress();

        assert!(app.progress.is_some(), "should show Fetching comments when descendants>0 and kids=None");
    }

    #[test]
    fn tick_progress_hidden_when_no_descendants() {
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        let mut story = make_item(1);
        story.kids = None;
        story.descendants = Some(0);
        app.stories = vec![story];

        app.tick_progress();

        assert!(app.progress.is_none());
    }

    #[tokio::test]
    async fn load_comments_hydrates_kids_from_hn_for_algolia_story() {
        let server = MockServer::start().await;
        // HN item endpoint returns kids that Algolia didn't provide
        Mock::given(method("GET"))
            .and(path("/item/1.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 1, "type": "story", "title": "Story 1",
                "kids": [10], "descendants": 1,
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/item/10.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 10, "type": "comment", "text": "hello", "by": "alice",
            })))
            .mount(&server)
            .await;

        let mut app = test_app(&server.uri());
        let mut story = make_item(1);
        story.kids = None; // Algolia-style: no kids
        story.descendants = Some(1);
        app.stories = vec![story];

        app.load_comments().await;

        assert_eq!(app.comments.len(), 1);
        assert_eq!(app.comments[0].item.id, 10);
    }

    #[tokio::test]
    async fn load_comments_uses_cache_when_prefetched() {
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        let mut story = make_item(1);
        story.kids = Some(vec![10]);
        app.stories = vec![story];
        // Simulate already-prefetched comments
        app.comments_story_id = Some(1);
        app.comments = vec![algolia_node(10)];

        app.load_comments().await;

        // Should reuse cache, not re-fetch (no network available in this test)
        assert_eq!(app.comments.len(), 1);
        assert_eq!(app.comments[0].item.id, 10);
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
