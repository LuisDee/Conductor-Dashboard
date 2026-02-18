use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler,
};

use crate::model::{Priority, Status, Track, TrackId};
use crate::parser;

use super::types::*;

// ---------------------------------------------------------------------------
// ConductorService
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ConductorService {
    tracks: Arc<BTreeMap<TrackId, Track>>,
    conductor_dir: PathBuf,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ConductorService {
    pub fn new(conductor_dir: &Path) -> Result<Self, crate::parser::error::ParseError> {
        let tracks = parser::load_all_tracks(conductor_dir)?;
        Ok(Self {
            tracks: Arc::new(tracks),
            conductor_dir: conductor_dir.to_path_buf(),
            tool_router: Self::tool_router(),
        })
    }

    // -- helpers (not tools) ------------------------------------------------

    fn format_datetime(dt: &Option<chrono::DateTime<chrono::Utc>>) -> Option<String> {
        dt.map(|d| d.format("%Y-%m-%d").to_string())
    }

    fn track_to_summary(track: &Track) -> TrackSummaryResponse {
        TrackSummaryResponse {
            id: track.id.as_str().to_string(),
            title: track.title.clone(),
            status: format!("{}", track.status),
            priority: format!("{}", track.priority),
            track_type: format!("{}", track.track_type),
            progress_percent: track.progress_percent(),
            tasks_completed: track.tasks_completed,
            tasks_total: track.tasks_total,
            tags: track.tags.clone(),
            created_at: Self::format_datetime(&track.created_at),
            updated_at: Self::format_datetime(&track.updated_at),
        }
    }

    fn track_to_detail(&self, track: &Track) -> TrackDetailResponse {
        let tracks_dir = self.conductor_dir.join("tracks");
        let track_dir = tracks_dir.join(track.id.as_str());

        let plan_md = track_dir.join("plan.md");
        let metadata_json = track_dir.join("metadata.json");
        let meta_yaml = track_dir.join("meta.yaml");

        TrackDetailResponse {
            id: track.id.as_str().to_string(),
            title: track.title.clone(),
            status: format!("{}", track.status),
            priority: format!("{}", track.priority),
            track_type: format!("{}", track.track_type),
            phase: track.phase.clone(),
            progress_percent: track.progress_percent(),
            tasks_completed: track.tasks_completed,
            tasks_total: track.tasks_total,
            tags: track.tags.clone(),
            dependencies: track
                .dependencies
                .iter()
                .map(|d| d.as_str().to_string())
                .collect(),
            branch: track.branch.clone(),
            description: track.description.clone(),
            created_at: Self::format_datetime(&track.created_at),
            updated_at: Self::format_datetime(&track.updated_at),
            plan_phases: track
                .plan_phases
                .iter()
                .map(|p| PhaseResponse {
                    name: p.name.clone(),
                    status: format!("{}", p.status),
                    tasks_completed: p.tasks_completed(),
                    tasks_total: p.tasks.len(),
                    progress_percent: p.progress_percent(),
                    tasks: p
                        .tasks
                        .iter()
                        .map(|t| TaskResponse {
                            text: t.text.clone(),
                            done: t.done,
                        })
                        .collect(),
                })
                .collect(),
            file_paths: FilePathsResponse {
                track_dir: track_dir.to_string_lossy().to_string(),
                plan_md: plan_md
                    .exists()
                    .then(|| plan_md.to_string_lossy().to_string()),
                metadata_json: metadata_json
                    .exists()
                    .then(|| metadata_json.to_string_lossy().to_string()),
                meta_yaml: meta_yaml
                    .exists()
                    .then(|| meta_yaml.to_string_lossy().to_string()),
            },
        }
    }

    // -- tools --------------------------------------------------------------

    #[tool(
        description = "List all tracks with optional filtering by status and sorting. Returns summary info for each track including progress, tasks, tags, and dates."
    )]
    pub fn list_tracks(&self, Parameters(params): Parameters<ListTracksParams>) -> String {
        let status_filter = params
            .status
            .as_deref()
            .unwrap_or("all")
            .to_ascii_lowercase();
        let sort = params
            .sort
            .as_deref()
            .unwrap_or("updated")
            .to_ascii_lowercase();

        let mut tracks: Vec<&Track> = self.tracks.values().collect();

        // Filter by status
        if status_filter != "all" {
            let target = Status::from_str_loose(&status_filter);
            tracks.retain(|t| t.status == target);
        }

        // Sort
        match sort.as_str() {
            "progress" => tracks.sort_by(|a, b| {
                b.progress_percent()
                    .partial_cmp(&a.progress_percent())
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            _ => tracks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
        }

        let summaries: Vec<TrackSummaryResponse> =
            tracks.iter().map(|t| Self::track_to_summary(t)).collect();

        serde_json::to_string_pretty(&summaries).unwrap_or_else(|e| format!("Error: {e}"))
    }

    #[tool(
        description = "Get full detail for a single track including plan phases, tasks, dependencies, file paths, and all metadata."
    )]
    pub fn get_track_detail(&self, Parameters(params): Parameters<GetTrackDetailParams>) -> String {
        let track_id = TrackId::new(&params.track_id);
        match self.tracks.get(&track_id) {
            Some(track) => {
                let detail = self.track_to_detail(track);
                serde_json::to_string_pretty(&detail).unwrap_or_else(|e| format!("Error: {e}"))
            }
            None => {
                // Try substring match
                let matches: Vec<&Track> = self
                    .tracks
                    .values()
                    .filter(|t| t.id.as_str().contains(&params.track_id))
                    .collect();
                match matches.len() {
                    0 => format!("No track found matching '{}'", params.track_id),
                    1 => {
                        let detail = self.track_to_detail(matches[0]);
                        serde_json::to_string_pretty(&detail)
                            .unwrap_or_else(|e| format!("Error: {e}"))
                    }
                    _ => {
                        let ids: Vec<&str> = matches.iter().map(|t| t.id.as_str()).collect();
                        format!(
                            "Multiple tracks match '{}': {}. Please be more specific.",
                            params.track_id,
                            ids.join(", ")
                        )
                    }
                }
            }
        }
    }

    #[tool(
        description = "Get aggregate summary stats: total track count, counts per status, overall progress percentage, and total task counts."
    )]
    pub fn get_summary(&self) -> String {
        let total = self.tracks.len();
        let mut new = 0;
        let mut in_progress = 0;
        let mut blocked = 0;
        let mut complete = 0;
        let mut total_tasks = 0usize;
        let mut total_completed = 0usize;

        for track in self.tracks.values() {
            match track.status {
                Status::New => new += 1,
                Status::InProgress => in_progress += 1,
                Status::Blocked => blocked += 1,
                Status::Complete => complete += 1,
            }
            total_tasks += track.tasks_total;
            total_completed += track.tasks_completed;
        }

        let overall = if total_tasks > 0 {
            (total_completed as f32 / total_tasks as f32) * 100.0
        } else {
            0.0
        };

        let resp = SummaryResponse {
            total_tracks: total,
            by_status: StatusCounts {
                new,
                in_progress,
                blocked,
                complete,
            },
            overall_progress: overall,
            total_tasks,
            total_tasks_completed: total_completed,
        };

        serde_json::to_string_pretty(&resp).unwrap_or_else(|e| format!("Error: {e}"))
    }

    #[tool(
        description = "Search tracks by title, ID, or tag substring (case-insensitive). Returns matching track summaries."
    )]
    pub fn search_tracks(&self, Parameters(params): Parameters<SearchTracksParams>) -> String {
        let query = params.query.to_ascii_lowercase();
        let matches: Vec<TrackSummaryResponse> = self
            .tracks
            .values()
            .filter(|t| {
                t.id.as_str().to_ascii_lowercase().contains(&query)
                    || t.title.to_ascii_lowercase().contains(&query)
                    || t.tags
                        .iter()
                        .any(|tag| tag.to_ascii_lowercase().contains(&query))
            })
            .map(|t| Self::track_to_summary(t))
            .collect();

        serde_json::to_string_pretty(&matches).unwrap_or_else(|e| format!("Error: {e}"))
    }

    #[tool(
        description = "Get the dependency graph showing what each track depends on and what it blocks. Optionally filter to a single track."
    )]
    pub fn get_track_dependencies(
        &self,
        Parameters(params): Parameters<GetTrackDependenciesParams>,
    ) -> String {
        // Build reverse map: track_id -> list of tracks that depend on it
        let mut blocked_by: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for track in self.tracks.values() {
            for dep in &track.dependencies {
                blocked_by
                    .entry(dep.as_str().to_string())
                    .or_default()
                    .push(track.id.as_str().to_string());
            }
        }

        let tracks_iter: Box<dyn Iterator<Item = &Track>> = if let Some(ref tid) = params.track_id {
            let track_id = TrackId::new(tid.as_str());
            if let Some(t) = self.tracks.get(&track_id) {
                Box::new(std::iter::once(t))
            } else {
                return format!("No track found with ID '{}'", tid);
            }
        } else {
            Box::new(self.tracks.values())
        };

        let deps: Vec<DependencyInfo> = tracks_iter
            .map(|t| {
                let id_str = t.id.as_str().to_string();
                DependencyInfo {
                    track_id: id_str.clone(),
                    title: t.title.clone(),
                    status: format!("{}", t.status),
                    depends_on: t
                        .dependencies
                        .iter()
                        .map(|d| d.as_str().to_string())
                        .collect(),
                    blocks: blocked_by.get(&id_str).cloned().unwrap_or_default(),
                }
            })
            .collect();

        serde_json::to_string_pretty(&deps).unwrap_or_else(|e| format!("Error: {e}"))
    }

    #[tool(
        description = "Filter tracks by tag (case-insensitive). Returns matching track summaries."
    )]
    pub fn get_tracks_by_tag(
        &self,
        Parameters(params): Parameters<GetTracksByTagParams>,
    ) -> String {
        let tag = params.tag.to_ascii_lowercase();
        let matches: Vec<TrackSummaryResponse> = self
            .tracks
            .values()
            .filter(|t| t.tags.iter().any(|tt| tt.to_ascii_lowercase() == tag))
            .map(|t| Self::track_to_summary(t))
            .collect();

        serde_json::to_string_pretty(&matches).unwrap_or_else(|e| format!("Error: {e}"))
    }

    #[tool(
        description = "Filter tracks by priority level (critical, high, medium, low). Returns matching track summaries."
    )]
    pub fn get_tracks_by_priority(
        &self,
        Parameters(params): Parameters<GetTracksByPriorityParams>,
    ) -> String {
        let target = Priority::from_str_loose(&params.priority);
        let matches: Vec<TrackSummaryResponse> = self
            .tracks
            .values()
            .filter(|t| t.priority == target)
            .map(|t| Self::track_to_summary(t))
            .collect();

        serde_json::to_string_pretty(&matches).unwrap_or_else(|e| format!("Error: {e}"))
    }

    #[tool(
        description = "Get all incomplete (outstanding) tasks across all tracks. Returns the track, phase, and task text for each incomplete task."
    )]
    pub fn get_outstanding_tasks(&self) -> String {
        let mut tasks = Vec::new();
        for track in self.tracks.values() {
            if track.status == Status::Complete {
                continue;
            }
            for phase in &track.plan_phases {
                for task in &phase.tasks {
                    if !task.done {
                        tasks.push(OutstandingTask {
                            track_id: track.id.as_str().to_string(),
                            track_title: track.title.clone(),
                            phase: phase.name.clone(),
                            task: task.text.clone(),
                        });
                    }
                }
            }
        }

        serde_json::to_string_pretty(&tasks).unwrap_or_else(|e| format!("Error: {e}"))
    }

    #[tool(
        description = "Get filesystem paths for a track's directory, plan.md, and metadata files."
    )]
    pub fn get_track_file_paths(
        &self,
        Parameters(params): Parameters<GetTrackFilePathsParams>,
    ) -> String {
        let tracks_dir = self.conductor_dir.join("tracks");
        let track_dir = tracks_dir.join(&params.track_id);

        if !track_dir.exists() {
            return format!("Track directory not found for '{}'", params.track_id);
        }

        let plan_md = track_dir.join("plan.md");
        let metadata_json = track_dir.join("metadata.json");
        let meta_yaml = track_dir.join("meta.yaml");

        let resp = FilePathsResponse {
            track_dir: track_dir.to_string_lossy().to_string(),
            plan_md: plan_md
                .exists()
                .then(|| plan_md.to_string_lossy().to_string()),
            metadata_json: metadata_json
                .exists()
                .then(|| metadata_json.to_string_lossy().to_string()),
            meta_yaml: meta_yaml
                .exists()
                .then(|| meta_yaml.to_string_lossy().to_string()),
        };

        serde_json::to_string_pretty(&resp).unwrap_or_else(|e| format!("Error: {e}"))
    }
}

#[tool_handler]
impl ServerHandler for ConductorService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Conductor Dashboard MCP Server — read-only access to track progress, \
                 statuses, plans, dependencies, and tasks. Use list_tracks to see all tracks, \
                 get_track_detail for full info on a specific track, and get_summary for \
                 aggregate stats."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn conductor_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("conductor")
    }

    #[test]
    fn test_service_loads() {
        let service = ConductorService::new(&conductor_dir()).expect("should load tracks");
        assert!(!service.tracks.is_empty(), "should have tracks");
    }

    #[test]
    fn test_list_tracks_returns_json() {
        let service = ConductorService::new(&conductor_dir()).unwrap();
        let params = ListTracksParams {
            status: None,
            sort: None,
        };
        let result = service.list_tracks(Parameters(params));
        let parsed: Vec<TrackSummaryResponse> =
            serde_json::from_str(&result).expect("should be valid JSON");
        assert!(!parsed.is_empty());
    }

    #[test]
    fn test_list_tracks_filter_by_status() {
        let service = ConductorService::new(&conductor_dir()).unwrap();
        let params = ListTracksParams {
            status: Some("complete".into()),
            sort: None,
        };
        let result = service.list_tracks(Parameters(params));
        let parsed: Vec<TrackSummaryResponse> = serde_json::from_str(&result).unwrap();
        for t in &parsed {
            assert_eq!(t.status, "Complete");
        }
    }

    #[test]
    fn test_get_summary_returns_json() {
        let service = ConductorService::new(&conductor_dir()).unwrap();
        let result = service.get_summary();
        let parsed: SummaryResponse = serde_json::from_str(&result).expect("should be valid JSON");
        assert!(parsed.total_tracks > 0);
        assert_eq!(
            parsed.by_status.new
                + parsed.by_status.in_progress
                + parsed.by_status.blocked
                + parsed.by_status.complete,
            parsed.total_tracks
        );
    }

    #[test]
    fn test_get_track_detail_existing() {
        let service = ConductorService::new(&conductor_dir()).unwrap();
        // Use the first track ID
        let first_id = service.tracks.keys().next().unwrap().as_str().to_string();
        let params = GetTrackDetailParams {
            track_id: first_id.clone(),
        };
        let result = service.get_track_detail(Parameters(params));
        let parsed: TrackDetailResponse =
            serde_json::from_str(&result).expect("should be valid JSON");
        assert_eq!(parsed.id, first_id);
    }

    #[test]
    fn test_get_track_detail_not_found() {
        let service = ConductorService::new(&conductor_dir()).unwrap();
        let params = GetTrackDetailParams {
            track_id: "nonexistent_track_xyz".into(),
        };
        let result = service.get_track_detail(Parameters(params));
        assert!(result.contains("No track found"));
    }

    #[test]
    fn test_search_tracks() {
        let service = ConductorService::new(&conductor_dir()).unwrap();
        // Search for something we know should exist
        let first_track = service.tracks.values().next().unwrap();
        let word = first_track
            .title
            .split_whitespace()
            .next()
            .unwrap_or("test");
        let params = SearchTracksParams {
            query: word.to_string(),
        };
        let result = service.search_tracks(Parameters(params));
        let parsed: Vec<TrackSummaryResponse> = serde_json::from_str(&result).unwrap();
        assert!(!parsed.is_empty());
    }

    #[test]
    fn test_get_dependencies() {
        let service = ConductorService::new(&conductor_dir()).unwrap();
        let params = GetTrackDependenciesParams { track_id: None };
        let result = service.get_track_dependencies(Parameters(params));
        let parsed: Vec<DependencyInfo> =
            serde_json::from_str(&result).expect("should be valid JSON");
        assert_eq!(parsed.len(), service.tracks.len());
    }

    #[test]
    fn test_get_outstanding_tasks() {
        let service = ConductorService::new(&conductor_dir()).unwrap();
        let result = service.get_outstanding_tasks();
        let parsed: Vec<OutstandingTask> =
            serde_json::from_str(&result).expect("should be valid JSON");
        // Should have some outstanding tasks (unless all tracks are complete)
        let has_incomplete = service
            .tracks
            .values()
            .any(|t| t.status != Status::Complete && t.tasks_total > t.tasks_completed);
        if has_incomplete {
            assert!(!parsed.is_empty());
        }
    }

    #[test]
    fn test_get_track_file_paths() {
        let service = ConductorService::new(&conductor_dir()).unwrap();
        let first_id = service.tracks.keys().next().unwrap().as_str().to_string();
        let params = GetTrackFilePathsParams { track_id: first_id };
        let result = service.get_track_file_paths(Parameters(params));
        let parsed: FilePathsResponse =
            serde_json::from_str(&result).expect("should be valid JSON");
        assert!(!parsed.track_dir.is_empty());
    }
}
