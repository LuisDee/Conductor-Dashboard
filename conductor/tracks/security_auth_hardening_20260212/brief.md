# Track Brief: Security & Authentication Hardening

**Goal**: Fix all CRITICAL security findings that block production deployment.

**Source**: `.autopsy/REVIEW_REPORT.md` + `.autopsy/ARCHITECTURE_REPORT.md` - verified against code.

## Scope
5 verified security issues: dev auth bypass in production, CORS wildcard with credentials, IAP JWT not verified, terminated employees can authenticate, hardcoded API token.

## Key Files
- `dashboard/src/api/client.ts` (auth header)
- `src/pa_dealing/api/main.py` (CORS)
- `src/pa_dealing/api/auth.py` (IAP + auth pipeline)
- `src/pa_dealing/identity/postgres.py` (employee lookup)
- `src/pa_dealing/config/settings.py` (hardcoded token)

## Effort Estimate
M (1-2 weeks) - requires end-to-end auth testing
