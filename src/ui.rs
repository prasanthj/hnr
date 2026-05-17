use crate::app::{App, LoginField, Mode, Pane, ViewMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

const ORANGE: Color = Color::Rgb(255, 102, 0);
const GRAY: Color = Color::Rgb(150, 150, 150);
const DARK: Color = Color::Rgb(30, 30, 30);
const SELECTED_BG: Color = Color::Rgb(50, 40, 30);

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // body
            Constraint::Length(1), // hints
            Constraint::Length(1), // status / command
        ])
        .split(area);

    draw_header(f, app, root[0]);
    draw_body(f, app, root[1]);
    draw_hints(f, app, root[2]);
    draw_statusbar(f, app, root[3]);

    match app.mode {
        Mode::Login => draw_login_overlay(f, app, area),
        Mode::Compose => draw_compose_overlay(f, app, area),
        Mode::CommentDetail => draw_comment_detail_overlay(f, app, area),
        _ => {}
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let feeds = [
        ("1", "Top"),
        ("2", "New"),
        ("3", "Best"),
        ("4", "Ask"),
        ("5", "Show"),
        ("6", "Bookmarks"),
    ];
    let current = app.feed.label();
    let mut spans = vec![
        Span::styled(" hnr ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::raw("│ "),
    ];
    for (num, label) in &feeds {
        let is_current = current.starts_with(label);
        if is_current {
            spans.push(Span::styled(
                format!("{num}:{label}"),
                Style::default().fg(Color::Black).bg(ORANGE).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                num.to_string(),
                Style::default().fg(ORANGE),
            ));
            spans.push(Span::styled(
                format!(":{label}"),
                Style::default().fg(GRAY),
            ));
        }
        spans.push(Span::raw("  "));
    }

    if let Some(session) = &app.session {
        spans.push(Span::raw("│ "));
        spans.push(Span::styled(
            format!(" {} ", session.username),
            Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
        ));
    }

    let p = Paragraph::new(Line::from(spans)).style(Style::default().bg(DARK));
    f.render_widget(p, area);
}

fn draw_hints(f: &mut Frame, app: &App, area: Rect) {
    let spans = match app.mode {
        Mode::Login => vec![
            Span::styled(" Tab", Style::default().fg(ORANGE)),
            Span::styled(" field  ", Style::default().fg(GRAY)),
            Span::styled("Enter", Style::default().fg(ORANGE)),
            Span::styled(" submit  ", Style::default().fg(GRAY)),
            Span::styled("Esc", Style::default().fg(ORANGE)),
            Span::styled(" cancel", Style::default().fg(GRAY)),
        ],
        Mode::Compose => vec![
            Span::styled(" Ctrl+S", Style::default().fg(ORANGE)),
            Span::styled(" post  ", Style::default().fg(GRAY)),
            Span::styled("Esc", Style::default().fg(ORANGE)),
            Span::styled(" cancel", Style::default().fg(GRAY)),
        ],
        Mode::Command => vec![
            Span::styled(" Enter", Style::default().fg(ORANGE)),
            Span::styled(" run  ", Style::default().fg(GRAY)),
            Span::styled("Esc", Style::default().fg(ORANGE)),
            Span::styled(" cancel", Style::default().fg(GRAY)),
        ],
        Mode::CommentDetail => vec![
            Span::styled(" j/k", Style::default().fg(ORANGE)),
            Span::styled(" scroll  ", Style::default().fg(GRAY)),
            Span::styled("Esc", Style::default().fg(ORANGE)),
            Span::styled(" close", Style::default().fg(GRAY)),
        ],
        Mode::Search => vec![
            Span::styled(" Type", Style::default().fg(ORANGE)),
            Span::styled(" to search  ", Style::default().fg(GRAY)),
            Span::styled("Enter", Style::default().fg(ORANGE)),
            Span::styled(" run  ", Style::default().fg(GRAY)),
            Span::styled("Esc", Style::default().fg(ORANGE)),
            Span::styled(" cancel", Style::default().fg(GRAY)),
        ],
        Mode::Normal => {
            let login_hint = if app.session.is_some() { "logout" } else { "login" };
            // global + pane-specific, every shortcut unique
            let global: &[(&str, &str)] = &[
                ("h", "help"),
                ("r", "refresh"),
                ("l", login_hint),
                ("/", "cmd"),
                ("q", "quit"),
            ];
            let pane: &[(&str, &str)] = match app.active_pane {
                Pane::Stories => &[
                    ("j/k", "nav"),
                    ("Enter", "comments"),
                    ("Tab", "→pane"),
                    ("b", "bookmark"),
                    ("o", "url"),
                    ("O", "hn"),
                    ("y", "copy url"),
                    ("v", "vote"),
                    ("c", "reply"),
                    ("u", "profile"),
                ],
                Pane::Comments => &[
                    ("j/k", "nav"),
                    ("Enter", "expand"),
                    ("Space", "collapse"),
                    ("Esc", "←back"),
                    ("o", "url"),
                    ("O", "hn"),
                    ("y", "copy url"),
                    ("v", "vote"),
                    ("c", "reply"),
                    ("u", "profile"),
                ],
            };
            let sep = Span::styled("  ", Style::default().fg(GRAY));
            let divider = Span::styled(" │ ", Style::default().fg(Color::Rgb(60, 60, 60)));
            let mut s = vec![Span::raw(" ")];
            for (i, (key, desc)) in global.iter().enumerate() {
                s.push(Span::styled(key.to_string(), Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));
                s.push(Span::styled(format!(" {desc}"), Style::default().fg(GRAY)));
                if i < global.len() - 1 { s.push(sep.clone()); }
            }
            s.push(divider);
            for (i, (key, desc)) in pane.iter().enumerate() {
                s.push(Span::styled(key.to_string(), Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));
                s.push(Span::styled(format!(" {desc}"), Style::default().fg(GRAY)));
                if i < pane.len() - 1 { s.push(sep.clone()); }
            }
            s
        }
    };

    let p = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Rgb(20, 20, 20)));
    f.render_widget(p, area);
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);
    draw_story_list(f, app, chunks[0]);
    draw_detail_panel(f, app, chunks[1]);
}

fn draw_story_list(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.active_pane == Pane::Stories;
    let border_style = if focused { Style::default().fg(ORANGE) } else { Style::default().fg(GRAY) };
    let visible = (area.height as usize).saturating_sub(2);

    let items: Vec<ListItem> = app
        .stories
        .iter()
        .enumerate()
        .skip(app.story_scroll)
        .take(visible)
        .map(|(i, story)| {
            let selected = i == app.story_cursor;
            let rank = format!("{:>3}. ", i + 1);
            let ago = story.time_ago();
            let meta = format!(" ▲{} {} | {} comments{}", story.score(), story.display_by(), story.comment_count(), if ago.is_empty() { String::new() } else { format!(" | {ago}") });

            let title_style = if selected {
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let rank_style = if selected { Style::default().fg(ORANGE) } else { Style::default().fg(GRAY) };
            let bg = if selected { SELECTED_BG } else { Color::Reset };
            let bookmarked = app.bookmark_ids.contains(&story.id);

            let line1 = Line::from(vec![
                Span::styled(rank, rank_style),
                Span::styled(if bookmarked { "★ " } else { "" }, Style::default().fg(Color::Yellow)),
                Span::styled(story.display_title(), title_style),
            ]);
            let line2 = Line::from(vec![
                Span::raw("     "),
                Span::styled(meta, Style::default().fg(GRAY)),
            ]);

            ListItem::new(ratatui::text::Text::from(vec![line1, line2])).style(Style::default().bg(bg))
        })
        .collect();

    let mut state = ListState::default();
    if focused {
        state.select(Some(app.story_cursor.saturating_sub(app.story_scroll)));
    }

    let title = if let Some(q) = &app.search_query {
        format!(" Search: {q} ")
    } else {
        format!(" {} Stories ", app.feed.label())
    };
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_detail_panel(f: &mut Frame, app: &App, area: Rect) {
    if app.view_mode == ViewMode::User {
        if let Some(user) = &app.user_profile {
            draw_user_profile(f, user, area);
            return;
        }
    }

    let focused = app.active_pane == Pane::Comments;
    if let Some(story) = app.selected_story() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(0)])
            .split(area);
        draw_story_header(f, story, chunks[0]);
        draw_comments(f, app, chunks[1], focused);
    } else {
        let p = Paragraph::new("Select a story to read comments.")
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(GRAY)))
            .style(Style::default().fg(GRAY));
        f.render_widget(p, area);
    }
}

fn draw_story_header(f: &mut Frame, story: &crate::api::Item, area: Rect) {
    let url = story.url.as_deref().unwrap_or("(self post)");
    let ago = story.time_ago();
    let meta = format!("▲ {}  by {}  {}  | {} comments  | o: url  O: HN", story.score(), story.display_by(), ago, story.comment_count());
    let text_body = story.text_plain();

    let mut lines = vec![
        Line::from(Span::styled(story.display_title(), Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(url, Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED))),
        Line::from(Span::styled(meta, Style::default().fg(GRAY))),
    ];
    for l in text_body.lines().take(2) {
        lines.push(Line::from(Span::styled(l.to_string(), Style::default().fg(Color::White))));
    }

    let p = Paragraph::new(lines)
        .block(Block::default().title(" Story ").borders(Borders::ALL).border_style(Style::default().fg(ORANGE)))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_comments(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    let border_style = if focused { Style::default().fg(ORANGE) } else { Style::default().fg(GRAY) };
    let flat = app.flat_comments();
    let visible = (area.height as usize).saturating_sub(2);
    let depth_colors = [Color::Cyan, Color::Green, Color::Yellow, Color::Magenta, Color::Red, Color::Blue, Color::White];

    let items: Vec<ListItem> = flat
        .iter()
        .enumerate()
        .skip(app.comment_scroll)
        .take(visible)
        .map(|(i, (node, depth))| {
            let selected = i + app.comment_scroll == app.comment_cursor;
            let indent = "  ".repeat(*depth);
            let collapse = if node.collapsed && !node.children.is_empty() {
                " [+]"
            } else if !node.collapsed && !node.children.is_empty() {
                " [-]"
            } else {
                ""
            };

            let dc = depth_colors[depth % depth_colors.len()];
            let header = format!("{indent}▸ {}{collapse}", node.item.display_by());
            let header_style = if selected {
                Style::default().fg(dc).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(dc)
            };

            let text = node.item.text_plain();
            let preview: String = text
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(area.width as usize - depth * 2 - 4)
                .collect();

            let bg = if selected { SELECTED_BG } else { Color::Reset };
            ListItem::new(ratatui::text::Text::from(vec![
                Line::from(Span::styled(header, header_style)),
                Line::from(vec![Span::raw(format!("{indent}  ")), Span::styled(preview, Style::default().fg(Color::White))]),
            ]))
            .style(Style::default().bg(bg))
        })
        .collect();

    let title = if focused { " Comments (Space collapse · v vote · c reply) " } else { " Comments " };
    f.render_widget(
        List::new(items).block(Block::default().title(title).borders(Borders::ALL).border_style(border_style)),
        area,
    );
}

fn draw_user_profile(f: &mut Frame, user: &crate::api::User, area: Rect) {
    let about = user.about_plain();
    let mut lines = vec![
        Line::from(Span::raw("")),
        Line::from(Span::styled(format!("  {}", user.id), Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("▲ {} karma", user.karma), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("Joined {}  ·  {} submissions", user.joined_ago(), user.submission_count()),
                Style::default().fg(GRAY),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("https://news.ycombinator.com/user?id={}", user.id),
                Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED),
            ),
        ]),
    ];
    if !about.is_empty() {
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled("  About", Style::default().fg(GRAY))));
        lines.push(Line::from(Span::raw("  ─────────────────────────────────────")));
        for l in about.lines().take(20) {
            lines.push(Line::from(vec![Span::raw("  "), Span::styled(l.to_string(), Style::default().fg(Color::White))]));
        }
    }
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled("  Esc back  ·  o open in browser", Style::default().fg(GRAY))));

    let p = Paragraph::new(lines)
        .block(Block::default().title(format!(" User: {} ", user.id)).borders(Borders::ALL).border_style(Style::default().fg(ORANGE)))
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_login_overlay(f: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(50, 40, area);
    f.render_widget(Clear, popup);

    let state = &app.login_state;
    let username_style = if state.field == LoginField::Username {
        Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let password_style = if state.field == LoginField::Password {
        Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let masked = "*".repeat(state.password.len());
    let cursor = "█";

    let lines = vec![
        Line::from(Span::raw("")),
        Line::from(Span::styled("  Username", Style::default().fg(GRAY))),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{}{}", state.username, if state.field == LoginField::Username { cursor } else { "" }),
                username_style,
            ),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled("  Password", Style::default().fg(GRAY))),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{}{}", masked, if state.field == LoginField::Password { cursor } else { "" }),
                password_style,
            ),
        ]),
        Line::from(Span::raw("")),
        if state.error.is_empty() {
            Line::from(Span::raw(""))
        } else {
            Line::from(Span::styled(format!("  {}", state.error), Style::default().fg(Color::Red)))
        },
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "  Tab: switch field  ·  Enter: login  ·  Esc: cancel",
            Style::default().fg(GRAY),
        )),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Login to Hacker News ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ORANGE)),
        )
        .style(Style::default().bg(DARK));
    f.render_widget(p, popup);
}

fn draw_compose_overlay(f: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(70, 60, area);
    f.render_widget(Clear, popup);

    let state = match &app.compose_state {
        Some(s) => s,
        None => return,
    };

    let mut lines = vec![];
    for line in state.text.lines() {
        lines.push(Line::from(Span::styled(line.to_string(), Style::default().fg(Color::White))));
    }
    // cursor indicator at end
    let cursor_line = if state.text.ends_with('\n') || state.text.is_empty() {
        Line::from(Span::styled("█", Style::default().fg(ORANGE)))
    } else {
        let last = state.text.lines().last().unwrap_or("").to_string();
        lines.pop();
        Line::from(vec![
            Span::styled(last, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(ORANGE)),
        ])
    };
    lines.push(cursor_line);

    let title = format!(" Reply to {} ", state.parent_by);
    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ORANGE))
                .title_bottom(Line::from(Span::styled(
                    " Ctrl+S submit  ·  Esc cancel ",
                    Style::default().fg(GRAY),
                ))),
        )
        .style(Style::default().bg(DARK))
        .wrap(Wrap { trim: false });
    f.render_widget(p, popup);
}

fn draw_statusbar(f: &mut Frame, app: &App, area: Rect) {
    let p = match app.mode {
        Mode::Command => Paragraph::new(format!(":{}", app.command_input))
            .style(Style::default().fg(Color::White).bg(DARK)),
        Mode::Search => Paragraph::new(format!("?{}", app.search_input))
            .style(Style::default().fg(Color::White).bg(DARK)),
        _ => {
            let style = if app.loading {
                Style::default().fg(ORANGE)
            } else {
                Style::default().fg(GRAY)
            };
            Paragraph::new(Line::from(Span::styled(&app.status_message, style)))
                .style(Style::default().bg(DARK))
        }
    };
    f.render_widget(p, area);
}

fn draw_comment_detail_overlay(f: &mut Frame, app: &App, area: Rect) {
    if let Some((author, text)) = &app.comment_detail {
        let popup = centered_rect(82, 72, area);
        f.render_widget(Clear, popup);

        let lines: Vec<Line> = text
            .lines()
            .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(Color::White))))
            .collect();

        let p = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" {} ", author))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(ORANGE))
                    .title_bottom(Line::from(Span::styled(
                        " j/k scroll  ·  Esc close ",
                        Style::default().fg(GRAY),
                    ))),
            )
            .scroll((app.comment_detail_scroll as u16, 0))
            .wrap(Wrap { trim: false });
        f.render_widget(p, popup);
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;
    use crate::api::{make_item, User};
    use crate::app::{App, ComposeState, Feed, LoginState, Mode, Pane, ViewMode};
    use crate::session::Session;
    use ratatui::{backend::TestBackend, Terminal};

    fn test_app(n: usize) -> App {
        let client = reqwest::Client::new();
        let mut app = App::new(client);
        app.stories = (1..=n as u64).map(make_item).collect();
        app
    }

    fn render(app: &App) -> String {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        terminal.backend().buffer().content.iter().map(|c| c.symbol()).collect()
    }

    #[test]
    fn story_list_renders_titles() {
        let app = test_app(3);
        let screen = render(&app);
        assert!(screen.contains("Story 1"), "Story 1 missing");
        assert!(screen.contains("Story 2"), "Story 2 missing");
        assert!(screen.contains("Story 3"), "Story 3 missing");
    }

    #[test]
    fn story_list_shows_score_and_author() {
        let app = test_app(2);
        let screen = render(&app);
        assert!(screen.contains("100"), "score missing");
        assert!(screen.contains("user1"), "author missing");
    }

    #[test]
    fn header_shows_active_feed_top() {
        let app = test_app(1);
        let screen = render(&app);
        assert!(screen.contains("1:Top"), "active Top feed label missing");
    }

    #[test]
    fn header_shows_active_feed_new() {
        let mut app = test_app(1);
        app.feed = Feed::New;
        let screen = render(&app);
        assert!(screen.contains("2:New"), "active New feed label missing");
    }

    #[test]
    fn header_shows_hnr_branding() {
        let app = test_app(0);
        let screen = render(&app);
        assert!(screen.contains("hnr"), "hnr brand missing");
    }

    #[test]
    fn hints_stories_pane_shows_nav_and_copy() {
        let app = test_app(1);
        let screen = render(&app);
        assert!(screen.contains("j/k"), "nav hint missing");
        assert!(screen.contains("copy url"), "copy url hint missing");
    }

    #[test]
    fn hints_comments_pane_shows_collapse_and_copy() {
        let mut app = test_app(1);
        app.active_pane = Pane::Comments;
        let screen = render(&app);
        assert!(screen.contains("collapse"), "collapse hint missing");
        assert!(screen.contains("copy url"), "copy url hint missing");
    }

    #[test]
    fn empty_story_list_shows_placeholder() {
        let client = reqwest::Client::new();
        let app = App::new(client);
        let screen = render(&app);
        assert!(screen.contains("Select a story"), "placeholder missing");
    }

    #[test]
    fn story_header_shows_score_and_author() {
        let app = test_app(1);
        let screen = render(&app);
        assert!(screen.contains("user1"), "author in story header missing");
        assert!(screen.contains("100"), "score in story header missing");
    }

    #[test]
    fn story_header_shows_self_post_when_no_url() {
        let mut app = test_app(1);
        app.stories[0].url = None;
        let screen = render(&app);
        assert!(screen.contains("(self post)"), "self post indicator missing");
    }

    // ── Hints bar ─────────────────────────────────────────────────────────

    #[test]
    fn hints_bar_shows_login_when_no_session() {
        let app = test_app(1);
        let screen = render(&app);
        assert!(screen.contains("login"), "login hint missing when not logged in");
    }

    #[test]
    fn hints_bar_shows_logout_when_logged_in() {
        let mut app = test_app(1);
        app.session = Some(Session { username: "pg".into(), cookie: "user=abc".into() });
        let screen = render(&app);
        assert!(screen.contains("logout"), "logout hint missing when logged in");
    }

    // ── Status bar ────────────────────────────────────────────────────────

    #[test]
    fn status_bar_shows_status_message() {
        let mut app = test_app(0);
        app.status_message = "42 stories loaded".into();
        let screen = render(&app);
        assert!(screen.contains("42 stories loaded"), "status message missing");
    }

    #[test]
    fn status_bar_shows_command_input() {
        let mut app = test_app(0);
        app.mode = Mode::Command;
        app.command_input = "top".into();
        let screen = render(&app);
        assert!(screen.contains(":top"), "command input missing");
    }

    // ── Login overlay ─────────────────────────────────────────────────────

    #[test]
    fn login_overlay_renders_fields() {
        let mut app = test_app(0);
        app.mode = Mode::Login;
        let screen = render(&app);
        assert!(screen.contains("Login to Hacker News"), "login dialog title missing");
        assert!(screen.contains("Username"), "username label missing");
        assert!(screen.contains("Password"), "password label missing");
    }

    #[test]
    fn login_overlay_shows_typed_username() {
        let mut app = test_app(0);
        app.mode = Mode::Login;
        app.login_state = LoginState { username: "alice".into(), ..LoginState::new() };
        let screen = render(&app);
        assert!(screen.contains("alice"), "username not displayed");
    }

    #[test]
    fn login_overlay_shows_error() {
        let mut app = test_app(0);
        app.mode = Mode::Login;
        app.login_state.error = "Bad credentials".into();
        let screen = render(&app);
        assert!(screen.contains("Bad credentials"), "login error not shown");
    }

    // ── Compose overlay ───────────────────────────────────────────────────

    #[test]
    fn compose_overlay_renders_reply_and_text() {
        let mut app = test_app(1);
        app.mode = Mode::Compose;
        app.compose_state = Some(ComposeState {
            text: "Hello HN!".into(),
            parent_id: 1,
            story_id: 1,
            hmac: "abc".into(),
            parent_by: "pg".into(),
        });
        let screen = render(&app);
        assert!(screen.contains("Reply to pg"), "compose title missing");
        assert!(screen.contains("Hello HN!"), "compose text missing");
        assert!(screen.contains("Ctrl+S"), "submit hint missing");
    }

    // ── User profile ──────────────────────────────────────────────────────

    #[test]
    fn user_profile_renders_karma_and_about() {
        let mut app = test_app(1);
        app.view_mode = ViewMode::User;
        app.user_profile = Some(User {
            id: "pg".into(),
            karma: 155000,
            created: 0,
            about: Some("<p>Lisp hacker</p>".into()),
            submitted: Some(vec![1, 2, 3]),
        });
        let screen = render(&app);
        assert!(screen.contains("pg"), "username missing");
        assert!(screen.contains("155000"), "karma missing");
        assert!(screen.contains("Lisp hacker"), "about text missing");
    }

    // ── Comments ──────────────────────────────────────────────────────────

    #[test]
    fn comments_render_author_and_text() {
        use crate::app::CommentNode;
        let mut app = test_app(1);
        app.active_pane = Pane::Comments;
        let mut item = make_item(100);
        item.text = Some("<p>Great post!</p>".into());
        item.by = Some("alice".into());
        app.comments = vec![CommentNode::new(item, 0)];
        let screen = render(&app);
        assert!(screen.contains("alice"), "commenter name missing");
        assert!(screen.contains("Great post!"), "comment text missing");
    }

    #[test]
    fn collapsed_comment_shows_expand_marker() {
        use crate::app::CommentNode;
        let mut app = test_app(1);
        app.active_pane = Pane::Comments;
        let child = CommentNode::new(make_item(101), 1);
        let mut root = CommentNode::new(make_item(100), 0);
        root.children = vec![child];
        root.collapsed = true;
        app.comments = vec![root];
        let screen = render(&app);
        assert!(screen.contains("[+]"), "expand marker missing for collapsed comment");
    }

    // ── Bookmarks ─────────────────────────────────────────────────────────

    #[test]
    fn header_shows_bookmarks_tab() {
        let app = test_app(0);
        let screen = render(&app);
        assert!(screen.contains("Bookmarks"), "bookmarks tab missing from header");
    }

    #[test]
    fn story_list_shows_bookmark_star_for_bookmarked_item() {
        use std::collections::HashSet;
        let mut app = test_app(1);
        app.bookmark_ids = HashSet::from([1u64]);
        let screen = render(&app);
        assert!(screen.contains('★'), "bookmark star missing");
    }

    #[test]
    fn story_list_no_star_when_not_bookmarked() {
        let app = test_app(1);
        let screen = render(&app);
        assert!(!screen.contains('★'), "unexpected star for unbookmarked story");
    }

    #[test]
    fn bookmarks_feed_title_in_story_list() {
        let mut app = test_app(1);
        app.feed = Feed::Bookmarks;
        let screen = render(&app);
        assert!(screen.contains("Bookmarks"), "Bookmarks not in story list title");
    }

    // ── Comment detail overlay ────────────────────────────────────────────

    #[test]
    fn comment_detail_overlay_renders_author_and_text() {
        let mut app = test_app(1);
        app.mode = Mode::CommentDetail;
        app.comment_detail = Some(("alice".into(), "Rust is amazing for systems programming.".into()));
        let screen = render(&app);
        assert!(screen.contains("alice"), "author missing from overlay");
        assert!(screen.contains("Rust is amazing"), "text missing from overlay");
        assert!(screen.contains("Esc"), "close hint missing");
    }

    #[test]
    fn comment_detail_hints_show_scroll_and_close() {
        let mut app = test_app(0);
        app.mode = Mode::CommentDetail;
        app.comment_detail = Some(("bob".into(), "some text".into()));
        let screen = render(&app);
        assert!(screen.contains("j/k"), "scroll hint missing");
        assert!(screen.contains("close"), "close label missing");
    }

    // ── Search ────────────────────────────────────────────────────────────

    #[test]
    fn search_mode_status_bar_shows_query_input() {
        let mut app = test_app(0);
        app.mode = Mode::Search;
        app.search_input = "rust async".into();
        let screen = render(&app);
        assert!(screen.contains("?rust async"), "search input not shown in status bar");
    }

    #[test]
    fn search_mode_hints_show_type_and_enter() {
        let mut app = test_app(0);
        app.mode = Mode::Search;
        let screen = render(&app);
        assert!(screen.contains("Enter"), "Enter hint missing in search mode");
        assert!(screen.contains("cancel"), "cancel hint missing in search mode");
    }

    #[test]
    fn story_list_title_shows_search_query_when_active() {
        let mut app = test_app(2);
        app.search_query = Some("rust".into());
        let screen = render(&app);
        assert!(screen.contains("Search: rust"), "search query not shown in story list title");
    }

    #[test]
    fn story_list_title_shows_feed_name_when_no_search() {
        let app = test_app(1);
        let screen = render(&app);
        assert!(screen.contains("Top Stories"), "feed name not shown without search");
        assert!(!screen.contains("Search:"), "unexpected search prefix without search query");
    }

    #[test]
    fn expanded_comment_shows_collapse_marker() {
        use crate::app::CommentNode;
        let mut app = test_app(1);
        app.active_pane = Pane::Comments;
        let child = CommentNode::new(make_item(101), 1);
        let mut root = CommentNode::new(make_item(100), 0);
        root.children = vec![child];
        app.comments = vec![root];
        let screen = render(&app);
        assert!(screen.contains("[-]"), "collapse marker missing for expanded comment");
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}
