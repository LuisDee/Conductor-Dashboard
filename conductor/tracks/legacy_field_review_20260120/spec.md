# Legacy PAD Field Review & Integration

## Context

During migration planning from `personal_account_dealing` (legacy monolithic table) to the new normalized schema (`pad_request`, `pad_approval`, `pad_execution`, `audit_log`), we identified 12 fields from the legacy system that don't have a clear home in the new schema.

**Strategy Chosen**: Add ALL legacy fields to `pad_request` to preserve data during migration, then review and properly integrate them later (this track).

**Migration**: `alembic/versions/20260120_1858_1e2bea66afb6_add_legacy_pad_fields_for_migration.py`

## Legacy Fields Added

### 1. Compliance & Regulatory
- `conflict_comments` (TEXT) - Compliance explanation of identified conflicts
- `other_comments` (TEXT) - General comments about request
- `broker_reporting` (BOOLEAN) - Does broker report trades to Mako?
- `is_derivative` (BOOLEAN) - Is this a derivative contract?
- `is_leveraged` (BOOLEAN) - Is this a leveraged product?

### 2. Related Party
- `related_party_name` (VARCHAR 250) - Name of person trading is for

### 3. Compliance Declarations
- `signed_declaration` (BOOLEAN) - Employee signed compliance declaration

### 4. Audit Trail
- `updated_by_id` (BIGINT FK) - Employee who last modified request
- `deleted_at` (TIMESTAMP) - Soft delete timestamp
- `deleted_by_id` (BIGINT FK) - Employee who deleted/withdrew request

### 5. Execution Tracking
- `executed_within_two_days` (BOOLEAN) - Legacy field for 2-day execution compliance

## Objectives

1. **Review Each Field**: Determine if it should be:
   - **Integrated**: Added to bot workflow, dashboard UI, compliance process
   - **Deprecated**: Historical data only, not used going forward
   - **Refactored**: Moved to different table or stored differently

2. **Bot Workflow Integration**: Add questions for derivative/leveraged products to Slack bot conversation

3. **Dashboard UI**: Display conflict comments, other comments in appropriate sections

4. **Compliance Process**: Ensure compliance officers can add conflict documentation

5. **Audit Trail**: Implement soft delete workflow (withdraw requests) with proper audit logging

6. **Refactor**: Move fields to appropriate locations if needed (e.g., conflict_comments might belong in pad_approval for compliance)

## User Feedback (from discussion)

- **Derivative & Leveraged**: "We ask if it's a derivative when user talks to bot" → Add these questions to bot flow
- **Name field**: "The user's name making the request" → Already in oracle_employee, might be related party name
- **Conflict comments**: "Maybe we want a section for manager to add this" → Add UI section for conflict documentation
- **Deleted**: Confused about soft delete → Explain: withdrawn/cancelled requests stay in DB for audit
- **Two business days**: "We can calculate this" → Don't need to store, calculate from approval vs execution dates
- **Signed**: "Really not sure" → Review if this is still a requirement
- **Overall**: "Let's not drop fields, create scope to review later" → This track!

## Non-Goals

- Don't remove any fields until reviewed and approved
- Don't break historical data migration (these fields are needed for old data)

## Success Criteria

1. Each of 12 legacy fields has documented decision (integrate/deprecate/refactor)
2. Bot workflow updated with derivative/leveraged questions (if integrated)
3. Dashboard displays conflict comments, other comments (if integrated)
4. Soft delete workflow implemented (if integrated)
5. All decisions documented in this track plan
6. Migration mapping updated to reflect final field usage

## References

- Legacy schema: `/home/coder/repos/bodev/backoffice-web/database_models/models/tables/table_models.py:6585`
- New schema: `src/pa_dealing/db/models/pad.py`
- Migration plan: `conductor/tracks/db_migration_20251230/plan.md`
- Migration code: `alembic/versions/20260120_1858_1e2bea66afb6_add_legacy_pad_fields_for_migration.py`
