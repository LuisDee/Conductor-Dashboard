use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Tool parameter types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTracksParams {
    /// Filter by status: "new", "in_progress", "blocked", "complete", or "all" (default)
    #[schemars(default)]
    pub status: Option<String>,
    /// Sort by: "updated" (default) or "progress"
    #[schemars(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTrackDetailParams {
    /// The track ID (directory name), e.g. "otel_observability_20260210"
    pub track_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchTracksParams {
    /// Search query — matches against title, ID, or tags (case-insensitive)
    pub query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTrackDependenciesParams {
    /// Optional track ID to get dependencies for a specific track. If omitted, returns all.
    #[schemars(default)]
    pub track_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTracksByTagParams {
    /// Tag to filter by (case-insensitive)
    pub tag: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTracksByPriorityParams {
    /// Priority level: "critical", "high", "medium", or "low"
    pub priority: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTrackFilePathsParams {
    /// The track ID
    pub track_id: String,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackSummaryResponse {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub track_type: String,
    pub progress_percent: f32,
    pub tasks_completed: usize,
    pub tasks_total: usize,
    pub tags: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackDetailResponse {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub track_type: String,
    pub phase: String,
    pub progress_percent: f32,
    pub tasks_completed: usize,
    pub tasks_total: usize,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub branch: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub plan_phases: Vec<PhaseResponse>,
    pub file_paths: FilePathsResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhaseResponse {
    pub name: String,
    pub status: String,
    pub tasks_completed: usize,
    pub tasks_total: usize,
    pub progress_percent: f32,
    pub tasks: Vec<TaskResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub text: String,
    pub done: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SummaryResponse {
    pub total_tracks: usize,
    pub by_status: StatusCounts,
    pub overall_progress: f32,
    pub total_tasks: usize,
    pub total_tasks_completed: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCounts {
    pub new: usize,
    pub in_progress: usize,
    pub blocked: usize,
    pub complete: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub track_id: String,
    pub title: String,
    pub status: String,
    pub depends_on: Vec<String>,
    pub blocks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutstandingTask {
    pub track_id: String,
    pub track_title: String,
    pub phase: String,
    pub task: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilePathsResponse {
    pub track_dir: String,
    pub plan_md: Option<String>,
    pub metadata_json: Option<String>,
    pub meta_yaml: Option<String>,
}
