use crate::api::{fetch_ask_ids, fetch_best_ids, fetch_new_ids, fetch_show_ids, fetch_top_ids, Item, User};
use crate::session::Session;
use std::collections::HashMap;
use arboard::Clipboard;

#[derive(Debug, Clone, PartialEq)]
pub enum Feed {
    Top,
    New,
    Best,
    Ask,
    Show,
}

impl Feed {
    pub fn label(&self) -> &str {
        match self {
            Feed::Top => "Top",
            Feed::New => "New",
            Feed::Best => "Best",
            Feed::Ask => "Ask HN",
            Feed::Show => "Show HN",
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

    pub loading: bool,
    pub client: reqwest::Client,
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
            loading: false,
            client,
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
        self.loading = true;
        self.status_message = format!("Loading {} stories...", self.feed.label());
        let client = self.client.clone();
        let ids = match self.feed {
            Feed::Top => fetch_top_ids(&client).await,
            Feed::New => fetch_new_ids(&client).await,
            Feed::Best => fetch_best_ids(&client).await,
            Feed::Ask => fetch_ask_ids(&client).await,
            Feed::Show => fetch_show_ids(&client).await,
        };
        match ids {
            Ok(mut ids) => {
                ids.truncate(60);
                self.story_ids = ids.clone();
                self.status_message = format!("Fetching {} items...", ids.len());
                let items = crate::api::fetch_items(&client, &ids).await;
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
            let nodes = load_comment_tree(&client, &kids, 0).await;
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
        match crate::api::fetch_user(&client, &username).await {
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
    use crate::api::make_item;

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

async fn load_comment_tree(client: &reqwest::Client, ids: &[u64], depth: usize) -> Vec<CommentNode> {
    let items = crate::api::fetch_items(client, ids).await;
    let mut nodes = vec![];
    for item in items {
        if item.is_deleted_or_dead() {
            continue;
        }
        let kids = item.kids.clone().unwrap_or_default();
        let mut node = CommentNode::new(item, depth);
        if !kids.is_empty() && depth < 6 {
            node.children = Box::pin(load_comment_tree(client, &kids, depth + 1)).await;
        }
        nodes.push(node);
    }
    nodes
}
