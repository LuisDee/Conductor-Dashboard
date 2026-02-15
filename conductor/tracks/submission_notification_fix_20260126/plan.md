# Track: Submission Notification Data Consistency Fix

**Jira:** DSS-4074
**Created:** 2026-01-26
**Spec:** /home/coder/repos/ai-research/pa-dealing/initial_submission_notification_spec.md

## Problem

Discrepancies exist between submission notification and manager notification data because they use different data sources:

| Field | Submission Notification | Manager Notification |
|-------|------------------------|---------------------|
| employee_id | `analysis.get("employee_id")` (raw chatbot) | `employee.mako_id` (identity provider) |
| employee_name | `analysis.get("employee_name")` | Enriched from identity |
| cost_centre | NOT INCLUDED | `employee.cost_centre` |
| desk_name | NOT INCLUDED | `employee.cost_centre` |

## Root Cause

`_send_submission_notification()` in handlers.py (line 778) only uses raw data from `result.get("analysis", {})` and does NOT enrich with identity provider lookup.

`_notify_manager_of_new_request()` (line 689) properly enriches data:
```python
employee = await identity.get_by_email(lookup_email)
# Then uses employee.mako_id, employee.cost_centre, etc.
```

## Fix

Update `_send_submission_notification()` to:
1. Look up employee from identity provider (same as manager notification)
2. Use enriched data for all fields
3. Ensure consistency between both notifications

## Implementation

### Phase 1: Fix _send_submission_notification data sources

**File:** `src/pa_dealing/agents/slack/handlers.py`

Add identity provider lookup to `_send_submission_notification()`:

```python
async def _send_submission_notification(self, result: dict, user_id: str) -> None:
    # ... existing code ...

    # NEW: Enrich with identity provider data (same pattern as manager notification)
    async with get_session() as session:
        identity = get_identity_provider_with_session(session)

        # Try lookup by email first
        lookup_email = result.get("employee_email") or result.get("google_email")
        employee = None
        if lookup_email:
            employee = await identity.get_by_email(lookup_email)

        # Fallback to mako_id lookup
        if not employee and employee_name:
            employee = await identity.get_by_mako_id(employee_name)

        # Use enriched data if available
        if employee:
            employee_id = employee.mako_id
            employee_name = employee.full_name or employee.mako_id
            # cost_centre available if needed later
```

### Phase 2: Verify data consistency

After fix, both notifications should show identical:
- employee_id (mako_id format)
- employee_name
- All shared fields

## Validation

- [x] Submission notification employee_id matches manager notification
- [x] Submission notification employee_name matches manager notification
- [x] No hardcoded or "Unknown" values when employee exists in DB
- [x] Unit tests pass
- [x] E2E test with real submission shows consistent data

## Files to Modify

1. `src/pa_dealing/agents/slack/handlers.py` - Add identity provider lookup to `_send_submission_notification()`
