# Implementation Plan: Fix Graph Poller Audit Error

## Phase 1: Diagnosis & Fix
- **Task:** Read `src/pa_dealing/audit.py` to find the correct method for logging discovery events.
- **Task:** Update `src/pa_dealing/services/graph_email_poller.py` to use the correct `AuditLogger` method.

## Phase 2: Verification
- **Task:** Restart `pad_graph_email_poller` container.
- **Task:** Monitor logs for successful poll cycle.
