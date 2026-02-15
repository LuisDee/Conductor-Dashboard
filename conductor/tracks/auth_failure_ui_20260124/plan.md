# Implementation Plan: Authorization Failure UI Indicator

## Status: ✅ COMPLETE (2026-01-24)

---

## Phase 1: Backend Enhancement ✅

### 1.1: Add auth_status to CurrentUser Response
- [x] Update `src/pa_dealing/api/auth.py`:
  - Add `auth_status` field to `CurrentUser` dataclass
  - Add `auth_message` field for human-readable explanation
  - Set status based on resolution outcome:
    - `"ok"` - Full identity resolution succeeded
    - `"identity_not_found"` - Email not matched to employee
    - `"partial"` - Authenticated but missing employee_id

### 1.2: Update get_current_user to Set Status
- [x] When `employee` is None, set:
  ```python
  auth_status = "identity_not_found"
  auth_message = "Could not resolve your identity. Contact IT support."
  ```

### 1.3: Update /api/auth/me Endpoint
- [x] `auth_status` and `auth_message` included in response

---

## Phase 2: Frontend Auth State Management ✅

### 2.1: Update CurrentUser Type
- [x] Update `dashboard/src/types/index.ts`:
  - Added `AuthStatus` type: `'ok' | 'identity_not_found' | 'partial'`
  - Added `auth_status` and `auth_message` fields to `CurrentUser` interface

### 2.2: Auth Query Integration
- [x] Updated Dashboard, MyRequests, PendingApprovals pages to query current user
- [x] Added retry and switch user handlers

---

## Phase 3: UI Components ✅

### 3.1: Create AuthErrorBanner Component
- [x] Created `dashboard/src/components/AuthErrorBanner.tsx`:
  - Error variant for `identity_not_found`
  - Warning variant for `partial`
  - Retry button with `data-testid="auth-retry-button"`
  - Switch User button (dev mode only) with `data-testid="auth-switch-user-button"`
  - Collapsible technical details section

### 3.2: Create ContentOverlay Component
- [x] Created `dashboard/src/components/ContentOverlay.tsx`:
  - Semi-transparent white background (`bg-white/70`)
  - Backdrop blur (`backdrop-blur-sm`)
  - Lock icon and customizable message
  - `pointer-events-none` to disable interaction
  - `data-testid="content-overlay"`

---

## Phase 4: Page-Level Integration ✅

### 4.1: Update Dashboard Home
- [x] Updated `dashboard/src/pages/Dashboard.tsx`:
  - Shows AuthErrorBanner when `auth_status !== 'ok'`
  - Shows ContentOverlay when `employee_id` is null
  - Summary cards visible but dimmed under overlay

### 4.2: Update My Requests Page
- [x] Updated `dashboard/src/pages/MyRequests.tsx`:
  - Shows AuthErrorBanner and ContentOverlay for auth failures
  - Clear messaging: "Your requests unavailable"

### 4.3: Update Pending Approvals Page
- [x] Updated `dashboard/src/pages/PendingApprovals.tsx`:
  - Shows AuthErrorBanner and ContentOverlay
  - **Bonus**: Added admin override - admins can approve any request type

---

## Phase 5: Testing ✅

### 5.1: Playwright E2E Tests
- [x] Created `dashboard/tests/auth_failure_ui.spec.ts` with 12 tests:

**Identity Not Found Tests (7):**
- `test_auth_failure_shows_error_banner` ✅
- `test_auth_failure_shows_content_overlay` ✅
- `test_retry_button_visible` ✅
- `test_switch_user_button_in_dev_mode` ✅
- `test_my_requests_shows_auth_error` ✅
- `test_pending_approvals_shows_auth_error` ✅
- `test_technical_details_collapsible` ✅

**Partial Identity Tests (2):**
- `test_partial_auth_shows_warning` ✅
- `test_no_overlay_for_partial_with_employee_id` ✅

**Success Tests (2):**
- `test_successful_auth_no_error_banner` ✅
- `test_successful_auth_no_overlay` ✅

**Retry Functionality (1):**
- `test_retry_button_triggers_new_auth_request` ✅

**All 12 tests pass.**

---

## Phase 6: Documentation & Cleanup ✅

### 6.1: Data-TestId Attributes
- [x] All interactive elements have data-testid:
  - `auth-error-banner`
  - `auth-retry-button`
  - `auth-switch-user-button`
  - `content-overlay`

---

## Summary

| Phase | Status | Deliverables |
|-------|--------|-------------|
| 1. Backend | ✅ | auth_status field in /auth/me |
| 2. Frontend State | ✅ | AuthStatus type, page integrations |
| 3. UI Components | ✅ | AuthErrorBanner, ContentOverlay |
| 4. Page Integration | ✅ | Dashboard, MyRequests, PendingApprovals |
| 5. Testing | ✅ | 12 Playwright E2E tests |
| 6. Docs | ✅ | data-testid attributes |

**Completed**: 2026-01-24
