//! Parse `tracks.md` — the master registry of all tracks.
//!
//! Uses pulldown-cmark to walk the markdown AST rather than fragile regexes.
//! Each H2 heading with the pattern `[x] Track: Title` starts a new track entry.
//! The body below each H2 contains metadata lines (`**Priority**: High`, etc.)
//! and an optional description.

use std::collections::BTreeMap;
use std::path::Path;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::model::{CheckboxStatus, Priority, Status, Track, TrackId};
use crate::parser::error::ParseError;

/// Result of parsing a single track entry from tracks.md.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub id: TrackId,
    pub title: String,
    pub checkbox: CheckboxStatus,
    pub status: Status,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub branch: Option<String>,
    pub dependencies: Vec<String>,
}

/// Parse `tracks.md` from the given conductor directory.
/// Returns a map of TrackId → Track (with only index-level data populated).
pub fn parse_index(conductor_dir: &Path) -> Result<BTreeMap<TrackId, Track>, ParseError> {
    let index_path = conductor_dir.join("tracks.md");
    let content = std::fs::read_to_string(&index_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ParseError::IndexNotFound(index_path.clone())
        } else {
            ParseError::Io {
                path: index_path.clone(),
                source: e,
            }
        }
    })?;

    let entries = parse_index_content(&content);

    let mut tracks = BTreeMap::new();
    for entry in entries {
        let status = if entry.status != Status::New {
            entry.status
        } else {
            entry.checkbox.to_status()
        };

        let track = Track {
            id: entry.id.clone(),
            title: entry.title,
            status,
            priority: entry.priority,
            checkbox_status: entry.checkbox,
            tags: entry.tags,
            branch: entry.branch,
            dependencies: entry.dependencies.into_iter().map(TrackId::new).collect(),
            ..Track::default()
        };
        tracks.insert(entry.id, track);
    }

    Ok(tracks)
}

/// Parse the raw markdown content of tracks.md into index entries.
/// This is the core logic, separated for testability.
pub fn parse_index_content(content: &str) -> Vec<IndexEntry> {
    let opts = Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(content, opts);

    let mut entries = Vec::new();
    let mut in_h2 = false;
    let mut h2_text = String::new();
    let mut current_entry: Option<IndexEntry> = None;
    let mut body_text = String::new();
    let mut in_paragraph = false;
    let mut in_strong = false;
    let mut strong_text = String::new();
    let mut field_key: Option<String> = None;

    for event in parser {
        match event {
            // Start of an H2 heading
            Event::Start(Tag::Heading { level: HeadingLevel::H2, .. }) => {
                // Flush previous entry
                if let Some(entry) = current_entry.take() {
                    entries.push(entry);
                }
                in_h2 = true;
                h2_text.clear();
                body_text.clear();
            }

            // End of H2 heading — parse the heading text
            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                in_h2 = false;
                if let Some(entry) = parse_h2_heading(&h2_text) {
                    current_entry = Some(entry);
                }
            }

            // Bold text (for field keys like **Priority**)
            Event::Start(Tag::Strong) => {
                in_strong = true;
                strong_text.clear();
            }
            Event::End(TagEnd::Strong) => {
                in_strong = false;
                if current_entry.is_some() {
                    // Check if this is a field key like "Priority", "Status", etc.
                    let key = strong_text.trim_end_matches(':').trim().to_string();
                    field_key = Some(key);
                }
            }

            // Paragraph boundaries
            Event::Start(Tag::Paragraph) => {
                in_paragraph = true;
            }
            Event::End(TagEnd::Paragraph) => {
                in_paragraph = false;
                field_key = None;
            }

            // Italic text (for Link lines: *Link: [...]*)
            Event::Start(Tag::Emphasis) => {}
            Event::End(TagEnd::Emphasis) => {}

            // Links — extract track ID from link target (first link only)
            Event::Start(Tag::Link { dest_url, .. }) => {
                if let Some(ref mut entry) = current_entry {
                    if entry.id.as_str().is_empty() {
                        if let Some(track_id) = extract_track_id_from_link(&dest_url) {
                            entry.id = TrackId::new(track_id);
                        }
                    }
                }
            }

            // Text content
            Event::Text(text) => {
                if in_h2 {
                    h2_text.push_str(&text);
                } else if in_strong {
                    strong_text.push_str(&text);
                } else if let Some(ref mut entry) = current_entry {
                    if in_paragraph {
                        // Process field values
                        if let Some(ref key) = field_key {
                            let value = text.trim();
                            if value.starts_with(':') {
                                let value = value.trim_start_matches(':').trim();
                                apply_field(entry, key, value);
                                field_key = None;
                            } else if !value.is_empty() {
                                apply_field(entry, key, value);
                                field_key = None;
                            }
                        }
                    }
                }
            }

            // Thematic break (---) between tracks — not structurally important
            Event::Rule => {}

            _ => {}
        }
    }

    // Flush last entry
    if let Some(entry) = current_entry {
        entries.push(entry);
    }

    entries
}

/// Parse an H2 heading line like `[x] Track: Dashboard UI Overhaul ✅ COMPLETE`
fn parse_h2_heading(text: &str) -> Option<IndexEntry> {
    let text = text.trim();

    // Must contain "Track:" to be a track entry
    // Some headings are section headers like "## Autopsy Remediation Tracks"
    let track_marker = text.find("Track:")?;

    // Parse checkbox: [x], [ ], [~], [-]
    let checkbox = if text.starts_with("[x]") || text.starts_with("[X]") {
        CheckboxStatus::Checked
    } else if text.starts_with("[~]") || text.starts_with("[-]") {
        CheckboxStatus::InProgress
    } else if text.starts_with("[ ]") {
        CheckboxStatus::Unchecked
    } else {
        CheckboxStatus::Unchecked
    };

    // Extract title: everything after "Track:" until ✅ or end
    let after_track = &text[track_marker + "Track:".len()..];
    let title = after_track
        .split('✅')
        .next()
        .unwrap_or(after_track)
        .trim()
        .to_string();

    if title.is_empty() {
        return None;
    }

    Some(IndexEntry {
        id: TrackId::new(""), // will be filled from link
        title,
        checkbox,
        status: Status::New, // will be overridden from **Status** field
        priority: Priority::Medium,
        tags: Vec::new(),
        branch: None,
        dependencies: Vec::new(),
    })
}

/// Extract track ID from a link like `./conductor/tracks/some_track_id/`
/// or `./tracks/some_track_id/`
fn extract_track_id_from_link(url: &str) -> Option<String> {
    let url = url.trim_end_matches('/');
    // Look for /tracks/ in the path
    if let Some(pos) = url.rfind("/tracks/") {
        let after = &url[pos + "/tracks/".len()..];
        let id = after.trim_end_matches('/');
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }
    // Fallback: last path segment
    url.rsplit('/').next().map(|s| s.to_string())
}

/// Apply a parsed field value to the current entry.
fn apply_field(entry: &mut IndexEntry, key: &str, value: &str) {
    let value = value.trim();
    match key {
        "Priority" => {
            entry.priority = Priority::from_str_loose(value);
        }
        "Status" => {
            entry.status = Status::from_str_loose(value);
        }
        "Tags" => {
            entry.tags = value
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
        }
        "Branch" => {
            let branch = value.trim_matches('`').to_string();
            if !branch.is_empty() {
                entry.branch = Some(branch);
            }
        }
        "Dependencies" | "Depends on" => {
            entry.dependencies = value
                .split(',')
                .map(|d| d.trim().trim_matches('`').trim_matches('(').split(')').next().unwrap_or("").trim().to_string())
                .filter(|d| !d.is_empty())
                .collect();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_h2_checked() {
        let entry = parse_h2_heading("[x] Track: Dashboard UI Overhaul ✅ COMPLETE").unwrap();
        assert_eq!(entry.checkbox, CheckboxStatus::Checked);
        assert_eq!(entry.title, "Dashboard UI Overhaul");
    }

    #[test]
    fn test_parse_h2_unchecked() {
        let entry = parse_h2_heading("[ ] Track: Compliance Workflow Enhancements").unwrap();
        assert_eq!(entry.checkbox, CheckboxStatus::Unchecked);
        assert_eq!(entry.title, "Compliance Workflow Enhancements");
    }

    #[test]
    fn test_parse_h2_in_progress() {
        let entry = parse_h2_heading("[~] Track: Chatbot Robustness Hardening").unwrap();
        assert_eq!(entry.checkbox, CheckboxStatus::InProgress);
        assert_eq!(entry.title, "Chatbot Robustness Hardening");
    }

    #[test]
    fn test_parse_h2_dash_progress() {
        let entry = parse_h2_heading("[-] Track: Security & Authentication Hardening - IN PROGRESS (3/5 findings)").unwrap();
        assert_eq!(entry.checkbox, CheckboxStatus::InProgress);
        assert_eq!(entry.title, "Security & Authentication Hardening - IN PROGRESS (3/5 findings)");
    }

    #[test]
    fn test_parse_h2_no_track_marker() {
        assert!(parse_h2_heading("Autopsy Remediation Tracks (2026-02-12)").is_none());
    }

    #[test]
    fn test_extract_track_id() {
        assert_eq!(
            extract_track_id_from_link("./conductor/tracks/pad_compliance_20260125/"),
            Some("pad_compliance_20260125".to_string())
        );
        assert_eq!(
            extract_track_id_from_link("./tracks/otel_collector_20260212/"),
            Some("otel_collector_20260212".to_string())
        );
    }

    #[test]
    fn test_parse_simple_index() {
        let md = r#"# Project Tracks

## [x] Track: Dashboard UI Overhaul ✅ COMPLETE
*Link: [./conductor/tracks/dashboard_overhaul_20260206/](./conductor/tracks/dashboard_overhaul_20260206/)*
**Priority**: High
**Tags**: frontend, ui, dashboard
**Status**: Completed (2026-02-06)
**Branch**: feat/dashboard-overhaul

---

## [ ] Track: Compliance Workflow Enhancements
*Link: [./conductor/tracks/compliance_enhancements_20260127/](./conductor/tracks/compliance_enhancements_20260127/)*
**Priority**: High
**Status**: Not_started
"#;
        let entries = parse_index_content(md);
        assert_eq!(entries.len(), 2);

        assert_eq!(entries[0].title, "Dashboard UI Overhaul");
        assert_eq!(entries[0].id.as_str(), "dashboard_overhaul_20260206");
        assert_eq!(entries[0].checkbox, CheckboxStatus::Checked);
        assert_eq!(entries[0].priority, Priority::High);

        assert_eq!(entries[1].title, "Compliance Workflow Enhancements");
        assert_eq!(entries[1].id.as_str(), "compliance_enhancements_20260127");
        assert_eq!(entries[1].checkbox, CheckboxStatus::Unchecked);
    }
}
