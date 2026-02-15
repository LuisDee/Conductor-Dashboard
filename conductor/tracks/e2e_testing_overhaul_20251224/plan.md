# Implementation Plan: E2E Testing Infrastructure Overhaul

## Area 1: Mock Slack Server Fidelity (COMPLETED)

### Work Item 1.1: Implement Stateful User Registry in Mock Server (DONE)
- [x] Add `users` and `users_by_email` registry to `SlackServer`.
- [x] Implement `UsersInfoHandler` and `UsersLookupByEmailHandler` using the registry.
- [x] Pre-seed with test users matching `conftest.py`.

### Work Item 1.2: Implement Per-User DM Channel Tracking (DONE)
- [x] Implement `open_dm` logic in `SlackServer` to return stable IDs per user pair.
- [x] Update `ConversationsOpenHandler`.

### Work Item 1.3: Add Block Kit Schema Validation (DONE)
- [x] Create `validation/block_kit.py` for schema rules.
- [x] Integrate into `ChatPostMessageHandler`.

### Work Item 1.4: Add Error Simulation Endpoints (DONE)
- [x] Implement `FailureConfigHandler` and `Actor.should_fail()`.
- [x] Update `BaseSlackHandler.prepare()` to inject failures.

### Work Item 1.5: Add Rate Limit Simulation (DONE)
- [x] Implement `Actor.check_rate_limit()`.
- [x] Return 429 with `Retry-After` in `BaseSlackHandler`.

## Area 2: Eliminate Timing-Based Tests (COMPLETED)

### Work Item 2.1: Create Async Wait Utilities (DONE)
- [x] Create `tests/utils/async_helpers.py` with `wait_for`, `wait_for_message`, etc.

### Work Item 2.2: Update tests to Use Wait Utilities (DONE)
- [x] Replace `asyncio.sleep()` in `test_e2e_scenarios.py`, `test_slack_mock.py`, etc.

### Work Item 2.3: Add Event-Based Synchronization to Mock Server (DONE)
- [x] Implement `WaitForMessageHandler` in the mock actor.

## Area 3: Stop Bypassing Authorization (COMPLETED)

### Work Item 3.1: Create Role-Aware Test Fixtures (DONE)
- [x] Update `conftest.py` to assign roles (`compliance`, `smf16`) to employees.
- [x] Create `assign_role` fixture.

### Work Item 3.2: Remove Authorization Patches from E2E Tests (DONE)
- [x] Remove `patch(...has_role...)`.
- [x] Add negative tests for unauthorized access.

### Work Item 3.3: Remove Email Resolution Patches (DONE)
- [x] Rely on mock server stateful user registry instead of side-effect patches.

## Area 4: Database Test Isolation (COMPLETED)

### Work Item 4.1: Implement Transaction Rollback Isolation (DONE)
- [x] Update `session` fixture in `conftest.py` to use nested transactions and shared session state.

### Work Item 4.2: Create Data Builders (Factories) (DONE)
- [x] Create `tests/factories.py` for flexible data creation.

## Area 5: Add Missing Test Coverage (COMPLETED)

### Work Item 5.1: Add Auto-Approve Path Tests (DONE)
- [x] Create `tests/test_auto_approve.py`.

### Work Item 5.2: Add Holding Period Edge Case Tests (DONE)
- [x] Create `tests/test_holding_period.py`.

### Work Item 5.3: Add Concurrent Submission Tests (DONE)
- [x] Create `tests/test_concurrency.py`.

### Work Item 5.4: Add Document Processing Error Tests (DONE)
- [x] Create `tests/test_document_errors.py`.

## Area 6: Improve Test Assertions (COMPLETED)

### Work Item 6.1: Create Structured Message Matchers (DONE)
- [x] Create `tests/utils/message_matchers.py`.

### Work Item 6.2: Update Existing Tests to Use Structured Matchers (DONE)
- [x] Refactor assertions in existing suites.

## Phase 7: Compliance Dashboard (Playwright) (COMPLETED)

### Work Item 7.1: Implement Visual Regression Testing (DONE)
- Configure Playwright screenshots and baseline comparisons.

### Work Item 7.2: Stateful Mock API Integration for Frontend (DONE)
- Coordinate `playwright.config.ts` with `slack-mock`.

### Work Item 7.3: Targeted Component Testing (DONE)
- Add tests for Risk Assessment Summary, Audit Log Filters, and PDF Previews.

### Work Item 7.4: Flakiness & Performance Remediation (DONE)
- Replace `waitForTimeout` and fix hydration race conditions.

---

## Verification & Completion Protocol

### Phase 1 Completion (DONE)
- [x] Task: Conductor - User Manual Verification 'Phase 1: Foundation' (Protocol in workflow.md)

### Phase 2 Completion (DONE)
- [x] Task: Conductor - User Manual Verification 'Phase 2: Mock Fidelity' (Protocol in workflow.md)

### Phase 3 Completion (DONE)
- [x] Task: Conductor - User Manual Verification 'Phase 3: Remove Patches' (Protocol in workflow.md)

### Phase 4 Completion (DONE)
- [x] Task: Conductor - User Manual Verification 'Phase 4: Coverage' (Protocol in workflow.md)

### Phase 5 Completion (DONE)
- [x] Task: Conductor - User Manual Verification 'Phase 5: Assertions' (Protocol in workflow.md)

### Phase 6 Completion (DONE)
- [x] Task: Conductor - User Manual Verification 'Phase 6: Timing' (Protocol in workflow.md)

### Phase 7 Completion (DONE)
- [x] Task: Conductor - User Manual Verification 'Phase 7: Compliance Dashboard' (Protocol in workflow.md)

### Final Regression (DONE)
- [x] Run full regression suite (Unit + E2E + Playwright) inside containers.
- [x] Verify `ruff` and `eslint` pass for all new/modified test files.