use crate::app::{App, Mode, Pane, ViewMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
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
            Constraint::Length(1), // status / command bar
        ])
        .split(area);

    draw_header(f, app, root[0]);
    draw_body(f, app, root[1]);
    draw_statusbar(f, app, root[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let feeds = ["Top", "New", "Best", "Ask HN", "Show HN"];
    let current = app.feed.label();
    let mut spans = vec![
        Span::styled(" hnr ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
    ];
    for feed in &feeds {
        if *feed == current {
            spans.push(Span::styled(
                *feed,
                Style::default().fg(Color::Black).bg(ORANGE).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(*feed, Style::default().fg(GRAY)));
        }
        spans.push(Span::raw("  "));
    }
    let line = Line::from(spans);
    let p = Paragraph::new(line).style(Style::default().bg(DARK));
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
    let border_style = if focused {
        Style::default().fg(ORANGE)
    } else {
        Style::default().fg(GRAY)
    };

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
            let title = story.display_title();
            let meta = format!(
                " ▲{} {} | {} comments",
                story.score(),
                story.display_by(),
                story.comment_count()
            );

            let title_style = if selected {
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let meta_style = Style::default().fg(GRAY);
            let rank_style = if selected {
                Style::default().fg(ORANGE)
            } else {
                Style::default().fg(GRAY)
            };

            let bg = if selected { SELECTED_BG } else { Color::Reset };

            let line1 = Line::from(vec![
                Span::styled(rank, rank_style),
                Span::styled(title, title_style),
            ]);
            let line2 = Line::from(vec![
                Span::raw("     "),
                Span::styled(meta, meta_style),
            ]);

            let content = ratatui::text::Text::from(vec![line1, line2]);
            ListItem::new(content).style(Style::default().bg(bg))
        })
        .collect();

    let mut state = ListState::default();
    if focused {
        state.select(Some(app.story_cursor.saturating_sub(app.story_scroll)));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" {} Stories ", app.feed.label()))
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
        let border_style = Style::default().fg(GRAY);
        let p = Paragraph::new("Select a story to read comments.")
            .block(Block::default().borders(Borders::ALL).border_style(border_style))
            .style(Style::default().fg(GRAY));
        f.render_widget(p, area);
    }
}

fn draw_user_profile(f: &mut Frame, user: &crate::api::User, area: Rect) {
    let about = user.about_plain();

    let mut lines = vec![
        Line::from(vec![
            Span::styled(" ", Style::default()),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(&user.id, Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("▲ {} karma", user.karma),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("Joined {} ago  ·  {} submissions", user.joined_ago(), user.submission_count()),
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
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(l.to_string(), Style::default().fg(Color::White)),
            ]));
        }
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "  Esc · back   o · open HN profile",
        Style::default().fg(GRAY),
    )));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(" User: {} ", user.id))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ORANGE)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_story_header(f: &mut Frame, story: &crate::api::Item, area: Rect) {
    let title = story.display_title();
    let url = story.url.as_deref().unwrap_or("(self post)");
    let meta = format!(
        "▲ {}  by {}  | {} comments  | o: open url  O: open HN",
        story.score(),
        story.display_by(),
        story.comment_count()
    );

    let text_body = story.text_plain();
    let mut lines = vec![
        Line::from(Span::styled(title, Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(url, Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED))),
        Line::from(Span::styled(meta, Style::default().fg(GRAY))),
    ];
    if !text_body.is_empty() {
        for l in text_body.lines().take(2) {
            lines.push(Line::from(Span::styled(l.to_string(), Style::default().fg(Color::White))));
        }
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Story ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ORANGE)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_comments(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(ORANGE)
    } else {
        Style::default().fg(GRAY)
    };

    let flat = app.flat_comments();
    let visible = (area.height as usize).saturating_sub(2);

    let items: Vec<ListItem> = flat
        .iter()
        .enumerate()
        .skip(app.comment_scroll)
        .take(visible)
        .map(|(i, (node, depth))| {
            let selected = i + app.comment_scroll == app.comment_cursor;
            let indent = "  ".repeat(*depth);
            let by = node.item.display_by();
            let collapsed_marker = if node.collapsed && !node.children.is_empty() {
                " [+]"
            } else if !node.collapsed && !node.children.is_empty() {
                " [-]"
            } else {
                ""
            };

            let depth_colors = [
                Color::Cyan,
                Color::Green,
                Color::Yellow,
                Color::Magenta,
                Color::Red,
                Color::Blue,
                Color::White,
            ];
            let dc = depth_colors[depth % depth_colors.len()];

            let header = format!("{indent}▸ {by}{collapsed_marker}");
            let header_style = if selected {
                Style::default().fg(dc).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(dc)
            };

            let text = node.item.text_plain();
            let preview = text
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(area.width as usize - depth * 2 - 4)
                .collect::<String>();

            let bg = if selected { SELECTED_BG } else { Color::Reset };

            let line1 = Line::from(Span::styled(header, header_style));
            let line2 = Line::from(vec![
                Span::raw(format!("{indent}  ")),
                Span::styled(preview, Style::default().fg(Color::White)),
            ]);

            ListItem::new(ratatui::text::Text::from(vec![line1, line2]))
                .style(Style::default().bg(bg))
        })
        .collect();

    let title = if focused { " Comments (Space: expand/collapse) " } else { " Comments " };
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    f.render_widget(list, area);
}

fn draw_statusbar(f: &mut Frame, app: &App, area: Rect) {
    match app.mode {
        Mode::Command => {
            let input = format!(":{}", app.command_input);
            let p = Paragraph::new(input)
                .style(Style::default().fg(Color::White).bg(DARK));
            f.render_widget(p, area);
        }
        Mode::Normal => {
            let msg = if app.loading {
                Span::styled(&app.status_message, Style::default().fg(ORANGE))
            } else {
                Span::styled(&app.status_message, Style::default().fg(GRAY))
            };
            let p = Paragraph::new(Line::from(vec![msg]))
                .style(Style::default().bg(DARK));
            f.render_widget(p, area);
        }
    }
}
