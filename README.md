# hnr

A fast terminal UI for Hacker News built with Rust and [ratatui](https://ratatui.rs).

![hnr screenshot](screenshot.png)

## Install

```bash
cargo install hnr
```

## Usage

```bash
hnr
```

## Keybindings

| Key | Action |
|-----|--------|
| `1` – `6` | Switch feed: Top / New / Best / Ask / Show / Bookmarks |
| `j` / `k` / `↑` / `↓` | Navigate up/down |
| `Enter` | Open comments (story pane) · Full text overlay (comment pane) |
| `Tab` | Switch between story list and comments pane |
| `Esc` | Back to story pane (from comments) · Close overlay |
| `Space` | Collapse / expand comment thread |
| `b` | Bookmark / unbookmark selected story (★) |
| `?` | Search stories via Algolia |
| `v` | Vote on selected story or comment |
| `c` | Reply to selected story or comment |
| `u` | View author profile (karma, about) |
| `o` | Open story URL in browser |
| `O` | Open HN discussion page in browser |
| `y` | Copy story URL to clipboard |
| `l` | Login / logout |
| `r` | Refresh current feed |
| `h` | Show help / shortcut reference |
| `/` | Command mode |
| `q` | Quit |

## Slash Commands

```
/login      /logout     /top        /new        /best
/ask        /show       /bookmarks  /search     /refresh
/user <n>   /bookmark   /open       /hn         /vote
/help       /quit
```

## Features

- **Feeds** — Top, New, Best, Ask HN, Show HN
- **Threaded comments** — recursive tree with collapse/expand (`Space`) and full-text overlay (`Enter`)
- **Bookmarks** — press `b` to save/remove; browse saved stories with `6` (persisted to `~/.hnr/bookmarks.json`)
- **Search** — press `?`, type a query, press `Enter` to search via Algolia; results appear in the story list with normal navigation; switch to any numbered feed to clear search
- **Voting & replies** — upvote or reply to any story or comment when logged in
- **User profiles** — press `u` to view karma, bio, and submission count for any author
- **Clipboard** — press `y` to copy the story URL
- **Session** — login cookie saved to `~/.hnr/session` and restored on next launch

## License

MIT
