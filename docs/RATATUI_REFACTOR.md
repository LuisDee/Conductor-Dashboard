# Conductor Dashboard — Iced → Ratatui Refactoring Plan

## Overview

This document is a **companion** to the existing `conductor-dashboard-rewrite-plan.md` (the "original plan"). It details every change needed to refactor the Rust rewrite from **Iced** (native GUI window) to **Ratatui + Crossterm** (terminal-native TUI) so the dashboard runs *inside* the terminal — SSH-friendly, tmux-composable, no window manager required.

**What stays the same:** All data models, type-safe enums, parsers, error types, caching logic, file watcher core, and business logic from the original plan are **unchanged**. This refactoring only touches the framework layer, event loop, rendering, and theme.

**What changes:** The Iced application shell, Subscription system, pane_grid, custom widgets, font bundling, and GPU rendering are replaced with ratatui's immediate-mode terminal rendering, crossterm's event stream, Layout constraints, and the Table/Gauge/Paragraph widget set.

---

## 1. Why Ratatui Instead of Iced

| Concern | Iced (Original) | Ratatui (Refactored) |
|---------|-----------------|----------------------|
| **Where it runs** | Separate native window (wgpu) | Inside the terminal — same pane, same tmux session |
| **SSH / headless** | Requires display server | Works over SSH, in containers, headless servers |
| **tmux / split** | External window, can't tile with terminal panes | Runs in a tmux pane next to your editor and Conductor agent |
| **Custom fonts** | Montserrat + JetBrains Mono via `include_bytes!` | Not available — use Unicode symbols + bold/dim styling instead |
| **Colour fidelity** | Exact hex via GPU | 24-bit true colour via crossterm (`Color::Rgb`) — identical hex values, requires modern terminal |
| **Resizable split** | `pane_grid` with drag handle | `Layout::horizontal` with percentage constraints — no drag, but keyboard-adjustable |
| **Binary deps** | Needs wgpu/vulkan/metal drivers | Zero graphics dependencies — pure terminal I/O |
| **Architecture** | Elm: State → Message → Update → View | Same Elm-style loop, manually wired with `tokio::select!` |

**The trade-off is clear:** we lose custom font rendering and mouse-draggable panes, but gain terminal nativeness — which is the whole point for a tool that monitors a CLI agent.

---

## 2. Dependency Changes

### Remove

```toml
# These are no longer needed
iced = { version = "0.13", features = ["tokio", "advanced"] }
```

### Add

```toml
[dependencies]
# TUI framework
ratatui = { version = "0.29", features = ["crossterm"] }
crossterm = { version = "0.28", features = ["event-stream"] }

# Async runtime + utilities
tokio = { version = "1", features = ["full"] }
futures = "0.3"

# File watching (unchanged)
notify = "7"
notify-debouncer-mini = "0.5"

# Parsing (unchanged)
pulldown-cmark = "0.12"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }

# CLI + errors (unchanged)
clap = { version = "4", features = ["derive"] }
thiserror = "2"
color-eyre = "0.6"
tracing = "0.1"
tracing-subscriber = "0.3"

# Optional: better fuzzy matching for search
fuzzy-matcher = "0.3"
```

### Full Dependency Table

| Crate | Purpose | Changed? |
|-------|---------|----------|
| `ratatui` | Terminal UI framework — Layout, Table, Gauge, Paragraph, Block | **NEW** (replaces `iced`) |
| `crossterm` | Terminal backend — raw mode, alternate screen, key/mouse events, true colour | **NEW** (replaces `iced`) |
| `tokio` | Async runtime — drives event loop, file watcher channel, async parsing | Unchanged (was required by Iced too) |
| `futures` | `StreamExt` for crossterm's `EventStream` | **NEW** |
| `notify` + `notify-debouncer-mini` | File system watching | Unchanged |
| `pulldown-cmark` | Markdown AST parsing | Unchanged |
| `serde` + `serde_json` | JSON deserialization | Unchanged |
| `chrono` | DateTime handling | Unchanged |
| `clap` | CLI args | Unchanged |
| `thiserror` | Typed errors | Unchanged |
| `color-eyre` | Error reporting | Unchanged |
| `tracing` + `tracing-subscriber` | Logging (writes to file, not terminal, since we own the screen) | Unchanged (but log to file) |
| `fuzzy-matcher` | Fuzzy search scoring | **NEW** (optional) |

---

## 3. Architecture Changes

### 3.1 Event Loop: Iced Subscription → tokio::select!

**Before (Iced):** The framework owned the event loop. We declared `subscription()` returning a batch of Subscriptions (file watcher, keyboard, timer) and Iced dispatched Messages to `update()`.

**After (Ratatui):** We own the event loop explicitly. A `tokio::select!` macro multiplexes three async sources into a single `Event` enum, which feeds the same update/render cycle.

```rust
// src/event.rs — The event hub

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::StreamExt;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    /// Terminal key/mouse/resize event
    Key(KeyEvent),
    Resize(u16, u16),

    /// File watcher detected changes
    FilesChanged(Vec<PathBuf>),

    /// Periodic tick (1 second) for clock, error auto-dismiss
    Tick,

    /// Async parse completed
    TracksLoaded(Result<TracksPayload, DashboardError>),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(conductor_dir: PathBuf) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn crossterm event reader
        let tx_key = tx.clone();
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            while let Some(Ok(evt)) = reader.next().await {
                match evt {
                    CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => {
                        let _ = tx_key.send(Event::Key(key));
                    }
                    CrosstermEvent::Resize(w, h) => {
                        let _ = tx_key.send(Event::Resize(w, h));
                    }
                    _ => {}
                }
            }
        });

        // Spawn tick timer
        let tx_tick = tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(1)
            );
            loop {
                interval.tick().await;
                if tx_tick.send(Event::Tick).is_err() {
                    break;
                }
            }
        });

        // Spawn file watcher
        let tx_watch = tx.clone();
        tokio::spawn(async move {
            let (wtx, mut wrx) = mpsc::channel::<Vec<PathBuf>>(100);

            let mut debouncer = notify_debouncer_mini::new_debouncer(
                std::time::Duration::from_millis(300),
                move |result: notify_debouncer_mini::DebounceEventResult| {
                    if let Ok(events) = result {
                        let paths: Vec<_> = events.iter()
                            .filter(|e| is_conductor_file(&e.path))
                            .map(|e| e.path.clone())
                            .collect();
                        if !paths.is_empty() {
                            let _ = wtx.blocking_send(paths);
                        }
                    }
                },
            ).expect("Failed to create file watcher");

            debouncer.watcher()
                .watch(&conductor_dir, notify::RecursiveMode::Recursive)
                .expect("Failed to watch conductor directory");

            // Keep debouncer alive; forward events
            while let Some(paths) = wrx.recv().await {
                if tx_watch.send(Event::FilesChanged(paths)).is_err() {
                    break;
                }
            }
        });

        EventHandler { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
```

**Key difference:** Instead of Iced's `Subscription::batch` abstracting this away, we explicitly spawn three tokio tasks that all feed into a single `mpsc::UnboundedChannel`. The main loop just calls `events.next().await`.

### 3.2 Main Loop: Framework-Managed → Explicit

**Before (Iced):**
```rust
fn main() -> iced::Result {
    ConductorDashboard::run(Settings::default())
}
```

**After (Ratatui):**
```rust
// src/main.rs

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Parse CLI args
    let args = Cli::parse();

    // Set up tracing to a log file (terminal is ours now)
    let file_appender = tracing_appender::rolling::daily(
        &args.log_dir, "conductor-dashboard.log"
    );
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .init();

    // Set up terminal
    let mut terminal = ratatui::init();

    // Run the app
    let result = App::new(args)?.run(&mut terminal).await;

    // Restore terminal (this is critical — even on panic)
    ratatui::restore();

    result
}
```

```rust
// src/app.rs — The main loop

impl App {
    pub async fn run(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
    ) -> color_eyre::Result<()> {
        // Initial load
        self.load_tracks().await?;

        // Start event handler
        let mut events = EventHandler::new(self.conductor_dir.clone());

        loop {
            // RENDER
            terminal.draw(|frame| self.render(frame))?;

            // WAIT FOR EVENT
            let Some(event) = events.next().await else {
                break; // channel closed
            };

            // UPDATE
            if self.handle_event(event).await? == Action::Quit {
                break;
            }
        }

        Ok(())
    }
}
```

This is the **same Elm architecture** — state → render → event → update — just expressed as an explicit loop instead of framework callbacks.

### 3.3 Message/Event Mapping

The `Message` enum from the original plan maps directly:

| Iced Message | Ratatui Event | Notes |
|-------------|---------------|-------|
| `FilesChanged(Vec<PathBuf>)` | `Event::FilesChanged(Vec<PathBuf>)` | Identical |
| `TracksLoaded(Result<...>)` | `Event::TracksLoaded(Result<...>)` | Identical |
| `TrackSelected(TrackId)` | Handled inside `Event::Key` handler | `↑`/`↓`/`j`/`k` changes selection in state |
| `KeyPressed(keyboard::Key)` | `Event::Key(KeyEvent)` | crossterm `KeyEvent` instead of iced `keyboard::Key` |
| `CycleFilter` | `Event::Key` → `f` key | Mapped in key handler |
| `CycleSort` | `Event::Key` → `s` key | Mapped in key handler |
| `ForceRefresh` | `Event::Key` → `r` key | Triggers full reload |
| `ToggleSearch` | `Event::Key` → `/` key | Shows search input |
| `SearchQueryChanged(String)` | `Event::Key` in search mode | Each keystroke updates query |
| `ToggleHelp` | `Event::Key` → `?` key | Shows help overlay |
| `ToggleFullscreen` | `Event::Key` → `Enter`/`Esc` | Toggles detail-only view |
| `PaneResized(...)` | `Event::Key` → `[`/`]` keys | **Changed:** keyboard-driven resize, not mouse drag |
| `Tick(Instant)` | `Event::Tick` | Updates clock, dismisses errors |
| `ErrorDismissed` | Handled within `Tick` handler | Auto-dismiss after 10s |

### 3.4 Application State Changes

The `ConductorDashboard` struct from the original plan changes minimally:

```rust
pub struct App {
    // === UNCHANGED from original plan ===
    tracks: BTreeMap<TrackId, Track>,
    conductor_dir: PathBuf,
    selected_track: Option<TrackId>,
    filter: FilterMode,
    sort: SortMode,
    search_query: String,
    watcher_active: bool,
    last_refresh: Instant,
    error_message: Option<(String, Instant)>,
    track_cache: TrackCache,

    // === CHANGED: Iced types → ratatui types ===
    table_state: TableState,          // was: scrollable::Id (track_list_scroll)
    detail_scroll: u16,               // was: scrollable::Id (detail_scroll)
    split_percent: u16,               // was: pane_grid::State<PaneKind>
    detail_maximised: bool,           // was: pane_grid maximisation

    // === NEW: TUI-specific state ===
    search_active: bool,              // is search input focused?
    help_visible: bool,               // is help overlay showing?
    mode: InputMode,                  // Normal | Search | Help
    clock: String,                    // pre-formatted clock string, updated on Tick
    filtered_track_ids: Vec<TrackId>, // cached list after filter+sort applied
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum InputMode {
    Normal,
    Search,
    Help,
}
```

**Key differences:**
- `pane_grid::State` → `split_percent: u16` (a simple number like `50` for 50/50 split, adjusted with `[`/`]` keys)
- `scrollable::Id` → `TableState` (ratatui's built-in stateful table selection) for the track list and a plain `u16` offset for detail scroll
- New `InputMode` enum to handle modal states (search input captures all keystrokes)

---

## 4. Rendering Architecture

### 4.1 Layout: pane_grid → Layout::horizontal

**Before (Iced):**
```rust
pane_grid(&self.panes, |pane, kind, _is_maximized| {
    match kind {
        PaneKind::TrackList => self.view_track_list(),
        PaneKind::Detail => self.view_detail_panel(),
    }
})
.on_resize(10, Message::PaneResized)
```

**After (Ratatui):**
```rust
fn render(&mut self, frame: &mut Frame) {
    let area = frame.area();

    // Vertical: title bar | stats bar | main content | status bar
    let [title_area, stats_area, main_area, status_area] =
        Layout::vertical([
            Constraint::Length(1),    // title bar
            Constraint::Length(2),    // stats bar + filter chips
            Constraint::Fill(1),     // main content (fills remaining)
            Constraint::Length(1),    // status bar
        ])
        .areas(area);

    self.render_title_bar(frame, title_area);
    self.render_stats_bar(frame, stats_area);
    self.render_status_bar(frame, status_area);

    // Horizontal split: track list | detail panel
    if self.detail_maximised {
        self.render_detail_panel(frame, main_area);
    } else {
        let [list_area, detail_area] = Layout::horizontal([
            Constraint::Percentage(self.split_percent),
            Constraint::Percentage(100 - self.split_percent),
        ])
        .areas(main_area);

        self.render_track_list(frame, list_area);
        self.render_detail_panel(frame, detail_area);
    }

    // Overlays (drawn last, on top)
    if self.mode == InputMode::Search {
        self.render_search_overlay(frame, area);
    }
    if self.mode == InputMode::Help {
        self.render_help_overlay(frame, area);
    }
}
```

**Split resizing** is keyboard-driven: `[` shrinks left pane by 5%, `]` grows it, clamped to 20–80%.

### 4.2 Terminal Layout Diagram

```
┌─ Title Bar ─────────────────────────────────────────────────┐ Length(1)
│ ◇ Conductor Dashboard               12:34:56  ● WATCHING   │
├─ Stats Bar ─────────────────────────────────────────────────┤ Length(2)
│ 6 Total │ 2 Active │ 1 Blocked │ 3 Complete                │
│ [F]ilter: All  [S]ort: Recent  [/]Search                   │
├─ Track List (left) ──────────┬─ Detail Panel (right) ──────┤ Fill(1)
│                              │                             │
│ Track         Status  Prog   │ FEATURE · pad_compliance    │
│ ────────────────────────────│                             │
│▸ PAD Compliance  ⚙ ACT 62%  │ PAD Compliance System       │
│  Ph.3 · Jan 20   ████░ 18/29│ ⚙ Active  Created: Jan 20  │
│                              │                             │
│  Oracle→PG       ⚙ ACT 41%  │ ┌ Overall Progress ───────┐ │
│  Ph.2 · Jan 15   ███░░  7/17│ │ 18/29  ████████░░░ 62%  │ │
│                              │ └─────────────────────────┘ │
│  GCS Cost Opt    ⚠ BLK 15%  │                             │
│  Ph.1 · Feb 01   █░░░░  2/13│ IMPLEMENTATION PLAN         │
│                              │                             │
│  Vol Box         ✓ DON 100% │ ● Phase 1: Parsing    5/5  │
│  Done · Jan 05   █████ 22/22│   ✓ Build PDF reader       │
│                              │   ✓ Extract fields         │
│                              │                             │
│                              │ ◐ Phase 2: Database   3/6  │
│                              │   ✓ Design schema          │
│                              │   ○ Write migrations       │
├─ Status Bar ────────────────┴──────────────────────────────┤ Length(1)
│ ↑↓/jk Navigate  Enter Expand  f Filter  s Sort  ? Help    │
└─────────────────────────────────────────────────────────────┘
```

### 4.3 TUI Layout ASCII vs Original Design

The original plan's visual spec had explicit pixel widths for columns (Status ~110px, Progress ~180px, etc.). In a terminal, everything is character-cell based. Here's the column mapping:

| Original Column | TUI Constraint | Rendering |
|----------------|---------------|-----------|
| Track (flex width) | `Constraint::Fill(1)` — takes remaining space | Title on line 1, "Ph.{n} · {date}" on line 2 (Row height = 2) |
| Status (~110px) | `Constraint::Length(5)` | Short codes: `⚙ ACT`, `⚠ BLK`, `✓ DON`, `○ NEW` — coloured |
| Created (~80px) | **Removed** — shown in track subtitle | "Ph.2 · Jan 15" combines phase + date |
| Progress (~180px) | `Constraint::Length(12)` | `████░░ 62%` using Unicode block chars |
| Tasks (~70px) | `Constraint::Length(6)` | `18/29` right-aligned |

**Net effect:** The table is more compact but communicates the same information. Created date moves to the subtitle line in the track column.

---

## 5. Theme: Mako Colours in True Colour

Crossterm supports 24-bit RGB colour. The Mako palette maps directly:

```rust
// src/theme/colors.rs

use ratatui::style::Color;

pub struct MakoColors;

impl MakoColors {
    // Primary palette
    pub const NAVY: Color      = Color::Rgb(14, 30, 63);     // #0E1E3F
    pub const BLUE: Color      = Color::Rgb(84, 113, 223);   // #5471DF
    pub const GOLD: Color      = Color::Rgb(178, 140, 84);   // #B28C54
    pub const LIGHT_BLUE: Color = Color::Rgb(219, 225, 245); // #DBE1F5
    pub const SUCCESS: Color   = Color::Rgb(44, 95, 45);     // #2C5F2D
    pub const ERROR: Color     = Color::Rgb(184, 80, 66);    // #B85042

    // Surfaces
    pub const BG: Color        = Color::Rgb(244, 246, 251);  // #F4F6FB
    pub const SURFACE: Color   = Color::Rgb(255, 255, 255);  // #FFFFFF
    pub const BORDER: Color    = Color::Rgb(209, 217, 232);  // #D1D9E8

    // Text
    pub const TEXT_PRIMARY: Color   = Color::Rgb(30, 30, 30);
    pub const TEXT_SECONDARY: Color = Color::Rgb(120, 120, 140);
    pub const TEXT_ON_NAVY: Color   = Color::Rgb(255, 255, 255);
}
```

**Usage in styles:**
```rust
use ratatui::style::{Style, Modifier};

// Title bar (navy background, white text)
let title_style = Style::default()
    .bg(MakoColors::NAVY)
    .fg(MakoColors::TEXT_ON_NAVY)
    .add_modifier(Modifier::BOLD);

// Selected track row (blue highlight)
let selected_style = Style::default()
    .bg(MakoColors::BLUE)
    .fg(Color::White)
    .add_modifier(Modifier::BOLD);

// Status badges
fn status_style(status: Status) -> Style {
    match status {
        Status::InProgress => Style::default().fg(MakoColors::BLUE).add_modifier(Modifier::BOLD),
        Status::Blocked    => Style::default().fg(MakoColors::GOLD).add_modifier(Modifier::BOLD),
        Status::Complete   => Style::default().fg(MakoColors::SUCCESS),
        Status::New        => Style::default().fg(MakoColors::TEXT_SECONDARY),
    }
}
```

**Font replacement:** Since terminals can't render Montserrat, we use:
- **Bold** (`Modifier::BOLD`) where ExtraBold/SemiBold was specified
- **Dim** (`Modifier::DIM`) for secondary/muted text
- **No italic** (most terminals render it poorly) — use colour differentiation instead

---

## 6. Widget Mapping

Every Iced custom widget from the original plan maps to a ratatui equivalent:

### 6.1 Track List → `Table` (StatefulWidget)

The track list left pane uses ratatui's built-in `Table` with `TableState` for selection:

```rust
fn render_track_list(&mut self, frame: &mut Frame, area: Rect) {
    let header = Row::new(vec!["Track", "Status", "Progress", "Tasks"])
        .style(Style::default().fg(MakoColors::TEXT_SECONDARY).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = self.filtered_track_ids.iter()
        .filter_map(|id| self.tracks.get(id))
        .map(|track| {
            let title = Line::from(vec![
                Span::styled(&track.title, Style::default().add_modifier(Modifier::BOLD)),
            ]);
            let subtitle = Line::from(vec![
                Span::styled(
                    format!("Ph.{} · {}", track.phase, track.created_at.format("%b %d")),
                    Style::default().fg(MakoColors::TEXT_SECONDARY),
                ),
            ]);

            Row::new(vec![
                Cell::from(Text::from(vec![title, subtitle])),
                Cell::from(status_span(&track.status)),
                Cell::from(progress_bar_text(track.progress_percent())),
                Cell::from(format!("{}/{}", track.tasks_completed, track.tasks_total)),
            ])
            .height(2) // two lines per row
        })
        .collect();

    let widths = [
        Constraint::Fill(1),       // Track name
        Constraint::Length(5),     // Status
        Constraint::Length(12),    // Progress bar
        Constraint::Length(6),     // Tasks
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::bordered().border_style(Style::default().fg(MakoColors::BORDER)))
        .row_highlight_style(
            Style::default().bg(MakoColors::BLUE).fg(Color::White)
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(table, area, &mut self.table_state);
}
```

### 6.2 Progress Bar → Unicode Block Characters

**Before (Iced):** Custom `widgets/progress_bar.rs` with gradient fill.

**After:** A pure function that returns a styled `Span`:

```rust
fn progress_bar_text(percent: f32) -> Text<'static> {
    let width = 8; // character cells for the bar
    let filled = ((percent / 100.0) * width as f32).round() as usize;
    let empty = width - filled;

    let bar = format!(
        "{}{} {:>3.0}%",
        "█".repeat(filled),
        "░".repeat(empty),
        percent,
    );

    let color = if percent >= 100.0 {
        MakoColors::SUCCESS
    } else if percent > 0.0 {
        MakoColors::BLUE
    } else {
        MakoColors::TEXT_SECONDARY
    };

    Text::from(Span::styled(bar, Style::default().fg(color)))
}
```

### 6.3 Status Badge → Coloured Span

**Before (Iced):** Custom `widgets/status_badge.rs` with coloured dot + label.

**After:**

```rust
fn status_span(status: &Status) -> Span<'static> {
    match status {
        Status::InProgress => Span::styled(
            "⚙ ACT", Style::default().fg(MakoColors::BLUE).add_modifier(Modifier::BOLD)
        ),
        Status::Blocked => Span::styled(
            "⚠ BLK", Style::default().fg(MakoColors::GOLD).add_modifier(Modifier::BOLD)
        ),
        Status::Complete => Span::styled(
            "✓ DON", Style::default().fg(MakoColors::SUCCESS)
        ),
        Status::New => Span::styled(
            "○ NEW", Style::default().fg(MakoColors::TEXT_SECONDARY)
        ),
    }
}
```

### 6.4 Detail Panel → Paragraph + Block Composition

**Before (Iced):** Custom view composing containers, text, and the plan_view widget.

**After:** A `render_detail_panel` function that manually composes ratatui widgets into the allocated area:

```rust
fn render_detail_panel(&self, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .border_style(Style::default().fg(MakoColors::BORDER))
        .title(" Detail ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(track_id) = &self.selected_track else {
        // Empty state
        let msg = Paragraph::new("Select a track to view details")
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(MakoColors::TEXT_SECONDARY));
        frame.render_widget(msg, inner);
        return;
    };

    let Some(track) = self.tracks.get(track_id) else { return; };

    // Build detail content as a Vec<Line>
    let mut lines: Vec<Line> = Vec::new();

    // Type label + track ID
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:?}", track.track_type).to_uppercase(),
            Style::default().fg(MakoColors::TEXT_SECONDARY).add_modifier(Modifier::DIM),
        ),
        Span::raw(" · "),
        Span::styled(
            track.id.as_str(),
            Style::default().fg(MakoColors::TEXT_SECONDARY),
        ),
    ]));

    // Title
    lines.push(Line::from(Span::styled(
        &track.title,
        Style::default().add_modifier(Modifier::BOLD),
    )));

    // Status + created date
    lines.push(Line::from(vec![
        status_span(&track.status),
        Span::raw("  Created: "),
        Span::raw(track.created_at.format("%b %d, %Y").to_string()),
    ]));

    lines.push(Line::raw(""));

    // Progress bar (full width)
    let pct = track.progress_percent();
    let bar_width = inner.width.saturating_sub(12) as usize;
    let filled = ((pct / 100.0) * bar_width as f32).round() as usize;
    let empty = bar_width.saturating_sub(filled);
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}/{} ", track.tasks_completed, track.tasks_total),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled("█".repeat(filled), Style::default().fg(MakoColors::BLUE)),
        Span::styled("░".repeat(empty), Style::default().fg(MakoColors::BORDER)),
        Span::raw(format!(" {:.0}%", pct)),
    ]));

    lines.push(Line::raw(""));

    // Dependencies (if any)
    if !track.dependencies.is_empty() {
        lines.push(Line::styled(
            format!("⚠ Blocked by: {}", track.dependencies.iter()
                .map(|d| d.as_str()).collect::<Vec<_>>().join(", ")),
            Style::default().fg(MakoColors::GOLD),
        ));
        lines.push(Line::raw(""));
    }

    // Implementation Plan heading
    lines.push(Line::styled(
        "IMPLEMENTATION PLAN",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::raw(""));

    // Phase blocks
    for phase in &track.plan_phases {
        let phase_icon = match phase.status {
            PhaseStatus::Complete => "●",
            PhaseStatus::Active   => "◐",
            PhaseStatus::Pending  => "○",
            PhaseStatus::Blocked  => "⊘",
        };
        let done = phase.tasks.iter().filter(|t| t.done).count();
        let total = phase.tasks.len();

        lines.push(Line::from(vec![
            Span::styled(phase_icon, status_color(phase.status)),
            Span::raw(format!(" {} ", phase.name)),
            Span::styled(
                format!("{}/{}", done, total),
                Style::default().fg(MakoColors::TEXT_SECONDARY),
            ),
        ]));

        for task in &phase.tasks {
            let (icon, style) = if task.done {
                ("  ✓ ", Style::default().fg(MakoColors::SUCCESS).add_modifier(Modifier::DIM))
            } else {
                ("  ○ ", Style::default().fg(MakoColors::TEXT_PRIMARY))
            };
            lines.push(Line::styled(format!("{}{}", icon, task.text), style));
        }

        lines.push(Line::raw(""));
    }

    // Render as scrollable paragraph
    let paragraph = Paragraph::new(lines)
        .scroll((self.detail_scroll, 0));

    frame.render_widget(paragraph, inner);

    // Scrollbar for detail panel
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state = ScrollbarState::new(total_lines)
        .position(self.detail_scroll as usize);
    frame.render_stateful_widget(
        scrollbar,
        inner.inner(Margin { vertical: 0, horizontal: 0 }),
        &mut scrollbar_state,
    );
}
```

### 6.5 Title Bar → Styled Paragraph (Length 1)

```rust
fn render_title_bar(&self, frame: &mut Frame, area: Rect) {
    let watcher_indicator = if self.watcher_active {
        Span::styled("● WATCHING", Style::default().fg(MakoColors::SUCCESS))
    } else {
        Span::styled("○ STATIC", Style::default().fg(MakoColors::TEXT_SECONDARY))
    };

    let title = Line::from(vec![
        Span::styled(" ◇ Conductor Dashboard", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        // Right-align clock and watcher by padding
        Span::raw(" ".repeat(
            area.width.saturating_sub(45) as usize
        )),
        Span::raw(&self.clock),
        Span::raw("  "),
        watcher_indicator,
        Span::raw(" "),
    ]);

    frame.render_widget(
        Paragraph::new(title).style(
            Style::default().bg(MakoColors::NAVY).fg(MakoColors::TEXT_ON_NAVY)
        ),
        area,
    );
}
```

### 6.6 Status Bar → Styled Paragraph (Length 1)

```rust
fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
    let shortcuts = Line::from(vec![
        Span::styled(" ↑↓", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Navigate  "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Expand  "),
        Span::styled("f", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Filter  "),
        Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Sort  "),
        Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Search  "),
        Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Help  "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Quit"),
    ]);

    frame.render_widget(
        Paragraph::new(shortcuts).style(
            Style::default().bg(MakoColors::NAVY).fg(MakoColors::TEXT_ON_NAVY)
        ),
        area,
    );
}
```

### 6.7 Stats Bar → Line with Styled Spans

```rust
fn render_stats_bar(&self, frame: &mut Frame, area: Rect) {
    let [counts_area, controls_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
    ]).areas(area);

    // Line 1: track counts
    let total = self.tracks.len();
    let active = self.tracks.values().filter(|t| t.status == Status::InProgress).count();
    let blocked = self.tracks.values().filter(|t| t.status == Status::Blocked).count();
    let complete = self.tracks.values().filter(|t| t.status == Status::Complete).count();

    let counts = Line::from(vec![
        Span::styled(format!(" {} Total", total), Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" │ "),
        Span::styled(format!("{} Active", active), Style::default().fg(MakoColors::BLUE)),
        Span::raw(" │ "),
        Span::styled(format!("{} Blocked", blocked), Style::default().fg(MakoColors::GOLD)),
        Span::raw(" │ "),
        Span::styled(format!("{} Complete", complete), Style::default().fg(MakoColors::SUCCESS)),
    ]);
    frame.render_widget(Paragraph::new(counts), counts_area);

    // Line 2: filter + sort controls
    let filter_label = match self.filter {
        FilterMode::All      => "[All]  Active  Blocked  Done",
        FilterMode::Active   => " All  [Active] Blocked  Done",
        FilterMode::Blocked  => " All   Active [Blocked] Done",
        FilterMode::Complete => " All   Active  Blocked [Done]",
    };
    let controls = Line::from(vec![
        Span::styled(format!(" Filter: {}", filter_label), Style::default().fg(MakoColors::TEXT_SECONDARY)),
        Span::raw("  │  "),
        Span::styled(
            format!("Sort: {}", if self.sort == SortMode::Updated { "[Recent] Progress" } else { " Recent [Progress]" }),
            Style::default().fg(MakoColors::TEXT_SECONDARY),
        ),
    ]);
    frame.render_widget(Paragraph::new(controls), controls_area);
}
```

### 6.8 Help Overlay → Centered Clear + Paragraph

```rust
fn render_help_overlay(&self, frame: &mut Frame, area: Rect) {
    // Centered popup area (60x20)
    let popup_area = centered_rect(60, 20, area);

    // Clear background
    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::styled("Keyboard Shortcuts", Style::default().add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::raw("  ↑/k       Move selection up"),
        Line::raw("  ↓/j       Move selection down"),
        Line::raw("  Enter     Maximise detail panel"),
        Line::raw("  Esc       Return to split view / close"),
        Line::raw("  f         Cycle filter (All → Active → Blocked → Done)"),
        Line::raw("  s         Cycle sort (Recent ↔ Progress)"),
        Line::raw("  /         Open search"),
        Line::raw("  r         Force refresh"),
        Line::raw("  [/]       Resize split (left/right)"),
        Line::raw("  ?         Toggle this help"),
        Line::raw("  q         Quit"),
        Line::raw(""),
        Line::styled("Press any key to close", Style::default().fg(MakoColors::TEXT_SECONDARY)),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::bordered()
                .title(" Help ")
                .border_style(Style::default().fg(MakoColors::BLUE))
                .style(Style::default().bg(MakoColors::SURFACE))
        );

    frame.render_widget(help, popup_area);
}

/// Returns a centered Rect of given width% and height lines
fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ]).split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ]).split(popup_layout[1])[1]
}
```

### 6.9 Search Overlay → Input Line at Top

```rust
fn render_search_overlay(&self, frame: &mut Frame, area: Rect) {
    // Render a search bar at the top of the main area
    let search_area = Rect {
        x: area.x + 1,
        y: area.y + 3, // below title + stats bars
        width: area.width.saturating_sub(2),
        height: 1,
    };

    frame.render_widget(Clear, search_area);

    let search_line = Line::from(vec![
        Span::styled(" / ", Style::default().fg(MakoColors::BLUE).add_modifier(Modifier::BOLD)),
        Span::raw(&self.search_query),
        Span::styled("█", Style::default().fg(MakoColors::BLUE)), // cursor
    ]);

    frame.render_widget(
        Paragraph::new(search_line)
            .style(Style::default().bg(MakoColors::SURFACE).fg(MakoColors::TEXT_PRIMARY)),
        search_area,
    );
}
```

---

## 7. Key Event Handling

```rust
// src/app.rs

fn handle_key_event(&mut self, key: KeyEvent) -> Action {
    // Global keys (always active)
    match key.code {
        KeyCode::Char('q') if self.mode == InputMode::Normal => return Action::Quit,
        KeyCode::Char('?') => {
            self.mode = if self.mode == InputMode::Help {
                InputMode::Normal
            } else {
                InputMode::Help
            };
            return Action::Continue;
        }
        KeyCode::Esc => {
            match self.mode {
                InputMode::Search => {
                    self.mode = InputMode::Normal;
                    self.search_query.clear();
                    self.recompute_filtered_tracks();
                }
                InputMode::Help => {
                    self.mode = InputMode::Normal;
                }
                InputMode::Normal if self.detail_maximised => {
                    self.detail_maximised = false;
                }
                _ => {}
            }
            return Action::Continue;
        }
        _ => {}
    }

    // Search mode: capture all input
    if self.mode == InputMode::Search {
        match key.code {
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.recompute_filtered_tracks();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.recompute_filtered_tracks();
            }
            KeyCode::Enter => {
                self.mode = InputMode::Normal;
                // keep filter applied
            }
            _ => {}
        }
        return Action::Continue;
    }

    // Normal mode keys
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => self.select_next(),
        KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
        KeyCode::Home => self.select_first(),
        KeyCode::End => self.select_last(),
        KeyCode::Enter => self.detail_maximised = true,
        KeyCode::Char('f') => {
            self.filter = self.filter.next();
            self.recompute_filtered_tracks();
        }
        KeyCode::Char('s') => {
            self.sort = self.sort.next();
            self.recompute_filtered_tracks();
        }
        KeyCode::Char('/') => {
            self.mode = InputMode::Search;
        }
        KeyCode::Char('r') => {
            // Force refresh — will be handled by caller
            return Action::ForceRefresh;
        }
        KeyCode::Char('[') => {
            self.split_percent = (self.split_percent.saturating_sub(5)).max(20);
        }
        KeyCode::Char(']') => {
            self.split_percent = (self.split_percent + 5).min(80);
        }
        // Detail panel scrolling (when detail has focus)
        KeyCode::Char('d') => self.detail_scroll = self.detail_scroll.saturating_add(5),
        KeyCode::Char('u') => self.detail_scroll = self.detail_scroll.saturating_sub(5),
        _ => {}
    }

    Action::Continue
}

fn select_next(&mut self) {
    let len = self.filtered_track_ids.len();
    if len == 0 { return; }
    let i = self.table_state.selected().map(|s| (s + 1).min(len - 1)).unwrap_or(0);
    self.table_state.select(Some(i));
    self.selected_track = self.filtered_track_ids.get(i).cloned();
    self.detail_scroll = 0; // reset scroll on selection change
}

fn select_previous(&mut self) {
    let len = self.filtered_track_ids.len();
    if len == 0 { return; }
    let i = self.table_state.selected().map(|s| s.saturating_sub(1)).unwrap_or(0);
    self.table_state.select(Some(i));
    self.selected_track = self.filtered_track_ids.get(i).cloned();
    self.detail_scroll = 0;
}
```

---

## 8. Updated Module Structure

```
conductor-dashboard/
├── Cargo.toml
├── src/
│   ├── main.rs                 # tokio::main, CLI args, terminal setup/restore
│   ├── app.rs                  # App struct, run(), handle_event(), render()
│   ├── event.rs                # Event enum, EventHandler (crossterm + watcher + tick)
│   │
│   ├── model/                  # *** UNCHANGED from original plan ***
│   │   ├── mod.rs
│   │   ├── track.rs            # Track, PlanPhase, PlanTask
│   │   ├── state.rs            # Filter/sort logic
│   │   ├── enums.rs            # Status, Priority, FilterMode, SortMode, TrackType
│   │   └── cache.rs            # TrackCache (mtime tracking)
│   │
│   ├── parser/                 # *** UNCHANGED from original plan ***
│   │   ├── mod.rs
│   │   ├── index.rs            # Parse tracks.md (pulldown-cmark AST)
│   │   ├── metadata.rs         # Parse metadata.json (serde defaults)
│   │   ├── plan.rs             # Parse plan.md (phases + tasks)
│   │   └── error.rs            # ParseError enum (thiserror)
│   │
│   ├── theme/
│   │   ├── mod.rs
│   │   └── colors.rs           # Mako palette as Color::Rgb constants
│   │   *** REMOVED: mako_theme.rs (no Iced theme impl needed) ***
│   │   *** REMOVED: fonts.rs (no custom fonts in terminal) ***
│   │
│   └── ui/                     # Renamed from views/ — pure render functions
│       ├── mod.rs
│       ├── title_bar.rs        # render_title_bar(frame, area)
│       ├── stats_bar.rs        # render_stats_bar(frame, area)
│       ├── track_list.rs       # render_track_list(frame, area, state)
│       ├── detail_panel.rs     # render_detail_panel(frame, area)
│       ├── plan_view.rs        # render_plan(lines, phases) → appends to Vec<Line>
│       ├── status_bar.rs       # render_status_bar(frame, area)
│       ├── help_overlay.rs     # render_help_overlay(frame, area)
│       ├── search_bar.rs       # render_search_overlay(frame, area)
│       └── components.rs       # status_span(), progress_bar_text(), centered_rect()
│
│   *** REMOVED: widgets/ directory ***
│   *** (progress_bar, status_badge, filter_chip — all inlined as functions) ***
│
│   *** REMOVED: watcher/subscription.rs ***
│   *** (file watcher is now a tokio::spawn in event.rs, not an Iced Subscription) ***
│
└── tests/                      # *** UNCHANGED from original plan ***
    ├── parser_tests.rs
    ├── model_tests.rs
    └── integration_tests.rs
```

**Changes summarised:**
- `app.rs` — Complete rewrite (Iced application → explicit event loop)
- `event.rs` — **New** (replaces `watcher/subscription.rs` + Iced keyboard/timer subscriptions)
- `theme/` — Simplified (just colours, no Iced theme impl or font loading)
- `views/` → `ui/` — Rewritten as `fn render_*(frame, area)` instead of `fn view_*() -> Element<Message>`
- `widgets/` — **Removed** (progress_bar, status_badge, filter_chip are now inline functions in `ui/components.rs`)
- `assets/fonts/` — **Removed** (no custom fonts in TUI)
- `model/`, `parser/`, `tests/` — **Unchanged**

---

## 9. Updated Implementation Phases

### Phase 1: Foundation (Core Data + Parsing) — NO CHANGES

This phase is **identical** to the original plan. Nothing in Phase 1 touches the UI framework.

Tasks 1–8 are unchanged: scaffold Cargo project, define enums, define data structs, implement index/metadata/plan parsers with pulldown-cmark, implement TrackCache, write parser unit tests.

**Only difference:** The `Cargo.toml` dependencies use ratatui/crossterm instead of iced. See Section 2 of this document.

### Phase 2: TUI Shell + Theme (was "Iced Shell + Theme")

**Goal:** Render a terminal UI with the Mako colour scheme, title bar, status bar, and empty split layout.

**Tasks (rewritten):**

1. **Set up terminal application** in `main.rs` — `ratatui::init()`, `ratatui::restore()`, tokio runtime. Ensure terminal is always restored, even on panic (use `std::panic::set_hook`).

2. **Implement EventHandler** in `event.rs` — Spawn three tokio tasks (crossterm EventStream, tick timer, file watcher) feeding into a single `mpsc::UnboundedChannel<Event>`. This replaces Iced's `Subscription::batch`.

3. **Implement main event loop** in `app.rs` — `terminal.draw(|f| self.render(f))` → `events.next().await` → `self.handle_event(event)`. This replaces Iced's framework-managed loop.

4. **Implement Mako colour constants** in `theme/colors.rs` — All colours as `Color::Rgb(r, g, b)`. No Iced theme impl needed.

5. **Build title bar** (`ui/title_bar.rs`) — Single `Paragraph` with navy background, app name, clock placeholder, watcher indicator. Uses `Layout::vertical` `Constraint::Length(1)`.

6. **Build status bar** (`ui/status_bar.rs`) — Single `Paragraph` with navy background, keyboard shortcut hints. Uses `Constraint::Length(1)`.

7. **Build layout shell** — `Layout::vertical` for title/stats/main/status, `Layout::horizontal` for left/right panes. Render empty `Block::bordered()` in each pane as placeholder.

8. **CLI argument parsing** (`main.rs`) — Unchanged from original plan (`clap` derive for `--conductor-dir`, `--no-watch`, `--filter`).

9. **Set up logging to file** — Since we own the terminal, `tracing-subscriber` must write to a file, not stderr. Use `tracing-appender` for daily log rotation.

### Phase 3: Track List (Left Pane) — REWRITTEN

**Goal:** Populate the left pane with a navigable, styled track table.

**Tasks (rewritten):**

1. **Implement stats bar** (`ui/stats_bar.rs`) — Two `Line`s: counts row + filter/sort indicator row. Wire `f` and `s` keys to cycle filter/sort in `handle_key_event`.

2. **Implement track table** (`ui/track_list.rs`) — Use ratatui's `Table` (StatefulWidget) with `TableState`. Each `Row` has height 2 (title + subtitle). Columns: Track (Fill), Status (Length 5), Progress (Length 12), Tasks (Length 6).

3. **Implement progress bar function** (`ui/components.rs`) — `fn progress_bar_text(percent) -> Text` using Unicode block characters (`█░`).

4. **Implement status badge function** (`ui/components.rs`) — `fn status_span(status) -> Span` returning coloured abbreviated status.

5. **Implement filter/sort logic** — `fn recompute_filtered_tracks(&mut self)` applies filter + sort + search query, stores result in `self.filtered_track_ids`. Called whenever filter, sort, or search changes.

6. **Wire keyboard navigation** — `j`/`k`/`↑`/`↓` call `select_next()`/`select_previous()` which update `TableState` and `selected_track`. `Home`/`End` for first/last. `TableState` handles scroll automatically.

### Phase 4: Detail Panel (Right Pane) — REWRITTEN

**Goal:** Show full track details with scrollable plan view.

**Tasks (rewritten):**

1. **Implement detail header** — Type label, track ID, title (bold), status span + created date. All rendered as `Line`s in a `Vec<Line>`.

2. **Implement progress summary** — Full-width Unicode progress bar with task count, using the wider inner area width for more granular fill.

3. **Implement dependency warning** — Gold-coloured `Line` listing blocking track IDs. Only rendered when `track.dependencies` is non-empty.

4. **Implement plan view** (`ui/plan_view.rs`) — Iterate `track.plan_phases`, render phase headers with status dot + name + task count, then task items with `✓`/`○` icons and appropriate styling (dim + green for complete, normal for pending).

5. **Implement scrollable rendering** — All detail content builds a `Vec<Line>`, rendered via `Paragraph::new(lines).scroll((self.detail_scroll, 0))`. `d`/`u` keys scroll the detail panel. Render a `Scrollbar` widget alongside.

6. **Empty state** — When no track is selected, render "Select a track to view details" centred in the detail area.

7. **Reset scroll on selection change** — When `selected_track` changes, set `self.detail_scroll = 0`.

### Phase 5: File Watcher + Live Updates — SIMPLIFIED

**Goal:** Watch conductor directory and update the dashboard in real time.

**Tasks (rewritten):**

1. **File watcher already implemented** — The `EventHandler` from Phase 2 already spawns the notify watcher as a tokio task. No Iced `Subscription` wrapping needed.

2. **Handle `Event::FilesChanged`** in the main loop — Classify changes via `TrackCache`, spawn async parse task, send `Event::TracksLoaded` back through the event channel.

3. **Incremental refresh** — Same logic as original plan: specific tracks if only their files changed, full re-parse if `tracks.md` changed.

4. **Update watcher indicator** — Set `self.watcher_active` based on watcher status, reflected in title bar render.

5. **Test with running Conductor agent** — Same as original plan.

### Phase 6: Search, Help, and Polish — ADAPTED

**Goal:** Add search, help overlay, error display, and polish.

**Tasks (rewritten):**

1. **Search overlay** (`ui/search_bar.rs`) — `/` sets `mode = InputMode::Search`. All keystrokes go to search query. Track list filters live by substring match on title. `Enter` confirms, `Esc` cancels and clears. Render as a single highlighted line at the top of the main area.

2. **Help overlay** (`ui/help_overlay.rs`) — `?` sets `mode = InputMode::Help`. Render a centred popup using `Clear` widget + bordered `Paragraph`. Any key dismisses.

3. **Error display** — Parse/watcher errors stored in `self.error_message: Option<(String, Instant)>`. Rendered as a gold-coloured `Line` between the stats bar and track list. Auto-dismissed after 10 seconds via `Tick` handler checking elapsed time.

4. **Fullscreen detail toggle** — `Enter` sets `self.detail_maximised = true`, causing `render()` to give the detail panel the full `main_area`. `Esc` restores split view.

5. **Polish:**
   - Unicode status indicators: `⚙` (in-progress), `⚠` (blocked), `✓` (complete), `○` (new)
   - Progress bar uses `█` (filled) and `░` (empty) with colour per status
   - Selected row highlight with `▸` marker
   - Pane resize with `[`/`]` keys (5% increments, clamped 20–80%)
   - Error bar auto-dismiss animation (not true animation, just disappears after timeout)

6. **Config file support** (optional) — Same as original plan: `.conductor-dashboard.toml` for defaults.

### Phase 7: Testing and Distribution — MINOR CHANGES

**Tasks:**

1–4: **Unchanged** — Parser tests, model tests, integration tests, property-based tests.

5. **Release build** — Same: `cargo build --release`, strip symbols. Binary should be **even smaller** than the Iced version (no wgpu/GPU deps). Target: < 5MB (down from < 15MB).

6. **CI pipeline** — Same: fmt, clippy, test, build on Linux/macOS/Windows.

7. **Distribution** — Same: GitHub Releases + `cargo install`. **Bonus:** No graphics driver requirements means it works on any server/container out of the box.

---

## 10. Terminal Compatibility Notes

### True Colour Requirement

The Mako palette uses specific RGB values (`#0E1E3F`, `#5471DF`, etc.). These require a terminal that supports 24-bit true colour. Most modern terminals do:

- ✅ iTerm2, Alacritty, WezTerm, Kitty, Windows Terminal, GNOME Terminal, foot
- ⚠️ macOS Terminal.app (limited — may fall back to 256 colours)
- ❌ xterm (unless compiled with `--enable-direct-color`)

**Fallback strategy:** Detect `$COLORTERM` env var. If not `truecolor` or `24bit`, fall back to the closest 256-colour approximations:

```rust
fn resolve_color(rgb: Color, supports_true_color: bool) -> Color {
    if supports_true_color {
        rgb
    } else {
        // Map to closest 256-color
        match rgb {
            MakoColors::NAVY => Color::Indexed(17),    // dark blue
            MakoColors::BLUE => Color::Indexed(69),    // medium blue
            MakoColors::GOLD => Color::Indexed(178),   // gold
            // ... etc
            _ => rgb,
        }
    }
}
```

### Unicode Symbols

The dashboard uses Unicode characters that require a font with good symbol coverage:

| Symbol | Name | Usage | Fallback |
|--------|------|-------|----------|
| `◇` | Diamond | App logo | `*` |
| `●` / `◐` / `○` | Circles | Phase status | `[X]` / `[~]` / `[ ]` |
| `⚙` | Gear | In-progress | `>` |
| `⚠` | Warning | Blocked | `!` |
| `✓` | Check | Complete | `+` |
| `█` / `░` | Blocks | Progress bar | `#` / `-` |
| `▸` | Triangle | Selection marker | `>` |

Most modern monospace fonts (JetBrains Mono, Fira Code, Cascadia Code, SF Mono) support all of these. Nerd Fonts are not required.

### Minimum Terminal Size

The layout needs at least **80 columns × 24 rows** to render properly. Below that, the detail panel won't fit. The app should detect terminal size on startup and on `Resize` events, degrading gracefully:

- < 80 cols: Hide detail panel, show track list only
- < 60 cols: Simplify track rows to single line (title + status only)
- < 40 cols: Show a "Terminal too small" message

---

## 11. Logging Strategy

Since the TUI owns the terminal (alternate screen, raw mode), we can't write logs to stderr. Options:

1. **File logging** (recommended) — `tracing-subscriber` with `tracing-appender` writing to `~/.conductor-dashboard/logs/`. Set via `CONDUCTOR_DASHBOARD_LOG` env var.

2. **In-app log viewer** (optional future) — A hidden panel (toggle with `L`) showing the last N log lines, rendered as a scrollable `Paragraph`.

```rust
// In main.rs
let log_dir = dirs::data_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("conductor-dashboard")
    .join("logs");
std::fs::create_dir_all(&log_dir)?;

let file_appender = tracing_appender::rolling::daily(&log_dir, "dashboard.log");
let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
tracing_subscriber::fmt()
    .with_writer(non_blocking)
    .with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("conductor_dashboard=info".parse()?)
    )
    .init();
```

---

## 12. Performance Targets (Updated)

| Metric | Iced Target | Ratatui Target | Rationale |
|--------|-------------|----------------|-----------|
| Cold start | < 500ms | < 300ms | No GPU init, no font loading |
| File change → UI | < 200ms | < 100ms | Simpler render pipeline |
| Render frame | < 16ms (60fps) | < 5ms | Terminal buffer diff is trivial |
| Memory | < 50MB | < 20MB | No texture atlas, no GPU buffers |
| Binary size | < 15MB | < 5MB | No wgpu, no system graphics deps |

---

## 13. Prompting Strategy for AI Coding Agents (Updated)

### For Claude Code

1. **Place BOTH plans in repo:**
   - `docs/REWRITE_PLAN.md` — The original Iced plan (for data model + parser reference)
   - `docs/RATATUI_REFACTOR.md` — This document (for architecture + rendering)

2. **Reference in CLAUDE.md:**
   ```markdown
   ## Conductor Dashboard Rewrite

   Read docs/RATATUI_REFACTOR.md for architecture decisions.
   Read docs/REWRITE_PLAN.md for data models, parsers, and enums (Phase 1 is shared).

   This is a Ratatui + Crossterm TUI, NOT an Iced GUI.
   Follow phases in order. Complete each fully before moving to next.
   Run `cargo check` and `cargo test` after each change.
   Use the real `./conductor/` directory for testing parsers.
   ```

3. **Phase-by-phase prompting:**
   ```
   Start Phase 1 from docs/REWRITE_PLAN.md (it's shared between both plans).
   Scaffold Cargo project with ratatui/crossterm deps from docs/RATATUI_REFACTOR.md
   Section 2. Define all enums, data models, implement three parsers with
   unit tests against real data in ./conductor/
   ```

   Then:
   ```
   Implement Phase 2 from docs/RATATUI_REFACTOR.md. Set up terminal with
   ratatui::init/restore, implement EventHandler with three tokio tasks,
   build the main event loop, render title bar + status bar + empty split layout.
   Should compile and show the Mako-themed shell in the terminal.
   ```

### For Claude.ai

Upload both documents:
```
I'm building a Conductor Dashboard TUI in Rust using Ratatui + Crossterm.
The attached RATATUI_REFACTOR.md has the full architecture. The original
REWRITE_PLAN.md has the data models and parsers (Phase 1 is shared).
Let's start with Phase 2 — I've already completed Phase 1.
Here's my current app.rs: [paste]
```

### Key tips

- Always reference RATATUI_REFACTOR.md for rendering decisions — the original plan's view code is Iced-specific and won't compile
- The parser code from Phase 1 is identical between plans — point the agent at the original plan for that
- If the agent starts using `iced::` imports, redirect: "This is a ratatui TUI, not an Iced GUI. See docs/RATATUI_REFACTOR.md Section 3"
- For colour drift, reference: "Use exact Mako RGB values from theme/colors.rs"
- The `event.rs` EventHandler pattern is the most complex new piece — make sure Phase 2 gets it right before moving on

---

## 14. Summary of Changes from Original Plan

| Component | Original (Iced) | Refactored (Ratatui) | Effort |
|-----------|-----------------|----------------------|--------|
| Framework | `iced 0.13` | `ratatui 0.29` + `crossterm 0.28` | Rewrite |
| Event loop | `Subscription::batch` | `tokio::select!` via `EventHandler` | Rewrite |
| Layout | `pane_grid` + `column!` | `Layout::vertical` + `Layout::horizontal` | Rewrite |
| Track list | Custom scrollable + widgets | `Table` (StatefulWidget) + `TableState` | Rewrite |
| Progress bar | Custom `Widget` impl | `fn progress_bar_text() -> Text` | Simplify |
| Status badge | Custom `Widget` impl | `fn status_span() -> Span` | Simplify |
| Detail panel | Custom view composition | `Paragraph` + `Scrollbar` | Rewrite |
| Help overlay | Custom modal | `Clear` + `Paragraph` in centred `Rect` | Rewrite |
| Theme | `iced::Theme::Custom` | `Color::Rgb` constants + `Style` builders | Simplify |
| Fonts | Montserrat + JetBrains Mono via `include_bytes!` | **Removed** — use terminal font + bold/dim | Remove |
| File watcher | Iced `Subscription::run` | `tokio::spawn` with `mpsc` channel | Adapt |
| Enums | — | **Unchanged** | None |
| Data models | — | **Unchanged** | None |
| Parsers | — | **Unchanged** | None |
| Error types | — | **Unchanged** | None |
| Cache | — | **Unchanged** | None |
| Tests | — | **Unchanged** | None |

**Estimated effort split:** ~50% of the code (model + parser + tests) is unchanged. ~50% (app shell + rendering + events) needs rewriting. The rewritten code is simpler (no GPU, no custom Widget traits, no Subscription abstraction).
