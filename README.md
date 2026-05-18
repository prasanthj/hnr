# h.n.r

[![Crates.io](https://img.shields.io/crates/v/hnr.svg)](https://crates.io/crates/hnr)

### A terminal UI for Hacker News — pronounced **HoNoR**

Browse stories, read threaded comments, search, vote, reply, and bookmark — all without leaving the terminal. Built with Rust and [ratatui](https://ratatui.rs). Mostly vibe coded with Claude Code over a weekend.

## Features

- **Six feeds** — Top, New, Best, Ask HN, Show HN, Jobs
- **Threaded comments** — recursive tree with inline expand/collapse per comment
- **Comment prefetch** — comments are fetched automatically after a short dwell on any story and the pane switches without pressing Enter; works for Algolia search results too
- **Reader mode** — press `r` to fetch and render any article as structured text with headings, code blocks, quotes and lists (powered by Mozilla Readability); loads in the background with a progress indicator
- **Shimmer progress** — all async operations (feed refresh, article fetch, comment prefetch, search) show an animated shimmer indicator in the status bar
- **Search** — Algolia-powered full-text search; press `?` to open, results drop into the story list
- **Seen tracking** — stories dim automatically when you open their comments; press `u` to mark unread (persisted to `~/.hnr/seen.json`)
- **Bookmarks** — save stories with `b`, browse them on feed `7`, persisted to `~/.hnr/bookmarks.json`
- **Voting & replies** — upvote or reply to any story or comment when logged in
- **User profiles** — view karma, bio, and submission count for any author
- **Open in browser** — jump to the story URL or HN discussion page with `o` / `O`
- **Clipboard** — copy the story URL with `y`
- **Session** — login cookie saved to `~/.hnr/session` and restored on next launch

## Demo

![hnr showcase](showcase.gif)

## Install

```bash
# Homebrew
brew tap prasanthj/hnr
brew install hnr

# Cargo
cargo install hnr
```

## Usage

```bash
hnr
```

## Keybindings

| Key | Action |
|-----|--------|
| `1` – `7` | Switch feed: Top / New / Best / Ask / Show / Jobs / Bookmarks |
| `j` / `k` / `↑` / `↓` | Navigate up / down |
| `Enter` | Fetch comments (story pane) · Expand / collapse comment inline |
| `Tab` | Switch between story list and comments pane |
| `Esc` | Back to story pane · Close modal |
| `Space` | Collapse / expand comment thread |
| `r` | Reader mode — fetch and render article as structured text |
| `R` | Refresh current feed |
| `b` | Bookmark / unbookmark selected story |
| `u` | Mark selected story as unread |
| `?` | Search stories via Algolia |
| `v` | Vote on selected story or comment |
| `c` | Reply to selected story or comment |
| `p` | View author profile |
| `o` | Open story URL in browser |
| `O` | Open HN discussion page in browser |
| `y` | Copy story URL to clipboard |
| `l` | Login / logout |
| `h` | Show help |
| `/` | Command mode |
| `q` | Quit |

### Reader mode keys

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll line by line |
| `d` / `u` | Half-page down / up |
| `n` / `N` | Jump to next / previous section |
| `g` / `G` | Jump to top / bottom |
| `o` | Open article in browser |
| `Esc` / `q` / `r` | Close reader |

## Slash Commands

```
/login      /logout     /top        /new        /best
/ask        /show       /jobs       /bookmarks  /search
/refresh    /user <n>   /bookmark   /open       /hn
/vote       /help       /quit
```

## License

MIT
