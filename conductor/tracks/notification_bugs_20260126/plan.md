# Implementation Plan: Notification Bugs Fix

**Track:** notification_bugs_20260126
**Status:** Complete
**Branch:** DSS-4074

---

## Phase 1: Fix USD Formatting ✅
- [x] Find all `:,.0f` and `:,.1f` patterns in ui.py
- [x] Replace with `:,.2f` (2 decimal places) - Fixed 4 locations
- [x] Test manager notification shows USD 1,177.50

## Phase 2: Remove Manager Execution Deadline ✅
- [x] Delete lines 819-822 in ui.py
- [x] Renumber block comments
- [x] Verify employee confirmation still has deadline

## Phase 3: Add Risk Factors to Manager Notification ✅
- [x] Extract risk factors in handlers.py (line 720)
- [x] Pass to request_manager_approval() (line 755)
- [x] Update agent.py method signature (line ~170)
- [x] Update ui.py function signature (line 564)
- [x] Add risk factors display block (after line 810)
- [x] Test with AAPL request (has Mako position)

## Phase 4: Restructure Approval Notification ✅
- [x] Move reference ID to header in handlers.py (line 2012)
- [x] Simplify footer message (line 2042)
- [x] Test approval confirmation format

## Phase 5: Fix Dashboard Summary Count ✅
- [x] Add current_user_id parameter in dashboard.py (line 210)
- [x] Test dashboard count matches page for luis
- [x] Test dashboard count matches page for manager

## Testing ✅
- [x] Manual E2E test complete
- [x] All 8 fixes verified
- [x] No regressions in Slack notifications

## Phase 6: Fix Compliance Notification Header ✅
- [x] Change header from "⚖️ Compliance Approval Required" to "PAD Approval REFERENCE-ID"
- [x] Updated ui.py line 1908
- [x] Test compliance notification header format

## Phase 7: Fix Compliance Notification Timestamp ✅
- [x] Add created_at/approved_at fields to SlackMessageRequest schema
- [x] Pass created_at in compliance notification (handlers.py line 2129)
- [x] Pass created_at in SMF16 escalation (handlers.py line 2172)
- [x] Test timestamp shows submission time (not approval time)

## Phase 8: Move Risk Factors Earlier in Compliance Notification ✅
- [x] Moved risk factors block to after core fields (ui.py line 1933)
- [x] Risk factors now appear before conflict warning
- [x] Test risk factors visibility in compliance notification

**Status:** ✅ COMPLETE - All 8 fixes implemented and committed
