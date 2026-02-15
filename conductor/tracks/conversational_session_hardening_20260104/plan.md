# Conversational Session Hardening Plan

## Objective
Refactor the Chatbot's session management to strictly use Slack Threads for state isolation. This solves the "Sticky Session" problem by ensuring that every new request in the Main DM spawns a dedicated, isolated thread. Additionally, we will implement "Justification Coaching" as a soft quality gate.

## Core Architectural Changes
1.  **Thread-Centric Sessions:** The `SessionManager` will key sessions by `slack_thread_ts`. The Main DM (where `thread_ts` is null) will be stateless, acting only as a "Launcher."
2.  **Multi-Draft Support:** By virtue of threading, users can maintain multiple concurrent drafts in parallel threads.
3.  **Justification Coaching:** An LLM-driven step to advise users on weak justifications without blocking them.

## Implementation Steps

### Phase 1: Session & State Refactoring
- [x] **Task 1:** Modify `SessionManager.get_draft` to require `thread_ts`.
    - If `thread_ts` is missing (Main DM), it should likely return `None` or a temporary "New Intent" object, triggering the creation of a new thread.
- [x] **Task 2:** Update `chatbot.py:process_message` to handle Main DM vs. Thread logic.
    - **Case A (Main DM):** Detect intent -> Post "Starting request..." -> Create Thread -> Save Draft keyed to new `thread_ts`.
    - **Case B (Thread):** Fetch draft by `thread_ts`. Apply updates.

### Phase 2: Justification Coaching
- [x] **Task 3:** Update `SYSTEM_PROMPT` to evaluate justification quality.
- [x] **Task 4:** Add a `justification_quality` field to the schema (e.g., `GOOD`, `WEAK`, `MISSING`).
- [x] **Task 5:** Implement the "Soft Gate" logic in `chatbot.py`.
    - If `WEAK` and not `user_confirmed_weak`, ask: "This is brief. Add more?"
    - If `user_confirmed_weak` (e.g., "Yes, submit it"), proceed.

### Phase 3: Cleanup & Management
- [x] **Task 6:** Implement `my drafts` command.
    - Scans DB for active drafts associated with the user.
    - Returns a list with links to the Slack threads.
- [x] **Task 7:** Stale Draft Cleanup.
    - Periodic job to archive/warn about threads untouched for > 24 hours.

### Phase 4: Testing
- [x] **Task 8:** Unit Tests for `SessionManager` (key generation).
- [x] **Task 9:** E2E Test (Mock Slack) for Parallel Drafts.
    - User starts Draft A (AAPL).
    - User starts Draft B (BARC).
    - User replies to Draft A.
    - Verify B is untouched.

## Success Criteria
- [x] A user can start a new request in the Main DM at any time without "polluting" previous requests.
- [x] A user can switch between threads to work on multiple drafts.
- [x] The bot advises on weak justifications but allows override.