# Plan: Legacy PAD Field Review & Integration

## Phase 1: Field Review & Decision

### 1.1: Compliance & Regulatory Fields
- [ ] Task: Review `conflict_comments` field
  - [ ] Determine: Should compliance officers add conflict explanations?
  - [ ] Decision: Integrate into compliance approval workflow OR keep as historical-only
  - [ ] If integrate: Design UI section in dashboard for compliance to document conflicts
  - [ ] Alternative: Move to `pad_approval.comments` for compliance approval type?

- [ ] Task: Review `other_comments` field
  - [ ] Determine: Who adds these comments? When? Why?
  - [ ] Decision: Integrate as general notes field OR deprecate (use justification instead)
  - [ ] Consider: Append to `justification` during migration for historical data

- [ ] Task: Review `broker_reporting` field
  - [ ] Determine: Is broker reporting to Mako still a regulatory requirement?
  - [ ] Decision: Add to bot questions OR store in compliance_assessment JSONB OR deprecate
  - [ ] If integrate: Add question to Slack bot workflow

- [ ] Task: Review `is_derivative` field
  - [ ] User feedback: "We ask if it's a derivative when user talks to bot"
  - [ ] Decision: **INTEGRATE** into bot workflow
  - [ ] Action: Add "Is this a derivative contract?" question to Slack bot
  - [ ] Action: Update risk engine to consider derivative flag

- [ ] Task: Review `is_leveraged` field
  - [ ] User feedback: "Add is this a leveraged product y/n to questions we ask"
  - [ ] Decision: **INTEGRATE** into bot workflow
  - [ ] Action: Add "Is this a leveraged product?" question to Slack bot
  - [ ] Action: Update risk scoring to consider leverage (higher risk)

### 1.2: Related Party Field
- [ ] Task: Review `related_party_name` field
  - [ ] User feedback: "The name field is who the user is, the user's name making the request"
  - [ ] Question: Is this redundant with oracle_employee.name?
  - [ ] Alternative interpretation: Name of related party (if is_related_party=true)?
  - [ ] Decision: Clarify intended use, then integrate or deprecate
  - [ ] If related party name: Add to bot "Who is the related party?" question

### 1.3: Compliance Declarations
- [ ] Task: Review `signed_declaration` field
  - [ ] User feedback: "Signed I am really not sure"
  - [ ] Determine: Is physical/digital signature still required?
  - [ ] Decision: Integrate signature workflow OR deprecate (assume all requests are "signed" by submission)
  - [ ] Consider: Modern systems use submission timestamp as implicit signature

### 1.4: Audit Trail Fields
- [ ] Task: Review `updated_by_id` field
  - [ ] Determine: Do we need to track who modified requests?
  - [ ] Decision: **LIKELY INTEGRATE** for compliance audit trail
  - [ ] Action: Update request modification endpoints to capture updated_by_id

- [ ] Task: Review `deleted_at` / `deleted_by_id` soft delete
  - [ ] User feedback: "I don't know what deleted means if the record is in the DB how is it deleted?"
  - [ ] Explain: Soft delete = withdrawn/cancelled requests stay in DB for audit
  - [ ] Decision: **INTEGRATE** soft delete workflow
  - [ ] Action: Implement "Withdraw Request" feature (sets deleted_at, deleted_by_id)
  - [ ] Action: Filter deleted requests from dashboard (but show in audit view)

### 1.5: Execution Tracking
- [ ] Task: Review `executed_within_two_days` field
  - [ ] User feedback: "We can calculate as you mentioned"
  - [ ] Decision: **DEPRECATE** (calculate on-demand from approval_date vs executed_at)
  - [ ] Migration: Preserve historical value, don't populate for new requests
  - [ ] Action: Create calculated property or database view for 2-day compliance check

## Phase 2: Bot Workflow Integration

- [ ] Task: Add derivative question to Slack bot
  - [ ] Update conversation flow to ask "Is this a derivative contract? (Yes/No)"
  - [ ] Store answer in `pad_request.is_derivative`
  - [ ] Update risk assessment to flag derivatives

- [ ] Task: Add leverage question to Slack bot
  - [ ] Update conversation flow to ask "Is this a leveraged product? (Yes/No)"
  - [ ] Store answer in `pad_request.is_leveraged`
  - [ ] Update risk scoring (leveraged products = higher risk)

- [ ] Task: Conditionally ask broker_reporting (if integrated)
  - [ ] If decided to integrate: Add question about broker reporting
  - [ ] Store in `pad_request.broker_reporting`

- [ ] Task: Clarify related_party_name usage
  - [ ] If related party name needed: Add "Who is the related party?" after is_related_party=true

## Phase 3: Dashboard UI Integration

- [ ] Task: Display conflict_comments (if integrated)
  - [ ] Add "Conflict Details" section in compliance review view
  - [ ] Allow compliance officers to add/edit conflict_comments
  - [ ] Show in request detail view

- [ ] Task: Display other_comments (if integrated)
  - [ ] Add "Additional Comments" section in request view
  - [ ] Determine: Who can add these? When?

- [ ] Task: Show soft delete status
  - [ ] Add "Withdrawn" badge for deleted_at != null
  - [ ] Filter withdrawn requests from default list view
  - [ ] Add "Show Withdrawn" toggle for audit purposes

## Phase 4: Compliance Workflow Integration

- [ ] Task: Conflict documentation workflow (if integrated)
  - [ ] Allow compliance to add conflict_comments during approval
  - [ ] Require conflict_comments if has_conflict=true?
  - [ ] Show conflict history in audit log

- [ ] Task: Soft delete implementation
  - [ ] Add "Withdraw Request" button (for pending requests only)
  - [ ] Capture deleted_at (timestamp) and deleted_by_id (current user)
  - [ ] Create audit_log entry for withdrawal
  - [ ] Prevent actions on withdrawn requests

- [ ] Task: Audit trail for modifications
  - [ ] Capture updated_by_id on any request modification
  - [ ] Show modification history in audit view

## Phase 5: Data Migration Update

- [ ] Task: Update migration mapping with field decisions
  - [ ] Map legacy fields to new fields based on integration decisions
  - [ ] Handle deprecated fields (migrate historical data, don't populate going forward)
  - [ ] Document transformation logic for each field

- [ ] Task: Update migration script with field mappings
  - [ ] Implement all field transformations in `scripts/ops/migrate_historical_pad_data.py`
  - [ ] Test migration on sample historical data

## Phase 6: Documentation & Cleanup

- [ ] Task: Document field decisions in migration plan
  - [ ] Update `conductor/tracks/db_migration_20251230/plan.md` with final mappings
  - [ ] Document which fields are active vs historical-only

- [ ] Task: Update API documentation
  - [ ] Document new bot questions (derivative, leveraged)
  - [ ] Document soft delete workflow
  - [ ] Update schema documentation

- [ ] Task: User Manual Verification
  - [ ] Review all integration decisions with stakeholders
  - [ ] Verify bot workflow changes
  - [ ] Verify dashboard UI updates
  - [ ] Approve migration field mappings

## Summary of Current Decisions

Based on user feedback, preliminary decisions:

✅ **INTEGRATE**:
- `is_derivative` → Add to bot questions, risk assessment
- `is_leveraged` → Add to bot questions, risk scoring
- `deleted_at`, `deleted_by_id` → Implement soft delete (withdraw requests)
- `updated_by_id` → Track who modifies requests (audit trail)
- `conflict_comments` → Compliance documentation (likely)

❓ **NEEDS CLARIFICATION**:
- `related_party_name` → Is this redundant or actual related party name?
- `broker_reporting` → Still a regulatory requirement?
- `other_comments` → Who uses this? Append to justification?
- `signed_declaration` → Still needed or implicit in submission?

📊 **DEPRECATE** (calculate/derive):
- `executed_within_two_days` → Calculate from approval vs execution dates

---

**Next Steps**:
1. Complete Phase 1 field reviews
2. Get stakeholder sign-off on integration decisions
3. Implement bot workflow changes (Phase 2)
4. Implement dashboard UI (Phase 3)
5. Update migration script (Phase 5)
