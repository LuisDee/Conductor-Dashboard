# Specification: E2E Testing Infrastructure Overhaul

**Track ID:** `e2e_testing_overhaul_20251224`
**Type:** Infrastructure / Refactor
**Status:** In-Progress (Formalized)

## 1. Overview
The current E2E testing suite provides insufficient confidence. Tests often pass in CI but fail to catch real-world bugs due to excessive mocking, bypassed authorization logic, flaky timing-based assertions, and incomplete coverage of the Compliance Dashboard. This track aims to overhaul the entire testing infrastructure (Backend, Slack Mock, and Frontend) to ensure that when users interact with the system, it works as intended.

## 2. Functional Requirements

### 2.1 Backend & Slack Mock Fidelity
- **Stateful Mock Slack:** Implement a user registry and per-user DM channel tracking to mirror real Slack behavior.
- **Protocol Validation:** Add Block Kit schema validation to catch malformed UI blocks before they reach a "real" client.
- **Resilience Testing:** Implement error and rate-limit simulation (429s, 5xx) to verify retry logic and error handling.
- **Auth Integrity:** Remove all patches that bypass `has_role` checks. Use real, role-aware test data to verify security boundaries.

### 2.2 Test Reliability & Performance
- **Deterministic Waiting:** Eliminate `asyncio.sleep()` calls. Implement robust async wait utilities and event-driven synchronization between the app and the mock server.
- **Database Isolation:** Implement transaction-level rollback isolation to ensure tests are fast, side-effect-free, and capable of running in parallel.
- **Structured Assertions:** Replace fragile JSON/string matching with structured message matchers and data-aware predicates.

### 2.3 Compliance Dashboard (Frontend)
- **Visual Regression:** Implement Playwright-based screenshot comparison to detect unintended UI regressions.
- **State Integration:** Ensure the frontend interacts with a stateful backend/mock environment that reflects consistent data across the full user journey.
- **Component Integrity:** Add targeted tests for complex UI components (e.g., risk assessment summaries, filters) to ensure isolated reliability.
- **Flakiness Remediation:** Identify and resolve existing timing/hydration issues in the React application that cause non-deterministic test failures.

## 3. Scope of Coverage
The overhauled suite must provide high-fidelity coverage for:
- **Auto-Approval Paths:** Verify the logic that grants instant approval for low-risk trades.
- **Holding Periods:** Verify 30-day enforcement, including clock-reset edge cases.
- **Concurrency:** Ensure the system handles simultaneous requests and duplicate approval clicks gracefully.
- **Document Processing:** Verify error handling for malformed PDFs and discrepancies in trade extraction.

## 4. Acceptance Criteria
- [x] No `asyncio.sleep()` or hardcoded `patch("...has_role")` remain in the E2E suite.
- [x] The Mock Slack server rejects invalid Block Kit payloads with descriptive errors.
- [x] Playwright tests pass consistently with 4+ workers and include visual snapshots.
- [x] Test execution time is significantly reduced via database transaction rollbacks.
- [x] All "critical paths" (identified in Area 5 of the plan) have documented passing tests.
