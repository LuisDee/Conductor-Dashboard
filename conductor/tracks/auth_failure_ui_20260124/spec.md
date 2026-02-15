# Spec: Authorization Failure UI Indicator

## Problem Statement

When authorization fails (e.g., identity resolution cannot find the user, or the user lacks permissions), the dashboard currently shows empty data (all zeros) with no indication that something is wrong. Users have no way to know if:
- Their identity wasn't resolved
- They lack permissions
- There's a backend error
- There's simply no data

This leads to confusion and delayed debugging.

## User Stories

1. **As a user**, when my identity cannot be resolved, I want to see a clear error message explaining the issue so I know to contact support or retry.

2. **As a user**, when I see an authorization error, I want the dashboard content to be visually dimmed so I understand the data shown is not reliable.

3. **As a user**, I want a "Retry" button to attempt re-authentication without refreshing the entire page.

4. **As an admin**, when testing different user personas, I want immediate visual feedback if a user switch failed.

## Proposed Solution

### 1. Authorization Error Alert Banner

Display a prominent alert at the top of the dashboard when auth fails:

```tsx
{authError && (
  <Alert variant="destructive" className="mb-4">
    <AlertCircle className="h-4 w-4" />
    <AlertTitle>Authorization Failed</AlertTitle>
    <AlertDescription>
      {authError.message || "Unable to verify your identity. Please contact your administrator."}
      {authError.details && (
        <details className="mt-2 text-xs">
          <summary>Technical Details</summary>
          <pre>{authError.details}</pre>
        </details>
      )}
    </AlertDescription>
    <div className="mt-3 flex gap-2">
      <Button onClick={handleRetry} variant="outline" size="sm">
        Retry
      </Button>
      <Button onClick={handleLogout} variant="ghost" size="sm">
        Switch User
      </Button>
    </div>
  </Alert>
)}
```

### 2. Content Overlay When Unauthorized

Add a semi-transparent overlay over dashboard content to indicate data is unreliable:

```tsx
<div className="relative">
  {authError && (
    <div className="absolute inset-0 bg-white/70 backdrop-blur-sm z-10 flex items-center justify-center">
      <div className="text-center text-slate-500">
        <LockIcon className="w-12 h-12 mx-auto mb-2 opacity-50" />
        <p className="font-medium">Content unavailable</p>
        <p className="text-sm">Please resolve authorization to view data</p>
      </div>
    </div>
  )}
  {/* Dashboard cards here */}
</div>
```

### 3. Auth State Detection

The `/api/auth/me` endpoint already returns user info. We need to detect auth failures:

**Backend indicators of auth failure:**
- `employee_id` is `null` → Identity resolution failed
- `401 Unauthorized` response → Not authenticated
- `403 Forbidden` response → Authenticated but no permissions

**Frontend auth state:**
```typescript
interface AuthState {
  isAuthenticated: boolean;
  isAuthorized: boolean;  // Has employee_id and required roles
  user: CurrentUser | null;
  error: AuthError | null;
}

interface AuthError {
  type: 'not_authenticated' | 'identity_not_found' | 'no_permissions';
  message: string;
  details?: string;
}
```

### 4. Visual States

| State | Banner | Overlay | Dashboard Data |
|-------|--------|---------|----------------|
| Authenticated + Authorized | None | None | Normal |
| Authenticated + No employee_id | Warning (yellow) | Light blur | Show zeros (dimmed) |
| Not Authenticated (401) | Error (red) | Full overlay | Hidden |
| No Permissions (403) | Error (red) | Light blur | Show zeros (dimmed) |

## API Changes

### Backend: Enhanced `/api/auth/me` Response

Current response when identity not found:
```json
{
  "email": "alex.agombar@mako.com",
  "employee_id": null,
  "is_admin": false
}
```

Enhanced response:
```json
{
  "email": "alex.agombar@mako.com",
  "employee_id": null,
  "is_admin": false,
  "auth_status": "identity_not_found",
  "auth_message": "Could not resolve your identity in the employee database. Please contact IT support."
}
```

## Test Scenarios

### Unit Tests
1. AuthProvider correctly detects missing employee_id
2. AuthProvider correctly handles 401/403 responses
3. Alert renders with correct variant for each error type
4. Retry button triggers re-fetch of auth state
5. Overlay renders when authError is present

### Integration Tests
1. Dashboard shows error when identity resolution fails
2. Dashboard shows normal state when auth succeeds
3. User switch triggers auth re-fetch
4. Retry clears error and re-fetches

### E2E (Playwright) Tests
1. `test_auth_failure_shows_banner`: Visit dashboard with invalid user → see error banner
2. `test_auth_failure_shows_overlay`: Visit dashboard with invalid user → content is overlaid
3. `test_retry_button_works`: Click retry → auth re-attempted
4. `test_successful_auth_no_banner`: Login with valid user → no error banner
5. `test_user_switch_updates_auth`: Switch user → auth state updates

## Acceptance Criteria

- [x] Error banner appears within 2 seconds of auth failure
- [x] Banner message clearly explains the issue
- [x] Content overlay is visually distinct but still shows underlying structure
- [x] Retry button successfully re-attempts authentication
- [x] No false positives (banner doesn't show for normal empty data)
- [x] All unit tests pass
- [x] All Playwright E2E tests pass
- [x] Works in both dev user switcher and production IAP mode
