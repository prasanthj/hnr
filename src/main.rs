mod api;
mod app;
mod ui;

use app::{App, Feed, Mode, Pane, ViewMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let client = reqwest::Client::new();
    let mut app = App::new(client);
    app.load_feed().await;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let ev = event::read()?;
        if let Event::Key(key) = ev {
            match app.mode {
                Mode::Command => {
                    match key.code {
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                            app.command_input.clear();
                        }
                        KeyCode::Enter => {
                            let cmd = app.command_input.trim().to_lowercase();
                            app.command_input.clear();
                            app.mode = Mode::Normal;
                            handle_command(&mut app, &cmd).await;
                        }
                        KeyCode::Backspace => {
                            app.command_input.pop();
                        }
                        KeyCode::Char(c) => {
                            app.command_input.push(c);
                        }
                        _ => {}
                    }
                }
                Mode::Normal => {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break;
                    }

                    if key.code == KeyCode::Char('/') {
                        app.mode = Mode::Command;
                        continue;
                    }

                    // user profile view intercepts Esc and o
                    if app.view_mode == ViewMode::User {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => app.close_user_profile(),
                            KeyCode::Char('o') => {
                                if let Some(user) = &app.user_profile {
                                    let url = format!("https://news.ycombinator.com/user?id={}", user.id);
                                    let _ = open::that(url);
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match app.active_pane {
                        Pane::Stories => match key.code {
                            KeyCode::Char('j') | KeyCode::Down => {
                                let h = terminal.size()?.height as usize;
                                app.scroll_story_down(h.saturating_sub(10));
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                app.scroll_story_up();
                            }
                            KeyCode::Enter => {
                                app.active_pane = Pane::Comments;
                                app.load_comments().await;
                            }
                            KeyCode::Tab => {
                                if !app.comments.is_empty() {
                                    app.active_pane = Pane::Comments;
                                }
                            }
                            KeyCode::Char('u') => {
                                if let Some(name) = app.username_at_cursor() {
                                    app.load_user(name).await;
                                }
                            }
                            KeyCode::Char('o') => app.open_story_in_browser(),
                            KeyCode::Char('O') => app.open_hn_page_in_browser(),
                            KeyCode::Char('r') => app.load_feed().await,
                            KeyCode::Char('1') => switch_feed(&mut app, Feed::Top).await,
                            KeyCode::Char('2') => switch_feed(&mut app, Feed::New).await,
                            KeyCode::Char('3') => switch_feed(&mut app, Feed::Best).await,
                            KeyCode::Char('4') => switch_feed(&mut app, Feed::Ask).await,
                            KeyCode::Char('5') => switch_feed(&mut app, Feed::Show).await,
                            _ => {}
                        },
                        Pane::Comments => match key.code {
                            KeyCode::Char('j') | KeyCode::Down => {
                                let h = terminal.size()?.height as usize;
                                app.scroll_comment_down(h.saturating_sub(10));
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                app.scroll_comment_up();
                            }
                            KeyCode::Char(' ') => app.toggle_current_comment(),
                            KeyCode::Tab | KeyCode::Esc => app.active_pane = Pane::Stories,
                            KeyCode::Char('u') => {
                                if let Some(name) = app.username_at_cursor() {
                                    app.load_user(name).await;
                                }
                            }
                            KeyCode::Char('o') => app.open_story_in_browser(),
                            KeyCode::Char('O') => app.open_hn_page_in_browser(),
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

async fn switch_feed(app: &mut App, feed: Feed) {
    if app.feed != feed {
        app.feed = feed;
        app.load_feed().await;
    }
}

async fn handle_command(app: &mut App, cmd: &str) {
    let mut parts = cmd.splitn(2, ' ');
    let verb = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("").trim();

    match verb {
        "q" | "quit" | "exit" => std::process::exit(0),
        "top" | "1" => switch_feed(app, Feed::Top).await,
        "new" | "2" => switch_feed(app, Feed::New).await,
        "best" | "3" => switch_feed(app, Feed::Best).await,
        "ask" | "4" => switch_feed(app, Feed::Ask).await,
        "show" | "5" => switch_feed(app, Feed::Show).await,
        "refresh" | "r" => app.load_feed().await,
        "open" | "o" => app.open_story_in_browser(),
        "hn" => app.open_hn_page_in_browser(),
        "user" | "u" => {
            let name = if arg.is_empty() {
                app.username_at_cursor()
            } else {
                Some(arg.to_string())
            };
            if let Some(name) = name {
                app.load_user(name).await;
            } else {
                app.status_message = "Usage: /user <username>".into();
            }
        }
        "help" | "?" => {
            app.status_message =
                "Commands: top/new/best/ask/show | user <name> | refresh | open | hn | quit | 1-5 feeds".into();
        }
        other => {
            app.status_message = format!("Unknown command: '{other}'. Try /help");
        }
    }
}
