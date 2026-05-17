use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    pub id: u64,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub kind: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub text: Option<String>,
    pub by: Option<String>,
    pub score: Option<i64>,
    #[allow(dead_code)]
    pub time: Option<u64>,
    pub descendants: Option<u64>,
    pub kids: Option<Vec<u64>>,
    #[allow(dead_code)]
    pub parent: Option<u64>,
    pub deleted: Option<bool>,
    pub dead: Option<bool>,
}

impl Item {
    pub fn display_title(&self) -> &str {
        self.title.as_deref().unwrap_or("[untitled]")
    }

    pub fn display_by(&self) -> &str {
        self.by.as_deref().unwrap_or("?")
    }

    pub fn comment_count(&self) -> u64 {
        self.descendants.unwrap_or(0)
    }

    pub fn score(&self) -> i64 {
        self.score.unwrap_or(0)
    }

    pub fn is_deleted_or_dead(&self) -> bool {
        self.deleted.unwrap_or(false) || self.dead.unwrap_or(false)
    }

    pub fn time_ago(&self) -> String {
        let t = match self.time {
            Some(t) => t,
            None => return String::new(),
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let secs = now.saturating_sub(t);
        if secs < 60 { format!("{secs}s ago") }
        else if secs < 3600 { format!("{}m ago", secs / 60) }
        else if secs < 86400 { format!("{}h ago", secs / 3600) }
        else { format!("{}d ago", secs / 86400) }
    }

    pub fn text_plain(&self) -> String {
        match &self.text {
            Some(t) => html2text::from_read(t.as_bytes(), 80),
            None => String::new(),
        }
    }
}

pub async fn fetch_top_ids(client: &reqwest::Client, base: &str) -> anyhow::Result<Vec<u64>> {
    let ids: Vec<u64> = client
        .get(format!("{base}/topstories.json"))
        .send()
        .await?
        .json()
        .await?;
    Ok(ids)
}

pub async fn fetch_new_ids(client: &reqwest::Client, base: &str) -> anyhow::Result<Vec<u64>> {
    let ids: Vec<u64> = client
        .get(format!("{base}/newstories.json"))
        .send()
        .await?
        .json()
        .await?;
    Ok(ids)
}

pub async fn fetch_best_ids(client: &reqwest::Client, base: &str) -> anyhow::Result<Vec<u64>> {
    let ids: Vec<u64> = client
        .get(format!("{base}/beststories.json"))
        .send()
        .await?
        .json()
        .await?;
    Ok(ids)
}

pub async fn fetch_ask_ids(client: &reqwest::Client, base: &str) -> anyhow::Result<Vec<u64>> {
    let ids: Vec<u64> = client
        .get(format!("{base}/askstories.json"))
        .send()
        .await?
        .json()
        .await?;
    Ok(ids)
}

pub async fn fetch_show_ids(client: &reqwest::Client, base: &str) -> anyhow::Result<Vec<u64>> {
    let ids: Vec<u64> = client
        .get(format!("{base}/showstories.json"))
        .send()
        .await?
        .json()
        .await?;
    Ok(ids)
}

pub async fn fetch_job_ids(client: &reqwest::Client, base: &str) -> anyhow::Result<Vec<u64>> {
    let ids: Vec<u64> = client
        .get(format!("{base}/jobstories.json"))
        .send()
        .await?
        .json()
        .await?;
    Ok(ids)
}

pub async fn fetch_item(client: &reqwest::Client, id: u64, base: &str) -> anyhow::Result<Item> {
    let item: Item = client
        .get(format!("{base}/item/{id}.json"))
        .send()
        .await?
        .json()
        .await?;
    Ok(item)
}

pub async fn fetch_items(client: &reqwest::Client, ids: &[u64], base: &str) -> Vec<Item> {
    let futures: Vec<_> = ids
        .iter()
        .map(|&id| fetch_item(client, id, base))
        .collect();
    let results = futures::future::join_all(futures).await;
    results.into_iter().flatten().collect()
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: String,
    pub karma: i64,
    pub created: u64,
    pub about: Option<String>,
    pub submitted: Option<Vec<u64>>,
}

impl User {
    pub fn about_plain(&self) -> String {
        match &self.about {
            Some(t) => html2text::from_read(t.as_bytes(), 80),
            None => String::new(),
        }
    }

    pub fn submission_count(&self) -> usize {
        self.submitted.as_ref().map(|v| v.len()).unwrap_or(0)
    }

    pub fn joined_ago(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let secs = now.saturating_sub(self.created);
        let days = secs / 86400;
        if days < 30 {
            format!("{days} days ago")
        } else if days < 365 {
            format!("{} months ago", days / 30)
        } else {
            format!("{} years ago", days / 365)
        }
    }
}

#[cfg(test)]
pub(crate) fn make_item(id: u64) -> Item {
    Item {
        id,
        kind: None,
        title: Some(format!("Story {id}")),
        url: Some(format!("https://example.com/{id}")),
        text: None,
        by: Some(format!("user{id}")),
        score: Some(100),
        time: None,
        descendants: Some(10),
        kids: None,
        parent: None,
        deleted: None,
        dead: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u64) -> Item {
        make_item(id)
    }

    fn deleted_item(id: u64) -> Item {
        Item { id, deleted: Some(true), ..item(id) }
    }

    fn dead_item(id: u64) -> Item {
        Item { id, dead: Some(true), ..item(id) }
    }

    fn bare_item(id: u64) -> Item {
        Item {
            id,
            kind: None, title: None, url: None, text: None,
            by: None, score: None, time: None, descendants: None,
            kids: None, parent: None, deleted: None, dead: None,
        }
    }

    #[test]
    fn display_title_falls_back() {
        assert_eq!(bare_item(1).display_title(), "[untitled]");
        assert_eq!(item(1).display_title(), "Story 1");
    }

    #[test]
    fn display_by_falls_back() {
        assert_eq!(bare_item(1).display_by(), "?");
        assert_eq!(item(1).display_by(), "user1");
    }

    #[test]
    fn score_defaults_zero() {
        assert_eq!(bare_item(1).score(), 0);
        assert_eq!(item(1).score(), 100);
    }

    #[test]
    fn comment_count_defaults_zero() {
        assert_eq!(bare_item(1).comment_count(), 0);
        assert_eq!(item(1).comment_count(), 10);
    }

    #[test]
    fn deleted_and_dead_detection() {
        assert!(!item(1).is_deleted_or_dead());
        assert!(deleted_item(1).is_deleted_or_dead());
        assert!(dead_item(1).is_deleted_or_dead());
    }

    #[test]
    fn text_plain_empty_when_no_text() {
        assert_eq!(bare_item(1).text_plain(), "");
    }

    #[test]
    fn text_plain_strips_html() {
        let item = Item { text: Some("<p>Hello <b>world</b></p>".into()), ..bare_item(1) };
        let plain = item.text_plain();
        assert!(plain.contains("Hello"));
        assert!(plain.contains("world"));
        assert!(!plain.contains("<p>"));
    }

    #[test]
    fn user_submission_count() {
        let user = User {
            id: "pg".into(),
            karma: 155000,
            created: 1000000,
            about: None,
            submitted: Some(vec![1, 2, 3]),
        };
        assert_eq!(user.submission_count(), 3);

        let no_subs = User { submitted: None, ..user };
        assert_eq!(no_subs.submission_count(), 0);
    }

    #[test]
    fn user_joined_ago_formats_correctly() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let recent = User { id: "x".into(), karma: 1, created: now - 5 * 86400, about: None, submitted: None };
        assert!(recent.joined_ago().contains("days ago"), "got: {}", recent.joined_ago());

        let months = User { created: now - 60 * 86400, ..recent.clone() };
        assert!(months.joined_ago().contains("months ago"), "got: {}", months.joined_ago());

        let years = User { created: now - 400 * 86400, ..recent.clone() };
        assert!(years.joined_ago().contains("years ago"), "got: {}", years.joined_ago());
    }

    #[test]
    fn user_about_strips_html() {
        let user = User {
            id: "pg".into(),
            karma: 1,
            created: 0,
            about: Some("<a href='http://x.com'>My site</a>".into()),
            submitted: None,
        };
        let plain = user.about_plain();
        assert!(plain.contains("My site"));
        assert!(!plain.contains("<a"));
    }

    #[test]
    fn time_ago_none_returns_empty() {
        assert_eq!(bare_item(1).time_ago(), "");
    }

    #[test]
    fn time_ago_formats_correctly() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let seconds = Item { time: Some(now - 30), ..bare_item(1) };
        assert!(seconds.time_ago().contains("s ago"), "got: {}", seconds.time_ago());

        let minutes = Item { time: Some(now - 5 * 60), ..bare_item(1) };
        assert!(minutes.time_ago().contains("m ago"), "got: {}", minutes.time_ago());

        let hours = Item { time: Some(now - 3 * 3600), ..bare_item(1) };
        assert!(hours.time_ago().contains("h ago"), "got: {}", hours.time_ago());

        let days = Item { time: Some(now - 2 * 86400), ..bare_item(1) };
        assert!(days.time_ago().contains("d ago"), "got: {}", days.time_ago());
    }
}

#[derive(Deserialize)]
struct AlgoliaHit {
    #[serde(rename = "objectID")]
    object_id: String,
    title: Option<String>,
    url: Option<String>,
    author: Option<String>,
    points: Option<i64>,
    num_comments: Option<u64>,
    created_at_i: Option<u64>,
}

#[derive(Deserialize)]
struct AlgoliaResponse {
    hits: Vec<AlgoliaHit>,
}

impl AlgoliaHit {
    fn into_item(self) -> Option<Item> {
        let id = self.object_id.parse::<u64>().ok()?;
        Some(Item {
            id,
            kind: Some("story".into()),
            title: self.title,
            url: self.url,
            text: None,
            by: self.author,
            score: self.points,
            time: self.created_at_i,
            descendants: self.num_comments,
            kids: None,
            parent: None,
            deleted: None,
            dead: None,
        })
    }
}

pub async fn search_stories(
    client: &reqwest::Client,
    query: &str,
    base: &str,
) -> anyhow::Result<Vec<Item>> {
    let resp: AlgoliaResponse = client
        .get(format!("{base}/search"))
        .query(&[("query", query), ("tags", "story"), ("hitsPerPage", "30")])
        .send()
        .await?
        .json()
        .await?;
    Ok(resp.hits.into_iter().filter_map(|h| h.into_item()).collect())
}

pub async fn fetch_readable(client: &reqwest::Client, url: &str) -> anyhow::Result<(String, String)> {
    let html = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; hnr/1.0)")
        .send()
        .await?
        .text()
        .await?;
    let parsed = url::Url::parse(url)?;
    let mut cursor = std::io::Cursor::new(html.as_bytes());
    let product = readability::extractor::extract(&mut cursor, &parsed)?;
    let text = if product.text.trim().is_empty() {
        html2text::from_read(product.content.as_bytes(), 80)
    } else {
        product.text
    };
    Ok((product.title, text))
}

pub async fn fetch_latest_version(client: &reqwest::Client) -> anyhow::Result<String> {
    #[derive(Deserialize)]
    struct Krate { newest_version: String }
    #[derive(Deserialize)]
    struct Resp { #[serde(rename = "crate")] krate: Krate }

    let resp: Resp = client
        .get("https://crates.io/api/v1/crates/hnr")
        .header("User-Agent", concat!("hnr/", env!("CARGO_PKG_VERSION")))
        .send()
        .await?
        .json()
        .await?;
    Ok(resp.krate.newest_version)
}

pub async fn fetch_user(client: &reqwest::Client, username: &str, base: &str) -> anyhow::Result<User> {
    let user: User = client
        .get(format!("{base}/user/{username}.json"))
        .send()
        .await?
        .json()
        .await?;
    Ok(user)
}

pub async fn login(username: &str, password: &str) -> anyhow::Result<String> {
    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let params = [("acct", username), ("pw", password), ("goto", "news")];
    let resp = client
        .post("https://news.ycombinator.com/login")
        .form(&params)
        .send()
        .await?;

    for value in resp.headers().get_all(reqwest::header::SET_COOKIE) {
        let s = value.to_str().unwrap_or("");
        if s.starts_with("user=") && !s.contains("user=deleted") {
            let cookie = s.split(';').next().unwrap_or("").trim().to_string();
            return Ok(cookie);
        }
    }
    anyhow::bail!("Login failed — check your credentials")
}

pub async fn fetch_vote_auth(
    client: &reqwest::Client,
    cookie: &str,
    item_id: u64,
    story_id: u64,
) -> anyhow::Result<String> {
    let html = client
        .get(format!("https://news.ycombinator.com/item?id={story_id}"))
        .header("Cookie", cookie)
        .send()
        .await?
        .text()
        .await?;

    // HN HTML-encodes & as &amp; inside href attributes
    let needle = format!("vote?id={item_id}&amp;how=up&amp;auth=");
    if let Some(pos) = html.find(&needle) {
        let after = &html[pos + needle.len()..];
        let auth: String = after.chars().take_while(|c| c.is_alphanumeric()).collect();
        if !auth.is_empty() {
            return Ok(auth);
        }
    }
    anyhow::bail!("Vote auth not found — already voted, or item is too old")
}

pub async fn vote_item(
    client: &reqwest::Client,
    cookie: &str,
    item_id: u64,
    auth: &str,
    story_id: u64,
) -> anyhow::Result<()> {
    let url = format!(
        "https://news.ycombinator.com/vote?id={item_id}&how=up&auth={auth}&goto=item%3Fid%3D{story_id}"
    );
    client
        .get(&url)
        .header("Cookie", cookie)
        .send()
        .await?;
    Ok(())
}

pub async fn fetch_reply_hmac(
    client: &reqwest::Client,
    cookie: &str,
    parent_id: u64,
) -> anyhow::Result<String> {
    let html = client
        .get(format!("https://news.ycombinator.com/reply?id={parent_id}"))
        .header("Cookie", cookie)
        .send()
        .await?
        .text()
        .await?;

    let needle = r#"name="hmac" value=""#;
    if let Some(pos) = html.find(needle) {
        let after = &html[pos + needle.len()..];
        let hmac: String = after.chars().take_while(|c| *c != '"').collect();
        if !hmac.is_empty() {
            return Ok(hmac);
        }
    }
    anyhow::bail!("Could not get reply token — are you logged in?")
}

pub async fn post_comment(
    client: &reqwest::Client,
    cookie: &str,
    parent_id: u64,
    story_id: u64,
    hmac: &str,
    text: &str,
) -> anyhow::Result<()> {
    let params = [
        ("parent", parent_id.to_string()),
        ("goto", format!("item?id={story_id}")),
        ("hmac", hmac.to_string()),
        ("text", text.to_string()),
    ];
    client
        .post("https://news.ycombinator.com/comment")
        .header("Cookie", cookie)
        .form(&params)
        .send()
        .await?;
    Ok(())
}
