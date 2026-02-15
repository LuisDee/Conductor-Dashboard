use std::fmt;

use chrono::{DateTime, Utc};

use super::enums::{CheckboxStatus, PhaseStatus, Priority, Status, TrackType};

// ---------------------------------------------------------------------------
// TrackId — newtype for type safety
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TrackId(pub String);

impl TrackId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TrackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for TrackId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TrackId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ---------------------------------------------------------------------------
// Track — the core data model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Track {
    pub id: TrackId,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
    pub track_type: TrackType,
    pub phase: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub dependencies: Vec<TrackId>,
    pub tasks_total: usize,
    pub tasks_completed: usize,
    pub checkbox_status: CheckboxStatus,
    pub plan_phases: Vec<PlanPhase>,
    pub tags: Vec<String>,
    pub branch: Option<String>,
    pub description: Option<String>,
}

impl Track {
    pub fn progress_percent(&self) -> f32 {
        if self.tasks_total == 0 {
            return 0.0;
        }
        (self.tasks_completed as f32 / self.tasks_total as f32) * 100.0
    }

    pub fn is_complete(&self) -> bool {
        self.status == Status::Complete
            || (self.tasks_total > 0 && self.tasks_completed == self.tasks_total)
    }

    /// Merge metadata (from metadata.json or meta.yaml) into a track
    /// that was initially parsed from tracks.md.
    pub fn merge_metadata(&mut self, meta: TrackMetadata) {
        // Metadata status overrides checkbox if not default
        if meta.status != Status::New {
            self.status = meta.status;
        }
        if meta.priority != Priority::Medium {
            self.priority = meta.priority;
        }
        if meta.track_type != TrackType::Other {
            self.track_type = meta.track_type;
        }
        if let Some(dt) = meta.created_at {
            self.created_at = Some(dt);
        }
        if let Some(dt) = meta.updated_at {
            self.updated_at = Some(dt);
        }
        if !meta.dependencies.is_empty() {
            self.dependencies = meta.dependencies.into_iter().map(TrackId::new).collect();
        }
        if !meta.tags.is_empty() {
            self.tags = meta.tags;
        }
        if meta.branch.is_some() {
            self.branch = meta.branch;
        }
        if meta.description.is_some() {
            self.description = meta.description;
        }
    }

    /// Mark all plan tasks as complete (display-level normalization for tracks
    /// whose metadata status is Complete but whose plan.md has unticked tasks).
    pub fn mark_all_tasks_complete(&mut self) {
        for phase in &mut self.plan_phases {
            for task in &mut phase.tasks {
                task.done = true;
            }
            phase.status = PhaseStatus::Complete;
        }
        self.tasks_completed = self.tasks_total;
    }

    /// Merge plan data (from plan.md) into this track.
    pub fn merge_plan(&mut self, phases: Vec<PlanPhase>) {
        let (total, completed) = phases.iter().fold((0usize, 0usize), |(t, c), phase| {
            let phase_total = phase.tasks.len();
            let phase_done = phase.tasks.iter().filter(|t| t.done).count();
            (t + phase_total, c + phase_done)
        });
        self.tasks_total = total;
        self.tasks_completed = completed;
        self.plan_phases = phases;

        // Derive current phase name from first non-complete phase
        if let Some(active) = self
            .plan_phases
            .iter()
            .find(|p| p.status == PhaseStatus::Active || p.status == PhaseStatus::Pending)
        {
            self.phase = active.name.clone();
        } else if let Some(last) = self.plan_phases.last() {
            self.phase = last.name.clone();
        }
    }
}

impl Default for Track {
    fn default() -> Self {
        Self {
            id: TrackId::new(""),
            title: String::new(),
            status: Status::New,
            priority: Priority::Medium,
            track_type: TrackType::Other,
            phase: String::new(),
            created_at: None,
            updated_at: None,
            dependencies: Vec::new(),
            tasks_total: 0,
            tasks_completed: 0,
            checkbox_status: CheckboxStatus::Unchecked,
            plan_phases: Vec::new(),
            tags: Vec::new(),
            branch: None,
            description: None,
        }
    }
}

// ---------------------------------------------------------------------------
// PlanPhase / PlanTask — parsed from plan.md
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PlanPhase {
    pub name: String,
    pub status: PhaseStatus,
    pub tasks: Vec<PlanTask>,
}

impl PlanPhase {
    pub fn tasks_completed(&self) -> usize {
        self.tasks.iter().filter(|t| t.done).count()
    }

    pub fn progress_percent(&self) -> f32 {
        if self.tasks.is_empty() {
            return 0.0;
        }
        (self.tasks_completed() as f32 / self.tasks.len() as f32) * 100.0
    }
}

// ---------------------------------------------------------------------------
// PlanTask
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PlanTask {
    pub text: String,
    pub done: bool,
}

// ---------------------------------------------------------------------------
// TrackMetadata — intermediate struct from metadata.json / meta.yaml
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct TrackMetadata {
    pub status: Status,
    pub priority: Priority,
    pub track_type: TrackType,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub branch: Option<String>,
    pub description: Option<String>,
}
