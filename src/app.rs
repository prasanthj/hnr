use crate::api::{fetch_ask_ids, fetch_best_ids, fetch_new_ids, fetch_show_ids, fetch_top_ids, Item, User};
use std::collections::HashMap;

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
pub enum Mode {
    Normal,
    Command,
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
        Self {
            item,
            depth,
            collapsed: false,
            children: vec![],
        }
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

    pub loading: bool,
    pub client: reqwest::Client,
    pub item_cache: HashMap<u64, Item>,
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
            loading: false,
            client,
            item_cache: HashMap::new(),
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
                    "{} stories loaded | j/k navigate | Enter open | Tab switch pane | / command",
                    self.stories.len()
                );
            }
            Err(e) => {
                self.status_message = format!("Error: {e}");
            }
        }
        self.loading = false;
    }

    pub async fn load_comments(&mut self) {
        if let Some(story) = self.selected_story().cloned() {
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
            self.comment_cursor = 0;
            self.comment_scroll = 0;
            self.status_message = format!(
                "{} top-level comments | j/k navigate | Space collapse | Tab switch pane",
                self.comments.len()
            );
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

    pub async fn load_user(&mut self, username: String) {
        self.status_message = format!("Loading profile for {username}...");
        self.loading = true;
        let client = self.client.clone();
        match crate::api::fetch_user(&client, &username).await {
            Ok(user) => {
                self.status_message = format!(
                    "{} | karma: {} | Esc to go back",
                    user.id, user.karma
                );
                self.user_profile = Some(user);
                self.view_mode = ViewMode::User;
            }
            Err(e) => {
                self.status_message = format!("Failed to load user: {e}");
            }
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

fn toggle_in_tree(nodes: &mut Vec<CommentNode>, id: u64) {
    for node in nodes.iter_mut() {
        if node.item.id == id {
            node.collapsed = !node.collapsed;
            return;
        }
        toggle_in_tree(&mut node.children, id);
    }
}

async fn load_comment_tree(
    client: &reqwest::Client,
    ids: &[u64],
    depth: usize,
) -> Vec<CommentNode> {
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
