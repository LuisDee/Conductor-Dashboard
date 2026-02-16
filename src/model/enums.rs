use std::fmt;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Track status (from tracks.md checkbox + metadata)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Status {
    #[default]
    New,
    InProgress,
    Blocked,
    Complete,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::New => "New",
            Self::InProgress => "Active",
            Self::Blocked => "Blocked",
            Self::Complete => "Complete",
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Custom deserializer that handles the many status string variants found
/// across metadata.json and meta.yaml files.
impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Status::from_str_loose(&s))
    }
}

impl Status {
    /// Parse a status string leniently.  Handles all observed variants:
    /// `"not_started"`, `"new"`, `"in_progress"`, `"complete"`, `"completed"`,
    /// `"blocked"`, `"planning"`, `"planned"`, etc.
    pub fn from_str_loose(s: &str) -> Self {
        let lower = s.to_ascii_lowercase();
        let lower = lower.trim();
        match lower {
            "complete" | "completed" | "done" => Self::Complete,
            "in_progress" | "in-progress" | "active" | "implementation" => Self::InProgress,
            "blocked" | "on_hold" => Self::Blocked,
            _ => Self::New, // not_started, new, planning, planned, etc.
        }
    }
}

// ---------------------------------------------------------------------------
// Priority
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Priority {
    Critical = 0,
    High = 1,
    #[default]
    Medium = 2,
    Low = 3,
}

impl Priority {
    pub fn label(self) -> &'static str {
        match self {
            Self::Critical => "CRITICAL",
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        let lower = s.to_ascii_lowercase();
        let lower = lower.trim();
        match lower {
            "critical" => Self::Critical,
            "high" => Self::High,
            "medium" | "med" => Self::Medium,
            "low" => Self::Low,
            _ => Self::Medium,
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl<'de> Deserialize<'de> for Priority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Priority::from_str_loose(&s))
    }
}

// ---------------------------------------------------------------------------
// Checkbox status (from tracks.md H2 headings)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CheckboxStatus {
    #[default]
    Unchecked, // [ ]
    InProgress, // [~] or [-]
    Checked,    // [x]
}

impl CheckboxStatus {
    /// Map checkbox to a Status, used as a fallback when metadata is missing.
    pub fn to_status(self) -> Status {
        match self {
            Self::Unchecked => Status::New,
            Self::InProgress => Status::InProgress,
            Self::Checked => Status::Complete,
        }
    }
}

// ---------------------------------------------------------------------------
// Phase status (derived from task completion within a phase)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PhaseStatus {
    #[default]
    Pending,
    Active,
    Complete,
    Blocked,
}

impl PhaseStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Active => "Active",
            Self::Complete => "Complete",
            Self::Blocked => "Blocked",
        }
    }
}

impl fmt::Display for PhaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ---------------------------------------------------------------------------
// Track type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum TrackType {
    Feature,
    Bug,
    Migration,
    Refactor,
    #[default]
    Other,
}

impl TrackType {
    pub fn label(&self) -> &str {
        match self {
            Self::Feature => "FEATURE",
            Self::Bug => "BUG",
            Self::Migration => "MIGRATION",
            Self::Refactor => "REFACTOR",
            Self::Other => "TRACK",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        let lower = s.to_ascii_lowercase();
        match lower.trim() {
            "feature" | "feat" => Self::Feature,
            "bug" | "bugfix" | "fix" => Self::Bug,
            "migration" | "migrate" => Self::Migration,
            "refactor" | "refactoring" => Self::Refactor,
            _ => Self::Other,
        }
    }
}

impl fmt::Display for TrackType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl<'de> Deserialize<'de> for TrackType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(TrackType::from_str_loose(&s))
    }
}

// ---------------------------------------------------------------------------
// Filter / Sort modes (UI state — Phase 2+)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterMode {
    #[default]
    All,
    Active,
    Blocked,
    Complete,
}

impl FilterMode {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Active,
            Self::Active => Self::Blocked,
            Self::Blocked => Self::Complete,
            Self::Complete => Self::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Blocked => "Blocked",
            Self::Complete => "Done",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    Updated,
    Progress,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            Self::Updated => Self::Progress,
            Self::Progress => Self::Updated,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Updated => "Recent",
            Self::Progress => "Progress",
        }
    }
}
