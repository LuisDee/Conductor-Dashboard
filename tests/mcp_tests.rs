use std::path::PathBuf;

use conductor_dashboard::mcp::service::ConductorService;
use conductor_dashboard::mcp::types::*;
use rmcp::handler::server::wrapper::Parameters;

fn conductor_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("conductor")
}

fn service() -> ConductorService {
    ConductorService::new(&conductor_dir()).expect("should load tracks from real conductor dir")
}

// ---------------------------------------------------------------------------
// list_tracks
// ---------------------------------------------------------------------------

#[test]
fn test_list_all_tracks() {
    let svc = service();
    let result = svc.list_tracks(Parameters(ListTracksParams {
        status: None,
        sort: None,
    }));
    let tracks: Vec<TrackSummaryResponse> =
        serde_json::from_str(&result).expect("valid JSON array");
    assert!(!tracks.is_empty(), "should have at least one track");
}

#[test]
fn test_list_tracks_sort_by_progress() {
    let svc = service();
    let result = svc.list_tracks(Parameters(ListTracksParams {
        status: None,
        sort: Some("progress".into()),
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&result).unwrap();
    // Verify descending progress order
    for window in tracks.windows(2) {
        assert!(
            window[0].progress_percent >= window[1].progress_percent,
            "expected descending progress: {} >= {}",
            window[0].progress_percent,
            window[1].progress_percent,
        );
    }
}

#[test]
fn test_list_tracks_filter_new() {
    let svc = service();
    let result = svc.list_tracks(Parameters(ListTracksParams {
        status: Some("new".into()),
        sort: None,
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&result).unwrap();
    for t in &tracks {
        assert_eq!(t.status, "New", "expected status New, got {}", t.status);
    }
}

#[test]
fn test_list_tracks_filter_in_progress() {
    let svc = service();
    let result = svc.list_tracks(Parameters(ListTracksParams {
        status: Some("in_progress".into()),
        sort: None,
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&result).unwrap();
    for t in &tracks {
        assert_eq!(t.status, "Active", "expected Active, got {}", t.status);
    }
}

// ---------------------------------------------------------------------------
// get_summary
// ---------------------------------------------------------------------------

#[test]
fn test_summary_status_counts_add_up() {
    let svc = service();
    let result = svc.get_summary();
    let summary: SummaryResponse = serde_json::from_str(&result).unwrap();
    let sum = summary.by_status.new
        + summary.by_status.in_progress
        + summary.by_status.blocked
        + summary.by_status.complete;
    assert_eq!(
        sum, summary.total_tracks,
        "status counts should sum to total"
    );
}

#[test]
fn test_summary_progress_bounded() {
    let svc = service();
    let result = svc.get_summary();
    let summary: SummaryResponse = serde_json::from_str(&result).unwrap();
    assert!(
        summary.overall_progress >= 0.0 && summary.overall_progress <= 100.0,
        "progress should be 0-100, got {}",
        summary.overall_progress
    );
}

// ---------------------------------------------------------------------------
// get_track_detail
// ---------------------------------------------------------------------------

#[test]
fn test_detail_has_plan_phases() {
    let svc = service();
    // Get first track with plan phases
    let all = svc.list_tracks(Parameters(ListTracksParams {
        status: None,
        sort: None,
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&all).unwrap();

    // Try to find a track with tasks
    if let Some(t) = tracks.iter().find(|t| t.tasks_total > 0) {
        let result = svc.get_track_detail(Parameters(GetTrackDetailParams {
            track_id: t.id.clone(),
        }));
        let detail: TrackDetailResponse = serde_json::from_str(&result).unwrap();
        assert!(!detail.plan_phases.is_empty(), "should have plan phases");
        assert_eq!(detail.id, t.id);
    }
}

#[test]
fn test_detail_substring_match() {
    let svc = service();
    // Get first track, use partial ID
    let all = svc.list_tracks(Parameters(ListTracksParams {
        status: None,
        sort: None,
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&all).unwrap();
    let first = &tracks[0];

    // Use first 10 chars as substring
    let partial = &first.id[..first.id.len().min(10)];
    let result = svc.get_track_detail(Parameters(GetTrackDetailParams {
        track_id: partial.to_string(),
    }));
    // Should either find exactly one or report multiple matches
    assert!(
        !result.contains("No track found"),
        "substring should match at least one track"
    );
}

// ---------------------------------------------------------------------------
// search_tracks
// ---------------------------------------------------------------------------

#[test]
fn test_search_by_id_substring() {
    let svc = service();
    let all = svc.list_tracks(Parameters(ListTracksParams {
        status: None,
        sort: None,
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&all).unwrap();
    let first = &tracks[0];

    let result = svc.search_tracks(Parameters(SearchTracksParams {
        query: first.id.clone(),
    }));
    let matches: Vec<TrackSummaryResponse> = serde_json::from_str(&result).unwrap();
    assert!(!matches.is_empty(), "should find track by exact ID");
    assert!(matches.iter().any(|m| m.id == first.id));
}

#[test]
fn test_search_case_insensitive() {
    let svc = service();
    let all = svc.list_tracks(Parameters(ListTracksParams {
        status: None,
        sort: None,
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&all).unwrap();
    let first = &tracks[0];
    let upper_title = first.title.to_uppercase();

    let result = svc.search_tracks(Parameters(SearchTracksParams { query: upper_title }));
    let matches: Vec<TrackSummaryResponse> = serde_json::from_str(&result).unwrap();
    assert!(!matches.is_empty(), "search should be case-insensitive");
}

#[test]
fn test_search_no_results() {
    let svc = service();
    let result = svc.search_tracks(Parameters(SearchTracksParams {
        query: "zzz_nonexistent_query_xyz_12345".into(),
    }));
    let matches: Vec<TrackSummaryResponse> = serde_json::from_str(&result).unwrap();
    assert!(matches.is_empty(), "should find no results for gibberish");
}

// ---------------------------------------------------------------------------
// get_track_dependencies
// ---------------------------------------------------------------------------

#[test]
fn test_dependencies_all_tracks() {
    let svc = service();
    let result =
        svc.get_track_dependencies(Parameters(GetTrackDependenciesParams { track_id: None }));
    let deps: Vec<DependencyInfo> = serde_json::from_str(&result).unwrap();
    // Should have one entry per track
    let all = svc.list_tracks(Parameters(ListTracksParams {
        status: None,
        sort: None,
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&all).unwrap();
    assert_eq!(deps.len(), tracks.len());
}

#[test]
fn test_dependencies_single_track() {
    let svc = service();
    let all = svc.list_tracks(Parameters(ListTracksParams {
        status: None,
        sort: None,
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&all).unwrap();
    let first = &tracks[0];

    let result = svc.get_track_dependencies(Parameters(GetTrackDependenciesParams {
        track_id: Some(first.id.clone()),
    }));
    let deps: Vec<DependencyInfo> = serde_json::from_str(&result).unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].track_id, first.id);
}

#[test]
fn test_dependencies_nonexistent() {
    let svc = service();
    let result = svc.get_track_dependencies(Parameters(GetTrackDependenciesParams {
        track_id: Some("nonexistent_xyz".into()),
    }));
    assert!(result.contains("No track found"));
}

// ---------------------------------------------------------------------------
// get_tracks_by_priority
// ---------------------------------------------------------------------------

#[test]
fn test_filter_by_priority() {
    let svc = service();
    let result = svc.get_tracks_by_priority(Parameters(GetTracksByPriorityParams {
        priority: "high".into(),
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&result).unwrap();
    for t in &tracks {
        assert_eq!(t.priority, "HIGH");
    }
}

// ---------------------------------------------------------------------------
// get_outstanding_tasks
// ---------------------------------------------------------------------------

#[test]
fn test_outstanding_tasks_are_incomplete() {
    let svc = service();
    let result = svc.get_outstanding_tasks();
    let tasks: Vec<OutstandingTask> = serde_json::from_str(&result).unwrap();
    // All returned tasks should be from non-complete tracks
    let summary_result = svc.get_summary();
    let summary: SummaryResponse = serde_json::from_str(&summary_result).unwrap();
    if summary.total_tasks_completed < summary.total_tasks {
        assert!(
            !tasks.is_empty(),
            "should have outstanding tasks when not all complete"
        );
    }
}

// ---------------------------------------------------------------------------
// get_track_file_paths
// ---------------------------------------------------------------------------

#[test]
fn test_file_paths_existing_track() {
    let svc = service();
    let all = svc.list_tracks(Parameters(ListTracksParams {
        status: None,
        sort: None,
    }));
    let tracks: Vec<TrackSummaryResponse> = serde_json::from_str(&all).unwrap();
    let first = &tracks[0];

    let result = svc.get_track_file_paths(Parameters(GetTrackFilePathsParams {
        track_id: first.id.clone(),
    }));
    let paths: FilePathsResponse = serde_json::from_str(&result).unwrap();
    assert!(paths.track_dir.contains(&first.id));
}

#[test]
fn test_file_paths_nonexistent() {
    let svc = service();
    let result = svc.get_track_file_paths(Parameters(GetTrackFilePathsParams {
        track_id: "nonexistent_xyz".into(),
    }));
    assert!(result.contains("not found"));
}
