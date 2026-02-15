# Plan: PAD Policy Version Monitoring

**Status:** Planning

**Depends on:** `confluence_restricted_list_20260129` (provides ConfluenceClient)

---

## Phase 1: Extend Confluence Client

**Goal**: Add page metadata fetching to existing ConfluenceClient.

### Implementation

Add to `src/pa_dealing/integrations/confluence_client.py`:

```python
async def get_page_metadata(self, page_id: str) -> dict | None:
    """
    Fetch page metadata including version and last modified.

    Uses Confluence REST API:
    GET /wiki/rest/api/content/{page_id}?expand=version,history.lastUpdated

    Returns:
        {
            "page_id": "50384235",
            "title": "PA Dealing Policy",
            "version": 42,
            "last_modified": "2026-01-15T14:30:00Z",
            "last_modified_by": "john.smith@mako.com",
            "page_url": "https://..."
        }
    """
    if self.use_mock_data:
        return self._get_mock_page_metadata(page_id)

    try:
        page = self.confluence.get_page_by_id(
            page_id,
            expand="version,history.lastUpdated"
        )

        return {
            "page_id": page_id,
            "title": page.get("title"),
            "version": page.get("version", {}).get("number"),
            "last_modified": page.get("history", {}).get("lastUpdated", {}).get("when"),
            "last_modified_by": page.get("history", {}).get("lastUpdated", {}).get("by", {}).get("email"),
            "page_url": f"{self.base_url}/wiki/spaces/{page.get('space', {}).get('key')}/pages/{page_id}",
        }

    except Exception as e:
        logger.error(f"Failed to fetch page metadata: {e}")
        return None

def _get_mock_page_metadata(self, page_id: str) -> dict:
    """Return mock metadata for testing."""
    return {
        "page_id": page_id,
        "title": "PA Dealing Policy (Mock)",
        "version": 5,
        "last_modified": "2026-01-15T14:30:00Z",
        "last_modified_by": "mock@mako.com",
        "page_url": f"https://mock.atlassian.net/wiki/pages/{page_id}",
    }
```

### Tasks

- [ ] Add `get_page_metadata()` method to ConfluenceClient
- [ ] Add mock data support for testing
- [ ] Add unit tests for metadata fetching
- [ ] Test with real Confluence page

### Acceptance Criteria

- [ ] Page version number extracted correctly
- [ ] Last modified date and author extracted
- [ ] Mock mode works for testing

---

## Phase 2: Policy Version Service

**Goal**: Create service to check and track policy version status.

### Implementation

Create `src/pa_dealing/services/policy_version.py`:

```python
import logging
from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import Optional

from pa_dealing.integrations.confluence_client import ConfluenceClient
from pa_dealing.config import settings

logger = logging.getLogger(__name__)

# Cache
_cached_status: Optional["PolicyVersionStatus"] = None
_cache_expiry: Optional[datetime] = None


@dataclass
class PolicyVersionStatus:
    """Status of PAD policy version."""
    current_version: int          # Confluence page version
    known_version: int            # Version our spec is based on
    is_outdated: bool             # current > known
    last_checked: datetime
    last_modified: datetime | None
    last_modified_by: str | None
    page_url: str
    check_failed: bool = False
    error_message: str | None = None

    def to_dict(self) -> dict:
        return {
            "is_outdated": self.is_outdated,
            "current_version": self.current_version,
            "known_version": self.known_version,
            "last_checked": self.last_checked.isoformat(),
            "last_modified": self.last_modified.isoformat() if self.last_modified else None,
            "last_modified_by": self.last_modified_by,
            "page_url": self.page_url,
            "check_failed": self.check_failed,
            "error_message": self.error_message,
        }


def get_known_policy_version() -> int:
    """
    Get the policy version our spec is based on.
    """
    return settings.PAD_POLICY_KNOWN_VERSION


async def check_policy_version(force_refresh: bool = False) -> PolicyVersionStatus:
    """
    Check if PAD policy page has been updated.

    Uses cache unless force_refresh=True or cache expired.
    """
    global _cached_status, _cache_expiry

    # Return cached if valid
    if not force_refresh and _cached_status and _cache_expiry and datetime.utcnow() < _cache_expiry:
        return _cached_status

    known_version = get_known_policy_version()
    page_id = settings.PAD_POLICY_PAGE_ID

    try:
        client = ConfluenceClient()
        metadata = await client.get_page_metadata(page_id)

        if not metadata:
            raise Exception("Failed to fetch page metadata")

        current_version = metadata.get("version", 0)
        is_outdated = current_version > known_version

        if is_outdated:
            logger.warning(
                "PAD policy is outdated",
                extra={
                    "current_version": current_version,
                    "known_version": known_version,
                    "last_modified_by": metadata.get("last_modified_by"),
                }
            )

        status = PolicyVersionStatus(
            current_version=current_version,
            known_version=known_version,
            is_outdated=is_outdated,
            last_checked=datetime.utcnow(),
            last_modified=datetime.fromisoformat(metadata["last_modified"].replace("Z", "+00:00"))
                if metadata.get("last_modified") else None,
            last_modified_by=metadata.get("last_modified_by"),
            page_url=metadata.get("page_url", ""),
        )

        # Cache result
        _cached_status = status
        _cache_expiry = datetime.utcnow() + timedelta(hours=settings.PAD_POLICY_CHECK_INTERVAL_HOURS)

        return status

    except Exception as e:
        logger.error(f"Policy version check failed: {e}")

        # Return error status (keep old cache if available)
        return PolicyVersionStatus(
            current_version=_cached_status.current_version if _cached_status else 0,
            known_version=known_version,
            is_outdated=_cached_status.is_outdated if _cached_status else False,
            last_checked=datetime.utcnow(),
            last_modified=_cached_status.last_modified if _cached_status else None,
            last_modified_by=_cached_status.last_modified_by if _cached_status else None,
            page_url=f"https://makoglobal.atlassian.net/wiki/pages/viewpage.action?pageId={page_id}",
            check_failed=True,
            error_message=str(e),
        )
```

### Tasks

- [ ] Create `PolicyVersionStatus` dataclass
- [ ] Implement `get_known_policy_version()` from settings
- [ ] Implement `check_policy_version()` with caching
- [ ] Add unit tests with mocked Confluence
- [ ] Test cache expiry logic

### Acceptance Criteria

- [ ] Detects when `current_version > known_version`
- [ ] Caches result for configured interval
- [ ] Handles Confluence failures gracefully
- [ ] Logs warning when outdated

---

## Phase 3: Configuration

**Goal**: Add environment variables for policy monitoring.

### Implementation

Add to `src/pa_dealing/config.py`:

```python
# PAD Policy Monitoring
PAD_POLICY_PAGE_ID: str = "50384235"
PAD_POLICY_KNOWN_VERSION: int = 1  # Update when spec aligned to new policy
PAD_POLICY_CHECK_INTERVAL_HOURS: int = 6
```

### Environment Files

Add to `.env.example`, `.env.dev`, `.env.uat`:

```bash
# PAD Policy Monitoring
PAD_POLICY_PAGE_ID=50384235
PAD_POLICY_KNOWN_VERSION=1
PAD_POLICY_CHECK_INTERVAL_HOURS=6
```

### Tasks

- [ ] Add settings to config.py
- [ ] Update `.env.example` with new variables
- [ ] Update `.env.dev` and `.env.uat`
- [ ] Document what `PAD_POLICY_KNOWN_VERSION` means

### Acceptance Criteria

- [ ] All config variables accessible via settings
- [ ] Defaults are sensible
- [ ] Documentation clear on how to update known version

---

## Phase 4: API Endpoint

**Goal**: Add endpoint for dashboard to fetch policy status.

### Implementation

Add to `src/pa_dealing/api/routes/config.py` (or create):

```python
from fastapi import APIRouter
from pa_dealing.services.policy_version import check_policy_version

router = APIRouter(prefix="/api/config", tags=["config"])


@router.get("/policy-version-status")
async def get_policy_version_status():
    """
    Get PAD policy version status.

    Returns whether the policy has been updated since
    the version our spec is based on.
    """
    status = await check_policy_version()
    return status.to_dict()
```

### Tasks

- [ ] Create config routes file (if not exists)
- [ ] Add `GET /api/config/policy-version-status` endpoint
- [ ] Register router in main app
- [ ] Add API tests

### Acceptance Criteria

- [ ] Endpoint returns correct status
- [ ] Response uses cached value when valid
- [ ] Handles errors gracefully

---

## Phase 5: Dashboard Widget

**Goal**: Add warning banner to dashboard when policy is outdated.

### Implementation

Create React component `ui/src/components/PolicyVersionBanner.tsx`:

```tsx
import { useQuery } from '@tanstack/react-query';
import { useState, useEffect } from 'react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { AlertTriangle, ExternalLink, X } from 'lucide-react';

const DISMISS_KEY = 'policy-version-dismissed-until';

export function PolicyVersionBanner() {
  const [dismissed, setDismissed] = useState(false);

  const { data: status } = useQuery({
    queryKey: ['policy-version-status'],
    queryFn: async () => {
      const res = await fetch('/api/config/policy-version-status');
      return res.json();
    },
    refetchInterval: 1000 * 60 * 60, // Refetch hourly
  });

  // Check localStorage for dismissal
  useEffect(() => {
    const dismissedUntil = localStorage.getItem(DISMISS_KEY);
    if (dismissedUntil && new Date(dismissedUntil) > new Date()) {
      setDismissed(true);
    }
  }, []);

  const handleDismiss = () => {
    const until = new Date();
    until.setHours(until.getHours() + 24);
    localStorage.setItem(DISMISS_KEY, until.toISOString());
    setDismissed(true);
  };

  if (!status?.is_outdated || dismissed) {
    return null;
  }

  return (
    <Alert variant="warning" className="mb-4">
      <AlertTriangle className="h-4 w-4" />
      <AlertTitle>PAD Policy Updated</AlertTitle>
      <AlertDescription>
        <p>
          The PAD Policy was updated on{' '}
          {status.last_modified ? new Date(status.last_modified).toLocaleDateString() : 'unknown date'}
          {status.last_modified_by && ` by ${status.last_modified_by}`}.
        </p>
        <p className="text-sm text-muted-foreground mt-1">
          Current version: {status.current_version} | System spec version: {status.known_version}
        </p>
        <p className="text-sm mt-2">Some features may need reviewing.</p>
        <div className="flex gap-2 mt-3">
          <Button variant="outline" size="sm" asChild>
            <a href={status.page_url} target="_blank" rel="noopener noreferrer">
              <ExternalLink className="h-3 w-3 mr-1" />
              View Policy
            </a>
          </Button>
          <Button variant="ghost" size="sm" onClick={handleDismiss}>
            <X className="h-3 w-3 mr-1" />
            Dismiss for 24h
          </Button>
        </div>
      </AlertDescription>
    </Alert>
  );
}
```

### Integration

Add to main dashboard layout:

```tsx
import { PolicyVersionBanner } from '@/components/PolicyVersionBanner';

export function DashboardLayout({ children }) {
  return (
    <div>
      <PolicyVersionBanner />
      {children}
    </div>
  );
}
```

### Tasks

- [ ] Create `PolicyVersionBanner.tsx` component
- [ ] Add query for policy status
- [ ] Implement 24h dismissal with localStorage
- [ ] Add to dashboard layout
- [ ] Style warning banner appropriately
- [ ] Add "View Policy" link to Confluence

### Acceptance Criteria

- [ ] Banner only shows when `is_outdated=true`
- [ ] Shows version numbers and last modified info
- [ ] "Dismiss for 24h" hides banner and persists
- [ ] "View Policy" opens Confluence in new tab
- [ ] Banner reappears after 24h

---

## Phase 6: Scheduled Check

**Goal**: Add periodic check to monitoring jobs.

### Implementation

Add to `src/pa_dealing/agents/monitoring/jobs.py`:

```python
from pa_dealing.services.policy_version import check_policy_version

async def check_pad_policy_version():
    """
    Periodic check for PAD policy updates.
    Logs warning if new version detected.
    """
    status = await check_policy_version(force_refresh=True)

    if status.is_outdated:
        logger.warning(
            "PAD policy version outdated - review required",
            extra={
                "current_version": status.current_version,
                "known_version": status.known_version,
                "last_modified": status.last_modified.isoformat() if status.last_modified else None,
                "last_modified_by": status.last_modified_by,
            }
        )

    return status


# Register job
scheduler.add_job(
    check_pad_policy_version,
    trigger=IntervalTrigger(hours=settings.PAD_POLICY_CHECK_INTERVAL_HOURS),
    id="check_pad_policy_version",
    name="PAD Policy Version Check",
    replace_existing=True,
)
```

### Tasks

- [ ] Add `check_pad_policy_version()` job function
- [ ] Register with scheduler at configured interval
- [ ] Add logging for new version detection
- [ ] Add unit test for job

### Acceptance Criteria

- [ ] Job runs at configured interval
- [ ] Logs warning when outdated
- [ ] Job registration doesn't fail if scheduler not running

---

## Phase 7: Testing & Verification

**Goal**: Comprehensive tests and manual verification.

### Tasks

- [ ] Unit tests for policy version service
- [ ] Unit tests for API endpoint
- [ ] Integration test with mocked Confluence
- [ ] Manual test with real Confluence page
- [ ] Verify dashboard banner displays correctly
- [ ] Test dismissal persistence

### Manual Verification Checklist

- [ ] Set `PAD_POLICY_KNOWN_VERSION=1` (lower than actual)
- [ ] Load dashboard - verify banner appears
- [ ] Click "View Policy" - verify link works
- [ ] Click "Dismiss for 24h" - verify banner hides
- [ ] Refresh page - verify banner stays hidden
- [ ] Wait 24h (or clear localStorage) - verify banner returns
- [ ] Set `PAD_POLICY_KNOWN_VERSION` to current version
- [ ] Verify banner no longer appears

### Acceptance Criteria

- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Manual verification checklist complete

---

## Summary

| Phase | Description | Effort |
|-------|-------------|--------|
| 1 | Extend Confluence Client | Small |
| 2 | Policy Version Service | Medium |
| 3 | Configuration | Small |
| 4 | API Endpoint | Small |
| 5 | Dashboard Widget | Medium |
| 6 | Scheduled Check | Small |
| 7 | Testing | Medium |

**Total effort**: ~1-2 days

---

## Out of Scope (Future)

- Slack notification when policy updates
- Admin UI to update known version
- Attachment version tracking
- Policy diff/changelog view
