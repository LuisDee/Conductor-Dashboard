# Conductor Dashboard

A terminal dashboard that monitors [Conductor](https://github.com/anthropics/claude-code) track progress in real-time. Built with Rust, Ratatui, and Crossterm.

## Features

- **Live file watching** — automatically updates when track files change on disk
- **Track list** with status badges, progress bars, and task counts
- **Detail panel** showing implementation plan phases and individual tasks
- **6 switchable themes** — Mako, Warm Dark, Midnight, Ember, Dusk, Light
- **Filtering** by status (All / Active / Blocked / Complete)
- **Sorting** by last updated or progress percentage
- **Search** with live substring matching on track titles and IDs
- **Mouse support** — click to select tracks, scroll to navigate
- **Keyboard-driven** — vim-style navigation (j/k), resizable split panes

## Installation

```sh
cargo install --path .
```

Or build from source:

```sh
cargo build --release
# Binary at target/release/conductor-dashboard
```

## Usage

```sh
conductor-dashboard --conductor-dir ./conductor
```

### Options

| Flag | Description |
|------|-------------|
| `--conductor-dir <PATH>` | Path to the conductor directory (default: `./conductor`) |
| `--no-watch` | Disable live file watching |
| `--filter <MODE>` | Initial filter: `all`, `active`, `blocked`, `complete` |

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑`/`k` | Move selection up |
| `↓`/`j` | Move selection down |
| `Home`/`End` | First/last track |
| `Enter` | Maximise detail panel |
| `Esc` | Return to split view / close overlay |
| `f` | Cycle filter |
| `s` | Cycle sort |
| `/` | Open search |
| `r` | Force refresh |
| `t` | Cycle theme |
| `d`/`u` | Scroll detail down/up |
| `[`/`]` | Resize split panes |
| `?` | Toggle help overlay |
| `q` | Quit |

## Themes

Press `t` to cycle through the 6 built-in themes:

- **Mako** — the default, inspired by the Mako Group colour palette
- **Warm Dark** — earthy warm tones on a dark background
- **Midnight** — deep blue-grey dark theme
- **Ember** — warm amber/brown dark theme
- **Dusk** — medium-contrast grey dark theme
- **Light** — light background with dark text

## Tech Stack

- [Rust](https://www.rust-lang.org/) (edition 2021)
- [Ratatui](https://ratatui.rs/) 0.29 + [Crossterm](https://github.com/crossterm-rs/crossterm) 0.28
- [Tokio](https://tokio.rs/) async runtime
- [notify](https://github.com/notify-rs/notify) for file watching
- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) for markdown parsing
- [clap](https://github.com/clap-rs/clap) for CLI args

## License

MIT
