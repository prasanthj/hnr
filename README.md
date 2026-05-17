# hnr

A fast terminal UI for Hacker News built with Rust and [ratatui](https://ratatui.rs).

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
| `1-5` | Switch feed: Top / New / Best / Ask / Show |
| `j` / `k` | Navigate up/down |
| `Enter` | Open comments for selected story |
| `Tab` | Switch between story list and comments pane |
| `Space` | Collapse / expand comment thread |
| `v` | Vote on selected story or comment |
| `c` | Reply to selected story or comment |
| `u` | View author profile (karma, about) |
| `o` | Open story URL in browser |
| `O` | Open HN discussion page in browser |
| `l` | Login / logout |
| `r` | Refresh current feed |
| `h` | Show help / shortcut reference |
| `/` | Command mode |
| `q` | Quit |

## Slash Commands

```
/login      /logout     /top        /new        /best
/ask        /show       /refresh    /user <n>   /open
/hn         /vote       /help       /quit
```

## Login

Login is handled directly in the TUI (`l` or `/login`). Your session cookie is saved to `~/.hnr/session` and restored on next launch.

## License

MIT
