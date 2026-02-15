# Spec: Confluence Integration for Restricted Instruments List

## Overview

Integrate with Confluence to fetch and sync a restricted instruments list. The Confluence page serves as the source of truth, with the existing `RestrictedSecurity` database table acting as a cache. This enables Compliance to maintain the restricted list in Confluence while the PA Dealing system automatically syncs changes.

## Background

### Current System
- `RestrictedSecurity` table stores restricted instruments
- `check_restricted_list()` in `repository.py` queries this table
- Restricted instruments trigger CRITICAL advisory → trade rejected
- List is currently manually managed in the database

### Problem
- No easy way for Compliance to update the restricted list
- Requires database access or developer intervention
- No audit trail of changes in a user-friendly format

### Solution
- Sync restricted list from a Confluence page on a configurable schedule
- Confluence becomes the source of truth
- Existing `check_restricted_list()` continues unchanged (queries DB cache)

## Functional Requirements

### FR1: Confluence Client
Create `src/pa_dealing/integrations/confluence_client.py` with:

1. **`ConfluenceClient` class**
   - Initialize with URL, username, API token from environment
   - Use `atlassian-python-api` library (pattern from `../tax-psa`)

2. **`fetch_restricted_instruments_page()`**
   - Extract page ID from configured URL (handle redirects)
   - Fetch page content via Confluence API
   - Return raw HTML content

3. **`parse_restricted_instruments(html: str)`**
   - Parse HTML to extract instrument data
   - Flexible parser (exact format TBD - page will be provided later)
   - Return structured list:
     ```python
     [
         {
             "ticker": "AAPL",
             "isin": "US0378331005",  # optional
             "reason": "Insider trading investigation",
             "restriction_type": "full",  # or "buy_only", "sell_only"
         },
         ...
     ]
     ```

4. **`get_restricted_instruments()`**
   - Convenience method: fetch + parse
   - Return format:
     ```python
     {
         "success": True,
         "instruments": [...],
         "source": "confluence",
         "fetched_at": "2026-01-28T12:00:00Z"
     }
     ```

### FR2: Sync Service
Create `src/pa_dealing/services/restricted_list_sync.py`:

1. **`sync_restricted_list()`**
   - Fetch instruments from Confluence
   - Upsert into `RestrictedSecurity` table
   - Mark instruments NOT in Confluence as `is_active=False`
   - Log sync results (added, removed, unchanged counts)

2. **`get_sync_status()`**
   - Return last sync time, success/failure, instrument count
   - Used by dashboard for status display

### FR3: Scheduled Job
Add to existing monitoring jobs system:

1. **Configurable interval** (default: 1 hour)
   - Setting: `RESTRICTED_LIST_SYNC_INTERVAL_MINUTES` (default: 60)

2. **Job registration** in `monitoring/jobs.py`
   - Run `sync_restricted_list()` on schedule

### FR4: Dashboard Status
Add sync status to dashboard:

1. **Sync status indicator**
   - Last sync time
   - Success/failure status
   - Instrument count
   - Warning banner if sync failed (stale data)

### FR5: Environment Variables
Add to all `.env` files:

```bash
# Confluence Integration
CONFLUENCE_URL=https://mako-group.atlassian.net
CONFLUENCE_USERNAME=luis.deburnay-bastos@mako.com
CONFLUENCE_API_TOKEN=ATATT3xFfGF0cHMRi0C5M19sXsLXySksmNMJ0-CXRdbMdiplTDSy4ml1ryXlEDeTr1HoVIsbWw6Z-WOcYqmg0B5yGth9Vpp0x8KWD75_TzGnFtegmXOQ3BfvGsvpeMxbkKBAwU40WuZ2ITErfrMcpHo30tqpPE-EVvr5CucY5FQV7OU2ppqxJaQ=B4A1DA5F
RESTRICTED_INSTRUMENTS_PAGE_URL=  # TBD - page URL to be provided

# Sync Configuration
RESTRICTED_LIST_SYNC_INTERVAL_MINUTES=60
```

## Non-Functional Requirements

### NFR1: Error Handling
- On Confluence failure: log warning, keep cached DB data, continue trading
- Dashboard shows "Sync failed - using cached data from {timestamp}"
- Retry on next scheduled run

### NFR2: Mock Data Support
- When `USE_MOCK_DATA=true`, load from local JSON file
- Path: `tests/data/restricted_instruments.json`
- Enables testing without Confluence access

### NFR3: Logging
- Log all sync attempts (success/failure)
- Log instrument changes (added/removed)
- Use existing `pa_dealing` logger pattern

## Dependencies

Add to `pyproject.toml`:
```toml
atlassian-python-api = "^3.41.0"
beautifulsoup4 = "^4.12.0"  # if not already present
```

## Reference Implementation

Copy patterns from `../tax-psa/src/tax_psa/tools/rules_tools.py`:
- `_extract_page_id()` - Confluence URL parsing (lines 37-55)
- `get_psa_rules()` - API fetch pattern (lines 58-104)
- Error handling, logging, mock data patterns

## Acceptance Criteria

### AC1: Confluence Integration
- [ ] `ConfluenceClient` connects to Confluence with provided credentials
- [ ] Page content successfully fetched and parsed
- [ ] Instruments correctly extracted to structured format

### AC2: Database Sync
- [ ] Instruments synced to `RestrictedSecurity` table
- [ ] Removed instruments marked as `is_active=False`
- [ ] Sync is idempotent (running twice produces same result)

### AC3: Scheduled Execution
- [ ] Sync runs on configurable interval (default: hourly)
- [ ] Interval configurable via `RESTRICTED_LIST_SYNC_INTERVAL_MINUTES`

### AC4: Failure Handling
- [ ] Confluence failure doesn't break trading
- [ ] Cached data continues to work
- [ ] Dashboard shows sync status/warnings

### AC5: Environment Configuration
- [ ] All `.env` files updated with Confluence settings
- [ ] Credentials work in dev/UAT environments

## Out of Scope

- Slack alerts for sync failures (can add later)
- Manual sync trigger from dashboard
- Confluence page creation/editing
- Historical sync audit log

## Test Plan

1. **Unit Tests**: Mock Confluence API, test parsing logic
2. **Integration Tests**: Test sync to DB with test data
3. **Mock Mode**: Verify `USE_MOCK_DATA=true` works
4. **Failure Tests**: Simulate Confluence unavailable, verify graceful handling
