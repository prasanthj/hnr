mod api;
mod app;
mod bookmarks;
mod reader;
mod seen;
mod session;
mod shimmer;
mod theme;
mod ui;

use app::{App, Feed, LoginField, Mode, Pane, ViewMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("hnr {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let client = reqwest::Client::new();
    let mut app = App::new(client);

    if app.session.is_some() {
        let name = app.session.as_ref().unwrap().username.clone();
        app.status_message = format!("Logged in as {name} | Loading...");
    }

    let update_flag = app.update_available.clone();
    let version_client = app.client.clone();
    tokio::spawn(async move {
        if let Ok(latest) = crate::api::fetch_latest_version(&version_client).await {
            if semver_gt(&latest, env!("CARGO_PKG_VERSION")) {
                if let Ok(mut flag) = update_flag.lock() {
                    *flag = Some(latest);
                }
            }
        }
    });

    app.load_feed().await;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;
        app.apply_pending_reader();
        app.apply_pending_comments();
        app.maybe_start_prefetch();
        app.tick_progress();

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            match app.mode.clone() {
                Mode::Login => handle_login_keys(&mut app, key.code, key.modifiers).await,
                Mode::Compose => handle_compose_keys(&mut app, key.code, key.modifiers).await,
                Mode::Command => handle_command_keys(&mut app, key.code).await,
                Mode::Search => handle_search_keys(&mut app, key.code).await,
                Mode::Reader => {
                    let sz = terminal.size()?;
                    let inner_h = (sz.height as usize * 90 / 100).saturating_sub(2);
                    handle_reader_keys(&mut app, key.code, inner_h);
                }
                Mode::Normal => {
                    let h = terminal.size()?.height as usize;
                    handle_normal_keys(&mut app, key.code, key.modifiers, h).await;
                    if app.mode == Mode::Normal
                        && matches!(key.code, KeyCode::Char('q'))
                        && app.view_mode != ViewMode::User
                    {
                        break;
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

async fn handle_login_keys(app: &mut App, code: KeyCode, _mods: KeyModifiers) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.status_message = "Login cancelled.".into();
        }
        KeyCode::Tab | KeyCode::BackTab => {
            app.login_state.field = match app.login_state.field {
                LoginField::Username => LoginField::Password,
                LoginField::Password => LoginField::Username,
            };
        }
        KeyCode::Enter => match app.login_state.field {
            LoginField::Username => app.login_state.field = LoginField::Password,
            LoginField::Password => app.submit_login().await,
        },
        KeyCode::Backspace => {
            match app.login_state.field {
                LoginField::Username => { app.login_state.username.pop(); }
                LoginField::Password => { app.login_state.password.pop(); }
            }
        }
        KeyCode::Char(c) => {
            match app.login_state.field {
                LoginField::Username => app.login_state.username.push(c),
                LoginField::Password => app.login_state.password.push(c),
            }
        }
        _ => {}
    }
}

async fn handle_compose_keys(app: &mut App, code: KeyCode, mods: KeyModifiers) {
    match code {
        KeyCode::Esc => {
            app.compose_state = None;
            app.mode = Mode::Normal;
            app.status_message = "Compose cancelled.".into();
        }
        KeyCode::Char('s') if mods.contains(KeyModifiers::CONTROL) => {
            app.submit_comment().await;
        }
        KeyCode::Enter => {
            if let Some(state) = &mut app.compose_state {
                state.text.push('\n');
            }
        }
        KeyCode::Backspace => {
            if let Some(state) = &mut app.compose_state {
                state.text.pop();
            }
        }
        KeyCode::Char(c) => {
            if let Some(state) = &mut app.compose_state {
                state.text.push(c);
            }
        }
        _ => {}
    }
}

async fn handle_command_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.command_input.clear();
        }
        KeyCode::Enter => {
            let cmd = app.command_input.trim().to_lowercase().to_string();
            app.command_input.clear();
            app.mode = Mode::Normal;
            run_command(app, &cmd).await;
        }
        KeyCode::Backspace => { app.command_input.pop(); }
        KeyCode::Char(c) => app.command_input.push(c),
        _ => {}
    }
}

async fn handle_search_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.search_input.clear();
            app.status_message = "Search cancelled.".into();
        }
        KeyCode::Enter => app.run_search().await,
        KeyCode::Backspace => { app.search_input.pop(); }
        KeyCode::Char(c) => app.search_input.push(c),
        _ => {}
    }
}

async fn handle_normal_keys(app: &mut App, code: KeyCode, mods: KeyModifiers, height: usize) {
    if code == KeyCode::Char('c') && mods.contains(KeyModifiers::CONTROL) {
        std::process::exit(0);
    }
    if code == KeyCode::Char('/') {
        app.mode = Mode::Command;
        return;
    }
    if code == KeyCode::Char('?') {
        app.start_search();
        return;
    }

    // User profile view captures input
    if app.view_mode == ViewMode::User {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => app.close_user_profile(),
            KeyCode::Char('o') => {
                if let Some(user) = &app.user_profile {
                    let url = format!("https://news.ycombinator.com/user?id={}", user.id);
                    let _ = open::that(url);
                }
            }
            _ => {}
        }
        return;
    }

    // global keys work in both panes
    match code {
        KeyCode::Char('h') => {
            app.status_message =
                "1-7:feeds  j/k/↑↓:nav  Enter:comments/expand  Tab/Esc:pane  Space:collapse  r:read  y:copy  v:vote  c:reply  p:profile  u:unread  b:bookmark  ?:search  o:open  O:hn  R:refresh  l:login/logout  /:cmd  q:quit"
                    .into();
            return;
        }
        KeyCode::Char('R') => { app.load_feed().await; return; }
        KeyCode::Char('l') => {
            if app.session.is_some() {
                app.logout();
            } else {
                app.start_login();
            }
            return;
        }
        KeyCode::Char('1') => { switch_feed(app, Feed::Top).await; return; }
        KeyCode::Char('2') => { switch_feed(app, Feed::New).await; return; }
        KeyCode::Char('3') => { switch_feed(app, Feed::Best).await; return; }
        KeyCode::Char('4') => { switch_feed(app, Feed::Ask).await; return; }
        KeyCode::Char('5') => { switch_feed(app, Feed::Show).await; return; }
        KeyCode::Char('6') => { switch_feed(app, Feed::Jobs).await; return; }
        KeyCode::Char('7') => { switch_feed(app, Feed::Bookmarks).await; return; }
        _ => {}
    }

    let visible = height.saturating_sub(10);
    match app.active_pane {
        Pane::Stories => match code {
            KeyCode::Char('j') | KeyCode::Down  => app.scroll_story_down(visible),
            KeyCode::Char('k') | KeyCode::Up    => app.scroll_story_up(),
            KeyCode::Enter => {
                app.active_pane = Pane::Comments;
                app.load_comments().await;
            }
            KeyCode::Tab => {
                if !app.comments.is_empty() {
                    app.active_pane = Pane::Comments;
                }
            }
            KeyCode::Char('p') => {
                if let Some(name) = app.username_at_cursor() {
                    app.load_user(name).await;
                }
            }
            KeyCode::Char('u') => app.mark_unseen(),
            KeyCode::Char('b') => app.toggle_bookmark(),
            KeyCode::Char('v') => app.vote_current().await,
            KeyCode::Char('c') => app.start_compose().await,
            KeyCode::Char('o') => app.open_story_in_browser(),
            KeyCode::Char('O') => app.open_hn_page_in_browser(),
            KeyCode::Char('y') => app.copy_url_to_clipboard(),
            KeyCode::Char('r') => app.load_reader(),
            _ => {}
        },
        Pane::Comments => match code {
            KeyCode::Char('j') | KeyCode::Down  => app.scroll_comment_down(visible),
            KeyCode::Char('k') | KeyCode::Up    => app.scroll_comment_up(),
            KeyCode::Enter                       => app.toggle_comment_expand(),
            KeyCode::Char(' ')                   => app.toggle_current_comment(),
            KeyCode::Tab | KeyCode::Esc          => {
                app.save_comment_pos();
                app.active_pane = Pane::Stories;
            }
            KeyCode::Char('p') => {
                if let Some(name) = app.username_at_cursor() {
                    app.load_user(name).await;
                }
            }
            KeyCode::Char('u') => app.mark_unseen(),
            KeyCode::Char('b') => app.toggle_bookmark(),
            KeyCode::Char('v') => app.vote_current().await,
            KeyCode::Char('c') => app.start_compose().await,
            KeyCode::Char('o') => app.open_story_in_browser(),
            KeyCode::Char('O') => app.open_hn_page_in_browser(),
            KeyCode::Char('y') => app.copy_url_to_clipboard(),
            KeyCode::Char('r') => app.load_reader(),
            _ => {}
        },
    }
}

fn handle_reader_keys(app: &mut App, code: KeyCode, inner_h: usize) {
    let half = (inner_h / 2).max(1);
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.reader_scroll += 1,
        KeyCode::Char('k') | KeyCode::Up => app.reader_scroll = app.reader_scroll.saturating_sub(1),
        KeyCode::Char('d') => app.reader_scroll += half,
        KeyCode::Char('u') => app.reader_scroll = app.reader_scroll.saturating_sub(half),
        KeyCode::Char('g') => app.reader_scroll = 0,
        KeyCode::Char('G') => {
            app.reader_scroll = app.reader_total_lines.saturating_sub(inner_h);
        }
        KeyCode::Char('n') => {
            if let Some(&next) = app.reader_section_offsets.iter().find(|&&o| o > app.reader_scroll) {
                app.reader_scroll = next;
            }
        }
        KeyCode::Char('N') => {
            if let Some(&prev) = app.reader_section_offsets.iter().rev().find(|&&o| o < app.reader_scroll) {
                app.reader_scroll = prev;
            }
        }
        KeyCode::Char('o') => { app.close_reader(); app.open_story_in_browser(); }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('r') => app.close_reader(),
        _ => {}
    }
}


fn parse_ver(v: &str) -> (u64, u64, u64) {
    let mut parts = v.splitn(3, '.').map(|p| p.parse::<u64>().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

fn semver_gt(a: &str, b: &str) -> bool {
    parse_ver(a) > parse_ver(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn semver_gt_handles_double_digit_patch() {
        assert!(semver_gt("0.3.10", "0.3.9"));
        assert!(semver_gt("0.3.10", "0.3.8"));
        assert!(!semver_gt("0.3.8", "0.3.10"));
        assert!(!semver_gt("0.3.10", "0.3.10"));
        assert!(semver_gt("1.0.0", "0.9.99"));
    }
}

async fn switch_feed(app: &mut App, feed: Feed) {
    if app.feed != feed || app.search_query.is_some() {
        app.feed = feed;
        app.search_query = None;
        app.load_feed().await;
    }
}

async fn run_command(app: &mut App, cmd: &str) {
    let mut parts = cmd.splitn(2, ' ');
    let verb = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim();

    match verb {
        "q" | "quit" | "exit" => std::process::exit(0),
        "login"  | "l" if app.session.is_none() => app.start_login(),
        "logout" | "l" if app.session.is_some() => app.logout(),
        "l"                   => app.start_login(),
        "top"  | "1"          => switch_feed(app, Feed::Top).await,
        "new"  | "2"          => switch_feed(app, Feed::New).await,
        "best" | "3"          => switch_feed(app, Feed::Best).await,
        "ask"  | "4"          => switch_feed(app, Feed::Ask).await,
        "show"      | "5"      => switch_feed(app, Feed::Show).await,
        "jobs"      | "6"      => switch_feed(app, Feed::Jobs).await,
        "bookmarks" | "7"      => switch_feed(app, Feed::Bookmarks).await,
        "bookmark"  | "b"      => app.toggle_bookmark(),
        "search"    | "s"      => app.start_search(),
        "refresh"   | "r"      => app.load_feed().await,
        "open" | "o"          => app.open_story_in_browser(),
        "hn"                  => app.open_hn_page_in_browser(),
        "vote" | "v"          => app.vote_current().await,
        "user" | "u" | "p" => {
            let name = if arg.is_empty() { app.username_at_cursor() } else { Some(arg.to_string()) };
            if let Some(name) = name {
                app.load_user(name).await;
            } else {
                app.status_message = "Usage: /user <username>".into();
            }
        }
        "help" | "?" => {
            app.status_message =
                "login · logout · top/new/best/ask/show/bookmarks · search · user <n> · bookmark · refresh · open · hn · vote · quit".into();
        }
        other => app.status_message = format!("Unknown: '{other}' — try /help"),
    }
}
