//! TrackCache — tracks file modification times for incremental reloading.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::model::TrackId;

/// Determines what needs to be reloaded when files change.
#[derive(Debug, Clone)]
pub enum ReloadScope {
    /// tracks.md changed — full re-parse needed.
    Full,
    /// Only specific track files changed.
    Tracks(Vec<TrackId>),
}

/// Caches file modification times to enable incremental reloading.
#[derive(Debug, Default)]
pub struct TrackCache {
    mtimes: HashMap<PathBuf, SystemTime>,
}

impl TrackCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classify a set of changed file paths into a ReloadScope.
    pub fn classify_changes(&self, paths: &[PathBuf]) -> ReloadScope {
        let mut changed_tracks = Vec::new();
        let mut full_reload = false;

        for path in paths {
            if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                match name {
                    "tracks.md" => {
                        full_reload = true;
                    }
                    "metadata.json" | "meta.yaml" | "plan.md" | "spec.md" => {
                        if let Some(track_id) = extract_track_id_from_path(path) {
                            if !changed_tracks.contains(&track_id) {
                                changed_tracks.push(track_id);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if full_reload {
            ReloadScope::Full
        } else {
            ReloadScope::Tracks(changed_tracks)
        }
    }

    /// Update cached mtime for a path.
    pub fn update_mtime(&mut self, path: &Path) {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(mtime) = metadata.modified() {
                self.mtimes.insert(path.to_path_buf(), mtime);
            }
        }
    }

    /// Check if a path has changed since last cached mtime.
    pub fn has_changed(&self, path: &Path) -> bool {
        let current = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok());

        match (self.mtimes.get(path), current) {
            (Some(cached), Some(current)) => current > *cached,
            (None, Some(_)) => true,
            _ => false,
        }
    }
}

/// Extract a TrackId from a file path like `.../tracks/some_track_id/plan.md`
fn extract_track_id_from_path(path: &Path) -> Option<TrackId> {
    let parent = path.parent()?;
    let track_dir_name = parent.file_name()?.to_str()?;

    // Verify the grandparent is "tracks"
    let grandparent = parent.parent()?;
    if grandparent.file_name()?.to_str()? == "tracks" {
        Some(TrackId::new(track_dir_name))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_tracks_md_change() {
        let cache = TrackCache::new();
        let paths = vec![PathBuf::from("/project/conductor/tracks.md")];
        assert!(matches!(cache.classify_changes(&paths), ReloadScope::Full));
    }

    #[test]
    fn test_classify_plan_change() {
        let cache = TrackCache::new();
        let paths = vec![PathBuf::from(
            "/project/conductor/tracks/my_track_123/plan.md",
        )];
        match cache.classify_changes(&paths) {
            ReloadScope::Tracks(ids) => {
                assert_eq!(ids.len(), 1);
                assert_eq!(ids[0].as_str(), "my_track_123");
            }
            _ => panic!("expected Tracks scope"),
        }
    }

    #[test]
    fn test_classify_mixed_changes() {
        let cache = TrackCache::new();
        let paths = vec![
            PathBuf::from("/project/conductor/tracks/track_a/metadata.json"),
            PathBuf::from("/project/conductor/tracks.md"),
        ];
        // tracks.md change should trigger Full reload
        assert!(matches!(cache.classify_changes(&paths), ReloadScope::Full));
    }

    #[test]
    fn test_classify_multiple_track_changes() {
        let cache = TrackCache::new();
        let paths = vec![
            PathBuf::from("/project/conductor/tracks/track_a/plan.md"),
            PathBuf::from("/project/conductor/tracks/track_b/meta.yaml"),
        ];
        match cache.classify_changes(&paths) {
            ReloadScope::Tracks(ids) => {
                assert_eq!(ids.len(), 2);
            }
            _ => panic!("expected Tracks scope"),
        }
    }

    #[test]
    fn test_extract_track_id() {
        let path = PathBuf::from("/project/conductor/tracks/my_track_123/plan.md");
        let id = extract_track_id_from_path(&path).unwrap();
        assert_eq!(id.as_str(), "my_track_123");
    }

    #[test]
    fn test_extract_track_id_not_in_tracks_dir() {
        let path = PathBuf::from("/project/some_other/my_track/plan.md");
        assert!(extract_track_id_from_path(&path).is_none());
    }
}
