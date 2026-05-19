use crate::app::{App, Feed, LoginField, Mode, Pane, ViewMode};
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState,
        Padding, Paragraph, Wrap,
    },
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    f.render_widget(
        Block::default().style(Style::default().bg(Theme::BG)),
        f.area(),
    );

    let area = f.area();
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header(f, app, root[0]);
    draw_separator(f, root[1], false);
    draw_body(f, app, root[2]);
    draw_separator(f, root[3], false);
    draw_hints(f, app, root[4]);
    draw_separator(f, root[5], true);
    draw_statusbar(f, app, root[6]);

    match app.mode {
        Mode::Login => draw_login_overlay(f, app, area),
        Mode::Compose => draw_compose_overlay(f, app, area),
        Mode::Reader => {
            if let Some(content) = &app.reader_content {
                draw_reader_overlay(f, content, app.reader_scroll, area);
            }
        }
        _ => {}
    }
}

fn overlay_block(title: &str) -> Block<'static> {
    Block::default()
        .title(Span::styled(format!(" {} ", title), Theme::secondary()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::accent())
        .padding(Padding::horizontal(1))
}

fn draw_separator(f: &mut Frame, area: Rect, dotted: bool) {
    let line = if dotted {
        "· ".repeat(area.width as usize / 2 + 1)
            .chars()
            .take(area.width as usize)
            .collect::<String>()
    } else {
        "─".repeat(area.width as usize)
    };
    f.render_widget(
        Paragraph::new(line).style(Style::default().fg(Theme::DIM).bg(Theme::BG)),
        area,
    );
}

fn brand_spans() -> Vec<Span<'static>> {
    let cap = Theme::accent().add_modifier(Modifier::BOLD);
    let dot = Theme::secondary().add_modifier(Modifier::BOLD);
    vec![
        Span::raw(" "),
        Span::styled("H", cap),
        Span::styled("·", dot),
        Span::styled("N", cap),
        Span::styled("·", dot),
        Span::styled("R", cap),
        Span::raw("   "),
    ]
}

fn mode_span(num: u8, label: &str, active: bool) -> Vec<Span<'static>> {
    let label_upper = label.to_uppercase();
    if active {
        vec![
            Span::styled(
                format!(" {} ", label_upper),
                Style::default().fg(Theme::BG).bg(Theme::ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
        ]
    } else {
        vec![
            Span::styled(num.to_string(), Theme::dim()),
            Span::raw(" "),
            Span::styled(label_upper, Theme::secondary().add_modifier(Modifier::BOLD)),
            Span::raw("   "),
        ]
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let feeds: &[(u8, &str)] = &[
        (1, "Top"),
        (2, "New"),
        (3, "Best"),
        (4, "Ask"),
        (5, "Show"),
        (6, "Jobs"),
        (7, "Bookmarks"),
    ];
    let current = app.feed.label();

    let mut spans: Vec<Span<'static>> = brand_spans();

    for (num, label) in feeds {
        let active = current.starts_with(label);
        spans.extend(mode_span(*num, label, active));
    }

    if let Some(session) = &app.session {
        spans.push(Span::styled(
            format!(" {} ", session.username),
            Style::default().fg(Theme::BG).bg(Theme::ACCENT_SOFT),
        ));
    }

    let update = app.update_available.lock().ok().and_then(|g| g.clone());
    let version_str = match update {
        Some(ref v) => format!(" v{} ↑{} ", env!("CARGO_PKG_VERSION"), v),
        None => format!(" v{} ", env!("CARGO_PKG_VERSION")),
    };
    let version_style = if update.is_some() { Theme::accent() } else { Theme::dim() };

    // One blank row above, content on second row.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    let mid = rows[1];

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(version_str.len() as u16),
        ])
        .split(mid);

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Theme::BG)),
        cols[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(version_str, version_style)))
            .style(Style::default().bg(Theme::BG)),
        cols[1],
    );
}

fn hint_spans(pairs: &[(&str, &str)]) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = vec![Span::raw(" ")];
    for (i, (key, desc)) in pairs.iter().enumerate() {
        spans.push(Span::styled(
            key.to_string(),
            Theme::accent().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(desc.to_string(), Theme::secondary()));
        if i < pairs.len() - 1 {
            spans.push(Span::raw("  "));
        }
    }
    spans
}

fn draw_hints(f: &mut Frame, app: &App, area: Rect) {
    let spans = match app.mode {
        Mode::Login => hint_spans(&[("Tab", "field"), ("Enter", "submit"), ("Esc", "cancel")]),
        Mode::Compose => hint_spans(&[("Ctrl+S", "post"), ("Esc", "cancel")]),
        Mode::Command => hint_spans(&[("Enter", "run"), ("Esc", "cancel")]),
        Mode::Search => {
            hint_spans(&[("Type", "to search"), ("Enter", "run"), ("Esc", "cancel")])
        }
        Mode::Reader => hint_spans(&[
            ("j/k/↑↓", "scroll"),
            ("d/u", "page"),
            ("n/N", "section"),
            ("o", "browser"),
            ("Esc", "close"),
        ]),
        Mode::Normal => {
            let login_hint = if app.session.is_some() { "logout" } else { "login" };
            let global: &[(&str, &str)] = &[
                ("h", "help"),
                ("R", "refresh"),
                ("l", login_hint),
                ("/", "cmd"),
                ("q", "quit"),
            ];
            let pane: &[(&str, &str)] = match app.active_pane {
                Pane::Stories => &[
                    ("j/k/↑↓", "nav"),
                    ("Enter", "comments"),
                    ("Tab", "→pane"),
                    ("r", "read"),
                    ("b", "bookmark"),
                    ("u", "unread"),
                    ("o", "open"),
                    ("y", "copy url"),
                    ("v", "vote"),
                    ("p", "profile"),
                ],
                Pane::Comments => &[
                    ("j/k/↑↓", "nav"),
                    ("Enter", "expand/collapse"),
                    ("Space", "thread collapse"),
                    ("Esc", "←back"),
                    ("u", "unread"),
                    ("o", "open"),
                    ("O", "hn"),
                    ("y", "copy url"),
                    ("v", "vote"),
                    ("c", "reply"),
                    ("p", "profile"),
                ],
            };
            let mut s = hint_spans(global);
            s.push(Span::styled("  ·  ", Theme::dim()));
            s.extend(hint_spans(pane));
            s
        }
    };

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Theme::BG)),
        area,
    );
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

    // Right border acts as the pane separator.
    let outer = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Theme::dim());
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    // Section label.
    let label = if let Some(q) = &app.search_query {
        format!("Search: {}", q)
    } else {
        match app.feed {
            Feed::Bookmarks => "bookmarks".to_string(),
            Feed::Jobs => "jobs".to_string(),
            _ => format!("{} stories", app.feed.label().to_lowercase()),
        }
    };
    f.render_widget(
        Paragraph::new(Span::styled(label, Theme::dim()))
            .style(Style::default().bg(Theme::BG)),
        split[0],
    );

    let list_area = split[2];
    let visible = list_area.height as usize;

    let items: Vec<ListItem> = app
        .stories
        .iter()
        .enumerate()
        .skip(app.story_scroll)
        .take(visible)
        .map(|(i, story)| {
            let selected = i == app.story_cursor;
            let seen = app.seen_ids.contains(&story.id);
            let bookmarked = app.bookmark_ids.contains(&story.id);

            // The bar spans every line of the item for a continuous left rail.
            let bar: Span<'static> = if selected {
                Span::styled("▎ ", Theme::accent())
            } else {
                Span::raw("  ")
            };

            let title_style = if selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else if seen {
                Theme::secondary()
            } else {
                Theme::title()
            };

            let score_style = match story.score() {
                0..=99 => Theme::secondary(),
                100..=299 => Theme::accent_soft(),
                _ => Theme::accent(),
            };

            let title_line = Line::from(vec![
                bar.clone(),
                Span::styled(format!("{:>2}. ", i + 1), Theme::secondary()),
                Span::styled(
                    if bookmarked { "★ " } else { "" }.to_string(),
                    Theme::accent_soft(),
                ),
                Span::styled(story.display_title().to_string(), title_style),
            ]);

            let ago = story.time_ago();
            let mut meta_spans: Vec<Span<'static>> = vec![
                bar.clone(),
                Span::raw("    "),
                Span::styled("▲ ", Theme::dim()),
                Span::styled(story.score().to_string(), score_style),
                Span::styled(" · ", Theme::dim()),
                Span::styled(story.display_by().to_string(), Theme::secondary()),
                Span::styled(" · ", Theme::dim()),
                Span::styled(
                    format!("{} comments", story.comment_count()),
                    Theme::secondary(),
                ),
            ];
            if !ago.is_empty() {
                meta_spans.push(Span::styled(" · ", Theme::dim()));
                meta_spans.push(Span::styled(ago, Theme::secondary()));
            }

            ListItem::new(vec![
                title_line,
                Line::from(vec![bar.clone()]),
                Line::from(meta_spans),
                Line::from(vec![bar]),
            ])
        })
        .collect();

    let mut state = ListState::default();
    if focused {
        state.select(Some(app.story_cursor.saturating_sub(app.story_scroll)));
    }

    let list = List::new(items)
        .highlight_style(Style::default().bg(Theme::SELECT_BG));

    f.render_stateful_widget(list, list_area, &mut state);
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
            .constraints([
                Constraint::Length(1), // "story" label
                Constraint::Length(1), // blank
                Constraint::Length(5), // story panel
                Constraint::Length(1), // blank
                Constraint::Length(1), // "comments [hints]" label
                Constraint::Length(1), // blank
                Constraint::Min(0),    // comments
            ])
            .split(area);

        f.render_widget(
            Paragraph::new(Span::styled("story", Theme::dim()))
                .style(Style::default().bg(Theme::BG)),
            chunks[0],
        );
        draw_story_header(f, story, chunks[2]);
        draw_comments_label(f, focused, chunks[4]);
        draw_comments(f, app, chunks[6], focused);
    } else {
        f.render_widget(
            Paragraph::new(Span::styled(
                "Select a story to read comments.",
                Theme::secondary(),
            ))
            .style(Style::default().bg(Theme::BG)),
            area,
        );
    }
}

fn draw_comments_label(f: &mut Frame, focused: bool, area: Rect) {
    let mut spans: Vec<Span<'static>> = vec![Span::styled("comments", Theme::dim())];
    if focused {
        spans.push(Span::raw("  "));
        spans.push(Span::styled("space", Theme::accent_soft()));
        spans.push(Span::styled(" collapse · ", Theme::secondary()));
        spans.push(Span::styled("v", Theme::accent_soft()));
        spans.push(Span::styled(" vote · ", Theme::secondary()));
        spans.push(Span::styled("c", Theme::accent_soft()));
        spans.push(Span::styled(" reply", Theme::secondary()));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(Theme::BG)),
        area,
    );
}

fn draw_story_header(f: &mut Frame, story: &crate::api::Item, area: Rect) {
    let url = story.url.as_deref().unwrap_or("(self post)");
    let ago = story.time_ago();

    let lines = vec![
        Line::from(Span::styled(
            story.display_title().to_string(),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(url.to_string(), Theme::accent_soft())),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("▲ {}", story.score()), Theme::secondary()),
            Span::styled("  by ", Theme::dim()),
            Span::styled(story.display_by().to_string(), Theme::accent()),
            Span::styled("  ·  ", Theme::dim()),
            Span::styled(ago, Theme::secondary()),
            Span::styled("  ·  ", Theme::dim()),
            Span::styled(
                format!("{} comments", story.comment_count()),
                Theme::secondary(),
            ),
        ]),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::LEFT)
                .border_style(Theme::accent())
                .padding(Padding::horizontal(1)),
        )
        .style(Style::default().bg(Color::Rgb(0x22, 0x20, 0x1c)))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_comments(f: &mut Frame, app: &App, area: Rect, _focused: bool) {
    let placeholder = |msg: &'static str| {
        Paragraph::new(Span::styled(msg, Theme::secondary()))
            .style(Style::default().bg(Theme::BG))
    };

    if app.comments_loading {
        f.render_widget(placeholder("Loading comments…"), area);
        return;
    }

    if app.comments_story_id.is_none() {
        f.render_widget(placeholder("Press Enter to load comments."), area);
        return;
    }

    if app.comments.is_empty() {
        f.render_widget(placeholder("No comments yet."), area);
        return;
    }

    let flat = app.flat_comments();
    let visible = (area.height as usize).saturating_sub(2);

    let story_author: Option<String> = app
        .selected_story()
        .and_then(|s| s.by.clone());

    let items: Vec<ListItem> = flat
        .iter()
        .enumerate()
        .skip(app.comment_scroll)
        .take(visible)
        .map(|(i, (node, depth))| {
            let selected = i + app.comment_scroll == app.comment_cursor;
            let bg = if selected { Theme::SELECT_BG } else { Theme::BG };

            let glyph = if node.collapsed { "▸" } else { "▾" };
            let is_op = story_author
                .as_deref()
                .map(|a| Some(a) == node.item.by.as_deref())
                .unwrap_or(false);
            let user_style = if is_op { Theme::accent() } else { Theme::accent_soft() };

            let guides = || {
                (0..*depth)
                    .map(|_| Span::styled("│ ", Theme::dim()))
                    .collect::<Vec<_>>()
            };

            let mut header = guides();
            header.push(Span::styled(glyph.to_string(), Theme::secondary()));
            header.push(Span::raw(" "));
            header.push(Span::styled(node.item.display_by().to_string(), user_style));

            let is_expanded = app.expanded_comment == Some(node.item.id);
            let text = &node.text;
            let mut item_lines = vec![Line::from(header)];

            if !node.collapsed {
                if is_expanded {
                    if text.trim().is_empty() {
                        let mut spans = guides();
                        spans.push(Span::raw("  "));
                        spans.push(Span::styled("[no text]", Theme::secondary()));
                        item_lines.push(Line::from(spans));
                    } else {
                        for line in text.lines() {
                            let mut spans = guides();
                            spans.push(Span::raw("  "));
                            spans.push(Span::styled(line.to_string(), Theme::primary()));
                            item_lines.push(Line::from(spans));
                        }
                    }
                } else {
                    let max_width = area.width as usize - depth * 2 - 4;
                    let preview: String = text
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(max_width)
                        .collect();
                    let mut spans = guides();
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(preview, Theme::primary()));
                    item_lines.push(Line::from(spans));
                }
            }

            item_lines.push(Line::from(guides()));

            ListItem::new(ratatui::text::Text::from(item_lines))
                .style(Style::default().bg(bg))
        })
        .collect();

    f.render_widget(List::new(items), area);
}

fn draw_user_profile(f: &mut Frame, user: &crate::api::User, area: Rect) {
    let about = user.about_plain();
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", user.id),
            Theme::accent().add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("▲ {} karma", user.karma),
                Theme::accent_soft().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("Joined {}  ·  {} submissions", user.joined_ago(), user.submission_count()),
                Theme::secondary(),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("https://news.ycombinator.com/user?id={}", user.id),
                Theme::accent_soft(),
            ),
        ]),
    ];
    if !about.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  about", Theme::secondary())));
        lines.push(Line::from(Span::styled(
            "  ─────────────────────────────────────",
            Theme::dim(),
        )));
        for l in about.lines().take(20) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(l.to_string(), Theme::primary()),
            ]));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Esc back  ·  o open in browser",
        Theme::secondary(),
    )));

    let p = Paragraph::new(lines)
        .block(overlay_block(&format!("user: {}", user.id)))
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_login_overlay(f: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(50, 40, area);
    f.render_widget(Clear, popup);

    let state = &app.login_state;
    let username_style = if state.field == LoginField::Username {
        Theme::accent().add_modifier(Modifier::BOLD)
    } else {
        Theme::primary()
    };
    let password_style = if state.field == LoginField::Password {
        Theme::accent().add_modifier(Modifier::BOLD)
    } else {
        Theme::primary()
    };

    let masked = "*".repeat(state.password.len());
    let cursor = "█";

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Username", Theme::secondary())),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!(
                    "{}{}",
                    state.username,
                    if state.field == LoginField::Username { cursor } else { "" }
                ),
                username_style,
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Password", Theme::secondary())),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!(
                    "{}{}",
                    masked,
                    if state.field == LoginField::Password { cursor } else { "" }
                ),
                password_style,
            ),
        ]),
        Line::from(""),
        if state.error.is_empty() {
            Line::from("")
        } else {
            Line::from(Span::styled(
                format!("  {}", state.error),
                Style::default().fg(Color::Red),
            ))
        },
        Line::from(""),
        Line::from(Span::styled(
            "  Tab: switch field  ·  Enter: login  ·  Esc: cancel",
            Theme::secondary(),
        )),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Login to Hacker News ", Theme::secondary()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Theme::accent()),
        )
        .style(Style::default().bg(Theme::BG));
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
        lines.push(Line::from(Span::styled(line.to_string(), Theme::primary())));
    }
    let cursor_line = if state.text.ends_with('\n') || state.text.is_empty() {
        Line::from(Span::styled("█", Theme::accent()))
    } else {
        let last = state.text.lines().last().unwrap_or("").to_string();
        lines.pop();
        Line::from(vec![
            Span::styled(last, Theme::primary()),
            Span::styled("█", Theme::accent()),
        ])
    };
    lines.push(cursor_line);

    let title = format!(" Reply to {} ", state.parent_by);
    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(title, Theme::secondary()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Theme::accent())
                .title_bottom(Line::from(Span::styled(
                    " Ctrl+S submit  ·  Esc cancel ",
                    Theme::secondary(),
                ))),
        )
        .style(Style::default().bg(Theme::BG))
        .wrap(Wrap { trim: false });
    f.render_widget(p, popup);
}

fn blocks_to_lines(blocks: &[crate::reader::Block]) -> Vec<Line<'_>> {
    use crate::reader::Block;
    let mut lines: Vec<Line<'_>> = Vec::new();

    for block in blocks {
        match block {
            Block::Heading(level, text) => {
                let prefix = if *level == 1 {
                    "━━ "
                } else if *level == 2 {
                    "── "
                } else {
                    "·  "
                };
                let style = if *level <= 2 {
                    Theme::accent().add_modifier(Modifier::BOLD)
                } else {
                    Theme::accent()
                };
                lines.push(Line::from(Span::styled(format!("{prefix}{text}"), style)));
                lines.push(Line::raw(""));
            }
            Block::Paragraph(text) => {
                lines.push(Line::from(Span::styled(text.as_str(), Theme::primary())));
                lines.push(Line::raw(""));
            }
            Block::Code(text) => {
                for l in text.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {l}"),
                        Theme::secondary(),
                    )));
                }
                lines.push(Line::raw(""));
            }
            Block::Quote(text) => {
                for l in text.lines() {
                    lines.push(Line::from(vec![
                        Span::styled("│ ", Theme::accent()),
                        Span::styled(l, Theme::secondary()),
                    ]));
                }
                lines.push(Line::raw(""));
            }
            Block::ListItem(text) => {
                lines.push(Line::from(vec![
                    Span::styled("• ", Theme::accent()),
                    Span::styled(text.as_str(), Theme::primary()),
                ]));
            }
        }
    }

    lines
}

fn draw_reader_overlay(
    f: &mut Frame,
    content: &crate::app::ReaderContent,
    scroll: usize,
    area: Rect,
) {
    let popup = centered_rect(90, 90, area);
    f.render_widget(Clear, popup);

    let title = format!(" {} ", content.title);
    let block = Block::default()
        .title(Span::styled(title, Theme::secondary()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::accent())
        .title_bottom(Line::from(Span::styled(
            " g/G top/bot · j/k scroll · d/u page · n/N section · o browser · Esc close ",
            Theme::secondary(),
        )));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let lines = blocks_to_lines(&content.blocks);
    let p = Paragraph::new(lines)
        .style(Style::default().fg(Theme::PRIMARY).bg(Theme::BG))
        .wrap(Wrap { trim: true })
        .scroll((scroll as u16, 0));
    f.render_widget(p, inner);
}

fn draw_statusbar(f: &mut Frame, app: &App, area: Rect) {
    match app.mode {
        Mode::Command => {
            f.render_widget(
                Paragraph::new(format!(":{}", app.command_input))
                    .style(Theme::primary().bg(Theme::BG)),
                area,
            );
        }
        Mode::Search => {
            f.render_widget(
                Paragraph::new(format!("?{}", app.search_input))
                    .style(Theme::primary().bg(Theme::BG)),
                area,
            );
        }
        _ => {
            let msg_style = if app.loading {
                Theme::accent()
            } else {
                Theme::secondary()
            };
            if let Some(progress) = &app.progress {
                let spans = progress.spans();
                let progress_width = spans.len() as u16 + 2;
                if area.width > progress_width {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(progress_width),
                        ])
                        .split(area);
                    let mut progress_line: Vec<Span<'static>> = vec![Span::raw(" ")];
                    progress_line.extend(spans);
                    f.render_widget(
                        Paragraph::new(Line::from(Span::styled(
                            &app.status_message,
                            msg_style,
                        )))
                        .style(Style::default().bg(Theme::BG)),
                        chunks[0],
                    );
                    f.render_widget(
                        Paragraph::new(Line::from(progress_line))
                            .style(Style::default().bg(Theme::BG)),
                        chunks[1],
                    );
                    return;
                }
            }
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(&app.status_message, msg_style)))
                    .style(Style::default().bg(Theme::BG)),
                area,
            );
        }
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
        assert!(screen.contains("TOP"), "active Top feed label missing");
    }

    #[test]
    fn header_shows_active_feed_new() {
        let mut app = test_app(1);
        app.feed = Feed::New;
        let screen = render(&app);
        assert!(screen.contains("NEW"), "active New feed label missing");
    }

    #[test]
    fn header_shows_hnr_branding() {
        let app = test_app(0);
        let screen = render(&app);
        assert!(screen.contains("H·N·R"), "H·N·R brand missing");
    }

    #[test]
    fn hints_stories_pane_shows_nav_and_bookmark() {
        let app = test_app(1);
        let screen = render(&app);
        assert!(screen.contains("j/k"), "nav hint missing");
        assert!(screen.contains("bookmark"), "bookmark hint missing");
    }

    #[test]
    fn hints_comments_pane_shows_collapse_and_back() {
        let mut app = test_app(1);
        app.active_pane = Pane::Comments;
        let screen = render(&app);
        assert!(screen.contains("collapse"), "collapse hint missing");
        assert!(screen.contains("←back"), "back hint missing");
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

    #[test]
    fn comments_render_author_and_text() {
        use crate::app::CommentNode;
        let mut app = test_app(1);
        app.active_pane = Pane::Comments;
        let mut item = make_item(100);
        item.text = Some("<p>Great post!</p>".into());
        item.by = Some("alice".into());
        app.comments = vec![CommentNode::new(item, 0)];
        app.comments_story_id = Some(100);
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
        app.comments_story_id = Some(100);
        let screen = render(&app);
        assert!(screen.contains('▸'), "collapse glyph missing for collapsed comment");
    }

    #[test]
    fn header_shows_bookmarks_and_jobs_tabs() {
        let app = test_app(0);
        let screen = render(&app);
        assert!(screen.contains("BOOKMARKS"), "bookmarks tab missing from header");
        assert!(screen.contains("JOBS"), "jobs tab missing from header");
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
        assert!(screen.contains("bookmarks"), "bookmarks not in story list title");
    }

    #[test]
    fn expanded_comment_shows_full_text_inline() {
        use crate::app::CommentNode;
        let mut app = test_app(1);
        app.active_pane = Pane::Comments;
        let mut item = make_item(100);
        item.text = Some("<p>Rust is amazing for systems programming.</p>".into());
        app.comments = vec![CommentNode::new(item, 0)];
        app.comments_story_id = Some(100);
        app.expanded_comment = Some(100);
        let screen = render(&app);
        assert!(screen.contains("Rust is amazing"), "full text not shown when expanded");
    }

    #[test]
    fn collapsed_comment_shows_only_preview() {
        use crate::app::CommentNode;
        let mut app = test_app(1);
        app.active_pane = Pane::Comments;
        let mut item = make_item(100);
        item.text = Some("<p>First line.</p><p>Second line should not appear.</p>".into());
        app.comments = vec![CommentNode::new(item, 0)];
        app.comments_story_id = Some(100);
        app.expanded_comment = None;
        let screen = render(&app);
        assert!(screen.contains("First line"), "preview line missing");
    }

    #[test]
    fn header_shows_jobs_tab() {
        let app = test_app(0);
        let screen = render(&app);
        assert!(screen.contains("JOBS"), "Jobs tab missing from header");
    }

    #[test]
    fn seen_story_title_still_renders() {
        use std::collections::HashSet;
        let mut app = test_app(2);
        app.seen_ids = HashSet::from([1u64]);
        let screen = render(&app);
        assert!(screen.contains("Story 1"), "seen story title should still appear");
        assert!(screen.contains("Story 2"), "unseen story title should appear");
    }

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
        assert!(screen.contains("top stories"), "feed name not shown without search");
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
        app.comments_story_id = Some(100);
        let screen = render(&app);
        assert!(screen.contains('▾'), "expand glyph missing for expanded comment");
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
