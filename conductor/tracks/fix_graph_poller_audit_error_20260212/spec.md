# Specification: Fix Graph Poller Audit Error

## Goal
Fix the `AttributeError: 'AuditLogger' object has no attribute 'log_email_discovery'` crash in the `GraphEmailPoller` service.

## Problem
The `GraphEmailPoller` attempts to call `audit.log_email_discovery()`, but this method does not exist on the `AuditLogger` instance. This likely happened during the recent `audit_refinement` track which standardized the audit logging system.

## Requirements
1.  **Identify Correct Audit Method:** Review `src/pa_dealing/audit.py` to find the standard method for logging discovery events (likely `log_event` or similar).
2.  **Fix Access:** Update `GraphEmailPoller._process_message` to use the standardized audit logging pattern.
3.  **Verify:** Restart the poller and confirm it successfully processes messages.

## Deliverables
- [ ] Code fix in `src/pa_dealing/services/graph_email_poller.py`.
- [ ] Verification log showing "poll_cycle_complete" without errors.
