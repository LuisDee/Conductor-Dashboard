//! Event hub — multiplexes terminal, file watcher, and tick events
//! into a single async channel.

use std::path::{Path, PathBuf};

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind, MouseEvent};
use futures::StreamExt;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    /// Terminal key press
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resized
    #[allow(dead_code)]
    Resize(u16, u16),
    /// File watcher detected changes
    FilesChanged(Vec<PathBuf>),
    /// Periodic tick (1 second)
    Tick,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(conductor_dir: PathBuf, watch_enabled: bool) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn crossterm event reader
        let tx_key = tx.clone();
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            while let Some(Ok(evt)) = reader.next().await {
                match evt {
                    CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => {
                        if tx_key.send(Event::Key(key)).is_err() {
                            break;
                        }
                    }
                    CrosstermEvent::Mouse(mouse) => {
                        if tx_key.send(Event::Mouse(mouse)).is_err() {
                            break;
                        }
                    }
                    CrosstermEvent::Resize(w, h) => {
                        if tx_key.send(Event::Resize(w, h)).is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        });

        // Spawn tick timer
        let tx_tick = tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                if tx_tick.send(Event::Tick).is_err() {
                    break;
                }
            }
        });

        // Spawn file watcher (if enabled)
        if watch_enabled {
            let tx_watch = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = run_file_watcher(conductor_dir, tx_watch).await {
                    tracing::error!(error = %e, "file watcher failed");
                }
            });
        }

        EventHandler { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

async fn run_file_watcher(
    conductor_dir: PathBuf,
    tx: mpsc::UnboundedSender<Event>,
) -> color_eyre::Result<()> {
    let (wtx, mut wrx) = mpsc::channel::<Vec<PathBuf>>(100);

    let mut debouncer = notify_debouncer_mini::new_debouncer(
        std::time::Duration::from_millis(300),
        move |result: notify_debouncer_mini::DebounceEventResult| {
            if let Ok(events) = result {
                let paths: Vec<_> = events
                    .iter()
                    .filter(|e| is_conductor_file(&e.path))
                    .map(|e| e.path.clone())
                    .collect();
                if !paths.is_empty() {
                    let _ = wtx.blocking_send(paths);
                }
            }
        },
    )?;

    debouncer
        .watcher()
        .watch(&conductor_dir, notify::RecursiveMode::Recursive)?;

    // Keep debouncer alive; forward events
    while let Some(paths) = wrx.recv().await {
        if tx.send(Event::FilesChanged(paths)).is_err() {
            break;
        }
    }

    Ok(())
}

fn is_conductor_file(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|f| f.to_str()),
        Some("tracks.md" | "metadata.json" | "meta.yaml" | "plan.md" | "spec.md")
    )
}
