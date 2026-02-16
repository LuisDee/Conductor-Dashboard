//! Main application state, event handling, and rendering.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    TableState,
};
use ratatui::Frame;

use crate::event::Event;
use crate::model::{FilterMode, PhaseStatus, ReloadScope, SortMode, Status, Track, TrackCache, TrackId};
use crate::theme::Theme;

/// Return value from event handling.
#[derive(Debug, PartialEq)]
pub enum Action {
    Continue,
    Quit,
    ForceRefresh,
}

/// Input mode for modal states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    Help,
}

/// Core application state.
pub struct App {
    // Core data
    pub tracks: BTreeMap<TrackId, Track>,
    pub conductor_dir: PathBuf,

    // UI state
    pub table_state: TableState,
    pub selected_track: Option<TrackId>,
    pub filter: FilterMode,
    pub sort: SortMode,
    pub search_query: String,
    pub mode: InputMode,
    pub detail_scroll: u16,
    pub detail_total_lines: u16,
    pub split_percent: u16,
    pub detail_maximised: bool,

    // Theme
    pub theme: Theme,

    // Status
    pub watcher_active: bool,
    pub no_watch: bool,
    pub last_refresh: Option<Instant>,
    pub error_message: Option<(String, Instant)>,
    pub clock: String,

    // Cached filtered list
    pub filtered_track_ids: Vec<TrackId>,

    // Layout areas for mouse hit-testing
    pub list_area: Rect,
    pub detail_area: Rect,

    // Cache for incremental reloading
    pub track_cache: TrackCache,
}

impl App {
    pub fn new(conductor_dir: PathBuf, no_watch: bool, initial_filter: FilterMode) -> color_eyre::Result<Self> {
        Ok(Self {
            tracks: BTreeMap::new(),
            conductor_dir,
            table_state: TableState::default(),
            selected_track: None,
            filter: initial_filter,
            sort: SortMode::Updated,
            search_query: String::new(),
            mode: InputMode::Normal,
            detail_scroll: 0,
            detail_total_lines: 0,
            split_percent: 45,
            detail_maximised: false,
            theme: Theme::mako(),
            watcher_active: !no_watch,
            no_watch,
            last_refresh: None,
            error_message: None,
            clock: chrono::Local::now().format("%H:%M:%S").to_string(),
            filtered_track_ids: Vec::new(),
            list_area: Rect::default(),
            detail_area: Rect::default(),
            track_cache: TrackCache::new(),
        })
    }

    /// Load tracks from disk.
    pub fn load_tracks(&mut self) -> color_eyre::Result<()> {
        match crate::parser::load_all_tracks(&self.conductor_dir) {
            Ok(tracks) => {
                self.tracks = tracks;
                self.last_refresh = Some(Instant::now());
                self.recompute_filtered_tracks();
                if self.selected_track.is_none() {
                    self.select_first();
                }
                Ok(())
            }
            Err(e) => {
                self.error_message = Some((e.to_string(), Instant::now()));
                Ok(())
            }
        }
    }

    /// Reload specific tracks or do a full reload.
    pub fn reload_tracks(&mut self, scope: ReloadScope) {
        match scope {
            ReloadScope::Full => {
                if let Err(e) = self.load_tracks() {
                    self.error_message = Some((e.to_string(), Instant::now()));
                }
            }
            ReloadScope::Tracks(track_ids) => {
                let tracks_dir = self.conductor_dir.join("tracks");
                for id in &track_ids {
                    let track_dir = tracks_dir.join(id.as_str());

                    // Reload metadata
                    if let Some(track) = self.tracks.get_mut(id) {
                        if let Ok(Some(meta)) =
                            crate::parser::metadata::parse_metadata(&track_dir, id.as_str())
                        {
                            track.merge_metadata(meta);
                        }

                        // Reload plan
                        let plan_path = track_dir.join("plan.md");
                        if plan_path.exists() {
                            if let Ok(phases) = crate::parser::plan::parse_plan(&plan_path) {
                                track.merge_plan(phases);
                            }
                        }

                        // Auto-complete tasks for tracks marked as done
                        if track.status == Status::Complete {
                            track.mark_all_tasks_complete();
                        }
                    }
                }
                self.last_refresh = Some(Instant::now());
                self.recompute_filtered_tracks();
            }
        }
    }

    /// Main event loop.
    pub async fn run(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
    ) -> color_eyre::Result<()> {
        // Initial load
        self.load_tracks()?;

        // Start event handler
        let mut events =
            crate::event::EventHandler::new(self.conductor_dir.clone(), !self.no_watch);

        loop {
            // RENDER
            terminal.draw(|frame| self.render(frame))?;

            // WAIT FOR EVENT
            let Some(event) = events.next().await else {
                break;
            };

            // UPDATE
            match self.handle_event(event) {
                Action::Quit => break,
                Action::ForceRefresh => {
                    let _ = self.load_tracks();
                }
                Action::Continue => {}
            }
        }

        Ok(())
    }

    /// Handle a single event.
    pub fn handle_event(&mut self, event: Event) -> Action {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            Event::Mouse(mouse) => self.handle_mouse_event(mouse),
            Event::Tick => {
                self.clock = chrono::Local::now().format("%H:%M:%S").to_string();
                // Auto-dismiss errors after 10 seconds
                if let Some((_, when)) = &self.error_message {
                    if when.elapsed().as_secs() >= 10 {
                        self.error_message = None;
                    }
                }
                Action::Continue
            }
            Event::FilesChanged(paths) => {
                self.watcher_active = true;
                let scope = self.track_cache.classify_changes(&paths);
                self.reload_tracks(scope);
                Action::Continue
            }
            Event::Resize(_, _) => Action::Continue,
        }
    }

    /// Handle key events.
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        // Global keys
        match key.code {
            KeyCode::Char('q') if self.mode == InputMode::Normal => return Action::Quit,
            KeyCode::Char('?') if self.mode != InputMode::Search => {
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

        // Help mode: any key dismisses
        if self.mode == InputMode::Help {
            self.mode = InputMode::Normal;
            return Action::Continue;
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
                return Action::ForceRefresh;
            }
            KeyCode::Char('t') => {
                self.theme = self.theme.next();
            }
            KeyCode::Char('[') => {
                self.split_percent = self.split_percent.saturating_sub(5).max(20);
            }
            KeyCode::Char(']') => {
                self.split_percent = (self.split_percent + 5).min(80);
            }
            KeyCode::Char('d') => {
                self.detail_scroll = self
                    .detail_scroll
                    .saturating_add(5)
                    .min(self.detail_total_lines.saturating_sub(5));
            }
            KeyCode::Char('u') => {
                self.detail_scroll = self.detail_scroll.saturating_sub(5);
            }
            _ => {}
        }

        Action::Continue
    }

    /// Handle mouse events.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Action {
        match mouse.kind {
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                // Click in track list area → select that track
                if !self.detail_maximised && self.list_area.contains((mouse.column, mouse.row).into()) {
                    // Account for border (1) + header row (1) + header bottom margin (1) = 3 rows offset
                    let row_offset = mouse.row.saturating_sub(self.list_area.y + 3);
                    let track_index = (row_offset / 2) as usize; // each row is height 2
                    if track_index < self.filtered_track_ids.len() {
                        self.table_state.select(Some(track_index));
                        self.selected_track = self.filtered_track_ids.get(track_index).cloned();
                        self.detail_scroll = 0;
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                if self.detail_maximised
                    || self.detail_area.contains((mouse.column, mouse.row).into())
                {
                    // Scroll detail panel down
                    self.detail_scroll = self
                        .detail_scroll
                        .saturating_add(3)
                        .min(self.detail_total_lines.saturating_sub(5));
                } else if self.list_area.contains((mouse.column, mouse.row).into()) {
                    self.select_next();
                }
            }
            MouseEventKind::ScrollUp => {
                if self.detail_maximised
                    || self.detail_area.contains((mouse.column, mouse.row).into())
                {
                    // Scroll detail panel up
                    self.detail_scroll = self.detail_scroll.saturating_sub(3);
                } else if self.list_area.contains((mouse.column, mouse.row).into()) {
                    self.select_previous();
                }
            }
            _ => {}
        }
        Action::Continue
    }

    // ─────────────────────────────────────────────────────────
    // Selection helpers
    // ─────────────────────────────────────────────────────────

    fn select_next(&mut self) {
        let len = self.filtered_track_ids.len();
        if len == 0 {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map(|s| (s + 1).min(len - 1))
            .unwrap_or(0);
        self.table_state.select(Some(i));
        self.selected_track = self.filtered_track_ids.get(i).cloned();
        self.detail_scroll = 0;
    }

    fn select_previous(&mut self) {
        let len = self.filtered_track_ids.len();
        if len == 0 {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map(|s| s.saturating_sub(1))
            .unwrap_or(0);
        self.table_state.select(Some(i));
        self.selected_track = self.filtered_track_ids.get(i).cloned();
        self.detail_scroll = 0;
    }

    fn select_first(&mut self) {
        if self.filtered_track_ids.is_empty() {
            return;
        }
        self.table_state.select(Some(0));
        self.selected_track = self.filtered_track_ids.first().cloned();
        self.detail_scroll = 0;
    }

    fn select_last(&mut self) {
        let len = self.filtered_track_ids.len();
        if len == 0 {
            return;
        }
        self.table_state.select(Some(len - 1));
        self.selected_track = self.filtered_track_ids.last().cloned();
        self.detail_scroll = 0;
    }

    // ─────────────────────────────────────────────────────────
    // Filter / Sort
    // ─────────────────────────────────────────────────────────

    fn recompute_filtered_tracks(&mut self) {
        let search_lower = self.search_query.to_ascii_lowercase();

        let mut tracks: Vec<(TrackId, &Track)> = self
            .tracks
            .iter()
            .filter(|(_, track)| match self.filter {
                FilterMode::All => true,
                FilterMode::Active => track.status == Status::InProgress,
                FilterMode::Blocked => track.status == Status::Blocked,
                FilterMode::Complete => track.status == Status::Complete,
            })
            .filter(|(id, track)| {
                if search_lower.is_empty() {
                    return true;
                }
                track.title.to_ascii_lowercase().contains(&search_lower)
                    || id.as_str().to_ascii_lowercase().contains(&search_lower)
            })
            .map(|(id, track)| (id.clone(), track))
            .collect();

        match self.sort {
            SortMode::Updated => {
                tracks.sort_by(|(_, a), (_, b)| {
                    let a_time = a.updated_at.or(a.created_at);
                    let b_time = b.updated_at.or(b.created_at);
                    b_time.cmp(&a_time)
                });
            }
            SortMode::Progress => {
                tracks.sort_by(|(_, a), (_, b)| {
                    b.progress_percent()
                        .partial_cmp(&a.progress_percent())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }

        self.filtered_track_ids = tracks.into_iter().map(|(id, _)| id).collect();

        // Ensure selection is still visible
        if let Some(ref selected) = self.selected_track {
            if let Some(pos) = self.filtered_track_ids.iter().position(|id| id == selected) {
                self.table_state.select(Some(pos));
            } else {
                // Selection filtered out — select first
                self.table_state.select(if self.filtered_track_ids.is_empty() {
                    None
                } else {
                    Some(0)
                });
                self.selected_track = self.filtered_track_ids.first().cloned();
            }
        }
    }

    // ─────────────────────────────────────────────────────────
    // Rendering
    // ─────────────────────────────────────────────────────────

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Graceful degradation for tiny terminals
        if area.width < 40 || area.height < 10 {
            let msg = Paragraph::new("Terminal too small. Resize to at least 80x24.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(self.theme.error));
            frame.render_widget(msg, area);
            return;
        }

        let has_error = self.error_message.is_some();
        let constraints = if has_error {
            vec![
                Constraint::Length(1),  // title bar
                Constraint::Length(2),  // stats bar
                Constraint::Length(1),  // error bar
                Constraint::Fill(1),   // main content
                Constraint::Length(1),  // status bar
            ]
        } else {
            vec![
                Constraint::Length(1),  // title bar
                Constraint::Length(2),  // stats bar
                Constraint::Fill(1),   // main content
                Constraint::Length(1),  // status bar
            ]
        };

        let areas: Vec<Rect> = Layout::vertical(constraints).split(area).to_vec();

        let (title_area, stats_area, main_area, status_area) = if has_error {
            (areas[0], areas[1], areas[3], areas[4])
        } else {
            (areas[0], areas[1], areas[2], areas[3])
        };

        self.render_title_bar(frame, title_area);
        self.render_stats_bar(frame, stats_area);

        if has_error {
            self.render_error_bar(frame, areas[2]);
        }

        self.render_status_bar(frame, status_area);

        // Main content area
        if area.width < 80 || self.detail_maximised {
            // Narrow terminal or maximised: show only one pane
            if self.detail_maximised && self.selected_track.is_some() {
                self.detail_area = main_area;
                self.list_area = Rect::default();
                self.render_detail_panel(frame, main_area);
            } else {
                self.list_area = main_area;
                self.detail_area = Rect::default();
                self.render_track_list(frame, main_area);
            }
        } else {
            let [list_area, detail_area] = Layout::horizontal([
                Constraint::Percentage(self.split_percent),
                Constraint::Percentage(100 - self.split_percent),
            ])
            .areas(main_area);

            self.list_area = list_area;
            self.detail_area = detail_area;

            self.render_track_list(frame, list_area);
            self.render_detail_panel(frame, detail_area);
        }

        // Overlays
        if self.mode == InputMode::Search {
            self.render_search_overlay(frame, area);
        }
        if self.mode == InputMode::Help {
            self.render_help_overlay(frame, area);
        }
    }

    fn render_title_bar(&self, frame: &mut Frame, area: Rect) {
        let watcher_indicator = if self.no_watch {
            Span::styled("○ STATIC", Style::default().fg(self.theme.text_secondary))
        } else if self.watcher_active {
            Span::styled("● WATCHING", Style::default().fg(self.theme.success))
        } else {
            Span::styled("● WATCHER ERROR", Style::default().fg(self.theme.error))
        };

        let padding = area
            .width
            .saturating_sub(24 + self.clock.len() as u16 + 12) as usize;

        let title = Line::from(vec![
            Span::styled(
                " ◇ Conductor Dashboard",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ".repeat(padding)),
            Span::raw(&self.clock),
            Span::raw("  "),
            watcher_indicator,
            Span::raw(" "),
        ]);

        frame.render_widget(
            Paragraph::new(title).style(
                Style::default()
                    .bg(self.theme.bar_bg)
                    .fg(self.theme.text_on_bar),
            ),
            area,
        );
    }

    fn render_stats_bar(&self, frame: &mut Frame, area: Rect) {
        let [counts_area, controls_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

        let total = self.tracks.len();
        let active = self
            .tracks
            .values()
            .filter(|t| t.status == Status::InProgress)
            .count();
        let blocked = self
            .tracks
            .values()
            .filter(|t| t.status == Status::Blocked)
            .count();
        let complete = self
            .tracks
            .values()
            .filter(|t| t.status == Status::Complete)
            .count();

        let counts = Line::from(vec![
            Span::styled(
                format!(" {} Total", total),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" │ "),
            Span::styled(
                format!("{} Active", active),
                Style::default().fg(self.theme.accent),
            ),
            Span::raw(" │ "),
            Span::styled(
                format!("{} Blocked", blocked),
                Style::default().fg(self.theme.warning),
            ),
            Span::raw(" │ "),
            Span::styled(
                format!("{} Complete", complete),
                Style::default().fg(self.theme.success),
            ),
        ]);
        frame.render_widget(Paragraph::new(counts), counts_area);

        let filter_label = match self.filter {
            FilterMode::All => "[All]  Active  Blocked  Done",
            FilterMode::Active => " All  [Active] Blocked  Done",
            FilterMode::Blocked => " All   Active [Blocked] Done",
            FilterMode::Complete => " All   Active  Blocked [Done]",
        };
        let sort_label = match self.sort {
            SortMode::Updated => "[Recent] Progress",
            SortMode::Progress => " Recent [Progress]",
        };

        let controls = Line::from(vec![
            Span::styled(
                format!(" Filter: {filter_label}"),
                Style::default().fg(self.theme.text_secondary),
            ),
            Span::raw("  │  "),
            Span::styled(
                format!("Sort: {sort_label}"),
                Style::default().fg(self.theme.text_secondary),
            ),
        ]);
        frame.render_widget(Paragraph::new(controls), controls_area);
    }

    fn render_error_bar(&self, frame: &mut Frame, area: Rect) {
        if let Some((ref msg, _)) = self.error_message {
            let line = Line::from(vec![
                Span::styled(
                    format!(" ⚠ {msg}"),
                    Style::default()
                        .fg(self.theme.bar_bg)
                        .bg(self.theme.warning),
                ),
            ]);
            frame.render_widget(
                Paragraph::new(line).style(Style::default().bg(self.theme.warning)),
                area,
            );
        }
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let theme_name = self.theme.name;

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
            Span::styled("t", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Theme  "),
            Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Help  "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(" Quit  │ {theme_name}")),
        ]);

        frame.render_widget(
            Paragraph::new(shortcuts).style(
                Style::default()
                    .bg(self.theme.bar_bg)
                    .fg(self.theme.text_on_bar),
            ),
            area,
        );
    }

    fn render_track_list(&mut self, frame: &mut Frame, area: Rect) {
        let theme = self.theme;

        let header = Row::new(vec!["Track", "Status", "Progress", "Tasks"])
            .style(
                Style::default()
                    .fg(theme.text_secondary)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1);

        let rows: Vec<Row> = self
            .filtered_track_ids
            .iter()
            .filter_map(|id| self.tracks.get(id).map(|t| (id, t)))
            .map(|(_id, track)| {
                let title = Line::from(vec![Span::styled(
                    &track.title,
                    Style::default().add_modifier(Modifier::BOLD),
                )]);
                let date_str = track
                    .created_at
                    .map(|d| d.format("%b %d").to_string())
                    .unwrap_or_default();
                let subtitle = Line::from(vec![Span::styled(
                    format!(
                        "{}{}",
                        if track.phase.is_empty() {
                            String::new()
                        } else {
                            format!("{} · ", track.phase)
                        },
                        date_str
                    ),
                    Style::default().fg(theme.text_secondary),
                )]);

                Row::new(vec![
                    Cell::from(Text::from(vec![title, subtitle])),
                    Cell::from(status_span(&track.status, &theme)),
                    Cell::from(progress_bar_text(track.progress_percent(), &track.status, &theme)),
                    Cell::from(format!("{}/{}", track.tasks_completed, track.tasks_total)),
                ])
                .height(2)
            })
            .collect();

        let widths = [
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Length(12),
            Constraint::Length(6),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::bordered()
                    .border_style(Style::default().fg(theme.border))
                    .title(" Tracks "),
            )
            .row_highlight_style(
                Style::default()
                    .bg(theme.accent)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_detail_panel(&mut self, frame: &mut Frame, area: Rect) {
        let theme = self.theme;

        let block = Block::bordered()
            .border_style(Style::default().fg(theme.border))
            .title(" Detail ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(track_id) = &self.selected_track else {
            let msg = Paragraph::new("Select a track to view details")
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.text_secondary));
            frame.render_widget(msg, inner);
            return;
        };

        let Some(track) = self.tracks.get(track_id) else {
            return;
        };

        let mut lines: Vec<Line> = Vec::new();

        // Type label + track ID
        lines.push(Line::from(vec![
            Span::styled(
                track.track_type.label(),
                Style::default()
                    .fg(theme.text_secondary)
                    .add_modifier(Modifier::DIM),
            ),
            Span::raw(" · "),
            Span::styled(
                track.id.as_str(),
                Style::default().fg(theme.text_secondary),
            ),
        ]));

        // Title
        lines.push(Line::from(Span::styled(
            &track.title,
            Style::default().add_modifier(Modifier::BOLD),
        )));

        // Status + created date
        let date_str = track
            .created_at
            .map(|d| d.format("%b %d, %Y").to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        lines.push(Line::from(format!(
            "{} {}  Created: {}",
            match track.status {
                Status::InProgress => "⚙ Active",
                Status::Blocked => "⚠ Blocked",
                Status::Complete => "✓ Complete",
                Status::New => "○ New",
            },
            "",
            date_str
        )));

        lines.push(Line::raw(""));

        // Progress bar (full width)
        let pct = track.progress_percent();
        let bar_width = inner.width.saturating_sub(14) as usize;
        let filled = ((pct / 100.0) * bar_width as f32).round() as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar_color = if pct >= 100.0 {
            theme.progress_done
        } else if pct > 0.0 {
            theme.progress_active
        } else {
            theme.progress_new
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}/{} ", track.tasks_completed, track.tasks_total),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
            Span::styled("░".repeat(empty), Style::default().fg(theme.border)),
            Span::raw(format!(" {:.0}%", pct)),
        ]));

        lines.push(Line::raw(""));

        // Dependencies
        if !track.dependencies.is_empty() {
            let dep_str: Vec<&str> = track.dependencies.iter().map(|d| d.as_str()).collect();
            lines.push(Line::styled(
                format!("⚠ Blocked by: {}", dep_str.join(", ")),
                Style::default().fg(theme.warning),
            ));
            lines.push(Line::raw(""));
        }

        // Implementation Plan heading
        if !track.plan_phases.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(
                    "━━ ",
                    Style::default().fg(theme.accent),
                ),
                Span::styled(
                    "IMPLEMENTATION PLAN",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " ━━",
                    Style::default().fg(theme.accent),
                ),
            ]));
            lines.push(Line::raw(""));

            for phase in &track.plan_phases {
                let phase_icon = match phase.status {
                    PhaseStatus::Complete => "●",
                    PhaseStatus::Active => "◐",
                    PhaseStatus::Pending => "○",
                    PhaseStatus::Blocked => "⊘",
                };
                let icon_color = match phase.status {
                    PhaseStatus::Complete => theme.success,
                    PhaseStatus::Active => theme.accent,
                    PhaseStatus::Pending => theme.text_secondary,
                    PhaseStatus::Blocked => theme.warning,
                };
                let done = phase.tasks.iter().filter(|t| t.done).count();
                let total = phase.tasks.len();

                // Phase header with background highlight for active phases
                let phase_name_style = match phase.status {
                    PhaseStatus::Active => Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                    PhaseStatus::Complete => Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                    _ => Style::default().add_modifier(Modifier::BOLD),
                };

                let count_style = match phase.status {
                    PhaseStatus::Complete => Style::default().fg(theme.success),
                    PhaseStatus::Active => Style::default().fg(theme.accent),
                    _ => Style::default().fg(theme.text_secondary),
                };

                lines.push(Line::from(vec![
                    Span::styled(phase_icon, Style::default().fg(icon_color)),
                    Span::styled(format!(" {} ", phase.name), phase_name_style),
                    Span::styled(format!("({}/{})", done, total), count_style),
                ]));

                for task in &phase.tasks {
                    if task.done {
                        lines.push(Line::from(vec![
                            Span::styled("  ✓ ", Style::default().fg(theme.success)),
                            Span::styled(
                                &task.text,
                                Style::default().fg(theme.text_secondary),
                            ),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled("  ○ ", Style::default().fg(theme.warning)),
                            Span::styled(
                                &task.text,
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]));
                    }
                }

                lines.push(Line::raw(""));
            }
        }

        let total_lines = lines.len() as u16;
        self.detail_total_lines = total_lines;

        let paragraph = Paragraph::new(lines).scroll((self.detail_scroll, 0));
        frame.render_widget(paragraph, inner);

        // Scrollbar
        if total_lines > inner.height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::new(total_lines as usize)
                .position(self.detail_scroll as usize);
            frame.render_stateful_widget(
                scrollbar,
                inner.inner(Margin {
                    vertical: 0,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    }

    fn render_search_overlay(&self, frame: &mut Frame, area: Rect) {
        let search_area = Rect {
            x: area.x + 1,
            y: area.y + 3,
            width: area.width.saturating_sub(2),
            height: 1,
        };

        frame.render_widget(Clear, search_area);

        let search_line = Line::from(vec![
            Span::styled(
                " / ",
                Style::default()
                    .fg(self.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&self.search_query),
            Span::styled("█", Style::default().fg(self.theme.accent)),
        ]);

        frame.render_widget(
            Paragraph::new(search_line).style(
                Style::default()
                    .bg(self.theme.surface)
                    .fg(self.theme.text_primary),
            ),
            search_area,
        );
    }

    fn render_help_overlay(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(60, 20, area);
        frame.render_widget(Clear, popup_area);

        let help_text = vec![
            Line::styled(
                "Keyboard Shortcuts",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::raw("  ↑/k       Move selection up"),
            Line::raw("  ↓/j       Move selection down"),
            Line::raw("  Home/End  First/last track"),
            Line::raw("  Enter     Maximise detail panel"),
            Line::raw("  Esc       Return to split view / close"),
            Line::raw("  f         Cycle filter (All → Active → Blocked → Done)"),
            Line::raw("  s         Cycle sort (Recent ↔ Progress)"),
            Line::raw("  /         Open search"),
            Line::raw("  r         Force refresh"),
            Line::raw("  t         Cycle theme"),
            Line::raw("  d/u       Scroll detail down/up"),
            Line::raw("  [/]       Resize split (left/right)"),
            Line::raw("  ?         Toggle this help"),
            Line::raw("  q         Quit"),
            Line::raw(""),
            Line::styled(
                "Press any key to close",
                Style::default().fg(self.theme.text_secondary),
            ),
        ];

        let help = Paragraph::new(help_text).block(
            Block::bordered()
                .title(" Help ")
                .border_style(Style::default().fg(self.theme.accent))
                .style(Style::default().bg(self.theme.surface)),
        );

        frame.render_widget(help, popup_area);
    }
}

// ─────────────────────────────────────────────────────────
// Standalone helper functions
// ─────────────────────────────────────────────────────────

fn status_span(status: &Status, theme: &Theme) -> Text<'static> {
    let (label, style) = match status {
        Status::InProgress => (
            "⚙ ACT",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Status::Blocked => (
            "⚠ BLK",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Status::Complete => ("✓ DON", Style::default().fg(theme.success)),
        Status::New => ("○ NEW", Style::default().fg(theme.text_secondary)),
    };
    Text::from(Span::styled(label, style))
}

fn progress_bar_text(percent: f32, status: &Status, theme: &Theme) -> Text<'static> {
    let width: usize = 8;
    let filled = ((percent / 100.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);

    let color = match status {
        Status::Complete => theme.progress_done,
        Status::Blocked => theme.progress_blocked,
        _ if percent > 0.0 => theme.progress_active,
        _ => theme.progress_new,
    };

    let bar = format!("{}{} {:>3.0}%", "█".repeat(filled), "░".repeat(empty), percent);
    Text::from(Span::styled(bar, Style::default().fg(color)))
}

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
