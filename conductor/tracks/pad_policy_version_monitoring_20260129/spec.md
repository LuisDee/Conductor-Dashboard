# Spec: PAD Policy Version Monitoring

## Overview

Add a dashboard widget that monitors the PAD Policy Confluence page and alerts when a newer policy version is published. This ensures the PA Dealing system stays aligned with the latest compliance policy.

## Background

### Current Situation
- PAD policy is documented on Confluence: https://makoglobal.atlassian.net/wiki/pages/viewpageattachments.action?pageId=50384235
- The PA Dealing system spec references a specific policy version
- When Compliance updates the policy, developers need to review the spec and potentially update features
- Currently no automated notification when policy changes

### Problem
- Policy updates can go unnoticed
- Risk of system being out of sync with current compliance requirements
- Manual process to check for updates

### Solution
- Monitor the Confluence page for changes
- Store the "known policy version" in config
- Display warning banner when a newer version is detected
- Simple notification - no automated spec updates

## Functional Requirements

### FR1: Policy Page Fetcher

Extend `ConfluenceClient` (from `confluence_restricted_list` track) with:

```python
async def get_page_metadata(self, page_id: str) -> dict:
    """
    Fetch page metadata including version and last modified.

    Returns:
        {
            "page_id": "50384235",
            "title": "PA Dealing Policy",
            "version": 42,  # Confluence page version number
            "last_modified": "2026-01-15T14:30:00Z",
            "last_modified_by": "john.smith@mako.com",
        }
    """
```

### FR2: Policy Version Store

Create `src/pa_dealing/services/policy_version.py`:

```python
@dataclass
class PolicyVersionStatus:
    current_version: int          # Confluence page version
    known_version: int            # Version our spec is based on
    is_outdated: bool             # current > known
    last_checked: datetime
    last_modified: datetime
    last_modified_by: str
    page_url: str

async def check_policy_version() -> PolicyVersionStatus:
    """
    Check if PAD policy page has been updated since our known version.
    """

def get_known_policy_version() -> int:
    """
    Get the policy version our spec is based on.
    Read from config/environment.
    """
```

### FR3: Configuration

Add to environment:

```bash
# PAD Policy Monitoring
PAD_POLICY_PAGE_ID=50384235
PAD_POLICY_KNOWN_VERSION=1  # Update this when spec is updated for new policy
```

### FR4: Dashboard Widget

Add warning banner to dashboard when policy is outdated:

```
┌─────────────────────────────────────────────────────────────────┐
│ ⚠️  PAD Policy Updated                                          │
│                                                                 │
│ The PAD Policy was updated on 2026-01-28 by john.smith@mako.com │
│ Current version: 42 | System spec version: 38                   │
│                                                                 │
│ Some features may need reviewing.                               │
│ [View Policy] [Dismiss for 24h] [Mark as Reviewed]              │
└─────────────────────────────────────────────────────────────────┘
```

**Widget behavior:**
- Only shows when `current_version > known_version`
- "Dismiss for 24h" hides banner temporarily (localStorage)
- "Mark as Reviewed" requires updating `PAD_POLICY_KNOWN_VERSION` in config
- "View Policy" links to Confluence page

### FR5: API Endpoint

Add `GET /api/config/policy-version-status`:

```json
{
    "is_outdated": true,
    "current_version": 42,
    "known_version": 38,
    "last_checked": "2026-01-29T10:00:00Z",
    "last_modified": "2026-01-28T14:30:00Z",
    "last_modified_by": "john.smith@mako.com",
    "page_url": "https://makoglobal.atlassian.net/wiki/spaces/.../pages/50384235"
}
```

### FR6: Scheduled Check

Add to monitoring jobs:
- Check policy version every 6 hours (configurable)
- Log when new version detected
- Cache result to avoid excessive API calls

```bash
PAD_POLICY_CHECK_INTERVAL_HOURS=6
```

## Non-Functional Requirements

### NFR1: Minimal Impact
- Read-only monitoring (no Confluence writes)
- Failure doesn't affect trading
- Graceful degradation if Confluence unavailable

### NFR2: Caching
- Cache policy status for check interval
- Don't hit Confluence API on every dashboard load

### NFR3: Logging
- Log when new policy version detected
- Log check failures

## Dependencies

- `confluence_restricted_list_20260129` track (provides `ConfluenceClient`)

## Acceptance Criteria

### AC1: Version Detection
- [ ] System detects when Confluence page version > known version
- [ ] Correctly fetches page metadata from Confluence API

### AC2: Dashboard Warning
- [ ] Warning banner displays when policy is outdated
- [ ] Banner shows version numbers and last modified info
- [ ] "Dismiss for 24h" works (localStorage)
- [ ] "View Policy" links to correct Confluence page

### AC3: API Endpoint
- [ ] `/api/config/policy-version-status` returns correct status
- [ ] Response cached appropriately

### AC4: Scheduled Check
- [ ] Check runs on configurable interval
- [ ] New version detection logged

### AC5: Configuration
- [ ] `PAD_POLICY_PAGE_ID` configurable
- [ ] `PAD_POLICY_KNOWN_VERSION` configurable
- [ ] `PAD_POLICY_CHECK_INTERVAL_HOURS` configurable

## Out of Scope

- Automated spec updates based on policy changes
- Diff view of policy changes
- Slack notifications for policy updates (could add later)
- Attachment monitoring (just page version for now)
- Blocking trades when policy is outdated

## Future Enhancements

- Slack notification when new policy detected
- Show changelog/diff summary
- Track which spec sections relate to which policy sections
- Admin UI to update known version (instead of env var)
