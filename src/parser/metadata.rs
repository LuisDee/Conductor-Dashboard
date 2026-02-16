//! Parse `metadata.json` and `meta.yaml` — per-track metadata files.
//!
//! Two schemas exist in the wild:
//!   Schema A (older): { id, name, status, owner, start_date, end_date, description, dependencies, tags }
//!   Schema B (newer): { track_id, type, status, created_at, updated_at, description }
//!   YAML format:      { name, status, priority, created, branch, tags, completed, commits }
//!
//! We handle all three with serde defaults so missing fields are fine.

use std::path::Path;

use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;

use crate::model::{Priority, Status, TrackMetadata, TrackType};
use crate::parser::error::ParseError;

// ---------------------------------------------------------------------------
// JSON deserialization (handles both schema A and B)
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug, Default)]
#[allow(dead_code)]
struct RawJsonMetadata {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    track_id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    status: Option<Status>,
    #[serde(default)]
    priority: Option<Priority>,
    #[serde(default, rename = "type")]
    track_type: Option<TrackType>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    end_date: Option<String>,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    owner: Option<String>,
}

// ---------------------------------------------------------------------------
// YAML deserialization
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug, Default)]
#[allow(dead_code)]
struct RawYamlMetadata {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    status: Option<Status>,
    #[serde(default)]
    priority: Option<Priority>,
    #[serde(default)]
    created: Option<String>,
    #[serde(default)]
    completed: Option<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    commits: Vec<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Try to load metadata for a track. Tries metadata.json first, then meta.yaml.
/// Returns Ok(None) if neither file exists.
pub fn parse_metadata(
    track_dir: &Path,
    track_id: &str,
) -> Result<Option<TrackMetadata>, ParseError> {
    let json_path = track_dir.join("metadata.json");
    let yaml_path = track_dir.join("meta.yaml");

    if json_path.exists() {
        let content = std::fs::read_to_string(&json_path).map_err(|e| ParseError::Io {
            path: json_path.clone(),
            source: e,
        })?;
        return parse_json_metadata(&content, track_id).map(Some);
    }

    if yaml_path.exists() {
        let content = std::fs::read_to_string(&yaml_path).map_err(|e| ParseError::Io {
            path: yaml_path.clone(),
            source: e,
        })?;
        return parse_yaml_metadata(&content, track_id).map(Some);
    }

    Ok(None)
}

/// Parse JSON metadata content.
pub fn parse_json_metadata(content: &str, track_id: &str) -> Result<TrackMetadata, ParseError> {
    let raw: RawJsonMetadata =
        serde_json::from_str(content).map_err(|e| ParseError::MetadataInvalid {
            track_id: track_id.to_string(),
            message: e.to_string(),
        })?;

    let created_at = raw
        .created_at
        .as_deref()
        .or(raw.start_date.as_deref())
        .and_then(parse_datetime);

    let updated_at = raw
        .updated_at
        .as_deref()
        .or(raw.end_date.as_deref())
        .and_then(parse_datetime);

    Ok(TrackMetadata {
        status: raw.status.unwrap_or_default(),
        priority: raw.priority.unwrap_or_default(),
        track_type: raw.track_type.unwrap_or_default(),
        created_at,
        updated_at,
        dependencies: raw.dependencies,
        tags: raw.tags,
        branch: raw.branch,
        description: raw.description,
    })
}

/// Parse YAML metadata content.
pub fn parse_yaml_metadata(content: &str, track_id: &str) -> Result<TrackMetadata, ParseError> {
    let raw: RawYamlMetadata =
        serde_yaml::from_str(content).map_err(|e| ParseError::MetadataInvalid {
            track_id: track_id.to_string(),
            message: e.to_string(),
        })?;

    let created_at = raw.created.as_deref().and_then(parse_datetime);
    let updated_at = raw.completed.as_deref().and_then(parse_datetime);

    Ok(TrackMetadata {
        status: raw.status.unwrap_or_default(),
        priority: raw.priority.unwrap_or_default(),
        track_type: TrackType::Other,
        created_at,
        updated_at,
        dependencies: Vec::new(),
        tags: raw.tags,
        branch: raw.branch,
        description: None,
    })
}

/// Parse a datetime string flexibly. Handles:
/// - ISO 8601: `2026-02-12T14:45:00Z`
/// - Date only: `2026-02-04`
/// - Date with parens: `(2026-02-06)` → strip parens
fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim().trim_matches('(').trim_matches(')').trim();

    // Try ISO 8601 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }

    // Try date-only
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_schema_a() {
        let json = r#"{
            "id": "critical_data_integrity_bugs_20260212",
            "name": "Critical Data Integrity Bug Fixes",
            "status": "not_started",
            "owner": null,
            "start_date": null,
            "end_date": null,
            "description": "Fix all verified CRITICAL and HIGH logic bugs.",
            "dependencies": [],
            "tags": ["bugs", "data-integrity"]
        }"#;
        let meta = parse_json_metadata(json, "test").unwrap();
        assert_eq!(meta.status, Status::New);
        assert_eq!(meta.tags, vec!["bugs", "data-integrity"]);
        assert!(meta.description.is_some());
    }

    #[test]
    fn test_parse_json_schema_b() {
        let json = r#"{
            "track_id": "otel_collector_20260212",
            "type": "feature",
            "status": "new",
            "created_at": "2026-02-12T14:45:00Z",
            "updated_at": "2026-02-12T14:45:00Z",
            "description": "Add OTel Collector service."
        }"#;
        let meta = parse_json_metadata(json, "test").unwrap();
        assert_eq!(meta.status, Status::New);
        assert_eq!(meta.track_type, TrackType::Feature);
        assert!(meta.created_at.is_some());
    }

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
name: Dashboard Summary Latency Fix
status: in_progress
priority: high
created: 2026-02-04
branch: DSS-4074
tags:
  - performance
  - bug-fix
"#;
        let meta = parse_yaml_metadata(yaml, "test").unwrap();
        assert_eq!(meta.status, Status::InProgress);
        assert_eq!(meta.priority, Priority::High);
        assert!(meta.created_at.is_some());
        assert_eq!(meta.branch.as_deref(), Some("DSS-4074"));
        assert_eq!(meta.tags.len(), 2);
    }

    #[test]
    fn test_parse_datetime_iso() {
        let dt = parse_datetime("2026-02-12T14:45:00Z").unwrap();
        assert_eq!(dt.year(), 2026);
    }

    #[test]
    fn test_parse_datetime_date_only() {
        let dt = parse_datetime("2026-02-04").unwrap();
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 2);
        assert_eq!(dt.day(), 4);
    }

    #[test]
    fn test_parse_datetime_invalid() {
        assert!(parse_datetime("not a date").is_none());
        assert!(parse_datetime("").is_none());
    }

    #[test]
    fn test_empty_json() {
        let meta = parse_json_metadata("{}", "test").unwrap();
        assert_eq!(meta.status, Status::New);
        assert_eq!(meta.priority, Priority::Medium);
    }

    use chrono::Datelike;
}
