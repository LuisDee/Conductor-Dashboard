# Track Specification: Compliance Workflow Enhancements

## 1. Goal
Enhance compliance workflow with auto-approval transparency, dynamic risk configuration, SMF16 escalation capabilities, and manager comment functionality.

## 2. Problem Summary

**Current Limitations:**
1. Auto-approved requests indistinguishable from manual approvals (both show "approved" status)
2. No visibility that AI auto-approved a request (users may think manager reviewed it)
3. Department risk categories hardcoded (requires code changes to update)
4. Compliance can only escalate HIGH risk trades to SMF16 (no manual escalation for MEDIUM/LOW)
5. Managers can't add comments when approving (compliance can)
6. Dashboard lacks features present in Slack notifications

**User Impact:**
- Lack of transparency in auto-approval process
- Configuration changes require developer intervention
- Limited escalation flexibility for edge cases
- Inconsistent approval capabilities between manager and compliance roles
- Feature parity gaps between Slack and dashboard interfaces

## 3. Scope

### In Scope
- Add `auto_approved` status to distinguish from manual `approved`
- Add auto-approval notification text in Slack and dashboard
- Make department risk categories configurable in Settings page
- Add SMF16 escalation button to compliance notifications
- Add manager comment input to approval notifications
- Ensure dashboard feature parity with Slack

### Out of Scope
- Automatic SMF16 escalation rules (manual only)
- Email notifications for auto-approvals
- Audit trail UI for configuration changes
- Historical migration of manager comments
- Changes to risk scoring algorithm

## 4. User Stories

### Auto-Approval Transparency
- **As an Employee,** I want to know when my request was auto-approved by AI (vs manually reviewed) so I understand the approval process
- **As a Compliance Officer,** I want to quickly identify auto-approved trades in reports so I can audit the AI's decisions

### Dynamic Risk Configuration
- **As a Compliance Officer,** I want to configure department risk levels without developer help so I can respond to organizational changes quickly
- **As a Compliance Officer,** I want to add new departments to HIGH risk when restructuring happens so the risk model stays current

### SMF16 Escalation
- **As a Compliance Officer,** I want to escalate any trade to SMF16 (regardless of risk level) when I see concerning patterns so complex cases get senior review
- **As a Compliance Officer,** I want to provide escalation reasoning so SMF16 reviewers have context for my decision

### Manager Comments
- **As a Manager,** I want to add conditional approval notes (like compliance can) so I can communicate requirements to the employee
- **As a Compliance Officer,** I want to see manager comments in my notification so I have full context before making my decision

## 5. Technical Requirements

- Maintain backward compatibility with existing approval flows
- New status must work with existing status-based queries
- Department configuration must persist across restarts
- SMF16 escalation must skip compliance approval (not double-approve)
- Manager comments must flow through to compliance notifications
- Dashboard and Slack must have feature parity

## 6. Acceptance Criteria

**Auto-Approval:**
- [ ] Auto-approved requests have status = `"auto_approved"`
- [ ] Slack shows grey text: "This request was deemed LOW risk and has been auto-approved by AI"
- [ ] Dashboard Request Detail shows auto-approval banner with risk score
- [ ] Dashboard request lists show "AUTO-APPROVED" badge
- [ ] Existing approved requests migrated to auto_approved if applicable

**Department Configuration:**
- [ ] Settings page shows "+ Add Department" button for HIGH and MEDIUM risk
- [ ] Clicking button opens searchable modal with oracle_department names
- [ ] Selected departments shown as removable badges (X icon)
- [ ] Changes persist to database (RiskScoringConfig JSONB)
- [ ] Risk scoring reads from config instead of hardcoded constants
- [ ] Audit log records department configuration changes

**SMF16 Escalation:**
- [ ] Compliance notification has "⚠️ Escalate to SMF16" button
- [ ] Clicking button opens escalation reason modal
- [ ] Submitting escalation changes status to `"pending_smf16"`
- [ ] SMF16 user receives notification with escalation reason
- [ ] No compliance approval record created (escalated, not approved)
- [ ] Dashboard has matching escalation button on Request Detail page

**Manager Comments:**
- [ ] Manager notification has optional comment input field
- [ ] Manager comments stored in PADApproval.comments
- [ ] Compliance notification shows "Manager Notes:" section
- [ ] Dashboard Pending Approvals modal has comment field
- [ ] Request Detail timeline displays manager comments

**Dashboard Parity:**
- [ ] All Slack features available in dashboard
- [ ] UI consistency with MAKO design system
- [ ] Proper role-based access control for new features

## 7. Testing Requirements

- Unit tests for status transitions
- Unit tests for comment extraction
- Integration tests for SMF16 escalation flow
- E2E test: Submit → Auto-approve → Verify status and notifications
- E2E test: Submit → Manager comment → Compliance sees comment
- E2E test: Submit → Compliance escalate → SMF16 receives
- Manual UAT with real Slack notifications

## 8. Timeline

- Phase 0 (Setup): 0.5 day
- Phase 1 (Auto-approved status): 0.5 day
- Phase 2 (Auto-approval UX): 0.5 day
- Phase 3 (Dynamic departments): 1 day
- Phase 4 (SMF16 escalation): 1 day
- Phase 5 (Manager comments): 0.5 day
- Testing: 1 day

**Total:** 5 days
