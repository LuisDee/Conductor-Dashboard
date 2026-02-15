# Spec: Slack UI Dashboard Links & Cleanup

## Problem Statement

Slack notifications currently open modals for "View Full Details" actions. These modals are problematic:
1. Pop-ups within Slack are limited and not full-featured
2. Compliance review requires extensive form fields better suited to a web dashboard
3. Users expected expandable sections, not modals

Additionally, the Proposed Trade Summary is missing `is_derivative` and `is_leveraged` fields, and the Declaration button has an unnecessary emoji.

## Requirements

### Manager Approval Notification
- **KEEP** Approve and Decline buttons (action buttons)
- **CHANGE** "View Full Details" button to URL link opening dashboard
- **NO** expandable sections (current implementation already has none)
- **NO** modals

### Compliance Channel Notification
- **KEEP** Approve and Decline buttons (for quick low-risk approvals)
- **CHANGE** "Review Full Details" button to URL link opening dashboard
- **REMOVE** modal code (compliance_assessment_modal not needed)

### Other Changes
- **ADD** `is_derivative` and `is_leveraged` to Proposed Trade Summary
- **REMOVE** emoji from Declaration "Agree & Submit" button
- **VERIFY** dashboard `/requests/{id}` has all needed fields

### Auth Status Fix (Phase 5 - Outstanding)
- **FIX** `/api/auth/me` to return `auth_status` and `auth_message` fields
- **REMOVE** false "Limited Access" banner when auth succeeds

### Manager Authorization Fix (Phase 6 - Outstanding)
- **FIX** `is_manager_of()` to fallback to SQL `manager_id` when Google emails don't match
- **FIX** Remove non-existent `full_name` column from `get_visible_employees()` SQL
- **ENABLE** Managers to view their direct reports' requests even when Oracle/Google emails differ

## Success Criteria

1. All existing Slack UI tests pass
2. Dashboard link opens in browser (not modal)
3. Trade summary shows derivative/leveraged status
4. Declaration button renders correctly without emoji transformation
5. 5 failing chatbot tests fixed
6. **Phase 5**: `/api/auth/me` returns `auth_status: "ok"` for valid users, no false banner
7. **Phase 6**: Manager (aagombar) can view direct report's (ldeburna) request without 403
8. **Phase 6**: `/api/audit/employees` returns 200 (not 500 from `full_name` error)
