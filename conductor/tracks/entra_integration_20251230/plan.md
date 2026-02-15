# Plan: Azure Entra ID & Microsoft Graph Integration

## Phase 1: Azure Integration & Infrastructure
- [ ] Task: Add `msal` and `httpx` to `pyproject.toml` dependencies.
- [ ] Task: Update `.env.example` and local `.env` with Azure configuration placeholders (Tenant ID, Client ID, Secret).
- [ ] Task: Implement `EntraClient` in `src/pa_dealing/identity/entra.py` for token management and Graph API interactions.
- [ ] Task: **Write Tests**: Create unit tests for `EntraClient` mocking Graph API responses for user lookup, manager lookup, and direct reports.
- [ ] Task: **Implement**: Add `get_user_by_email`, `get_manager`, and `get_direct_reports` methods to `EntraClient`.
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Azure Integration' (Protocol in workflow.md)

## Phase 2: Hybrid Identity Service & TTL Caching
- [ ] Task: **Database Migration**: Add `entra_id` (string) and `last_refreshed_at` (timestamp) columns to the `oracle_employee` table.
- [ ] Task: **Write Tests**: Create integration tests for the `IdentityService` simulating the first-contact discovery and TTL-based refresh scenarios.
- [ ] Task: **Implement**: Create `IdentityService` to orchestrate resolution between Slack API (user_id -> email) and Entra ID (email -> profile/manager).
- [ ] Task: **Implement**: Add logic to `IdentityService` to check TTL and perform background/on-demand refreshes of employee metadata.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Identity Service' (Protocol in workflow.md)

## Phase 3: Application Integration (Routing & RBAC)
- [ ] Task: **Write Tests**: Update E2E tests to verify that `pad_service` correctly routes requests based on Entra-sourced manager IDs.
- [ ] Task: **Implement**: Update `pad_service` to use the new dynamic hierarchy for all approval routing logic.
- [ ] Task: **Implement**: Update Dashboard middleware/dependencies to dynamically determine `is_manager` status via `IdentityService` (checking `directReports` presence in Entra).
- [ ] Task: **Implement**: Update `scripts/db/seed_dev_database.py` to be compatible with the new identity discovery flow.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Application Integration' (Protocol in workflow.md)

## Phase 4: Verification & Regression
- [ ] Task: Verify the full E2E journey: Slack Submission -> Entra Manager Discovery -> DM Notification to Manager -> Manager Approval.
- [ ] Task: Run the full regression suite (`pytest` + `Playwright`) to ensure no regressions in existing conflict detection or audit trail logic.
- [ ] Task: Update `docs/ARCHITECTURE.md` to reflect the transition from Oracle-static to Entra-dynamic identity management.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Final Validation' (Protocol in workflow.md)
