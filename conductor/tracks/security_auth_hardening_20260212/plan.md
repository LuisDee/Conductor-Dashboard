# Security & Authentication Hardening - Implementation Plan

**Track ID:** security_auth_hardening_20260212
**System:** PA Dealing Compliance System
**Severity:** CRITICAL
**Source:** Autopsy Code Review Findings

## Executive Summary

This plan addresses 5 critical security vulnerabilities discovered during code review:

1. **Dev Auth Bypass** - Client unconditionally sends dev headers in production
2. **CORS Wildcard** - `allow_origins=["*"]` accepts requests from any domain
3. **Terminated Employee Access** - No `is_active` check after auth resolution
4. **Missing IAP JWT Verification** - Trusts raw headers without JWT validation
5. **Hardcoded Credentials** - EODHD API token committed to settings.py

Each phase includes detailed before/after code, tests, rollback strategy, and verification steps.

---

## Phase 1: Dev Auth Bypass Fix

### Problem Statement

**File:** `dashboard/src/api/client.ts` (lines 43-48)

The request interceptor unconditionally sends `X-Dev-User-Email` header on **every request**, regardless of environment. This bypasses authentication in production if the backend accepts it.

**Current Code (VULNERABLE):**
```typescript
// Lines 43-48 in dashboard/src/api/client.ts
// Request interceptor to add dev user header dynamically
api.interceptors.request.use((config) => {
  // Read current dev user from localStorage on each request
  config.headers['X-Dev-User-Email'] = getDevUserEmail();
  return config;
});
```

**File:** `dashboard/src/lib/devUser.ts` (lines 106-109)

`getDevUserEmail()` returns a default user email even when `isDevMode()` returns false:

```typescript
// Lines 106-109 in dashboard/src/lib/devUser.ts
export function getDevUserEmail(): string {
  if (!isDevMode()) {
    return DEV_USERS[0].email; // Default user in production ← SECURITY HOLE
  }
```

### Impact

- **Security:** Production clients send dev headers that could bypass IAP if backend accepts them
- **Compliance:** Audit logs may record wrong user if dev header is trusted over IAP
- **Risk:** Attacker could manipulate `X-Dev-User-Email` header to impersonate users

### Implementation

#### Changes Required

**File 1:** `dashboard/src/api/client.ts` (lines 43-48)

**BEFORE:**
```typescript
// Request interceptor to add dev user header dynamically
api.interceptors.request.use((config) => {
  // Read current dev user from localStorage on each request
  config.headers['X-Dev-User-Email'] = getDevUserEmail();
  return config;
});
```

**AFTER:**
```typescript
// Request interceptor to add dev user header dynamically (DEV ONLY)
api.interceptors.request.use((config) => {
  // Only send dev header in development mode (Vite's import.meta.env.MODE)
  if (import.meta.env.MODE === 'development') {
    config.headers['X-Dev-User-Email'] = getDevUserEmail();
  }
  return config;
});
```

**File 2:** `dashboard/src/lib/devUser.ts` (lines 106-109)

**BEFORE:**
```typescript
export function getDevUserEmail(): string {
  if (!isDevMode()) {
    return DEV_USERS[0].email; // Default user in production
  }
```

**AFTER:**
```typescript
export function getDevUserEmail(): string {
  if (!isDevMode()) {
    return ''; // No dev user in production - return empty string
  }
```

#### New Test File

**Location:** `dashboard/src/api/__tests__/client.test.ts`

```typescript
import axios from 'axios';
import { describe, it, expect, vi, beforeEach } from 'vitest';

describe('API Client - Dev Header Security', () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it('should NOT send X-Dev-User-Email header in production mode', async () => {
    // Mock import.meta.env.MODE as 'production'
    vi.stubGlobal('import', {
      meta: { env: { MODE: 'production', VITE_API_URL: '/api' } }
    });

    // Re-import client to pick up mocked env
    const { default: api } = await import('../client');

    // Mock the request
    const requestSpy = vi.fn((config) => config);
    api.interceptors.request.use(requestSpy);

    try {
      await api.get('/test');
    } catch {
      // Request will fail, we only care about headers
    }

    const lastCall = requestSpy.mock.calls[requestSpy.mock.calls.length - 1];
    const config = lastCall[0];

    expect(config.headers['X-Dev-User-Email']).toBeUndefined();
  });

  it('should send X-Dev-User-Email header in development mode', async () => {
    // Mock import.meta.env.MODE as 'development'
    vi.stubGlobal('import', {
      meta: { env: { MODE: 'development', VITE_API_URL: '/api' } }
    });

    const { default: api } = await import('../client');
    const requestSpy = vi.fn((config) => config);
    api.interceptors.request.use(requestSpy);

    try {
      await api.get('/test');
    } catch {
      // Request will fail, we only care about headers
    }

    const lastCall = requestSpy.mock.calls[requestSpy.mock.calls.length - 1];
    const config = lastCall[0];

    expect(config.headers['X-Dev-User-Email']).toBeDefined();
    expect(typeof config.headers['X-Dev-User-Email']).toBe('string');
  });
});
```

#### Caller Impact Analysis

**Files that import from `client.ts`:**
- All dashboard API consumers (requests, dashboard, auth, etc.)
- **Impact:** None - clients don't need changes, only interceptor behavior changes

**Files that import from `devUser.ts`:**
- `dashboard/src/components/DevUserSwitcher.tsx` (if exists)
- `dashboard/src/api/client.ts`
- **Impact:** None - `getDevUserEmail()` signature unchanged, only return value differs

#### Rollback Strategy

**If production breaks after deployment:**

1. Revert `client.ts` changes:
   ```bash
   git revert <commit-hash> --no-commit
   git checkout HEAD -- dashboard/src/api/client.ts
   git commit -m "Rollback: Restore unconditional dev header for emergency"
   ```

2. Revert `devUser.ts` changes:
   ```bash
   git checkout HEAD -- dashboard/src/lib/devUser.ts
   git commit --amend --no-edit
   ```

3. Rebuild and redeploy dashboard:
   ```bash
   cd dashboard
   npm run build
   # Deploy build/ to CDN/static hosting
   ```

**Rollback time:** ~5 minutes (git revert + rebuild + deploy)

#### Verification

**Pre-deployment:**
```bash
cd dashboard
npm run test -- src/api/__tests__/client.test.ts
npm run build
# Inspect build output - search for 'X-Dev-User-Email' in bundled JS
grep -r "X-Dev-User-Email" dist/
```

**Post-deployment (staging):**
1. Open browser DevTools → Network tab
2. Make API request from staging dashboard
3. Inspect request headers
4. **Expected:** No `X-Dev-User-Email` header present
5. Verify auth still works (IAP header used instead)

**Post-deployment (production):**
1. Same verification as staging
2. Monitor error logs for auth failures (15 minutes)
3. Check Sentry/logging for `401 Unauthorized` spike

---

## Phase 2: CORS Restriction

### Problem Statement

**File:** `src/pa_dealing/api/main.py` (lines 114-120)

CORS middleware accepts requests from **any origin** (`allow_origins=["*"]`), allowing malicious sites to make authenticated requests to the API if a user is logged in.

**Current Code (VULNERABLE):**
```python
# Lines 114-120 in src/pa_dealing/api/main.py
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, restrict to dashboard origin
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
```

### Impact

- **Security:** Any website can make authenticated API requests if user has active IAP session
- **Attack Vector:** CSRF attacks, data exfiltration via malicious JS
- **Compliance:** PCI-DSS, SOC2 require origin restrictions

### Implementation

#### Changes Required

**File 1:** `src/pa_dealing/config/settings.py` (add after line 304)

**BEFORE:**
```python
# Line 304 in settings.py
    )

    @field_validator("database_url")
```

**AFTER:**
```python
    )

    # CORS Security
    cors_allowed_origins: list[str] = Field(
        default_factory=lambda: ["http://localhost:3000", "http://localhost:5173"],
        description="Allowed CORS origins for API requests. In production, set to dashboard URL only.",
    )

    @field_validator("database_url")
```

**File 2:** `src/pa_dealing/api/main.py` (lines 114-120)

**BEFORE:**
```python
# Lines 114-120 in src/pa_dealing/api/main.py
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, restrict to dashboard origin
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
```

**AFTER:**
```python
# Lines 114-120 in src/pa_dealing/api/main.py
# Get CORS origins from settings
settings = get_settings()
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_allowed_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
```

**File 3:** `.env.prod` (create or update)

```bash
# Production CORS - only allow dashboard origin
CORS_ALLOWED_ORIGINS=["https://pad-dashboard.mako.com"]
```

**File 4:** `.env.dev` (create or update)

```bash
# Development CORS - allow local dev servers
CORS_ALLOWED_ORIGINS=["http://localhost:3000", "http://localhost:5173", "http://127.0.0.1:3000", "http://127.0.0.1:5173"]
```

#### New Test File

**Location:** `tests/api/test_cors_security.py`

```python
"""Test CORS security configuration."""

import pytest
from starlette.testclient import TestClient


def test_cors_blocks_unauthorized_origin(api_client):
    """Test that requests from unauthorized origins are blocked."""
    response = api_client.options(
        "/api/health",
        headers={
            "Origin": "https://evil.com",
            "Access-Control-Request-Method": "GET",
        },
    )

    # CORS preflight should succeed (OPTIONS always allowed)
    assert response.status_code == 200

    # But origin should NOT be in allowed origins
    assert response.headers.get("Access-Control-Allow-Origin") != "https://evil.com"


def test_cors_allows_configured_origin(api_client):
    """Test that requests from configured origins are allowed."""
    response = api_client.options(
        "/api/health",
        headers={
            "Origin": "http://localhost:3000",
            "Access-Control-Request-Method": "GET",
        },
    )

    assert response.status_code == 200
    assert response.headers.get("Access-Control-Allow-Origin") == "http://localhost:3000"
    assert response.headers.get("Access-Control-Allow-Credentials") == "true"


def test_cors_credentials_flag_present(api_client):
    """Test that credentials flag is set (required for IAP cookies)."""
    response = api_client.options(
        "/api/health",
        headers={
            "Origin": "http://localhost:3000",
            "Access-Control-Request-Method": "GET",
        },
    )

    assert response.headers.get("Access-Control-Allow-Credentials") == "true"


def test_production_cors_restrictive(monkeypatch):
    """Test that production environment has restrictive CORS."""
    from pa_dealing.config import get_settings

    # Mock production settings
    monkeypatch.setenv("ENVIRONMENT", "production")
    monkeypatch.setenv("CORS_ALLOWED_ORIGINS", '["https://pad-dashboard.mako.com"]')

    get_settings.cache_clear()
    settings = get_settings()

    assert len(settings.cors_allowed_origins) == 1
    assert "https://pad-dashboard.mako.com" in settings.cors_allowed_origins
    assert "*" not in settings.cors_allowed_origins
    assert "localhost" not in str(settings.cors_allowed_origins)
```

#### Caller Impact Analysis

**Affected Components:**
- **Frontend dashboard:** Must be served from origin in `cors_allowed_origins`
- **API Gateway/IAP:** Must preserve `Origin` header from client
- **CDN/Cloudflare:** Must not strip CORS headers

**Breaking Changes:**
- If dashboard is served from origin NOT in config → API requests blocked
- Dev servers on non-standard ports must be added to `.env.dev`

#### Rollback Strategy

**If CORS blocks legitimate traffic:**

1. Immediate hotfix - restore wildcard temporarily:
   ```bash
   # SSH to API server
   export CORS_ALLOWED_ORIGINS='["*"]'
   systemctl restart pa-dealing-api
   ```

2. Git rollback:
   ```bash
   git revert <commit-hash> --no-commit
   git checkout HEAD -- src/pa_dealing/api/main.py src/pa_dealing/config/settings.py
   git commit -m "Rollback: Restore CORS wildcard for emergency"
   ```

3. Redeploy:
   ```bash
   docker build -t pa-dealing-api:rollback .
   docker push pa-dealing-api:rollback
   kubectl rollout undo deployment/pa-dealing-api
   ```

**Rollback time:** ~2 minutes (env var) or ~10 minutes (full rollback + redeploy)

#### Verification

**Pre-deployment:**
```bash
# Run tests
pytest tests/api/test_cors_security.py -v

# Verify settings parse correctly
python -c "from pa_dealing.config import get_settings; print(get_settings().cors_allowed_origins)"
```

**Post-deployment (staging):**
1. Open browser console on staging dashboard
2. Run: `fetch('https://staging-api.mako.com/api/health').then(r => r.json())`
3. **Expected:** Request succeeds (origin allowed)

4. Simulate attack from another domain:
   ```javascript
   // Open browser console on https://google.com
   fetch('https://staging-api.mako.com/api/health', {credentials: 'include'})
     .then(r => r.json())
   ```
5. **Expected:** CORS error in console

**Post-deployment (production):**
1. Same verification as staging
2. Monitor error logs for CORS errors from legitimate clients (30 minutes)
3. Check Sentry for spike in CORS-related errors

---

## Phase 3: Terminated Employee Access Block

### Problem Statement

**File:** `src/pa_dealing/api/auth.py` (lines 106-143)

After resolving employee identity from email, the code does NOT check if employee is active (has `end_date`). Terminated employees can still authenticate and access the system.

**Current Code (VULNERABLE):**
```python
# Lines 106-143 in src/pa_dealing/api/auth.py
# Look up employee and roles using IdentityProvider
async with get_session() as session:
    identity = get_identity_provider_with_session(session)
    employee = await identity.get_by_email(email, resolve_manager=False)

    if not employee:
        # Employee not found - return basic user without roles
        log.warning("user_not_found_in_employee_table", email=email)
        return CurrentUser(
            email=email,
            user_id=user_id,
            # ... truncated
            auth_status="identity_not_found",
            auth_message="Could not resolve your identity in the employee database. Please contact IT support.",
        )

    is_admin = "admin" in employee.roles
    log.info("user_resolved", email=email, roles=employee.roles, is_admin=is_admin)

    return CurrentUser(
        email=email,
        user_id=user_id,
        employee_id=employee.employee_id,
        # ... truncated
        auth_status="ok",
        auth_message=None,
    )
    # ← MISSING: Check employee.is_active before returning
```

**Model Definition:** `src/pa_dealing/db/models/core.py` (lines 87-90)

```python
@property
def is_active(self) -> bool:
    if self.end_date is None:
        return True
    return self.end_date > datetime.now(UTC).replace(tzinfo=None)
```

**Identity Provider:** `src/pa_dealing/identity/base.py` (line 22)

`IdentityInfo` dataclass already has `is_active: bool = True` field.

### Impact

- **Security:** Terminated employees retain full access until removed from Google Workspace
- **Compliance:** SOX, GDPR require immediate access revocation upon termination
- **Audit Risk:** Ex-employees can view sensitive trading data, create requests
- **Data Integrity:** Terminated employees could approve requests if they had roles

### Implementation

#### Changes Required

**File:** `src/pa_dealing/api/auth.py` (after line 124)

**BEFORE:**
```python
    if not employee:
        # Employee not found - return basic user without roles
        log.warning("user_not_found_in_employee_table", email=email)
        return CurrentUser(
            email=email,
            user_id=user_id,
            is_compliance=False,
            is_smf16=False,
            is_admin=False,
            is_manager=False,
            roles=[],
            auth_status="identity_not_found",
            auth_message="Could not resolve your identity in the employee database. Please contact IT support.",
        )

    is_admin = "admin" in employee.roles
    log.info("user_resolved", email=email, roles=employee.roles, is_admin=is_admin)

    return CurrentUser(
```

**AFTER:**
```python
    if not employee:
        # Employee not found - return basic user without roles
        log.warning("user_not_found_in_employee_table", email=email)
        return CurrentUser(
            email=email,
            user_id=user_id,
            is_compliance=False,
            is_smf16=False,
            is_admin=False,
            is_manager=False,
            roles=[],
            auth_status="identity_not_found",
            auth_message="Could not resolve your identity in the employee database. Please contact IT support.",
        )

    # SECURITY: Block terminated employees from accessing system
    if not employee.is_active:
        log.warning(
            "terminated_employee_access_blocked",
            email=email,
            employee_id=employee.employee_id,
            mako_id=employee.mako_id,
        )
        raise HTTPException(
            status_code=HTTP_403_FORBIDDEN,
            detail="Your account has been deactivated. Please contact HR if you believe this is an error.",
        )

    is_admin = "admin" in employee.roles
    log.info("user_resolved", email=email, roles=employee.roles, is_admin=is_admin)

    return CurrentUser(
```

#### New Test File

**Location:** `tests/api/test_auth_terminated_employees.py`

```python
"""Test that terminated employees are blocked from authentication."""

import pytest
from datetime import datetime, timedelta
from fastapi import HTTPException
from starlette.status import HTTP_403_FORBIDDEN


@pytest.mark.asyncio
async def test_terminated_employee_blocked(session, factory):
    """Test that employee with end_date in the past is blocked."""
    from pa_dealing.api.auth import get_current_user
    from fastapi import Request
    from unittest.mock import Mock

    # Create terminated employee (end_date yesterday)
    employee = await factory.create_employee(
        mako_id="terminated_user",
        email="terminated@mako.com",
        status="Terminated",
        end_date=datetime.now() - timedelta(days=1),
    )

    # Mock request with dev header
    request = Mock(spec=Request)

    # Attempt to authenticate as terminated employee
    with pytest.raises(HTTPException) as exc_info:
        await get_current_user(
            request=request,
            x_goog_authenticated_user_email=None,
            x_goog_authenticated_user_id=None,
            x_dev_user_email="terminated@mako.com",
        )

    # Verify 403 Forbidden
    assert exc_info.value.status_code == HTTP_403_FORBIDDEN
    assert "deactivated" in exc_info.value.detail.lower()


@pytest.mark.asyncio
async def test_active_employee_allowed(session, factory):
    """Test that employee with no end_date can authenticate."""
    from pa_dealing.api.auth import get_current_user
    from fastapi import Request
    from unittest.mock import Mock

    # Create active employee (no end_date)
    employee = await factory.create_employee(
        mako_id="active_user",
        email="active@mako.com",
        status="Active",
        end_date=None,
    )

    request = Mock(spec=Request)

    # Attempt to authenticate as active employee
    current_user = await get_current_user(
        request=request,
        x_goog_authenticated_user_email=None,
        x_goog_authenticated_user_id=None,
        x_dev_user_email="active@mako.com",
    )

    # Verify authentication succeeded
    assert current_user.email == "active@mako.com"
    assert current_user.employee_id == employee.id
    assert current_user.auth_status == "ok"


@pytest.mark.asyncio
async def test_future_termination_allowed(session, factory):
    """Test that employee with future end_date can still authenticate."""
    from pa_dealing.api.auth import get_current_user
    from fastapi import Request
    from unittest.mock import Mock

    # Create employee with future end_date (termination scheduled)
    employee = await factory.create_employee(
        mako_id="future_term",
        email="future@mako.com",
        status="Active",
        end_date=datetime.now() + timedelta(days=30),
    )

    request = Mock(spec=Request)

    current_user = await get_current_user(
        request=request,
        x_goog_authenticated_user_email=None,
        x_goog_authenticated_user_id=None,
        x_dev_user_email="future@mako.com",
    )

    # Should still be allowed (end_date in future)
    assert current_user.email == "future@mako.com"
    assert current_user.auth_status == "ok"


@pytest.mark.asyncio
async def test_terminated_employee_audit_log(session, factory):
    """Test that terminated employee access attempts are audit logged."""
    from pa_dealing.api.auth import get_current_user
    from pa_dealing.audit import get_audit_logger
    from fastapi import Request
    from unittest.mock import Mock
    from sqlalchemy import select, text

    employee = await factory.create_employee(
        mako_id="audit_test",
        email="audit@mako.com",
        end_date=datetime.now() - timedelta(days=1),
    )

    request = Mock(spec=Request)

    # Attempt authentication
    with pytest.raises(HTTPException):
        await get_current_user(
            request=request,
            x_goog_authenticated_user_email=None,
            x_goog_authenticated_user_id=None,
            x_dev_user_email="audit@mako.com",
        )

    # Wait for audit log to flush (if async)
    import asyncio
    await asyncio.sleep(0.1)

    # Verify structured log was emitted (check via log capture if using pytest-logging)
    # This is a placeholder - actual implementation depends on log capture setup
    # In production, verify with: grep "terminated_employee_access_blocked" /var/log/pa-dealing.log
```

#### Caller Impact Analysis

**Affected Endpoints:**
- All API endpoints using `Depends(get_current_user)` dependency
- **Impact:** Terminated employees now get `403 Forbidden` instead of successfully authenticating

**Frontend Changes:**
- Dashboard should show user-friendly error when receiving 403 from `/api/auth/me`
- **Recommended:** Add error boundary to display "Account deactivated, contact HR" message

**Potential False Positives:**
- Employees with `end_date` set incorrectly in database
- Contractors with temporary `end_date` that should be extended
- **Mitigation:** HR must validate `end_date` values before deployment

#### Rollback Strategy

**If legitimate employees are blocked:**

1. Immediate fix - remove the check:
   ```bash
   # SSH to API server
   # Edit src/pa_dealing/api/auth.py, comment out lines 127-136
   systemctl restart pa-dealing-api
   ```

2. Git rollback:
   ```bash
   git revert <commit-hash> --no-commit
   git checkout HEAD -- src/pa_dealing/api/auth.py
   git commit -m "Rollback: Remove is_active check (false positives)"
   ```

3. Redeploy:
   ```bash
   docker build -t pa-dealing-api:rollback .
   docker push pa-dealing-api:rollback
   kubectl rollout undo deployment/pa-dealing-api
   ```

**Data Fix (if end_date is incorrect):**
```sql
-- Clear incorrect end_date values
UPDATE bo_airflow.oracle_employee
SET end_date = NULL
WHERE mako_id IN ('user1', 'user2')
  AND status = 'Active';
```

**Rollback time:** ~2 minutes (comment out code) or ~10 minutes (full rollback)

#### Verification

**Pre-deployment:**
```bash
# Run tests
pytest tests/api/test_auth_terminated_employees.py -v

# Validate test data - find terminated employees in DB
docker exec -it pa_dealing_db psql -U pad -d pa_dealing -c \
  "SELECT mako_id, status, end_date FROM bo_airflow.oracle_employee WHERE end_date IS NOT NULL LIMIT 5;"
```

**Post-deployment (staging):**
1. Create test employee with `end_date` yesterday:
   ```sql
   INSERT INTO bo_airflow.oracle_employee (mako_id, status, end_date)
   VALUES ('test_terminated', 'Terminated', NOW() - INTERVAL '1 day');

   INSERT INTO bo_airflow.oracle_contact (employee_id, email, contact_group_id, contact_type_id)
   SELECT id, 'test_terminated@mako.com', 2, 5
   FROM bo_airflow.oracle_employee WHERE mako_id = 'test_terminated';
   ```

2. Attempt to authenticate as terminated employee:
   ```bash
   curl -H "X-Dev-User-Email: test_terminated@mako.com" \
        https://staging-api.mako.com/api/auth/me
   ```
   **Expected:** `403 Forbidden` with "deactivated" message

3. Verify active employees unaffected:
   ```bash
   curl -H "X-Dev-User-Email: jsmith@mako.com" \
        https://staging-api.mako.com/api/auth/me
   ```
   **Expected:** `200 OK` with user details

**Post-deployment (production):**
1. Monitor logs for `terminated_employee_access_blocked` events (30 minutes)
2. Check Sentry for spike in 403 errors
3. Verify no complaints from active employees about access issues

---

## Phase 4: IAP JWT Verification

### Problem Statement

**File:** `src/pa_dealing/api/auth.py` (lines 72-78)

The code trusts the `X-Goog-Authenticated-User-Email` header **without verifying the IAP JWT**. An attacker controlling the proxy could forge this header.

**Current Code (VULNERABLE):**
```python
# Lines 72-78 in src/pa_dealing/api/auth.py
# 1. Check IAP headers (production)
if x_goog_authenticated_user_email:
    # Format: accounts.google.com:user@example.com
    email = x_goog_authenticated_user_email
    if ":" in email:
        email = email.split(":", 1)[1]
    user_id = x_goog_authenticated_user_id
    # ← MISSING: Verify JWT signature and claims
```

### Background

Google Identity-Aware Proxy (IAP) provides signed JWT assertions in the `X-Goog-IAP-JWT-Assertion` header. The application MUST verify this JWT to ensure the request actually came through IAP and wasn't forged.

**JWT Verification Steps:**
1. Fetch Google's public keys from https://www.gstatic.com/iap/verify/public_key-jwk
2. Verify JWT signature using public key
3. Verify JWT claims (issuer, audience, expiry)
4. Extract email from verified claims instead of trusting raw header

### Impact

- **Security:** Attacker bypassing IAP could forge user headers and impersonate anyone
- **Attack Scenario:** Internal network attacker sends requests directly to backend
- **Compliance:** Google Cloud security best practices require JWT verification
- **Audit:** Forged authentication would be undetectable in logs

### Implementation

#### Changes Required

**File 1:** `src/pa_dealing/config/settings.py` (add after line 296)

**BEFORE:**
```python
    # Application
    environment: Environment = Field(
        default=Environment.DEVELOPMENT,
        description="Application environment",
    )
```

**AFTER:**
```python
    # Google IAP JWT Verification
    iap_audience: str = Field(
        default="",
        description="Expected audience for IAP JWT verification (format: /projects/PROJECT_NUMBER/apps/PROJECT_ID)",
    )
    iap_issuer: str = Field(
        default="https://cloud.google.com/iap",
        description="Expected issuer for IAP JWT verification",
    )

    # Application
    environment: Environment = Field(
        default=Environment.DEVELOPMENT,
        description="Application environment",
    )
```

**File 2:** Create `src/pa_dealing/api/iap_verification.py`

```python
"""Google Identity-Aware Proxy JWT verification.

Reference: https://cloud.google.com/iap/docs/signed-headers-howto
"""

import structlog
from google.auth.transport import requests
from google.oauth2 import id_token

log = structlog.get_logger()


class IAPVerificationError(Exception):
    """Raised when IAP JWT verification fails."""
    pass


async def verify_iap_jwt(iap_jwt: str, expected_audience: str) -> dict:
    """Verify Google IAP JWT signature and claims.

    Args:
        iap_jwt: JWT from X-Goog-IAP-JWT-Assertion header
        expected_audience: Expected audience (format: /projects/PROJECT_NUMBER/apps/PROJECT_ID)

    Returns:
        Verified JWT claims dictionary with 'email', 'sub', 'iss', 'aud', etc.

    Raises:
        IAPVerificationError: If JWT verification fails
    """
    try:
        # Verify JWT signature using Google's public keys
        # id_token.verify_oauth2_token automatically fetches public keys from:
        # https://www.gstatic.com/iap/verify/public_key-jwk
        decoded_jwt = id_token.verify_oauth2_token(
            iap_jwt,
            requests.Request(),
            audience=expected_audience,
        )

        # Verify issuer
        if decoded_jwt.get("iss") != "https://cloud.google.com/iap":
            raise IAPVerificationError(
                f"Invalid issuer: {decoded_jwt.get('iss')} "
                "(expected https://cloud.google.com/iap)"
            )

        # Verify email is present
        if "email" not in decoded_jwt:
            raise IAPVerificationError("JWT missing email claim")

        log.info(
            "iap_jwt_verified",
            email=decoded_jwt.get("email"),
            subject=decoded_jwt.get("sub"),
        )

        return decoded_jwt

    except ValueError as e:
        # Invalid JWT signature, expired, or malformed
        log.error("iap_jwt_verification_failed", error=str(e))
        raise IAPVerificationError(f"JWT verification failed: {e}") from e
```

**File 3:** `src/pa_dealing/api/auth.py` (lines 52-78)

**BEFORE:**
```python
async def get_current_user(
    request: Request,
    x_goog_authenticated_user_email: str | None = Header(
        None, alias="X-Goog-Authenticated-User-Email"
    ),
    x_goog_authenticated_user_id: str | None = Header(None, alias="X-Goog-Authenticated-User-Id"),
    x_dev_user_email: str | None = Header(None, alias="X-Dev-User-Email"),
) -> CurrentUser:
    """
    Get the current authenticated user.

    Priority:
    1. IAP headers (production)
    2. Dev header (development)
    3. BYPASS_AUTH setting (testing)
    """
    settings = get_settings()
    email: str | None = None
    user_id: str | None = None

    # 1. Check IAP headers (production)
    if x_goog_authenticated_user_email:
        # Format: accounts.google.com:user@example.com
        email = x_goog_authenticated_user_email
        if ":" in email:
            email = email.split(":", 1)[1]
        user_id = x_goog_authenticated_user_id
```

**AFTER:**
```python
async def get_current_user(
    request: Request,
    x_goog_authenticated_user_email: str | None = Header(
        None, alias="X-Goog-Authenticated-User-Email"
    ),
    x_goog_authenticated_user_id: str | None = Header(None, alias="X-Goog-Authenticated-User-Id"),
    x_goog_iap_jwt_assertion: str | None = Header(None, alias="X-Goog-IAP-JWT-Assertion"),
    x_dev_user_email: str | None = Header(None, alias="X-Dev-User-Email"),
) -> CurrentUser:
    """
    Get the current authenticated user.

    Priority:
    1. IAP JWT (production) - verifies signature
    2. Dev header (development)
    3. BYPASS_AUTH setting (testing)
    """
    settings = get_settings()
    email: str | None = None
    user_id: str | None = None

    # 1. Check IAP JWT (production) - SECURE PATH
    if x_goog_iap_jwt_assertion and settings.iap_audience:
        from .iap_verification import verify_iap_jwt, IAPVerificationError

        try:
            # Verify JWT signature and extract claims
            jwt_claims = await verify_iap_jwt(
                x_goog_iap_jwt_assertion,
                expected_audience=settings.iap_audience,
            )

            # Extract email from verified claims (NOT from raw header)
            email = jwt_claims.get("email")
            user_id = jwt_claims.get("sub")  # Google user ID from JWT

            log.info("iap_authenticated", email=email, subject=user_id)

        except IAPVerificationError as e:
            log.error("iap_verification_failed", error=str(e))
            raise HTTPException(
                status_code=HTTP_401_UNAUTHORIZED,
                detail="IAP JWT verification failed. Authentication required.",
            )

    # 1b. Fallback to IAP headers (for backwards compatibility during migration)
    # TODO(SECURITY): Remove this fallback after IAP JWT verification is proven stable
    elif x_goog_authenticated_user_email and not settings.iap_audience:
        log.warning(
            "iap_jwt_verification_disabled",
            reason="iap_audience not configured",
            email=x_goog_authenticated_user_email,
        )
        # Format: accounts.google.com:user@example.com
        email = x_goog_authenticated_user_email
        if ":" in email:
            email = email.split(":", 1)[1]
        user_id = x_goog_authenticated_user_id
```

**File 4:** `.env.prod` (add IAP audience)

```bash
# Google IAP JWT Verification
IAP_AUDIENCE="/projects/123456789/apps/pa-dealing-prod"  # Replace with actual project number/ID
IAP_ISSUER="https://cloud.google.com/iap"
```

#### New Test File

**Location:** `tests/api/test_iap_jwt_verification.py`

```python
"""Test Google IAP JWT verification."""

import pytest
from unittest.mock import Mock, patch
from datetime import datetime, timedelta
from jose import jwt  # PyJWT alternative, or use google.auth directly


@pytest.mark.asyncio
async def test_iap_jwt_verification_success():
    """Test successful IAP JWT verification."""
    from pa_dealing.api.iap_verification import verify_iap_jwt

    # Mock JWT claims
    mock_claims = {
        "iss": "https://cloud.google.com/iap",
        "aud": "/projects/123456789/apps/pa-dealing",
        "email": "test@mako.com",
        "sub": "google-oauth2|123456",
        "exp": datetime.utcnow() + timedelta(hours=1),
        "iat": datetime.utcnow(),
    }

    # Mock google.oauth2.id_token.verify_oauth2_token
    with patch("pa_dealing.api.iap_verification.id_token.verify_oauth2_token") as mock_verify:
        mock_verify.return_value = mock_claims

        result = await verify_iap_jwt(
            iap_jwt="mock.jwt.token",
            expected_audience="/projects/123456789/apps/pa-dealing",
        )

        assert result["email"] == "test@mako.com"
        assert result["sub"] == "google-oauth2|123456"
        assert result["iss"] == "https://cloud.google.com/iap"


@pytest.mark.asyncio
async def test_iap_jwt_verification_invalid_signature():
    """Test that invalid JWT signature raises error."""
    from pa_dealing.api.iap_verification import verify_iap_jwt, IAPVerificationError

    # Mock verification failure (invalid signature)
    with patch("pa_dealing.api.iap_verification.id_token.verify_oauth2_token") as mock_verify:
        mock_verify.side_effect = ValueError("Invalid signature")

        with pytest.raises(IAPVerificationError) as exc_info:
            await verify_iap_jwt(
                iap_jwt="invalid.jwt.token",
                expected_audience="/projects/123456789/apps/pa-dealing",
            )

        assert "verification failed" in str(exc_info.value).lower()


@pytest.mark.asyncio
async def test_iap_jwt_verification_wrong_issuer():
    """Test that wrong issuer is rejected."""
    from pa_dealing.api.iap_verification import verify_iap_jwt, IAPVerificationError

    mock_claims = {
        "iss": "https://evil.com",  # Wrong issuer
        "aud": "/projects/123456789/apps/pa-dealing",
        "email": "test@mako.com",
        "sub": "12345",
    }

    with patch("pa_dealing.api.iap_verification.id_token.verify_oauth2_token") as mock_verify:
        mock_verify.return_value = mock_claims

        with pytest.raises(IAPVerificationError) as exc_info:
            await verify_iap_jwt(
                iap_jwt="mock.jwt.token",
                expected_audience="/projects/123456789/apps/pa-dealing",
            )

        assert "invalid issuer" in str(exc_info.value).lower()


@pytest.mark.asyncio
async def test_auth_with_valid_iap_jwt(api_client, monkeypatch):
    """Test authentication endpoint with valid IAP JWT."""
    from pa_dealing.api.iap_verification import verify_iap_jwt

    # Mock IAP audience configuration
    monkeypatch.setenv("IAP_AUDIENCE", "/projects/123456789/apps/pa-dealing")

    mock_claims = {
        "iss": "https://cloud.google.com/iap",
        "aud": "/projects/123456789/apps/pa-dealing",
        "email": "jsmith@mako.com",
        "sub": "google-oauth2|123",
    }

    with patch("pa_dealing.api.auth.verify_iap_jwt") as mock_verify:
        mock_verify.return_value = mock_claims

        response = api_client.get(
            "/api/auth/me",
            headers={"X-Goog-IAP-JWT-Assertion": "mock.jwt.token"},
        )

        assert response.status_code == 200
        data = response.json()
        assert data["data"]["email"] == "jsmith@mako.com"


@pytest.mark.asyncio
async def test_auth_rejects_forged_header_without_jwt(api_client, monkeypatch):
    """Test that forged X-Goog-Authenticated-User-Email is rejected without valid JWT."""
    # Mock production environment with IAP audience configured
    monkeypatch.setenv("ENVIRONMENT", "production")
    monkeypatch.setenv("IAP_AUDIENCE", "/projects/123456789/apps/pa-dealing")

    # Attempt to authenticate with forged header but no JWT
    response = api_client.get(
        "/api/auth/me",
        headers={
            "X-Goog-Authenticated-User-Email": "accounts.google.com:attacker@mako.com",
            # No X-Goog-IAP-JWT-Assertion header
        },
    )

    # Should fail because JWT is missing
    assert response.status_code == 401
```

#### Caller Impact Analysis

**Affected Components:**
- **IAP Configuration:** Must be configured to send `X-Goog-IAP-JWT-Assertion` header
- **All API endpoints:** No changes needed (dependency injection handles it)
- **Frontend:** No changes needed (IAP adds headers transparently)

**Breaking Changes:**
- If `IAP_AUDIENCE` is configured but IAP doesn't send JWT → all requests fail with 401
- Dev mode unaffected (still uses `X-Dev-User-Email`)

#### Rollback Strategy

**If JWT verification breaks production:**

1. Immediate hotfix - disable JWT verification:
   ```bash
   # SSH to API server
   unset IAP_AUDIENCE  # Fallback to header-based auth
   systemctl restart pa-dealing-api
   ```

2. Git rollback:
   ```bash
   git revert <commit-hash> --no-commit
   git checkout HEAD -- src/pa_dealing/api/auth.py src/pa_dealing/api/iap_verification.py src/pa_dealing/config/settings.py
   git commit -m "Rollback: Disable IAP JWT verification"
   ```

3. Redeploy without IAP_AUDIENCE env var

**Rollback time:** ~1 minute (env var) or ~10 minutes (full rollback)

#### Verification

**Pre-deployment:**
```bash
# Install google-auth (already in deps)
pip install google-auth

# Run tests
pytest tests/api/test_iap_jwt_verification.py -v

# Verify settings
python -c "from pa_dealing.config import get_settings; print(get_settings().iap_audience)"
```

**Post-deployment (staging - with IAP configured):**

1. Get actual IAP JWT from staging request:
   ```bash
   # Make request through IAP
   curl -H "Authorization: Bearer $(gcloud auth print-identity-token)" \
        https://staging-api.mako.com/api/auth/me -v 2>&1 | grep "X-Goog-IAP-JWT-Assertion"
   ```

2. Test JWT verification manually:
   ```python
   from pa_dealing.api.iap_verification import verify_iap_jwt
   import asyncio

   jwt_token = "eyJhbGc..."  # From step 1
   audience = "/projects/123456789/apps/pa-dealing-staging"

   claims = asyncio.run(verify_iap_jwt(jwt_token, audience))
   print(claims)
   ```

3. Verify authentication works through IAP:
   ```bash
   gcloud auth login
   curl -H "Authorization: Bearer $(gcloud auth print-identity-token)" \
        https://staging-api.mako.com/api/auth/me
   ```
   **Expected:** 200 OK with user details

**Post-deployment (production):**
1. Same verification as staging
2. Monitor logs for `iap_jwt_verified` events (30 minutes)
3. Check for `iap_verification_failed` errors (should be zero)

---

## Phase 5: Credential Rotation & Cleanup

### Problem Statement

**File:** `src/pa_dealing/config/settings.py` (lines 237-240)

The EODHD API token is **hardcoded as the default value** in settings.py, meaning it's committed to Git and visible to anyone with repo access.

**Current Code (VULNERABLE):**
```python
# Lines 237-240 in src/pa_dealing/config/settings.py
eodhd_api_token: str = Field(
    default="688c6b8c5ed5a0.06847867",  # ← HARDCODED CREDENTIAL IN GIT
    description="API token for EODHD (provided in gemini-plan-eodhd.plan)",
)
```

### Impact

- **Security:** API token visible in Git history, even if removed now
- **Compliance:** PCI-DSS, SOC2 require secrets in secure storage (not Git)
- **Cost Risk:** Anyone with token can make unlimited EODHD API calls on our account
- **Audit Finding:** Credential committed to version control = automatic compliance failure

### Implementation

#### Changes Required

**File 1:** `src/pa_dealing/config/settings.py` (lines 237-240)

**BEFORE:**
```python
eodhd_api_token: str = Field(
    default="688c6b8c5ed5a0.06847867",
    description="API token for EODHD (provided in gemini-plan-eodhd.plan)",
)
```

**AFTER:**
```python
eodhd_api_token: str = Field(
    default="",
    description="API token for EODHD. REQUIRED in production. Set via EODHD_API_TOKEN env var.",
)
```

**File 2:** `.env.prod` (create or update - DO NOT COMMIT)

```bash
# EODHD API Token (REQUIRED for production)
# Obtain from: https://eodhd.com/cp/settings/api-key
EODHD_API_TOKEN=your-new-token-here  # TODO: Generate new token and add to secret manager
```

**File 3:** `.env.dev` (create or update - DO NOT COMMIT)

```bash
# EODHD API Token (DEV - use free tier token)
EODHD_API_TOKEN=dev-token-from-free-tier
```

**File 4:** `.gitignore` (verify these entries exist)

```bash
# Environment files with secrets
.env
.env.*
.env.local
.env.production
.env.prod
.env.dev
.env.staging

# Exception: .env.example is allowed (no secrets)
!.env.example
```

**File 5:** `.env.example` (create as template - SAFE TO COMMIT)

```bash
# Example environment configuration
# Copy to .env.prod and fill in actual values

# Database
DATABASE_URL=postgresql+asyncpg://user:pass@localhost:5432/pa_dealing
DATABASE_SCHEMA=padealing
REFERENCE_SCHEMA=bo_airflow

# EODHD API (REQUIRED)
EODHD_API_TOKEN=your-token-here

# Google IAP
IAP_AUDIENCE=/projects/PROJECT_NUMBER/apps/PROJECT_ID

# CORS
CORS_ALLOWED_ORIGINS=["https://dashboard.example.com"]
```

#### Secret Rotation Steps

**Step 1: Generate New EODHD Token**
1. Log in to https://eodhd.com/cp/settings/api-key
2. Generate new API key
3. Copy to password manager (1Password, LastPass, etc.)
4. Store in secret manager (Google Secret Manager, AWS Secrets Manager, etc.)

**Step 2: Update Production Secrets**
```bash
# Google Secret Manager (example)
echo -n "new-eodhd-token-here" | gcloud secrets create eodhd-api-token \
  --data-file=- \
  --replication-policy=automatic

# Update deployment to mount secret as env var
kubectl create secret generic pa-dealing-secrets \
  --from-literal=EODHD_API_TOKEN="new-token" \
  --dry-run=client -o yaml | kubectl apply -f -
```

**Step 3: Revoke Old Token**
1. Return to https://eodhd.com/cp/settings/api-key
2. Delete/revoke old token `688c6b8c5ed5a0.06847867`
3. Verify new token works:
   ```bash
   curl "https://eodhd.com/api/real-time/AAPL.US?api_token=NEW_TOKEN&fmt=json"
   ```

**Step 4: Audit Git History**
```bash
# Search for other hardcoded secrets in Git history
git log -p -S "api_token" -S "password" -S "secret" --all

# If found, consider using git-filter-branch to remove from history (RISKY)
# Safer: Rotate all exposed credentials and document in security log
```

#### New Test File

**Location:** `tests/config/test_settings_security.py`

```python
"""Test that sensitive settings are not hardcoded."""

import pytest
import re


def test_no_hardcoded_api_tokens():
    """Test that API tokens are not hardcoded in settings.py."""
    from pathlib import Path

    settings_file = Path("src/pa_dealing/config/settings.py")
    content = settings_file.read_text()

    # Pattern to detect potential API tokens/secrets in default values
    # Look for Field(default="..." where value looks like a token
    suspicious_patterns = [
        r'default\s*=\s*["\'][a-f0-9]{16,}["\']',  # Hex tokens
        r'default\s*=\s*["\'][A-Za-z0-9_-]{20,}["\']',  # Base64-like tokens
        r'default\s*=\s*["\'].*api.*token.*["\']',  # Explicit api_token strings
        r'default\s*=\s*["\'].*secret.*["\']',  # Secret strings
    ]

    for pattern in suspicious_patterns:
        matches = re.findall(pattern, content, re.IGNORECASE)
        # Filter out obviously safe defaults like URLs, paths, etc.
        dangerous_matches = [
            m for m in matches
            if not any(safe in m.lower() for safe in ["http", "localhost", "example", "test"])
        ]

        if dangerous_matches:
            pytest.fail(
                f"Potential hardcoded secret found in settings.py: {dangerous_matches}\n"
                "Secrets MUST be loaded from environment variables, not hardcoded."
            )


def test_eodhd_token_not_hardcoded():
    """Specifically test that EODHD token is not hardcoded."""
    from pa_dealing.config import Settings
    from pydantic import Field
    import inspect

    # Get Field definition for eodhd_api_token
    field_info = Settings.model_fields["eodhd_api_token"]

    # Check default value is empty or None
    assert field_info.default in ("", None), \
        f"eodhd_api_token has hardcoded default: {field_info.default}"


def test_env_files_in_gitignore():
    """Test that .env files are in .gitignore."""
    from pathlib import Path

    gitignore = Path(".gitignore")
    if not gitignore.exists():
        pytest.fail(".gitignore file missing!")

    content = gitignore.read_text()

    required_ignores = [".env", ".env.prod", ".env.local"]
    for pattern in required_ignores:
        assert pattern in content, \
            f"{pattern} not found in .gitignore - secrets may be committed!"


def test_production_settings_require_env_vars(monkeypatch):
    """Test that production environment requires critical env vars."""
    from pa_dealing.config import get_settings

    # Mock production environment
    monkeypatch.setenv("ENVIRONMENT", "production")
    monkeypatch.delenv("EODHD_API_TOKEN", raising=False)

    get_settings.cache_clear()
    settings = get_settings()

    # In production, EODHD token should be empty if not set (forcing explicit config)
    assert settings.eodhd_api_token == "", \
        "Production should not have default EODHD token"
```

#### Caller Impact Analysis

**Affected Components:**
- **Instrument resolver:** Uses `settings.eodhd_api_token` to call EODHD API
- **Impact:** Will fail if env var not set (graceful degradation)

**Deployment Requirements:**
- **Production:** MUST set `EODHD_API_TOKEN` env var before deployment
- **Staging:** MUST set `EODHD_API_TOKEN` env var
- **Development:** MUST create `.env.dev` with token (from free tier)

**Breaking Changes:**
- Deployments without `EODHD_API_TOKEN` env var will fail instrument lookups
- Docker images must mount secrets or pass env vars

#### Rollback Strategy

**If instrument resolution breaks after deployment:**

1. Immediate hotfix - restore old token temporarily:
   ```bash
   # SSH to API server
   export EODHD_API_TOKEN="688c6b8c5ed5a0.06847867"  # Old token
   systemctl restart pa-dealing-api
   ```

2. Long-term fix - add secret to deployment:
   ```bash
   # Kubernetes
   kubectl set env deployment/pa-dealing-api EODHD_API_TOKEN="new-token"

   # Docker Compose
   echo "EODHD_API_TOKEN=new-token" >> .env.prod
   docker-compose up -d --force-recreate
   ```

**Rollback time:** ~1 minute (env var fix)

**NOTE:** Do NOT rollback settings.py code - keep default empty even after fixing env var

#### Verification

**Pre-deployment:**
```bash
# Run security tests
pytest tests/config/test_settings_security.py -v

# Verify .gitignore
cat .gitignore | grep ".env"

# Search for hardcoded secrets in codebase
rg -i "api.*token.*=.*['\"][a-z0-9]{16,}" --type py

# Verify EODHD_API_TOKEN is NOT in git
git log -p -S "688c6b8c5ed5a0.06847867" --all
```

**Post-deployment (staging):**
```bash
# Verify env var is loaded
kubectl exec -it deployment/pa-dealing-api -- env | grep EODHD_API_TOKEN
# Should show: EODHD_API_TOKEN=****** (masked)

# Test instrument resolution works
curl https://staging-api.mako.com/api/dashboard/instruments/search?q=AAPL
# Should return results (proves EODHD token is working)
```

**Post-deployment (production):**
1. Same verification as staging
2. Monitor logs for EODHD API errors (30 minutes)
3. Check Sentry for "API token missing" or "Authentication failed" errors

**Post-rotation:**
```bash
# Verify old token is revoked
curl "https://eodhd.com/api/real-time/AAPL.US?api_token=688c6b8c5ed5a0.06847867&fmt=json"
# Should return: {"error": "Invalid API token"}

# Verify new token works
curl "https://eodhd.com/api/real-time/AAPL.US?api_token=NEW_TOKEN&fmt=json"
# Should return: {"code": "AAPL.US", "timestamp": ..., "close": ...}
```

---

## Cross-Phase Dependencies

**Dependency Graph:**

```
Phase 1 (Dev Auth Bypass) ──┐
Phase 2 (CORS)              ├──> Phase 5 (Credential Rotation)
Phase 3 (Terminated Access) │
Phase 4 (IAP JWT)           ─┘
```

**Execution Order:**
1. **Phase 5 FIRST** - Rotate credentials before any security changes
2. **Phases 1-4 in parallel** - Independent changes, can be deployed simultaneously
3. **Full verification** - Test all phases together before production

**Why Phase 5 First?**
- If credentials are already compromised, rotating them is urgent
- Other phases depend on trust in credentials (e.g., IAP JWT uses Google creds)
- Credential rotation has no dependency on other phases

---

## Deployment Strategy

### Recommended Approach: Staged Rollout

**Week 1: Phase 5 (Credential Rotation)**
- Rotate EODHD token
- Audit Git history for other hardcoded secrets
- Update all environments with new token
- Verify no production impact

**Week 2: Phase 1 + Phase 2 (Dev Auth + CORS)**
- Deploy frontend changes (Phase 1)
- Deploy backend changes (Phase 2)
- Low risk: Dev mode changes, CORS config
- Monitor for 48 hours

**Week 3: Phase 3 (Terminated Employee Access)**
- Deploy is_active check
- **CRITICAL:** Audit employee database BEFORE deployment
  - Find employees with incorrect `end_date` values
  - Fix data issues before deploying code
- Monitor for false positives (24 hours)

**Week 4: Phase 4 (IAP JWT Verification)**
- Configure IAP audience in staging
- Deploy JWT verification code
- Test extensively in staging (1 week)
- **Gradual production rollout:**
  - Day 1: Deploy with `IAP_AUDIENCE` unset (fallback mode)
  - Day 2: Set `IAP_AUDIENCE` for 10% of traffic (canary)
  - Day 3: Increase to 50% of traffic
  - Day 4: 100% of traffic
  - Day 5: Remove fallback code

### Emergency Rollback Plan

**All-phases rollback:**
```bash
# 1. Git rollback to pre-security-hardening commit
git revert <first-security-commit>^..<last-security-commit>

# 2. Restore environment to pre-hardening state
export CORS_ALLOWED_ORIGINS='["*"]'
unset IAP_AUDIENCE
export EODHD_API_TOKEN="688c6b8c5ed5a0.06847867"  # Old token (temp)

# 3. Redeploy
docker build -t pa-dealing-api:rollback .
kubectl set image deployment/pa-dealing-api api=pa-dealing-api:rollback

# 4. Rebuild frontend without dev header gate
cd dashboard
git checkout <pre-security-commit> -- src/api/client.ts
npm run build
# Deploy to CDN
```

**Rollback time:** ~15 minutes (full stack rollback)

---

## Testing Checklist

### Pre-Deployment Tests

- [ ] **Phase 1:** `npm run test -- src/api/__tests__/client.test.ts`
- [ ] **Phase 2:** `pytest tests/api/test_cors_security.py -v`
- [ ] **Phase 3:** `pytest tests/api/test_auth_terminated_employees.py -v`
- [ ] **Phase 4:** `pytest tests/api/test_iap_jwt_verification.py -v`
- [ ] **Phase 5:** `pytest tests/config/test_settings_security.py -v`
- [ ] **Integration:** `pytest tests/ -v` (full test suite)
- [ ] **Frontend build:** `cd dashboard && npm run build`
- [ ] **Backend build:** `docker build -t pa-dealing-api:test .`

### Post-Deployment Verification (Staging)

- [ ] **Phase 1:** Browser DevTools → no `X-Dev-User-Email` in production build
- [ ] **Phase 2:** CORS test from unauthorized origin → blocked
- [ ] **Phase 3:** Terminated employee → 403 Forbidden
- [ ] **Phase 4:** IAP JWT verification → logs show `iap_jwt_verified`
- [ ] **Phase 5:** Instrument search works → EODHD API calls successful

### Post-Deployment Verification (Production)

- [ ] **Monitoring:** No spike in 401/403 errors (30 minutes)
- [ ] **Logging:** Check for security events in structured logs
- [ ] **Sentry:** No new auth-related errors
- [ ] **User Reports:** No complaints about access issues (24 hours)
- [ ] **Audit:** Verify all security controls active in production

---

## Success Criteria

### Phase 1: Dev Auth Bypass Fix
- ✅ Production dashboard does NOT send `X-Dev-User-Email` header
- ✅ Development dashboard DOES send `X-Dev-User-Email` header
- ✅ Backend auth works in both environments

### Phase 2: CORS Restriction
- ✅ Requests from dashboard origin → allowed
- ✅ Requests from unauthorized origin → blocked with CORS error
- ✅ `allow_origins=["*"]` removed from code

### Phase 3: Terminated Employee Access Block
- ✅ Employee with `end_date` in past → 403 Forbidden
- ✅ Employee with `end_date = NULL` → allowed
- ✅ Structured log event for terminated access attempts

### Phase 4: IAP JWT Verification
- ✅ Valid IAP JWT → authenticated
- ✅ Invalid/missing JWT → 401 Unauthorized
- ✅ Forged headers without JWT → rejected
- ✅ Logs show `iap_jwt_verified` events

### Phase 5: Credential Rotation
- ✅ No hardcoded tokens in settings.py (default="")
- ✅ Old EODHD token revoked
- ✅ New token working in production
- ✅ `.env*` files in `.gitignore`

---

## Documentation Updates

### Files to Update After Completion

1. **`README.md`** - Add security section:
   ```markdown
   ## Security

   - **Authentication:** Google Identity-Aware Proxy with JWT verification
   - **CORS:** Restricted to dashboard origin only
   - **Access Control:** Terminated employees automatically blocked
   - **Secrets:** All credentials in environment variables (never Git)
   ```

2. **`docs/deployment.md`** - Add required env vars:
   ```markdown
   ### Required Environment Variables (Production)

   - `IAP_AUDIENCE`: Google IAP audience for JWT verification
   - `CORS_ALLOWED_ORIGINS`: Dashboard URL (JSON array)
   - `EODHD_API_TOKEN`: EODHD API token (from secret manager)
   ```

3. **`SECURITY.md`** - Create security policy:
   ```markdown
   # Security Policy

   ## Reporting Vulnerabilities

   Email: security@mako.com

   ## Security Controls

   1. IAP JWT verification on all requests
   2. CORS restricted to dashboard origin
   3. Terminated employee access blocked
   4. No credentials in Git
   5. All secrets in Google Secret Manager
   ```

---

## Post-Implementation Audit

### Week 1 Post-Deployment

- [ ] Review auth logs for anomalies
- [ ] Check for false positive blocked users
- [ ] Verify EODHD API usage (no unexpected spikes)
- [ ] Confirm no hardcoded secrets in Git

### Month 1 Post-Deployment

- [ ] Security scan with Snyk/Trivy/Bandit
- [ ] Penetration test of auth flow
- [ ] CORS compliance test
- [ ] Credential rotation schedule established

### Ongoing Monitoring

- [ ] Weekly: Check for terminated employees in audit logs
- [ ] Monthly: Rotate EODHD API token
- [ ] Quarterly: Security audit of authentication code
- [ ] Annually: Full penetration test

---

## Appendix: Reference Links

### Google IAP Documentation
- JWT Verification: https://cloud.google.com/iap/docs/signed-headers-howto
- IAP Architecture: https://cloud.google.com/iap/docs/concepts-overview

### Security Best Practices
- OWASP CORS: https://cheatsheetseries.owasp.org/cheatsheets/CORS_Cheat_Sheet.html
- OWASP Authentication: https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html
- Secrets Management: https://cloud.google.com/secret-manager/docs/best-practices

### Testing Resources
- pytest-asyncio: https://pytest-asyncio.readthedocs.io/
- Starlette TestClient: https://www.starlette.io/testclient/
- Vitest: https://vitest.dev/

---

**Plan Version:** 1.0
**Last Updated:** 2026-02-12
**Owner:** Security Team
**Reviewers:** Backend Lead, Frontend Lead, Compliance Officer
