//! Model unit tests — enums, track state, filter/sort, merge logic.

use conductor_dashboard::model::*;

// ═══════════════════════════════════════════════════════════════════════════
// Enum cycling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_filter_mode_cycles() {
    assert_eq!(FilterMode::All.next(), FilterMode::Active);
    assert_eq!(FilterMode::Active.next(), FilterMode::Blocked);
    assert_eq!(FilterMode::Blocked.next(), FilterMode::Complete);
    assert_eq!(FilterMode::Complete.next(), FilterMode::All);
}

#[test]
fn test_sort_mode_toggles() {
    assert_eq!(SortMode::Updated.next(), SortMode::Progress);
    assert_eq!(SortMode::Progress.next(), SortMode::Updated);
}

// ═══════════════════════════════════════════════════════════════════════════
// Status parsing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_status_from_str_loose() {
    assert_eq!(Status::from_str_loose("complete"), Status::Complete);
    assert_eq!(Status::from_str_loose("Completed"), Status::Complete);
    assert_eq!(Status::from_str_loose("DONE"), Status::Complete);
    assert_eq!(Status::from_str_loose("in_progress"), Status::InProgress);
    assert_eq!(Status::from_str_loose("in-progress"), Status::InProgress);
    assert_eq!(Status::from_str_loose("active"), Status::InProgress);
    assert_eq!(Status::from_str_loose("implementation"), Status::InProgress);
    assert_eq!(Status::from_str_loose("blocked"), Status::Blocked);
    assert_eq!(Status::from_str_loose("on_hold"), Status::Blocked);
    assert_eq!(Status::from_str_loose("not_started"), Status::New);
    assert_eq!(Status::from_str_loose("new"), Status::New);
    assert_eq!(Status::from_str_loose("planning"), Status::New);
    assert_eq!(Status::from_str_loose("planned"), Status::New);
    assert_eq!(Status::from_str_loose("unknown_value"), Status::New);
    assert_eq!(Status::from_str_loose(""), Status::New);
}

#[test]
fn test_priority_from_str_loose() {
    assert_eq!(Priority::from_str_loose("critical"), Priority::Critical);
    assert_eq!(Priority::from_str_loose("high"), Priority::High);
    assert_eq!(Priority::from_str_loose("medium"), Priority::Medium);
    assert_eq!(Priority::from_str_loose("med"), Priority::Medium);
    assert_eq!(Priority::from_str_loose("low"), Priority::Low);
    assert_eq!(Priority::from_str_loose("unknown"), Priority::Medium);
}

#[test]
fn test_track_type_from_str_loose() {
    assert_eq!(TrackType::from_str_loose("feature"), TrackType::Feature);
    assert_eq!(TrackType::from_str_loose("feat"), TrackType::Feature);
    assert_eq!(TrackType::from_str_loose("bug"), TrackType::Bug);
    assert_eq!(TrackType::from_str_loose("bugfix"), TrackType::Bug);
    assert_eq!(TrackType::from_str_loose("fix"), TrackType::Bug);
    assert_eq!(TrackType::from_str_loose("migration"), TrackType::Migration);
    assert_eq!(TrackType::from_str_loose("refactor"), TrackType::Refactor);
    assert_eq!(TrackType::from_str_loose("other"), TrackType::Other);
}

#[test]
fn test_checkbox_to_status() {
    assert_eq!(CheckboxStatus::Unchecked.to_status(), Status::New);
    assert_eq!(CheckboxStatus::InProgress.to_status(), Status::InProgress);
    assert_eq!(CheckboxStatus::Checked.to_status(), Status::Complete);
}

// ═══════════════════════════════════════════════════════════════════════════
// Track progress
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_track_progress_zero_tasks() {
    let track = Track::default();
    assert!((track.progress_percent() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_track_progress_partial() {
    let track = Track {
        tasks_total: 10,
        tasks_completed: 3,
        ..Track::default()
    };
    assert!((track.progress_percent() - 30.0).abs() < 0.01);
}

#[test]
fn test_track_progress_full() {
    let track = Track {
        tasks_total: 5,
        tasks_completed: 5,
        ..Track::default()
    };
    assert!((track.progress_percent() - 100.0).abs() < f32::EPSILON);
}

#[test]
fn test_track_is_complete_by_status() {
    let track = Track {
        status: Status::Complete,
        ..Track::default()
    };
    assert!(track.is_complete());
}

#[test]
fn test_track_is_complete_by_tasks() {
    let track = Track {
        status: Status::InProgress,
        tasks_total: 5,
        tasks_completed: 5,
        ..Track::default()
    };
    assert!(track.is_complete());
}

#[test]
fn test_track_not_complete() {
    let track = Track {
        status: Status::InProgress,
        tasks_total: 5,
        tasks_completed: 3,
        ..Track::default()
    };
    assert!(!track.is_complete());
}

// ═══════════════════════════════════════════════════════════════════════════
// Track merge
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_merge_metadata_overrides_non_defaults() {
    let mut track = Track {
        id: TrackId::new("test"),
        title: "Test Track".to_string(),
        status: Status::New,
        priority: Priority::Medium,
        ..Track::default()
    };

    let meta = TrackMetadata {
        status: Status::InProgress,
        priority: Priority::High,
        track_type: TrackType::Feature,
        branch: Some("feat/test".to_string()),
        tags: vec!["backend".to_string()],
        ..TrackMetadata::default()
    };

    track.merge_metadata(meta);

    assert_eq!(track.status, Status::InProgress);
    assert_eq!(track.priority, Priority::High);
    assert_eq!(track.track_type, TrackType::Feature);
    assert_eq!(track.branch.as_deref(), Some("feat/test"));
    assert_eq!(track.tags, vec!["backend"]);
}

#[test]
fn test_merge_metadata_keeps_defaults_when_meta_is_default() {
    let mut track = Track {
        status: Status::Complete,
        priority: Priority::Critical,
        ..Track::default()
    };

    // Default metadata should not override non-default track fields
    let meta = TrackMetadata::default();
    track.merge_metadata(meta);

    // status=New and priority=Medium are defaults, so they don't override
    assert_eq!(track.status, Status::Complete);
    assert_eq!(track.priority, Priority::Critical);
}

#[test]
fn test_merge_plan_updates_task_counts() {
    let mut track = Track::default();

    let phases = vec![
        PlanPhase {
            name: "Phase 1".to_string(),
            status: PhaseStatus::Complete,
            tasks: vec![
                PlanTask {
                    text: "A".to_string(),
                    done: true,
                },
                PlanTask {
                    text: "B".to_string(),
                    done: true,
                },
            ],
        },
        PlanPhase {
            name: "Phase 2".to_string(),
            status: PhaseStatus::Active,
            tasks: vec![
                PlanTask {
                    text: "C".to_string(),
                    done: true,
                },
                PlanTask {
                    text: "D".to_string(),
                    done: false,
                },
                PlanTask {
                    text: "E".to_string(),
                    done: false,
                },
            ],
        },
    ];

    track.merge_plan(phases);

    assert_eq!(track.tasks_total, 5);
    assert_eq!(track.tasks_completed, 3);
    assert_eq!(track.plan_phases.len(), 2);
    assert_eq!(track.phase, "Phase 2");
}

// ═══════════════════════════════════════════════════════════════════════════
// PlanPhase progress
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_plan_phase_progress_empty() {
    let phase = PlanPhase {
        name: "Empty".to_string(),
        status: PhaseStatus::Pending,
        tasks: Vec::new(),
    };
    assert!((phase.progress_percent() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_plan_phase_progress_partial() {
    let phase = PlanPhase {
        name: "Partial".to_string(),
        status: PhaseStatus::Active,
        tasks: vec![
            PlanTask {
                text: "A".to_string(),
                done: true,
            },
            PlanTask {
                text: "B".to_string(),
                done: false,
            },
            PlanTask {
                text: "C".to_string(),
                done: false,
            },
            PlanTask {
                text: "D".to_string(),
                done: false,
            },
        ],
    };
    assert!((phase.progress_percent() - 25.0).abs() < f32::EPSILON);
    assert_eq!(phase.tasks_completed(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Display / label
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_status_labels() {
    assert_eq!(Status::New.label(), "New");
    assert_eq!(Status::InProgress.label(), "Active");
    assert_eq!(Status::Blocked.label(), "Blocked");
    assert_eq!(Status::Complete.label(), "Complete");
}

#[test]
fn test_priority_labels() {
    assert_eq!(Priority::Critical.label(), "CRITICAL");
    assert_eq!(Priority::High.label(), "HIGH");
    assert_eq!(Priority::Medium.label(), "MEDIUM");
    assert_eq!(Priority::Low.label(), "LOW");
}

#[test]
fn test_filter_mode_labels() {
    assert_eq!(FilterMode::All.label(), "All");
    assert_eq!(FilterMode::Active.label(), "Active");
    assert_eq!(FilterMode::Blocked.label(), "Blocked");
    assert_eq!(FilterMode::Complete.label(), "Done");
}

#[test]
fn test_sort_mode_labels() {
    assert_eq!(SortMode::Updated.label(), "Recent");
    assert_eq!(SortMode::Progress.label(), "Progress");
}

#[test]
fn test_track_id_display() {
    let id = TrackId::new("my_track_123");
    assert_eq!(format!("{}", id), "my_track_123");
    assert_eq!(id.as_str(), "my_track_123");
}

#[test]
fn test_track_id_from_string() {
    let id: TrackId = "test_track".into();
    assert_eq!(id.as_str(), "test_track");

    let id2: TrackId = String::from("another").into();
    assert_eq!(id2.as_str(), "another");
}

// ═══════════════════════════════════════════════════════════════════════════
// Priority ordering
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_priority_ordering() {
    assert!(Priority::Critical < Priority::High);
    assert!(Priority::High < Priority::Medium);
    assert!(Priority::Medium < Priority::Low);
}
