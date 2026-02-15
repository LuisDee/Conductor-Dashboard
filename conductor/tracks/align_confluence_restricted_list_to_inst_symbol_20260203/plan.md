# Plan: Align Confluence Restricted List to inst_symbol

## Phase 1: Confluence Page Update
- [ ] Task: Get current Confluence page content (storage format).
- [ ] Task: Replace the string `<th>ticker</th>` (or similar) with `<th>inst_symbol</th>` in the HTML table.
- [ ] Task: Update the Confluence page with the modified content using `confluence_update_page`.

## Phase 2: Code Refactoring
- [ ] Task: Update `ConfluenceClient.parse_restricted_instruments` in `src/pa_dealing/integrations/confluence_client.py`.
    - Modify the header detection logic to look for `inst_symbol`.
    - Add fallback logic to treat `ticker` header as `inst_symbol` if found.
    - Ensure resulting dict uses `inst_symbol` key.
- [ ] Task: Update `ModelFactory.create_restricted_security` in `tests/factories.py`.
    - Change parameter from `ticker` to `inst_symbol`.
- [ ] Task: Verify `RestrictedListSyncService.sync_restricted_list` in `src/pa_dealing/services/restricted_list_sync.py`.
    - Ensure it uses `inst_symbol` key from client results.

## Phase 3: Test Refactoring
- [ ] Task: Update `tests/unit/test_confluence_client.py`.
    - Change `ticker` to `inst_symbol` in mock HTML and expectations.
- [ ] Task: Update `tests/unit/test_restricted_list_sync.py`.
    - Align mock client return values.
- [ ] Task: Update `tests/integration/test_confluence_sync.py`.
    - Replace `RestrictedSecurity.ticker` with `RestrictedSecurity.inst_symbol` in queries and assertions.
- [ ] Task: Fix integration test environment.
    - Ensure tests can connect to the database (likely using `db` instead of `localhost` when running in-container).

## Phase 4: Verification
- [ ] Task: Run unit tests for Confluence client and sync service.
- [ ] Task: Run integration tests for full sync flow.
- [ ] Task: Manually trigger a sync from the dashboard or CLI and verify DB state.
