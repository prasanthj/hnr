# hnr

A fast terminal UI for Hacker News built with Rust and [ratatui](https://ratatui.rs).

```
 hnr │ 1:Top  2:New  3:Best  4:Ask  5:Show  6:Bookmarks
┌─────────────────────────────┐┌──────────────────────────────────────┐
│ Top Stories                 ││ Story                                │
│  1. Rust 2025 Edition       ││ Rust 2025 Edition                    │
│     ▲482 nincrementum | 91c ││ blog.rust-lang.org                   │
│  2. Show HN: hnr — TUI for  ││ ▲482  by nincrementum  2h  | 91 cmts │
│     ▲347 pg | 64 comments   │└──────────────────────────────────────┘
│  3. Ask HN: Best keymaps?   │┌── Comments ────────────────────────── ┐
│     ▲201 tptacek | 38 cmts  ││ ▸ nincrementum [-]                   │
│                             ││   Great release overall...           │
│                             ││   ▸ patio11                         │
│                             ││     Agreed, async is finally...      │
└─────────────────────────────┘└──────────────────────────────────────┘
```

**Feeds · Threaded comments · Search · Bookmarks · Voting · Replies · Profiles**

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

## License

MIT
