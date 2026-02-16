//! Parse `plan.md` — extract phases and tasks with checkbox state.
//!
//! Phases are identified by H2 (`##`) headings containing "Phase".
//! Tasks are list items starting with `- [x]` (done) or `- [ ]` (pending).
//! Nested content (code blocks, descriptions) is skipped.

use std::path::Path;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::model::{PhaseStatus, PlanPhase, PlanTask};
use crate::parser::error::ParseError;

/// Parse a plan.md file and return structured phases.
pub fn parse_plan(plan_path: &Path) -> Result<Vec<PlanPhase>, ParseError> {
    let content = std::fs::read_to_string(plan_path).map_err(|e| ParseError::Io {
        path: plan_path.to_path_buf(),
        source: e,
    })?;

    Ok(parse_plan_content(&content))
}

/// Parse plan.md content into phases.  This is the core logic.
pub fn parse_plan_content(content: &str) -> Vec<PlanPhase> {
    let opts = Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(content, opts);

    let mut phases: Vec<PlanPhase> = Vec::new();
    let mut in_heading = false;
    let mut _heading_level: Option<HeadingLevel> = None;
    let mut heading_text = String::new();
    let mut in_task_item = false;
    let mut task_text = String::new();
    let mut task_done = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // Flush any pending task
                flush_task(&mut phases, &mut in_task_item, &mut task_text, &task_done);

                in_heading = true;
                _heading_level = Some(level);
                heading_text.clear();
            }

            Event::End(TagEnd::Heading(level)) => {
                in_heading = false;
                let name = heading_text.trim().to_string();

                // Only treat H2 or H3 headings that look like phases
                if (level == HeadingLevel::H2 || level == HeadingLevel::H3)
                    && is_phase_heading(&name)
                {
                    phases.push(PlanPhase {
                        name,
                        status: PhaseStatus::Pending,
                        tasks: Vec::new(),
                    });
                }
            }

            // Task list checkbox events from pulldown-cmark
            Event::TaskListMarker(checked) => {
                // Flush any previous task first
                flush_task(&mut phases, &mut in_task_item, &mut task_text, &task_done);

                in_task_item = true;
                task_done = checked;
                task_text.clear();
            }

            Event::End(TagEnd::Item) => {
                flush_task(&mut phases, &mut in_task_item, &mut task_text, &task_done);
            }

            Event::Text(text) => {
                if in_heading {
                    heading_text.push_str(&text);
                } else if in_task_item {
                    task_text.push_str(&text);
                }
            }

            Event::Code(code) => {
                if in_heading {
                    heading_text.push_str(&code);
                } else if in_task_item {
                    task_text.push('`');
                    task_text.push_str(&code);
                    task_text.push('`');
                }
            }

            Event::SoftBreak | Event::HardBreak => {
                if in_heading {
                    heading_text.push(' ');
                } else if in_task_item {
                    task_text.push(' ');
                }
            }

            _ => {}
        }
    }

    // Flush final task
    flush_task(&mut phases, &mut in_task_item, &mut task_text, &task_done);

    // Compute phase statuses
    compute_phase_statuses(&mut phases);

    phases
}

/// Flush a pending task into the current (last) phase.
fn flush_task(
    phases: &mut Vec<PlanPhase>,
    in_task_item: &mut bool,
    task_text: &mut String,
    task_done: &bool,
) {
    if !*in_task_item {
        return;
    }
    let text = clean_task_text(task_text);
    if !text.is_empty() {
        // If no phase exists yet, create a default one
        if phases.is_empty() {
            phases.push(PlanPhase {
                name: "Tasks".to_string(),
                status: PhaseStatus::Pending,
                tasks: Vec::new(),
            });
        }
        phases.last_mut().unwrap().tasks.push(PlanTask {
            text,
            done: *task_done,
        });
    }
    *in_task_item = false;
    task_text.clear();
}

/// Check if a heading looks like a phase header.
/// Matches patterns like "Phase 1: Infrastructure", "Phase 2 (TDD)", etc.
fn is_phase_heading(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("phase")
}

/// Clean up task text: strip leading "Task:" prefix, trim whitespace.
fn clean_task_text(text: &str) -> String {
    let text = text.trim();
    let text = text.strip_prefix("Task:").unwrap_or(text).trim();
    text.to_string()
}

/// Derive phase statuses from task completion.
fn compute_phase_statuses(phases: &mut [PlanPhase]) {
    let mut found_active = false;

    for phase in phases.iter_mut() {
        if phase.tasks.is_empty() {
            phase.status = PhaseStatus::Pending;
            continue;
        }

        let all_done = phase.tasks.iter().all(|t| t.done);
        let any_done = phase.tasks.iter().any(|t| t.done);

        if all_done {
            phase.status = PhaseStatus::Complete;
        } else if any_done || !found_active {
            // First incomplete phase with some progress, or the first incomplete phase
            if any_done || !found_active {
                phase.status = PhaseStatus::Active;
                found_active = true;
            }
        } else {
            phase.status = PhaseStatus::Pending;
        }
    }

    // If we never found an active phase and there are incomplete phases,
    // mark the first incomplete one as active.
    if !found_active {
        if let Some(phase) = phases
            .iter_mut()
            .find(|p| p.status == PhaseStatus::Pending && !p.tasks.is_empty())
        {
            phase.status = PhaseStatus::Active;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_plan() {
        let md = r#"# Implementation Plan

## Phase 1: Setup
- [x] Task: Create project structure
- [x] Task: Add dependencies
- [ ] Task: Configure CI

## Phase 2: Implementation
- [ ] Task: Build parser
- [ ] Task: Add tests
"#;
        let phases = parse_plan_content(md);
        assert_eq!(phases.len(), 2);

        assert_eq!(phases[0].name, "Phase 1: Setup");
        assert_eq!(phases[0].tasks.len(), 3);
        assert!(phases[0].tasks[0].done);
        assert!(phases[0].tasks[1].done);
        assert!(!phases[0].tasks[2].done);
        assert_eq!(phases[0].status, PhaseStatus::Active);

        assert_eq!(phases[1].name, "Phase 2: Implementation");
        assert_eq!(phases[1].tasks.len(), 2);
        assert!(!phases[1].tasks[0].done);
        assert_eq!(phases[1].status, PhaseStatus::Pending);
    }

    #[test]
    fn test_all_complete() {
        let md = r#"## Phase 1: Done
- [x] Task: A
- [x] Task: B
"#;
        let phases = parse_plan_content(md);
        assert_eq!(phases[0].status, PhaseStatus::Complete);
    }

    #[test]
    fn test_empty_plan() {
        let phases = parse_plan_content("# Nothing here\n\nJust a description.\n");
        assert!(phases.is_empty());
    }

    #[test]
    fn test_task_text_cleanup() {
        assert_eq!(
            clean_task_text("Task: Build the parser"),
            "Build the parser"
        );
        assert_eq!(clean_task_text("  Build the parser  "), "Build the parser");
        assert_eq!(clean_task_text("Task:  Do stuff"), "Do stuff");
    }

    #[test]
    fn test_tasks_without_phase() {
        let md = r#"# Plan
- [x] Do thing one
- [ ] Do thing two
"#;
        let phases = parse_plan_content(md);
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].name, "Tasks");
        assert_eq!(phases[0].tasks.len(), 2);
    }

    #[test]
    fn test_phase_with_description_paragraph() {
        let md = r#"## Phase 1: Infrastructure & Foundation
Establish the base container environment and configuration structure.

- [ ] Task: Create OTel configuration directory
- [ ] Task: Add service to docker-compose

## Phase 2: Collection (TDD)
Configure the collector.

- [x] Task: Write verification script
- [ ] Task: Implement filelog receiver
"#;
        let phases = parse_plan_content(md);
        assert_eq!(phases.len(), 2);
        assert_eq!(phases[0].tasks.len(), 2);
        assert_eq!(phases[1].tasks.len(), 2);
        assert!(phases[1].tasks[0].done);
    }
}
