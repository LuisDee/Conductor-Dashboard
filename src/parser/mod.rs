pub mod error;
pub mod index;
pub mod metadata;
pub mod plan;

use std::collections::BTreeMap;
use std::path::Path;

use tracing::{debug, warn};

use crate::model::{Track, TrackId};
use crate::parser::error::ParseError;

/// Load all tracks from a conductor directory.
///
/// 1. Parse `tracks.md` to get the master list of tracks.
/// 2. For each track, try to load `metadata.json` or `meta.yaml`.
/// 3. For each track, try to load `plan.md`.
///
/// Partial failures (bad metadata, missing plan) are logged but don't
/// prevent other tracks from loading.
pub fn load_all_tracks(conductor_dir: &Path) -> Result<BTreeMap<TrackId, Track>, ParseError> {
    let mut tracks = index::parse_index(conductor_dir)?;

    let tracks_dir = conductor_dir.join("tracks");

    for (id, track) in tracks.iter_mut() {
        let track_dir = tracks_dir.join(id.as_str());

        // Load metadata
        match metadata::parse_metadata(&track_dir, id.as_str()) {
            Ok(Some(meta)) => {
                debug!(track_id = id.as_str(), "loaded metadata");
                track.merge_metadata(meta);
            }
            Ok(None) => {
                debug!(track_id = id.as_str(), "no metadata file found");
            }
            Err(e) => {
                warn!(track_id = id.as_str(), error = %e, "failed to parse metadata, using defaults");
            }
        }

        // Load plan
        let plan_path = track_dir.join("plan.md");
        if plan_path.exists() {
            match plan::parse_plan(&plan_path) {
                Ok(phases) => {
                    debug!(track_id = id.as_str(), phases = phases.len(), "loaded plan");
                    track.merge_plan(phases);
                }
                Err(e) => {
                    warn!(track_id = id.as_str(), error = %e, "failed to parse plan");
                }
            }
        }
    }

    // Auto-complete tasks for tracks marked as done — display-level normalization
    // so the dashboard shows 100% progress when metadata says Complete.
    for track in tracks.values_mut() {
        if track.status == crate::model::Status::Complete {
            track.mark_all_tasks_complete();
        }
    }

    Ok(tracks)
}
