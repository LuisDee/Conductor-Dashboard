# Spec: Security & Authentication Hardening

## Problem Statement
The autopsy review identified 5 CRITICAL security findings that are production blockers. The system cannot be safely deployed without addressing authentication bypass, CORS misconfiguration, unverified IAP headers, terminated employee access, and hardcoded credentials.

## Source
- `.autopsy/REVIEW_REPORT.md` - Findings #1, #2, #7, #8, #9
- `.autopsy/ARCHITECTURE_REPORT.md` - Section 5: "CRITICAL: No Production-Ready Authentication"

## Findings (All Verified Against Code)

### 1. Dev Auth Bypass Sent in Production (CRITICAL)
- **File:** `dashboard/src/api/client.ts` lines 44-48
- **Issue:** `X-Dev-User-Email` header sent unconditionally on every request via axios interceptor. `getDevUserEmail()` returns a value even in production (defaults to `DEV_USERS[0].email`).
- **Impact:** Any user can impersonate admin by manipulating localStorage.

### 2. CORS Wildcard with Credentials (CRITICAL)
- **File:** `src/pa_dealing/api/main.py` lines 114-120
- **Issue:** `allow_origins=["*"]` with `allow_credentials=True`. Comment acknowledges the problem but code is not gated.
- **Impact:** Any website can make authenticated cross-origin requests to the API.

### 3. IAP Header Without JWT Verification (CRITICAL)
- **File:** `src/pa_dealing/api/auth.py` lines 72-78
- **Issue:** `X-Goog-Authenticated-User-Email` header trusted without JWT signature verification.
- **Impact:** In environments not behind IAP, these headers can be spoofed.

### 4. Terminated Employees Can Authenticate (CRITICAL)
- **File:** `src/pa_dealing/identity/postgres.py` lines 76-156
- **Issue:** `get_by_email()` and `get_by_mako_id()` do not filter by `is_active`. The `is_active` field is computed (`end_date is None`) but never checked in the auth pipeline.
- **Impact:** Former employees retain full access until their credentials expire.

### 5. Hardcoded EODHD API Token (HIGH)
- **File:** `src/pa_dealing/config/settings.py` line 238
- **Issue:** Real API token `688c6b8c5ed5a0.06847867` committed as default value.
- **Impact:** Anyone with repo access can abuse the price API.

## Requirements
1. Gate `X-Dev-User-Email` header by `import.meta.env.MODE === 'development'` in client.ts
2. Replace CORS wildcard with explicit origin whitelist (configurable via env var)
3. Add JWT signature verification for IAP headers using Google's public keys
4. Add `is_active` / `end_date` check in auth pipeline before role resolution
5. Rotate EODHD token, remove default value, require env var
6. Add `.env*` files to `.gitignore` if not already present

## Acceptance Criteria
- [ ] Dev auth header only sent when `import.meta.env.MODE === 'development'`
- [ ] CORS restricted to configured origins (env var `CORS_ALLOWED_ORIGINS`)
- [ ] IAP JWT verified with Google public keys (or IAP validation library)
- [ ] Terminated employees (non-null `end_date`) rejected at auth
- [ ] No hardcoded API tokens in source control
- [ ] All existing tests pass
- [ ] New tests for each security fix
