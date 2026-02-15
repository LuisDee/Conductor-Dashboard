# Conductor Dashboard — Rust/Iced Rewrite Plan

## 1. Motivation & Background

### What is Conductor?

Conductor is a Gemini CLI extension (released December 2025, Apache 2.0) that enables **context-driven development**. Instead of relying on ephemeral AI chat sessions, Conductor stores project knowledge, specifications, and implementation plans as versioned Markdown files directly in the repository under a `conductor/` directory. It organises work into **tracks** — units of work like features or bug fixes — each with a spec, a phased plan, and metadata. An AI coding agent then executes the plan step by step, checking off tasks as it goes.

The Conductor file structure looks like this:

```
conductor/
├── product.md                          # Product context (users, goals, features)
├── product-guidelines.md               # Brand/prose standards
├── tech-stack.md                       # Language, DB, framework preferences
├── workflow.md                         # Team preferences (TDD, commit strategy)
├── code_styleguides/                   # Code style conventions
├── tracks.md                           # Master registry of all tracks + statuses
└── tracks/
    └── <track_id>/                     # e.g. pad_compliance_system_20260125
        ├── index.md                    # Links to spec, plan, metadata
        ├── spec.md                     # Detailed requirements (what & why)
        ├── plan.md                     # Phased to-do list with checkboxes
        └── metadata.json              # Track ID, status, timestamps, deps
```

Key files the dashboard reads:

- **`tracks.md`** — The master index. Contains `## [x] Track: Title` headers with checkbox status (`[ ]` = new, `[~]` = in-progress, `[x]` = complete), links to track folders, priority indicators, and dependency lists.
- **`metadata.json`** — Per-track JSON with fields like `status`, `priority`, `type`, `phase`, `created_at`, `updated_at`, `dependencies`.
- **`plan.md`** — Per-track Markdown with phased task lists using `- [ ]` / `- [x]` / `- [~]` checkboxes. Phases are `##` or `###` headings. The agent checks these off as it works.

### Why This Dashboard Exists

When the AI agent (Gemini CLI, Claude Code, etc.) is running through a Conductor plan, it's modifying `plan.md` and `tracks.md` in real time — checking off tasks, updating statuses, moving between phases. Right now the only way to see progress is to `cat` the files or run `/conductor:status`. For a team running multiple tracks simultaneously, there's no way to get a birds-eye view of what's happening across the project.

The Conductor Dashboard solves this by providing a **live GUI** that watches the `conductor/` directory and reflects changes in real time. It's the mission control for your agentic coding workflow.

### Why a Rewrite?

There is an existing Python/Textual TUI prototype. It works but has accumulated problems:

1. **Fragile regex parsing** — The Python version uses hand-rolled regexes to parse `tracks.md`. If Conductor changes its markdown format even slightly (a space, an emoji, a different link style), the parser silently returns zero tracks. No errors, just empty state.
2. **Stringly-typed state** — Status, priority, filter mode, and sort mode are all strings compared with `==`. A typo anywhere is a silent bug.
3. **Dead code and cruft** — An unused `_markdown_cache` dict, a `parse_plan=False` path that never fires, a benchmark file with hardcoded paths left in the package.
4. **No error visibility** — `except Exception: pass` in the display update, `_parse_datetime` silently falling back to `now()`. When things break, you don't know.
5. **TUI limitations** — The Textual version requires Enter → detail screen → Esc → back for every track. Constant context-switching. No split-pane view.
6. **No dependency on Python environment** — The team shouldn't need to manage a Python virtualenv with `textual` and `watchdog` installed. A single binary is better.

### Why Rust + Iced?

- **Single static binary** — `cargo install` or download a release. No Python, no pip, no virtualenvs. Anyone on the team can use it immediately.
- **Iced follows The Elm Architecture** — State → Message → Update → View. This maps perfectly to a reactive dashboard that watches files and updates UI. The architecture makes impossible states impossible at the type level.
- **Native GPU-accelerated rendering** — Iced uses wgpu for rendering. No terminal emulator limitations, full font rendering with Montserrat, proper colour support, smooth scrolling.
- **Iced has `pane_grid`** — A built-in widget for split-pane layouts with resizable dividers. Perfect for the track-list + detail-panel layout.
- **Iced has `Subscription`** — A first-class primitive for listening to external event streams (file changes, time ticks). The file watcher maps directly to a Subscription that emits messages into the Elm update loop.
- **Type safety** — Enums for status, priority, filter mode. `serde` with `#[serde(default)]` for JSON parsing. `thiserror` for typed errors. The compiler catches the bugs the Python version hides.

---

## 2. Design Specification

### Brand: Mako Group Design System

The dashboard uses Mako Group's corporate colour palette and typography:

```
COLOURS:
  Navy     #0E1E3F   — Sidebars, primary text, headers, title bar, status bar
  Blue     #5471DF   — Accent for active states, buttons, in-progress indicators, selected row highlight
  Gold     #B28C54   — Warnings, blocked status, attention states, dependency callouts
  LightBlue #DBE1F5  — Secondary backgrounds, hover states, filter bar, column headers
  Success  #2C5F2D   — Complete status, checkmarks, healthy indicators
  Error    #B85042   — Error states, critical priority indicator
  
  Background      #F4F6FB   — Main app background
  Surface         #FFFFFF   — Card/panel backgrounds
  Border          #D1D9E8   — Divider lines, subtle borders
  Text Body       #2D3748   — Primary body text
  Text Muted      #6B7A99   — Secondary text, timestamps, labels

TYPOGRAPHY:
  Font Family: Montserrat (bundled with the binary, not system-dependent)
  Title:       ExtraBold 800, used for track titles in detail panel, app name
  Section:     SemiBold 600, used for phase headers, column headers, status badges
  Body:        Regular 400, used for task text, descriptions, timestamps
  Monospace:   JetBrains Mono (for track IDs and technical labels)
```

### Layout (Split-Pane)

```
┌─ Title Bar (Navy) ──────────────────────────────────────────────────────┐
│ [●●●]  ◇ Conductor Dashboard              12:34:56  [● WATCHING]      │
├─ Stats Bar (White) ─────────────────────────────────────────────────────┤
│ 6 Total  2 Active  1 Blocked  3 Complete   [All|Active|Blocked|Done]   │
│                                            [↕ Recent|↕ Progress]       │
├─ Track List (Left Pane) ──────┬─ Detail Panel (Right Pane) ────────────┤
│ Track    Status  Created Prog │ FEATURE · pad_compliance_20260125      │
│───────────────────────────────│                                        │
│▸ PAD Compliance  ⚙ Active    │ PAD Compliance System                  │
│  Phase 3 · 2m ago  Jan 20    │ ⚙ Active  CRITICAL  Created Jan 20    │
│  ████████████░░░░░░ 62%      │                                        │
│                               │ ┌─ Overall Progress ───────────────┐  │
│  Oracle→PG Migration ⚙ Act   │ │ 18/29  ████████████░░░░░░ 62%    │  │
│  Phase 2 · 18m ago  Jan 15   │ └───────────────────────────────────┘  │
│  ██████████░░░░░░░░ 41%      │                                        │
│                               │ IMPLEMENTATION PLAN                    │
│  GCS Cost Dashboard ⚠ Block  │                                        │
│  Phase 1 · 3h ago   Feb 01   │ ● Phase 1: Document Parsing    5/5    │
│  ███░░░░░░░░░░░░░░░ 15%      │   ✓ Build PDF reader for trade confs  │
│                               │   ✓ Extract trade fields               │
│  Vol Box Delivery   ✓ Done   │   ✓ Validate parsed data              │
│  Complete · 2d ago  Jan 05   │   ✓ Handle multi-page documents       │
│  ██████████████████ 100%     │   ✓ Write unit tests for parser       │
│                               │                                        │
│  Airflow 3 Migration ✓ Done  │ ● Phase 2: Database & API      6/6    │
│  Complete · 5d ago  Jan 10   │   ✓ Design PostgreSQL schema          │
│  ██████████████████ 100%     │   ✓ Migrate Oracle tables             │
│                               │   ...                                  │
│  NSE Data Integration ✓ Done │                                        │
│  Complete · 1w ago  Jan 15   │ ◐ Phase 3: Slack Integration   7/10   │
│  ██████████████████ 100%     │   ✓ Create Slack app                  │
│                               │   ✓ Build webhook handler             │
├─ Status Bar (Navy) ──────────┴────────────────────────────────────────┤
│ [↑↓] Navigate  [Enter] Expand  [F] Filter  [S] Sort  [/] Search     │
│ [R] Refresh  [?] Help                     conductor-dashboard v0.1.0  │
└───────────────────────────────────────────────────────────────────────┘
```

### Column Layout (Left Pane — Track List)

The track list table has these columns (priority column removed, created date added):

| Column | Content | Width |
|--------|---------|-------|
| Track | Title (bold) + phase/updated subtitle | Flex (fills remaining) |
| Status | Badge with coloured dot: `⚙ Active`, `⚠ Blocked`, `✓ Complete`, `○ New` | ~110px |
| Created | Date string, e.g. "Jan 20" | ~80px |
| Progress | Visual progress bar + percentage | ~180px |
| Tasks | `18/29` completed/total | ~70px |

The selected row has a `Blue (#5471DF)` left border accent and a subtle blue background tint.

### Detail Panel (Right Pane)

When a track is selected, the right pane shows:

1. **Header** — Track type label (uppercased), track ID in monospace, title in ExtraBold, status badge + created date
2. **Progress card** — LightBlue background card with overall task count and progress bar
3. **Dependency warning** (if any) — Gold-tinted card showing blocking tracks
4. **Implementation Plan** — Full phase breakdown with:
   - Phase header with coloured dot (green=complete, blue=active, grey=pending, gold=blocked)
   - Task list with checkboxes (green filled for done, empty border for pending)
   - Strikethrough + muted colour for completed tasks

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate track list |
| `Enter` | Toggle full-screen detail view (when in split view) |
| `Esc` | Return to split view (when in full-screen detail) |
| `F` | Cycle filter: All → Active → Blocked → Complete |
| `S` | Cycle sort: Recent → Progress |
| `R` | Force refresh |
| `/` | Open search/filter input (fuzzy match on track titles) |
| `?` | Toggle help overlay |
| `Q` | Quit |

---

## 3. Architecture

### The Elm Architecture (Iced's Core Pattern)

Iced follows The Elm Architecture: **State → Message → Update → View**. Every user interaction and external event produces a `Message`. The `update` function processes messages and mutates state. The `view` function renders UI from current state. This is declarative and pure — the view is always a function of the state.

### Application State

```rust
struct ConductorDashboard {
    // Core data
    tracks: BTreeMap<TrackId, Track>,
    conductor_dir: PathBuf,
    
    // UI state
    selected_track: Option<TrackId>,
    filter: FilterMode,
    sort: SortMode,
    search_query: String,
    search_active: bool,
    detail_fullscreen: bool,
    show_help: bool,
    
    // Pane state
    panes: pane_grid::State<PaneKind>,
    track_list_pane: pane_grid::Pane,
    detail_pane: pane_grid::Pane,
    
    // Scroll state
    track_list_scroll: scrollable::Id,
    detail_scroll: scrollable::Id,
    
    // Status
    watcher_active: bool,
    last_refresh: Instant,
    error_message: Option<(String, Instant)>,  // auto-dismiss after timeout
    
    // Cache
    track_cache: TrackCache,
}
```

### Message Enum

```rust
#[derive(Debug, Clone)]
enum Message {
    // File watcher events
    FilesChanged(Vec<PathBuf>),
    TracksLoaded(Result<BTreeMap<TrackId, Track>, DashboardError>),
    
    // User interaction
    TrackSelected(TrackId),
    KeyPressed(keyboard::Key),
    CycleFilter,
    CycleSort,
    ForceRefresh,
    ToggleSearch,
    SearchQueryChanged(String),
    ToggleHelp,
    ToggleFullscreen,
    
    // Pane management
    PaneResized(pane_grid::ResizeEvent),
    
    // System
    Tick(Instant),          // periodic UI refresh (clock, auto-dismiss errors)
    ErrorDismissed,
}
```

### Type-Safe Enums (No More Strings)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    New,
    InProgress,
    Blocked,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Priority {
    Critical = 0,
    High = 1,
    Medium = 2,
    Low = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FilterMode {
    All,
    Active,      // in_progress or has progress > 0
    Blocked,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SortMode {
    Updated,     // most recently updated first
    Progress,    // highest progress first
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrackType {
    Feature,
    Bug,
    Migration,
    Other(/* store as String in Track */),
}
```

### Track Data Model

```rust
#[derive(Debug, Clone)]
struct Track {
    id: TrackId,                    // newtype wrapper around String
    title: String,
    status: Status,
    priority: Priority,
    track_type: TrackType,
    phase: String,                  // current phase name
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    dependencies: Vec<TrackId>,
    tasks_total: usize,
    tasks_completed: usize,
    checkbox_status: CheckboxStatus, // from tracks.md: Unchecked, InProgress, Checked
    plan_phases: Vec<PlanPhase>,    // parsed plan structure for detail view
}

#[derive(Debug, Clone)]
struct PlanPhase {
    name: String,
    status: PhaseStatus,            // Pending, Active, Complete, Blocked
    tasks: Vec<PlanTask>,
}

#[derive(Debug, Clone)]
struct PlanTask {
    text: String,
    done: bool,
}

impl Track {
    fn progress_percent(&self) -> f32 {
        if self.tasks_total == 0 { return 0.0; }
        (self.tasks_completed as f32 / self.tasks_total as f32) * 100.0
    }
}
```

### Subscription: File Watcher

Iced's `Subscription` is used to listen to file system events. The `subscription()` function returns an active subscription that wraps the `notify` crate's debounced watcher:

```rust
fn subscription(&self) -> Subscription<Message> {
    Subscription::batch([
        // File watcher subscription
        file_watcher::watch(self.conductor_dir.clone())
            .map(Message::FilesChanged),
        
        // Clock tick every second (for timestamps, error dismissal)
        iced::time::every(Duration::from_secs(1))
            .map(Message::Tick),
        
        // Keyboard events
        keyboard::on_key_press(|key, modifiers| {
            // Map keys to messages
            Some(Message::KeyPressed(key))
        }),
    ])
}
```

The file watcher subscription is its own module:

```rust
// src/watcher.rs
pub fn watch(conductor_dir: PathBuf) -> Subscription<Vec<PathBuf>> {
    Subscription::run(move || {
        stream::channel(100, move |mut output| async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            
            let mut debouncer = notify_debouncer_mini::new_debouncer(
                Duration::from_millis(300),
                move |result: DebounceEventResult| {
                    if let Ok(events) = result {
                        let paths: Vec<_> = events.iter()
                            .filter(|e| is_conductor_file(&e.path))
                            .map(|e| e.path.clone())
                            .collect();
                        if !paths.is_empty() {
                            let _ = tx.blocking_send(paths);
                        }
                    }
                },
            ).expect("Failed to create file watcher");
            
            debouncer.watcher()
                .watch(&conductor_dir, RecursiveMode::Recursive)
                .expect("Failed to watch conductor directory");
            
            // Keep debouncer alive and forward events
            loop {
                if let Some(paths) = rx.recv().await {
                    let _ = output.send(paths).await;
                }
            }
        })
    })
}

fn is_conductor_file(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|f| f.to_str()),
        Some("tracks.md" | "metadata.json" | "plan.md" | "spec.md")
    )
}
```

### Update Function (Message Processing)

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::FilesChanged(paths) => {
            // Determine what to reload
            let reload_scope = self.track_cache.classify_changes(&paths);
            
            // Spawn async parsing task
            let dir = self.conductor_dir.clone();
            Task::perform(
                async move { parse_tracks(dir, reload_scope).await },
                Message::TracksLoaded,
            )
        }
        
        Message::TracksLoaded(Ok(new_tracks)) => {
            self.tracks = new_tracks;
            self.last_refresh = Instant::now();
            self.watcher_active = true;
            Task::none()
        }
        
        Message::TracksLoaded(Err(e)) => {
            self.error_message = Some((format!("{}", e), Instant::now()));
            Task::none()
        }
        
        Message::TrackSelected(id) => {
            self.selected_track = Some(id);
            // Reset detail scroll to top
            scrollable::snap_to(
                self.detail_scroll.clone(),
                scrollable::RelativeOffset::START,
            )
        }
        
        Message::CycleFilter => {
            self.filter = self.filter.next();
            Task::none()
        }
        
        Message::CycleSort => {
            self.sort = self.sort.next();
            Task::none()
        }
        
        // ... other message handlers
    }
}
```

### View Function (Rendering)

The view composes the layout using Iced's built-in widgets:

```rust
fn view(&self) -> Element<Message> {
    let title_bar = self.view_title_bar();
    let stats_bar = self.view_stats_bar();
    let main_content = self.view_main_panes();  // pane_grid with track list + detail
    let status_bar = self.view_status_bar();
    
    column![title_bar, stats_bar, main_content, status_bar]
        .width(Fill)
        .height(Fill)
        .into()
}
```

The pane_grid provides the split-pane layout:

```rust
fn view_main_panes(&self) -> Element<Message> {
    pane_grid(&self.panes, |pane, kind, _is_maximized| {
        match kind {
            PaneKind::TrackList => self.view_track_list(),
            PaneKind::Detail => self.view_detail_panel(),
        }
    })
    .on_resize(10, Message::PaneResized)
    .into()
}
```

---

## 4. Module Structure

```
conductor-dashboard/
├── Cargo.toml
├── assets/
│   └── fonts/
│       ├── Montserrat-Regular.ttf
│       ├── Montserrat-SemiBold.ttf
│       ├── Montserrat-ExtraBold.ttf
│       └── JetBrainsMono-Regular.ttf
├── src/
│   ├── main.rs                 # Entry point, CLI arg parsing, app launch
│   ├── app.rs                  # ConductorDashboard struct, update(), view(), subscription()
│   ├── message.rs              # Message enum definition
│   ├── model/
│   │   ├── mod.rs
│   │   ├── track.rs            # Track, PlanPhase, PlanTask structs
│   │   ├── state.rs            # DashboardState (filter, sort, selection logic)
│   │   ├── enums.rs            # Status, Priority, FilterMode, SortMode, TrackType
│   │   └── cache.rs            # TrackCache for incremental updates (mtime tracking, hashing)
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── index.rs            # Parse tracks.md using pulldown-cmark (markdown AST)
│   │   ├── metadata.rs         # Parse metadata.json with serde (all fields default)
│   │   ├── plan.rs             # Parse plan.md — extract phases, tasks, checkbox counts
│   │   └── error.rs            # ParseError enum (typed, not stringly)
│   ├── watcher/
│   │   ├── mod.rs
│   │   └── subscription.rs     # Iced Subscription wrapping notify-debouncer-mini
│   ├── theme/
│   │   ├── mod.rs
│   │   ├── colors.rs           # Mako palette as iced::Color constants
│   │   ├── mako_theme.rs       # Custom Theme impl with all widget styles
│   │   └── fonts.rs            # Font loading and family definitions
│   ├── views/
│   │   ├── mod.rs
│   │   ├── title_bar.rs        # Navy top bar with app name, clock, watcher status
│   │   ├── stats_bar.rs        # Summary counts + filter/sort chip controls
│   │   ├── track_list.rs       # Left pane: scrollable table of tracks
│   │   ├── track_row.rs        # Individual track row widget (title, status, date, progress)
│   │   ├── detail_panel.rs     # Right pane: track header, progress card, plan
│   │   ├── plan_view.rs        # Phase blocks with task checkboxes
│   │   ├── status_bar.rs       # Navy bottom bar with keyboard shortcuts
│   │   ├── help_overlay.rs     # Modal help screen
│   │   └── search_bar.rs       # Fuzzy search input overlay
│   └── widgets/
│       ├── mod.rs
│       ├── progress_bar.rs     # Custom styled progress bar (gradient fill, shimmer)
│       ├── status_badge.rs     # Coloured status indicator (dot + label)
│       └── filter_chip.rs      # Toggle button group for filter/sort selection
└── tests/
    ├── parser_tests.rs         # Unit tests for markdown/json parsing
    ├── model_tests.rs          # State filter/sort logic tests
    └── snapshot_tests.rs       # (future) Visual regression tests
```

---

## 5. Implementation Phases

### Phase 1: Foundation (Core Data + Parsing)

**Goal:** Parse all Conductor files into strongly-typed Rust structs. No UI yet — just data.

**Tasks:**

1. **Project scaffold** — `cargo init`, add dependencies to `Cargo.toml`:
   ```toml
   [dependencies]
   iced = { version = "0.13", features = ["tokio", "advanced"] }
   tokio = { version = "1", features = ["full"] }
   notify = "7"
   notify-debouncer-mini = "0.5"
   pulldown-cmark = "0.12"
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   chrono = { version = "0.4", features = ["serde"] }
   clap = { version = "4", features = ["derive"] }
   thiserror = "2"
   color-eyre = "0.6"
   tracing = "0.1"
   tracing-subscriber = "0.3"
   ```
2. **Define all enums** in `model/enums.rs` — `Status`, `Priority`, `FilterMode`, `SortMode`, `TrackType`, `CheckboxStatus`, `PhaseStatus`. All with `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. Priority with `PartialOrd, Ord`. Status and Priority with serde `Deserialize`.
3. **Define data structs** in `model/track.rs` — `TrackId` (newtype), `Track`, `PlanPhase`, `PlanTask`. Implement `progress_percent()`, `status_display()`, etc.
4. **Implement index parser** (`parser/index.rs`) — Use `pulldown-cmark` to parse `tracks.md`. Walk the AST to extract:
   - H2 headings → track entries
   - Checkbox status from task list items
   - Links → track IDs
   - Bold text → priority hints
   - Inline text → dependency lists
   
   This is the **biggest robustness improvement** over the Python version. Instead of brittle regexes, the markdown AST handles whitespace/formatting variations gracefully.
5. **Implement metadata parser** (`parser/metadata.rs`) — Deserialize `metadata.json` with serde. All fields `#[serde(default)]` so missing fields degrade gracefully:
   ```rust
   #[derive(Deserialize, Default)]
   struct TrackMetadata {
       #[serde(default)]
       status: Status,
       #[serde(default)]
       priority: Priority,
       #[serde(default)]
       phase: String,
       #[serde(default)]
       dependencies: Vec<String>,
       #[serde(default)]
       created_at: Option<String>,
       #[serde(default)]
       updated_at: Option<String>,
   }
   ```
6. **Implement plan parser** (`parser/plan.rs`) — Use `pulldown-cmark` to parse `plan.md`. Extract phases (from headings), tasks (from checkbox list items), and build `Vec<PlanPhase>` structure. Count total/completed tasks as a side effect. Read line-by-line for the counting path (fast), full AST for the detail view path (rich).
7. **Implement TrackCache** (`model/cache.rs`) — Track file mtimes per-track. On change, diff against cached state and determine minimal reload scope:
   ```rust
   enum ReloadScope {
       Full,                           // tracks.md changed
       Tracks(Vec<TrackId>),           // specific plan.md or metadata.json changed
   }
   ```
8. **Write parser unit tests** — Test against known markdown strings. Include edge cases: empty tracks.md, missing metadata.json, malformed JSON, plan.md with no checkboxes, unicode in titles, varying whitespace.

### Phase 2: Iced Shell + Theme

**Goal:** Render a window with the Mako theme, title bar, status bar, and empty pane layout.

**Tasks:**

1. **Set up Iced application** in `app.rs` — Implement the core `ConductorDashboard` struct with `new()`, `update()`, `view()`, `subscription()`, `theme()`.
2. **Define the Message enum** in `message.rs`.
3. **Bundle fonts** — Include Montserrat (Regular, SemiBold, ExtraBold) and JetBrains Mono as embedded bytes using `include_bytes!`. Load via `iced::font::load()` in the app's initial `Task`.
4. **Implement Mako theme** (`theme/`) — Create a custom `Theme` with the Mako colour palette. Style all widgets: containers, buttons, scrollables, progress bars, text, rules. Use Iced's `theme::Custom` or implement `widget::container::Style`, etc.
5. **Build title bar** (`views/title_bar.rs`) — Navy background, window-style layout with app name "Conductor Dashboard", live clock (updated by `Tick` messages), "WATCHING" indicator with green dot.
6. **Build status bar** (`views/status_bar.rs`) — Navy background, keyboard shortcut hints rendered as styled key badges.
7. **Build pane_grid shell** — Initialise `pane_grid::State` with two panes (TrackList + Detail) at a 50/50 split. Render empty placeholder content in each. Wire up `PaneResized` message.
8. **CLI argument parsing** (`main.rs`) — Use `clap` derive to parse:
   - `--conductor-dir <PATH>` (default: `./conductor`)
   - `--no-watch` (disable file watcher)
   - `--filter <all|active|blocked|complete>` (initial filter)
   
   Validate that the directory and `tracks.md` exist before launching.

### Phase 3: Track List (Left Pane)

**Goal:** Populate the left pane with a scrollable, interactive track list.

**Tasks:**

1. **Implement stats bar** (`views/stats_bar.rs`) — Show total/active/blocked/complete counts. Render filter toggle chips (All | Active | Blocked | Complete) and sort chips (Recent | Progress). Wire to `CycleFilter` and `CycleSort` messages.
2. **Implement track row** (`views/track_row.rs`) — A row widget showing:
   - Title (Montserrat SemiBold) + subtitle line (phase + "· 2 min ago")
   - Status badge with coloured dot
   - Created date (e.g. "Jan 20")
   - Progress bar (custom widget)
   - Task count ("18/29")
   
   Selected row: Blue left border + subtle blue background. Hover: LightBlue background.
3. **Implement custom progress bar** (`widgets/progress_bar.rs`) — Gradient fill based on status (blue for active, green for complete, gold for blocked). Percentage label right-aligned.
4. **Implement status badge** (`widgets/status_badge.rs`) — Coloured dot + uppercase label. Dot pulses for in-progress tracks.
5. **Build track list view** (`views/track_list.rs`) — Column header row + scrollable list of `TrackRow` widgets. Apply current filter and sort from state. Each row emits `Message::TrackSelected(id)` on click.
6. **Wire up keyboard navigation** — `j`/`k` or `↑`/`↓` move selection, `Home`/`End` for first/last. Auto-scroll to keep selected track visible.

### Phase 4: Detail Panel (Right Pane)

**Goal:** Show full track details with the implementation plan when a track is selected.

**Tasks:**

1. **Implement detail header** — Track type label (uppercased, monospace), track ID, title in ExtraBold, status + created date badges.
2. **Implement progress summary card** — LightBlue background container with large task count and full-width progress bar.
3. **Implement dependency warning** — Gold-tinted container showing blocking tracks with their progress. Only shown when `track.dependencies` is non-empty.
4. **Implement plan view** (`views/plan_view.rs`) — Render `track.plan_phases` as:
   - Phase header with coloured status dot + name + task count
   - Task list with checkbox indicators (green filled SVG/icon for done, empty bordered square for pending)
   - Completed tasks: muted colour + strikethrough
   - Active phase: Blue dot with subtle glow
5. **Empty state** — When no track is selected, show centered placeholder: "Select a track to view details".
6. **Scroll behaviour** — Detail panel is scrollable. On track selection change, scroll resets to top.

### Phase 5: File Watcher + Live Updates

**Goal:** Watch the conductor directory and update the dashboard in real time.

**Tasks:**

1. **Implement file watcher subscription** (`watcher/subscription.rs`) — Wrap `notify-debouncer-mini` in an Iced `Subscription::run`. Emit `Vec<PathBuf>` of changed conductor files.
2. **Wire into update loop** — `Message::FilesChanged` → classify changes → spawn `Task::perform` for async parsing → `Message::TracksLoaded` updates state.
3. **Implement incremental refresh** — If only `plan.md` or `metadata.json` for specific tracks changed, only reload those tracks. If `tracks.md` changed, do a full index re-parse but still try to preserve existing plan data for unchanged tracks.
4. **Update the "WATCHING" indicator** — Green when watcher is active, red if watcher errors, grey if `--no-watch`.
5. **Test with a running Conductor agent** — Start the dashboard, run `/conductor:implement` in another terminal, verify the progress bars update live as the agent checks off tasks.

### Phase 6: Search, Help, and Polish

**Goal:** Add search overlay, help screen, error display, and UI polish.

**Tasks:**

1. **Fuzzy search** (`views/search_bar.rs`) — `/` opens a text input overlay. Filter track list by fuzzy-matching on title. `Esc` closes search. Use a simple substring/contains match initially (upgrade to fuzzy scoring later if needed).
2. **Help overlay** (`views/help_overlay.rs`) — `?` toggles a semi-transparent modal showing all keyboard shortcuts in a formatted table.
3. **Error display** — Parse errors and watcher errors show as a gold warning bar below the stats bar. Auto-dismiss after 10 seconds. Example: "⚠ Failed to parse metadata for track xyz — using defaults".
4. **Fullscreen detail toggle** — `Enter` on a selected track maximises the detail pane. `Esc` returns to split view.
5. **Animation polish** — Pulse animation on in-progress status dots. Smooth progress bar transitions on value change. Fade-in for newly appeared tracks.
6. **Config file support** (optional) — Read `.conductor-dashboard.toml` from the project root for persistent preferences:
   ```toml
   [display]
   default_filter = "active"
   default_sort = "progress"
   split_ratio = 0.5
   
   [watcher]
   debounce_ms = 300
   ```

### Phase 7: Testing and Distribution

**Goal:** Comprehensive tests and easy distribution.

**Tasks:**

1. **Parser unit tests** — Known markdown inputs → expected `Track` structs. Edge cases: empty files, missing fields, malformed content, unicode, varying Conductor format versions.
2. **Model tests** — Filter/sort logic: create tracks with various statuses/priorities/progress, apply filter combinations, assert correct ordering and inclusion.
3. **Integration tests** — Write a test that creates a temporary `conductor/` directory structure, launches the parser, and verifies all tracks are loaded correctly.
4. **Property-based tests** (optional) — Use `proptest` to fuzz the parsers with random markdown/JSON content. Verify no panics, only typed errors.
5. **Release build** — `cargo build --release`. Strip debug symbols. Verify binary size (target: < 15MB).
6. **CI pipeline** — GitHub Actions: `cargo fmt --check`, `cargo clippy`, `cargo test`, `cargo build --release` on Linux/macOS/Windows.
7. **Distribution** — Ship as downloadable binaries from GitHub Releases. Consider `cargo install conductor-dashboard` for Rust users.

---

## 6. Crate Dependencies

| Crate | Purpose | Why |
|-------|---------|-----|
| `iced` (with `tokio`, `advanced` features) | GUI framework | Elm architecture, pane_grid, scrollable, custom themes, Subscription for file watching |
| `tokio` | Async runtime | Required by Iced, powers async file parsing and watcher |
| `notify` + `notify-debouncer-mini` | File system watching | Battle-tested (used by alacritty, rust-analyzer, zed), built-in debouncing |
| `pulldown-cmark` | Markdown parsing | AST-based parsing of tracks.md and plan.md — far more robust than regex |
| `serde` + `serde_json` | JSON deserialization | Parse metadata.json with `#[serde(default)]` for graceful degradation |
| `chrono` | DateTime handling | Parse ISO timestamps from metadata, display relative times ("2 min ago") |
| `clap` (derive) | CLI argument parsing | `--conductor-dir`, `--no-watch`, `--filter` |
| `thiserror` | Error types | Typed error enums for parser, watcher, and app errors |
| `color-eyre` | Error reporting | Rich error context in development, clean panics in production |
| `tracing` + `tracing-subscriber` | Logging | Structured logging for debugging parser/watcher issues |

---

## 7. Key Design Decisions and Rationale

### Decision 1: Iced over TUI/Ratatui

The original was a terminal TUI. We're moving to a native GUI because:
- **Montserrat rendering** — Terminal can't render custom fonts. The Mako brand identity requires Montserrat.
- **Colour fidelity** — Terminal colour support varies by emulator. Iced renders exact hex colours via GPU.
- **Split-pane layout** — Iced's `pane_grid` gives us a resizable split with a proper drag handle. Terminal split-panes are possible but clunkier.
- **Future extensibility** — If we want to add dependency graph visualisation (Iced `Canvas`), image rendering, or clickable URLs, a native GUI handles these naturally.

### Decision 2: pulldown-cmark over Regex

The Python version had 4 compiled regexes for parsing tracks.md, each matching a very specific format. If Conductor updates its markdown style (which it will — it's in preview), the regexes break silently.

`pulldown-cmark` parses into an AST of events (heading start, text, checkbox, link, etc.). We walk the AST looking for structural patterns (H2 → track entry, checkbox → task) rather than character patterns. This is inherently more resilient to formatting changes.

### Decision 3: No Priority Column, Added Created Date

Priority is useful metadata but not useful enough for a column. It's available in the detail panel. The created date answers the question "how old is this track?" at a glance — much more useful for daily standup context than a priority badge you already know.

### Decision 4: Subscription-Based File Watching (Not Polling)

Iced's `Subscription` is the idiomatic way to handle external event streams. The file watcher emits events that flow through the same message pipeline as user input. No threads, no mutexes, no callback spaghetti. The Elm architecture keeps everything sequential and predictable.

### Decision 5: Parse Plan into Structured Phases (Not Just Checkbox Counts)

The Python version only counted checkboxes in plan.md — it didn't extract the phase structure. This meant the detail view had to re-read and render the raw markdown. By parsing plan.md into `Vec<PlanPhase>` with `Vec<PlanTask>`, we get:
- Phase-aware progress (which phase is active?)
- Structured rendering (no markdown-to-widget conversion at render time)
- Future features like phase-level filtering or collapse/expand

### Decision 6: Bundled Fonts

Montserrat and JetBrains Mono are embedded in the binary via `include_bytes!`. No system font dependency, no "font not found" issues across machines. The binary is self-contained.

---

## 8. Error Handling Strategy

Errors should be **visible, not silent**. The Python version swallowed errors everywhere. The Rust version:

1. **Parser errors** → Typed via `thiserror`:
   ```rust
   #[derive(thiserror::Error, Debug)]
   enum ParseError {
       #[error("tracks.md not found at {0}")]
       IndexNotFound(PathBuf),
       #[error("Invalid metadata.json for track {track_id}: {source}")]
       MetadataInvalid { track_id: String, source: serde_json::Error },
       #[error("Failed to read {path}: {source}")]
       IoError { path: PathBuf, source: std::io::Error },
   }
   ```
2. **Partial failures are okay** — If one track's metadata.json is invalid, log the error, use defaults for that track, and continue loading the rest. Show a warning in the UI: "⚠ 1 track has parse errors".
3. **Watcher errors** — If the file watcher fails, the dashboard still works in static mode. Show "WATCHER ERROR" in the title bar instead of "WATCHING".
4. **Display errors transiently** — Error messages appear as a gold bar below the stats bar and auto-dismiss after 10 seconds. They don't block interaction.

---

## 9. Performance Targets

| Metric | Target | Strategy |
|--------|--------|----------|
| Cold start | < 500ms for 30 tracks | Parse index first, lazy-load plans |
| File change → UI update | < 200ms | Incremental refresh, debounced watcher |
| Render frame | < 16ms (60fps) | Iced's immediate-mode + GPU rendering |
| Memory | < 50MB RSS | No full markdown strings kept; parsed into structs |
| Binary size | < 15MB | Release build with LTO, strip symbols |

---

## 10. Future Enhancements (Post-MVP)

These are not in scope for the initial rewrite but should be considered in the architecture:

1. **Dependency graph view** — Use Iced's `Canvas` widget to render a DAG of track dependencies. Blocked tracks are visually connected to their blockers.
2. **Desktop notifications** — When a track hits 100%, fire a system notification. Useful when the agent is running in another terminal.
3. **Multi-project support** — Tab bar or project switcher for monitoring multiple repos' conductor directories.
4. **Spec view** — Toggle between plan and spec in the detail panel.
5. **Terminal bell on completion** — Optional audible alert.
6. **Theme switching** — Light/dark mode toggle (the Mako palette works well in both).
