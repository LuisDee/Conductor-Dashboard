## Conductor Dashboard — Rust/Ratatui TUI

### Project Context
Building a terminal dashboard that monitors Conductor track progress in real-time.
Ratatui + Crossterm TUI, NOT a native GUI. Runs inside the terminal.

### Reference Documents (read before ANY implementation)
- `docs/REWRITE_PLAN.md` — Data models, enums, parsers, error types (Phase 1 source of truth)
- `docs/RATATUI_REFACTOR.md` — Architecture, event loop, rendering, layout (Phases 2–7 source of truth)

If these two docs conflict, RATATUI_REFACTOR.md wins — it's the newer plan.

### Rules
- Follow phases in order. Complete each phase fully before starting the next.
- Run `cargo check` and `cargo test` after every file change.
- Use `./conductor/` directory as real test data for parsers — don't invent synthetic fixtures.
- All colours must use exact Mako RGB values from `theme/colors.rs`.
- Never use `iced::` imports — this is a ratatui project.
- Log to file (not stderr) — we own the terminal.
- Enums over strings. `#[serde(default)]` on all deserialized fields.
- No `unwrap()` in production code paths — use `color_eyre` or `thiserror`.

### Tech Stack
- Rust (edition 2021)
- ratatui 0.29 + crossterm 0.28 (TUI)
- tokio (async runtime)
- notify + notify-debouncer-mini (file watching)
- pulldown-cmark (markdown parsing)
- serde + serde_json (JSON)
- clap (CLI), thiserror (errors), tracing (logging)
```

---

## Phase-by-Phase Prompts

Paste these one at a time as you complete each phase:

### Phase 1 — Foundation
```
Read docs/REWRITE_PLAN.md and docs/RATATUI_REFACTOR.md fully before starting.

Implement Phase 1: Foundation (Core Data + Parsing).

1. Scaffold the Cargo project with dependencies from RATATUI_REFACTOR.md Section 2
2. Define all enums in src/model/enums.rs — Status, Priority, FilterMode, SortMode, TrackType, CheckboxStatus, PhaseStatus. All with proper derives, serde rename_all snake_case on Status and Priority
3. Define data structs in src/model/track.rs — TrackId newtype, Track, PlanPhase, PlanTask with progress_percent() impl
4. Implement index parser (src/parser/index.rs) — parse tracks.md using pulldown-cmark AST, NOT regex. Extract H2 headings, checkbox status, links to track IDs, priority hints, dependencies
5. Implement metadata parser (src/parser/metadata.rs) — serde deserialize with #[serde(default)] on every field
6. Implement plan parser (src/parser/plan.rs) — pulldown-cmark AST to extract phases from headings and tasks from checkbox list items into Vec<PlanPhase>
7. Implement TrackCache (src/model/cache.rs) — track file mtimes, ReloadScope enum (Full vs Tracks(Vec<TrackId>))
8. Write unit tests for all three parsers against real data in ./conductor/

Test against the actual conductor/ directory in this repo. Run cargo test after each module.
```

### Phase 2 — TUI Shell
```
Read docs/RATATUI_REFACTOR.md Sections 3 and 4 before starting.

Implement Phase 2: TUI Shell + Theme.

1. Set up main.rs — tokio::main, ratatui::init/restore with panic hook to always restore terminal
2. Implement EventHandler in src/event.rs — three tokio::spawn tasks (crossterm EventStream for keys, 1-second tick timer, notify file watcher) all feeding mpsc::UnboundedChannel<Event>. Follow the exact pattern from Section 3.1
3. Implement the main event loop in src/app.rs — terminal.draw() → events.next().await → handle_event(). Follow Section 3.2
4. Implement Mako colour constants in src/theme/colors.rs as Color::Rgb values
5. Build title bar — navy background, "◇ Conductor Dashboard", clock placeholder, "● WATCHING" indicator
6. Build status bar — navy background, keyboard shortcut hints
7. Build layout shell — Layout::vertical for title/stats/main/status, Layout::horizontal for left/right pane placeholders with Block::bordered()
8. CLI args with clap derive — --conductor-dir, --no-watch, --filter
9. Set up file logging with tracing-appender (NOT stderr)

The app should compile, launch in the terminal, show the Mako-themed shell with empty panes, and quit on 'q'. The EventHandler should be receiving tick and key events.
```

### Phase 3 — Track List
```
Read docs/RATATUI_REFACTOR.md Section 6.1–6.3 before starting.

Implement Phase 3: Track List (Left Pane).

1. Stats bar (ui/stats_bar.rs) — two lines: track counts + filter/sort indicators
2. Track table (ui/track_list.rs) — ratatui Table with TableState. Row height 2 (title + "Ph.N · date" subtitle). Columns: Track (Fill), Status (Length 5), Progress (Length 12), Tasks (Length 6)
3. Progress bar function (ui/components.rs) — Unicode block chars ████░░ with colour by status
4. Status badge function (ui/components.rs) — coloured abbreviated spans (⚙ ACT, ⚠ BLK, ✓ DON, ○ NEW)
5. Filter/sort logic — recompute_filtered_tracks() applying FilterMode + SortMode + search query
6. Keyboard navigation — j/k/↑/↓ move selection via TableState, Home/End for first/last

Wire up real track data from the parsers. The left pane should show actual tracks from ./conductor/ with working selection, filtering (f key), and sorting (s key).
```

### Phase 4 — Detail Panel
```
Read docs/RATATUI_REFACTOR.md Section 6.4 before starting.

Implement Phase 4: Detail Panel (Right Pane).

1. Detail header — track type label, track ID, title (bold), status + created date
2. Progress summary — full-width Unicode progress bar using inner area width
3. Dependency warning — gold line listing blocking track IDs (only if deps non-empty)
4. Plan view (ui/plan_view.rs) — phase headers with ●/◐/○ status dots + task count, task items with ✓/○ icons, dim styling for completed tasks
5. Scrollable rendering — all content as Vec<Line> in Paragraph with .scroll(), d/u keys scroll, Scrollbar widget alongside
6. Empty state — "Select a track to view details" when nothing selected
7. Reset detail_scroll to 0 on selection change

Selecting a track in the left pane should immediately show its full detail in the right pane with the implementation plan phases and tasks.
```

### Phase 5 — Live Updates
```
Implement Phase 5: File Watcher + Live Updates.

The EventHandler already spawns the file watcher from Phase 2. Now wire it into the update cycle:

1. Handle Event::FilesChanged in app.rs — classify changes via TrackCache, determine ReloadScope
2. Spawn async parse task for changed tracks, send Event::TracksLoaded back through the channel
3. Incremental refresh — only reload changed tracks if possible, full re-parse if tracks.md changed
4. Update watcher_active flag — green indicator in title bar when active, red on error, grey if --no-watch

Test by editing a plan.md file in ./conductor/ while the dashboard is running — the progress should update live within 500ms.
```

### Phase 6 — Polish
```
Implement Phase 6: Search, Help, and Polish.

1. Search — / opens search input (InputMode::Search), all keystrokes filter track list by substring on title, Enter confirms, Esc cancels. Render as highlighted line at top of main area
2. Help overlay — ? toggles centred popup with keyboard shortcuts table using Clear + bordered Paragraph
3. Error display — gold warning line between stats bar and track list for parse/watcher errors, auto-dismiss after 10 seconds via Tick handler
4. Fullscreen detail — Enter maximises detail panel to full main_area, Esc returns to split
5. Split resize — [ and ] adjust split_percent by 5%, clamped 20-80%
6. Terminal size handling — graceful degradation below 80 cols (hide detail, simplify rows)

All features from Section 7 of RATATUI_REFACTOR.md should work.
```

### Phase 7 — Testing & Release
```
Implement Phase 7: Testing and Distribution.

1. Ensure all parser unit tests pass with edge cases (empty files, missing fields, malformed JSON, unicode titles)
2. Add model tests — filter/sort combinations with various track states
3. Add integration test — create temp conductor/ directory, parse all tracks, verify correctness
4. cargo build --release — strip symbols, verify binary < 5MB
5. Add CI workflow (.github/workflows/ci.yml) — cargo fmt --check, cargo clippy, cargo test, cargo build --release on ubuntu/macos/windows
6. Add README.md with installation instructions, screenshot, and usage
