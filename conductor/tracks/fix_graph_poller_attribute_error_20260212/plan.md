# Implementation Plan: Fix Graph Poller Attribute Error

## Phase 1: Diagnosis & Fix
- **Task:** Read `src/pa_dealing/services/graph_client.py` to inspect `MessageInfo` definition.
- **Task:** Update `src/pa_dealing/services/graph_email_poller.py` to replace `message.id` with `message.message_id` (or correct field).

## Phase 2: Verification
- **Task:** Restart `pad_graph_email_poller` container.
- **Task:** Monitor logs for successful poll cycle.
