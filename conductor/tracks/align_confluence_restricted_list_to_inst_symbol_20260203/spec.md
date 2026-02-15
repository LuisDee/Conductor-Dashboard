# Spec: Align Confluence Restricted List to inst_symbol

## Overview
Align the Restricted Instruments synchronization workflow with the authoritative identity refactor. Specifically, transition all remaining references from the ambiguous `ticker` identifier to the authoritative `inst_symbol` identifier across Confluence, the integration client, the sync service, and the test suite.

## Background
Migration `c98f6aae7452` (Authoritative Identity Anchor) dropped the `ticker` column from the `restricted_security` and `pad_request` tables in favor of `inst_symbol`. However, the Confluence synchronization logic, the actual Confluence page structure, and the existing integration tests still rely on the `ticker` column name, causing runtime failures and test regressions.

## Functional Requirements

### FR1: Confluence Page Update
- The header `ticker` in the "Restricted Instruments" Confluence page table must be renamed to `inst_symbol`.
- Data in this column should continue to be Mako internal symbols (which were previously referred to as tickers).

### FR2: Confluence Client Alignment
- Update `ConfluenceClient.parse_restricted_instruments` to look for the `inst_symbol` header.
- Maintain a backward-compatible fallback to `ticker` during the transition period if necessary, but prioritize `inst_symbol`.
- Ensure the returned dictionary uses `inst_symbol` as the key.

### FR3: Sync Service Alignment
- Verify `RestrictedListSyncService.sync_restricted_list` correctly maps the `inst_symbol` field from the client to the `RestrictedSecurity.inst_symbol` model property.
- Ensure the deactivation logic (instruments missing from Confluence) remains functional using `inst_symbol` as the join key.

### FR4: Test Suite Refactoring
- Update `tests/unit/test_confluence_client.py` to use `inst_symbol` in mock HTML and assertions.
- Update `tests/unit/test_restricted_list_sync.py` to align mock instruments with the `inst_symbol` key.
- Refactor `tests/integration/test_confluence_sync.py` to remove all references to `RestrictedSecurity.ticker` and use `RestrictedSecurity.inst_symbol`.
- Fix the DB connection issue in the integration test environment (localhost vs container networking).

### FR5: Model Factory Update
- Update `ModelFactory.create_restricted_security` in `tests/factories.py` to accept `inst_symbol` instead of `ticker`.

## Success Criteria
- [ ] Confluence table header renamed to `inst_symbol`.
- [ ] `RestrictedListSyncService` runs without `KeyError`.
- [ ] `tests/integration/test_confluence_sync.py` passes without `AttributeError`.
- [ ] All unit tests for Confluence integration pass.
- [ ] `ModelFactory` aligned with database schema.
