# PA Dealing - Production Readiness & Feature Completion Plan

## Overview
This plan outlines the steps required to move the PA Dealing system to a feature-complete and production-ready status.

## Progress Tracking

### [DONE] Stage 1: Dashboard Navigation & Link Integrity
- [x] Link all Dashboard tables to Request Detail.
- [x] Link Pending Approvals to Request Detail.
- [x] Link Execution Tracking to Request Detail.
- [x] Link Mako Conflicts to Request Detail.
- [x] Link Holding Periods to Request Detail.
- [x] Link Breaches to Request Detail.
- [x] Link Audit Log to Request Detail.
- [x] **Verification:** Playwright tests confirm all links lead to the correct `reference_id`.

### [DONE] Stage 2: Universal Filtering & Search
- [x] Implement `employee_name` and `security` (Ticker/ISIN) filters in Backend.
- [x] Update Frontend `api/client.ts` to support search params.
- [x] Add Filter UI to all Dashboard pages.
- [x] Standardize search behavior (case-insensitive partial match).
- [x] **Verification:** Playwright tests confirm filters update table results correctly.

### [DONE] Stage 3: Audit Log UX & Breach Actionability
- [x] Resolve Database Schema & Seeding issues (Hard Reset performed).
- [x] Implement Actor vs Employee distinction in Audit Log filters.
- [x] Implement Timeline/Visual history in Request Detail (Audit Trail).
- [x] Implement Resolve Breach Modal/Action.
- [x] Link Breach to Request Detail for immediate context.
- [x] **Verification:** Manual check of Audit Log filters and Breach resolution.

### [DONE] Stage 4: Automated Monitoring & Chasing
- [x] Implement Backend Scheduler (`APScheduler`) for automated jobs.
- [x] Implement "Overdue Execution" job (approvals > 2 business days).
- [x] Implement "Missing Contract Note" job (executed > 30 days ago).
- [x] Create Compliance Settings UI to manage thresholds (days, scores).
- [x] Add "Send Manual Chase" button to Request Detail for Slack reminders.
- [x] **Verification:** Manual verification of Settings UI and manual chase endpoint.

### [DONE] Stage 5: Production Hardening & Final Polish
- [x] Implement robust `/health` and `/ready` checks (DB, Slack, AI connectivity).
- [x] Implement Request Archiving/Cleanup job for old data.
- [x] Final UI/UX Polish: Loading skeletons, Error boundaries, and Toasts.
- [x] Review all logging for PII and sensitive data (Sanitized).
- [x] **Verification:** Final end-to-end smoke test of entire lifecycle passed (41/41 tests).

## Summary of Improvements
1. **Navigation:** Universal deep-linking from all dashboard tables to a comprehensive Request Detail view.
2. **Search:** String-based partial match search for Employees and Securities across the entire stack.
3. **Auditability:** Visual timelines for request history and a searchable, filtered audit log with Actor/Employee separation.
4. **Automation:** A background scheduler for proactive compliance chasing and an AI-driven contract note verification loop.
5. **Admin Control:** A dedicated Compliance Settings UI to manage all system thresholds and trigger manual data syncs.

