# Track Specification: Slack Notification Bugs & Dashboard Count Fix

## 1. Goal
Fix 5 bugs in Slack notifications and dashboard summary that affect user experience and data accuracy.

## 2. Problem Summary

**User:** luis.deburnay-bastos@mako.com
**Example:** LDEBURNA-260126-AAPL-2 (BUY 5 AAPL @ USD 1,177.50)

### Bugs Identified:

1. **USD Rounding Loss** - Manager sees "USD 1,178" instead of "USD 1,177.50"
2. **Execution Deadline Noise** - Hardcoded "⏰ Must execute within 2 business days" clutters manager view unnecessarily
3. **Risk Factors Hidden** - Important context (Instrument Type, Mako Position, Direction Match) only visible AFTER manager approval, not during initial decision
4. **Approval Message Redundancy** - Reference ID appears in both header and footer of approval confirmation
5. **Dashboard Count Mismatch** - Shows 16 pending approvals but page is empty (includes user's own requests)

## 3. Scope

### In Scope
- Fix USD formatting to always show 2 decimal places
- Remove execution deadline from manager notifications
- Add risk factors to manager approval notification
- Restructure approval confirmation message
- Fix dashboard summary to use role-based filtering

### Out of Scope
- Slack message complete redesign
- Risk scoring system changes
- Advisory criteria changes
- Dashboard UI changes beyond count fix

## 4. User Stories

- **As a Manager,** I want to see exact USD amounts with cents so I can make accurate approval decisions
- **As a Manager,** I want to see risk context (Mako position, direction match) BEFORE approving so I can make informed decisions
- **As a Manager,** I don't need execution deadlines (that's the employee's responsibility) so my notification is cleaner
- **As an Employee,** I want clear approval confirmations without redundant information so I can quickly understand the outcome
- **As a User,** I want the dashboard pending count to match what I see on the page so the dashboard is trustworthy

## 5. Technical Requirements

- Maintain backward compatibility with existing notification functions
- Don't break employee approval confirmations (they still need execution deadline)
- Ensure risk factors are displayed cleanly and don't clutter the message
- Dashboard count must respect role-based access control

## 6. Acceptance Criteria

- [x] All USD amounts show 2 decimal places (X,XXX.XX format)
- [x] Manager notifications have no execution deadline footer
- [x] Employee confirmations keep execution deadline
- [x] Risk factors visible in manager notification before approval
- [x] Approval header: "REFERENCE-ID Status" format
- [x] Approval footer: "Request processed by [name]" (no ID)
- [x] Dashboard count equals Pending Approvals page count
- [x] User's own requests excluded from dashboard count
- [x] All Slack notification tests pass

## 7. Testing Requirements

- Manual E2E test: Submit trade → Check manager notification → Approve → Check confirmation
- Verify with multiple users (employee, manager, compliance)
- Test edge cases (0 risk factors, multiple risk factors, high-value trades)
- Verify dashboard count for different user roles

## 8. Timeline

- Phase 1 (USD): 0.25 day
- Phase 2 (Deadline): 0.25 day
- Phase 3 (Risk factors): 0.5 day
- Phase 4 (Approval structure): 0.25 day
- Phase 5 (Dashboard count): 0.25 day
- Testing: 0.5 day

**Total:** 2 days
