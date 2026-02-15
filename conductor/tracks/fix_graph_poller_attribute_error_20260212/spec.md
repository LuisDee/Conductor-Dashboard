# Specification: Fix Graph Poller Attribute Error

## Goal
Fix the `AttributeError: 'MessageInfo' object has no attribute 'id'` crash in the `GraphEmailPoller` service to restore automated email ingestion.

## Problem
The `GraphEmailPoller` attempts to access `message.id` on the `MessageInfo` object returned by the `GraphClient`, but the attribute is likely named `message_id` (or similar), causing the worker to crash on every poll cycle.

## Requirements
1.  **Identify Correct Attribute:** Verify the `MessageInfo` schema definition to confirm the correct attribute name for the message ID.
2.  **Fix Access:** Update `GraphEmailPoller._process_message` (and any other call sites) to use the correct attribute.
3.  **Verify:** Restart the poller and confirm it successfully processes messages without crashing.

## Deliverables
- [ ] Code fix in `src/pa_dealing/services/graph_email_poller.py`.
- [ ] Verification log showing "poll_cycle_complete" without errors.
