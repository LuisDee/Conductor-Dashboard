//! Integration tests for parsers against real conductor data.
//!
//! These tests run against the real `conductor/` directory copied into the repo.
//! They verify that the parsers handle all real-world format variations.

use std::path::PathBuf;

use conductor_dashboard::model::*;
use conductor_dashboard::parser;

fn conductor_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("conductor")
}

// ═══════════════════════════════════════════════════════════════════════════
// Index parser (tracks.md)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_loads_real_tracks_md() {
    let tracks = parser::index::parse_index(&conductor_dir()).unwrap();
    // We have ~85 H2 track entries in the real tracks.md
    assert!(
        tracks.len() >= 80,
        "expected at least 80 tracks, got {}",
        tracks.len()
    );
}

#[test]
fn test_known_complete_track_parsed() {
    let tracks = parser::index::parse_index(&conductor_dir()).unwrap();
    let track = tracks
        .get(&TrackId::new("dashboard_overhaul_20260206"))
        .expect("dashboard_overhaul_20260206 should be in index");

    assert_eq!(track.title, "Dashboard UI Overhaul & Role-Based Views");
    assert_eq!(track.checkbox_status, CheckboxStatus::Checked);
    assert_eq!(track.priority, Priority::High);
}

#[test]
fn test_known_incomplete_track_parsed() {
    let tracks = parser::index::parse_index(&conductor_dir()).unwrap();
    let track = tracks
        .get(&TrackId::new("compliance_enhancements_20260127"))
        .expect("compliance_enhancements_20260127 should be in index");

    assert_eq!(track.title, "Compliance Workflow Enhancements");
    assert_eq!(track.checkbox_status, CheckboxStatus::Unchecked);
}

#[test]
fn test_in_progress_checkbox_parsed() {
    let tracks = parser::index::parse_index(&conductor_dir()).unwrap();
    // [~] or [-] tracks
    let track = tracks
        .get(&TrackId::new("critical_data_integrity_bugs_20260212"))
        .expect("critical_data_integrity_bugs should be in index");

    assert_eq!(track.checkbox_status, CheckboxStatus::InProgress);
}

#[test]
fn test_status_field_overrides_checkbox() {
    let tracks = parser::index::parse_index(&conductor_dir()).unwrap();
    let track = tracks
        .get(&TrackId::new("dashboard_overhaul_20260206"))
        .expect("dashboard_overhaul should be in index");

    // The **Status** field says "Completed" — should override to Complete
    assert_eq!(track.status, Status::Complete);
}

#[test]
fn test_all_tracks_have_ids() {
    let tracks = parser::index::parse_index(&conductor_dir()).unwrap();
    for (id, _track) in &tracks {
        assert!(!id.as_str().is_empty(), "track should have non-empty ID");
    }
}

#[test]
fn test_tags_parsed_for_known_track() {
    let tracks = parser::index::parse_index(&conductor_dir()).unwrap();
    let track = tracks
        .get(&TrackId::new("dashboard_overhaul_20260206"))
        .expect("dashboard_overhaul should be in index");

    assert!(
        track.tags.contains(&"frontend".to_string()),
        "expected 'frontend' in tags, got {:?}",
        track.tags
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Metadata parser (metadata.json / meta.yaml)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_json_schema_a_metadata() {
    let track_dir = conductor_dir()
        .join("tracks")
        .join("critical_data_integrity_bugs_20260212");
    let meta =
        parser::metadata::parse_metadata(&track_dir, "critical_data_integrity_bugs_20260212")
            .unwrap()
            .expect("metadata.json should exist");

    assert_eq!(meta.status, Status::New); // "not_started" → New
    assert!(!meta.tags.is_empty());
    assert!(meta.description.is_some());
}

#[test]
fn test_json_schema_b_metadata() {
    let track_dir = conductor_dir()
        .join("tracks")
        .join("otel_collector_20260212");
    let meta = parser::metadata::parse_metadata(&track_dir, "otel_collector_20260212")
        .unwrap()
        .expect("metadata.json should exist");

    assert_eq!(meta.track_type, TrackType::Feature);
    assert!(meta.created_at.is_some());
}

#[test]
fn test_yaml_metadata() {
    // Use a track that ONLY has meta.yaml (no metadata.json that would take precedence)
    let track_dir = conductor_dir()
        .join("tracks")
        .join("compliance_enhancements_20260127");
    let meta = parser::metadata::parse_metadata(&track_dir, "compliance_enhancements_20260127")
        .unwrap()
        .expect("meta.yaml should exist");

    assert_eq!(meta.status, Status::New); // "not_started" → New
    assert_eq!(meta.priority, Priority::High);
    assert_eq!(meta.branch.as_deref(), Some("DSS-4074"));
}

#[test]
fn test_missing_metadata_returns_none() {
    let track_dir = conductor_dir().join("tracks").join("nonexistent_track");
    let result = parser::metadata::parse_metadata(&track_dir, "nonexistent").unwrap();
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// Plan parser (plan.md)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_otel_plan_has_5_phases() {
    let plan_path = conductor_dir()
        .join("tracks")
        .join("otel_collector_20260212")
        .join("plan.md");
    let phases = parser::plan::parse_plan(&plan_path).unwrap();

    assert_eq!(
        phases.len(),
        5,
        "OTel plan should have 5 phases, got {}: {:?}",
        phases.len(),
        phases.iter().map(|p| &p.name).collect::<Vec<_>>()
    );
}

#[test]
fn test_otel_plan_task_counts() {
    let plan_path = conductor_dir()
        .join("tracks")
        .join("otel_collector_20260212")
        .join("plan.md");
    let phases = parser::plan::parse_plan(&plan_path).unwrap();

    let total_tasks: usize = phases.iter().map(|p| p.tasks.len()).sum();
    assert!(
        total_tasks >= 15,
        "OTel plan should have at least 15 tasks, got {}",
        total_tasks
    );
}

#[test]
fn test_critical_bugs_plan_has_phases() {
    let plan_path = conductor_dir()
        .join("tracks")
        .join("critical_data_integrity_bugs_20260212")
        .join("plan.md");
    let phases = parser::plan::parse_plan(&plan_path).unwrap();

    assert!(!phases.is_empty(), "critical bugs plan should have phases");
}

#[test]
fn test_plan_missing_file_returns_error() {
    let plan_path = conductor_dir()
        .join("tracks")
        .join("nonexistent")
        .join("plan.md");
    let result = parser::plan::parse_plan(&plan_path);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════
// Full pipeline (load_all_tracks)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_load_all_tracks_full_pipeline() {
    let tracks = parser::load_all_tracks(&conductor_dir()).unwrap();

    assert!(
        tracks.len() >= 80,
        "full pipeline should load at least 80 tracks, got {}",
        tracks.len()
    );

    // Spot-check a track with plan data
    let compliance = tracks.get(&TrackId::new("compliance_enhancements_20260127"));
    if let Some(track) = compliance {
        assert!(
            track.tasks_total > 0,
            "compliance_enhancements should have tasks from plan.md"
        );
    }
}

#[test]
fn test_metadata_overrides_index_status() {
    let tracks = parser::load_all_tracks(&conductor_dir()).unwrap();

    // dashboard_overhaul has [x] + "Complete" in tracks.md (→ Complete from index),
    // but meta.yaml says "in_progress" which overrides during merge.
    let dashboard = tracks
        .get(&TrackId::new("dashboard_overhaul_20260206"))
        .expect("dashboard_overhaul should exist");

    assert_eq!(dashboard.status, Status::InProgress);
}

#[test]
fn test_tracks_with_plans_have_task_counts() {
    let tracks = parser::load_all_tracks(&conductor_dir()).unwrap();

    let tracks_with_tasks: Vec<_> = tracks.values().filter(|t| t.tasks_total > 0).collect();

    assert!(
        tracks_with_tasks.len() >= 30,
        "at least 30 tracks should have plan tasks, got {}",
        tracks_with_tasks.len()
    );
}

#[test]
fn test_no_panics_on_full_load() {
    // This test primarily verifies no panic happens during full parsing.
    // If this test passes, all real-world format variations are handled.
    let result = parser::load_all_tracks(&conductor_dir());
    assert!(
        result.is_ok(),
        "full load should not error: {:?}",
        result.err()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_empty_index_content() {
    let entries = parser::index::parse_index_content("");
    assert!(entries.is_empty());
}

#[test]
fn test_parse_index_no_tracks_heading() {
    let entries = parser::index::parse_index_content("# Just a title\n\nSome text.\n");
    assert!(entries.is_empty());
}

#[test]
fn test_parse_plan_with_code_blocks() {
    // Plans with code blocks (before/after examples) should not create phantom tasks
    let md = r#"## Phase 1: Fix Something

### Before
```python
old_code()
```

### After
```python
new_code()
```

- [x] Task: Apply the fix
- [ ] Task: Write tests
"#;
    let phases = parser::plan::parse_plan_content(md);
    assert_eq!(phases.len(), 1);
    assert_eq!(phases[0].tasks.len(), 2);
}

#[test]
fn test_metadata_with_all_defaults() {
    let meta = parser::metadata::parse_json_metadata("{}", "test").unwrap();
    assert_eq!(meta.status, Status::New);
    assert_eq!(meta.priority, Priority::Medium);
    assert!(meta.tags.is_empty());
    assert!(meta.created_at.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge cases — malformed / unusual inputs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_malformed_json_returns_error() {
    let result = parser::metadata::parse_json_metadata("{ not valid json }", "test");
    assert!(result.is_err());
}

#[test]
fn test_json_with_extra_unknown_fields() {
    let json = r#"{
        "status": "in_progress",
        "totally_unknown_field": 42,
        "another_field": [1, 2, 3]
    }"#;
    let meta = parser::metadata::parse_json_metadata(json, "test").unwrap();
    assert_eq!(meta.status, Status::InProgress);
}

#[test]
fn test_yaml_with_minimal_content() {
    let yaml = "status: complete\n";
    let meta = parser::metadata::parse_yaml_metadata(yaml, "test").unwrap();
    assert_eq!(meta.status, Status::Complete);
    assert_eq!(meta.priority, Priority::Medium);
    assert!(meta.tags.is_empty());
}

#[test]
fn test_unicode_track_title_in_index() {
    let md = r#"# Tracks

## [~] Track: Système de Gestion des Données — résumé
*Link: [./conductor/tracks/unicode_track_123/](./conductor/tracks/unicode_track_123/)*
**Priority**: Critical
"#;
    let entries = parser::index::parse_index_content(md);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].title.contains("Système"));
    assert!(entries[0].title.contains("résumé"));
    assert_eq!(entries[0].priority, Priority::Critical);
}

#[test]
fn test_plan_whitespace_only() {
    let phases = parser::plan::parse_plan_content("   \n\n  \t  \n");
    assert!(phases.is_empty());
}

#[test]
fn test_plan_with_deeply_nested_tasks() {
    let md = r#"## Phase 1: Complex Structure

- [x] Top-level task
  - Sub-item description (not a checkbox)
  - More nested text
- [ ] Another top-level task
  - [x] Nested checked item
"#;
    let phases = parser::plan::parse_plan_content(md);
    assert_eq!(phases.len(), 1);
    // Should capture at least the top-level checkbox tasks
    assert!(phases[0].tasks.len() >= 2);
}

#[test]
fn test_index_with_dependencies() {
    let md = r#"# Tracks

## [ ] Track: Depends on Others
*Link: [./conductor/tracks/dependent_track/](./conductor/tracks/dependent_track/)*
**Priority**: High
**Dependencies**: track_a, track_b, track_c
"#;
    let entries = parser::index::parse_index_content(md);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].dependencies.len(), 3);
    assert!(entries[0].dependencies.contains(&"track_a".to_string()));
    assert!(entries[0].dependencies.contains(&"track_b".to_string()));
    assert!(entries[0].dependencies.contains(&"track_c".to_string()));
}

#[test]
fn test_json_null_optional_fields() {
    let json = r#"{
        "id": null,
        "name": null,
        "status": "blocked",
        "created_at": null,
        "dependencies": [],
        "tags": []
    }"#;
    let meta = parser::metadata::parse_json_metadata(json, "test").unwrap();
    assert_eq!(meta.status, Status::Blocked);
    assert!(meta.created_at.is_none());
}

#[test]
fn test_plan_phase_status_computation() {
    let md = r#"## Phase 1: Done Phase
- [x] A
- [x] B

## Phase 2: Active Phase
- [x] C
- [ ] D

## Phase 3: Future Phase
- [ ] E
- [ ] F
"#;
    let phases = parser::plan::parse_plan_content(md);
    assert_eq!(phases.len(), 3);
    assert_eq!(phases[0].status, PhaseStatus::Complete);
    assert_eq!(phases[1].status, PhaseStatus::Active);
    assert_eq!(phases[2].status, PhaseStatus::Pending);
}

// ═══════════════════════════════════════════════════════════════════════════
// Temp directory integration test
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_synthetic_conductor_directory() {
    use std::fs;

    let tmp = std::env::temp_dir().join("conductor_dashboard_test");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("tracks").join("alpha_track")).unwrap();
    fs::create_dir_all(tmp.join("tracks").join("beta_track")).unwrap();

    // Write tracks.md
    fs::write(
        tmp.join("tracks.md"),
        r#"# Tracks

## [x] Track: Alpha Feature ✅ COMPLETE
*Link: [./conductor/tracks/alpha_track/](./conductor/tracks/alpha_track/)*
**Priority**: High
**Status**: Completed

---

## [~] Track: Beta Enhancement
*Link: [./conductor/tracks/beta_track/](./conductor/tracks/beta_track/)*
**Priority**: Medium
**Status**: In_progress
**Tags**: backend, api
"#,
    )
    .unwrap();

    // Write metadata.json for alpha
    fs::write(
        tmp.join("tracks").join("alpha_track").join("metadata.json"),
        r#"{"status": "complete", "type": "feature", "created_at": "2026-01-15T10:00:00Z", "tags": ["frontend"]}"#,
    )
    .unwrap();

    // Write plan.md for alpha
    fs::write(
        tmp.join("tracks").join("alpha_track").join("plan.md"),
        r#"## Phase 1: Setup
- [x] Create project
- [x] Add deps

## Phase 2: Build
- [x] Implement core
- [x] Write tests
"#,
    )
    .unwrap();

    // Write meta.yaml for beta
    fs::write(
        tmp.join("tracks").join("beta_track").join("meta.yaml"),
        "name: Beta Enhancement\nstatus: in_progress\npriority: high\ncreated: 2026-02-01\nbranch: feat/beta\ntags:\n  - backend\n  - api\n",
    )
    .unwrap();

    // Write plan.md for beta
    fs::write(
        tmp.join("tracks").join("beta_track").join("plan.md"),
        r#"## Phase 1: Foundation
- [x] Setup database
- [x] Create models

## Phase 2: API Layer
- [x] Build endpoints
- [ ] Add validation

## Phase 3: Testing
- [ ] Unit tests
- [ ] Integration tests
"#,
    )
    .unwrap();

    // Parse everything
    let tracks = parser::load_all_tracks(&tmp).unwrap();
    assert_eq!(tracks.len(), 2);

    // Verify alpha
    let alpha = tracks.get(&TrackId::new("alpha_track")).unwrap();
    assert_eq!(alpha.title, "Alpha Feature");
    assert_eq!(alpha.status, Status::Complete);
    assert_eq!(alpha.tasks_total, 4);
    assert_eq!(alpha.tasks_completed, 4);
    assert!((alpha.progress_percent() - 100.0).abs() < f32::EPSILON);
    assert_eq!(alpha.plan_phases.len(), 2);

    // Verify beta
    let beta = tracks.get(&TrackId::new("beta_track")).unwrap();
    assert_eq!(beta.title, "Beta Enhancement");
    assert_eq!(beta.status, Status::InProgress);
    assert_eq!(beta.priority, Priority::High);
    assert_eq!(beta.tasks_total, 6);
    assert_eq!(beta.tasks_completed, 3);
    assert!((beta.progress_percent() - 50.0).abs() < f32::EPSILON);
    assert_eq!(beta.branch.as_deref(), Some("feat/beta"));
    assert_eq!(beta.plan_phases.len(), 3);
    assert_eq!(beta.plan_phases[0].status, PhaseStatus::Complete);
    assert_eq!(beta.plan_phases[1].status, PhaseStatus::Active);
    assert_eq!(beta.plan_phases[2].status, PhaseStatus::Pending);

    // Cleanup
    let _ = fs::remove_dir_all(&tmp);
}
