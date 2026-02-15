# Specification: Google Identity Integration (Hybrid Provider)

## Overview

This track implements a **Hybrid Identity Provider** that treats Google Workspace as the source of truth for **Human Metadata** (names, managers, emails, departments) while continuing to use SQL as the source of truth for **Trading Anchors** (employee IDs, positions, portfolios).

**Critical Design Principle:** We do NOT replace the database. We enhance identity resolution by merging Google's live data with the database's trading anchors.

---

## 1. Core Architecture

### Schema Distinction

**`bo_airflow` Schema (Reference Data)**
- **Purpose:** Read-only mirror of the firm's Backoffice Oracle database
- **Ownership:** Managed by data engineering pipeline. **We must not alter these tables.**
- **Key Tables:**
  - `oracle_employee` - Employee master data (lacks email, often has stale names)
  - `contact` - Contact information including work emails
  - `oracle_position` - Trading positions (future scope)

**`pa_dealing` Schema (Application Data)**
- **Purpose:** Our application's operational data
- **Ownership:** **We own these tables.** We can add columns (like `google_uid`)
- **Key Tables:**
  - `pad_request` - Trade requests
  - `pad_approval` - Approval records
  - `audit_log` - System audit trail
  - `employee_role` - Application-specific role assignments

---

## 2. The Identity Bridge (SQL Join Strategy)

### The Problem
The `oracle_employee` table lacks an `email` column. To link a Google user (Email) to a Database user (ID), we must use the `contact` table.

### The SQL Bridge Query

```sql
SELECT e.id as employee_id
FROM bo_airflow.oracle_employee e
JOIN bo_airflow.contact c ON (
    e.id = c.employee_id
    AND c.contact_group_id = 2  -- Work Group
    AND c.contact_type_id = 5   -- Email Type
)
WHERE LOWER(c.email) = LOWER('alex.agombar@mako.com')
```

**This `employee_id` is the anchor for all trading data (positions) and application data (requests).**

### The Achilles' Heel: Email Matching

**Risk:** The email in `contact` table might not match Google's `primaryEmail` exactly.
- Database: `alex@mako.com`
- Google: `alex.agombar@mako.com`
- Result: Bridge fails, user can't be anchored

### Fallback Strategy: mako_id Derivation

If email match fails, derive `mako_id` from Google name and match against `oracle_employee.mako_id`:

**Derivation Rule:**
```
mako_id = first_letter(givenName) + first_7_letters(familyName)
```

**Examples:**
- Alex Agombar → `aagombar`
- Luis De Burnay-Bastos → `ldeburna` (strip hyphens, take first 7 chars after first letter)
- David Rolfe → `drolfe`

**Fallback Query:**
```sql
SELECT e.id as employee_id
FROM bo_airflow.oracle_employee e
WHERE LOWER(e.mako_id) = LOWER('aagombar')
```

**Fallback Order:**
1. ✅ Try email match via `contact` table
2. ⚠️ If fails, derive `mako_id` from Google name and match against `oracle_employee`
3. ❌ If both fail, log warning: "User exists in Google but has no database anchor"

---

## 3. Data Mapping: The IdentityInfo Object

The application uses the `IdentityInfo` dataclass. Here's how each field is populated:

| Field | Source | Logic |
|-------|--------|-------|
| `employee_id` (int) | **SQL** | Resolved via SQL Bridge (with fallback) |
| `email` (str) | **Google** | `primaryEmail` from Google Admin API |
| `full_name` (str) | **Google** | `name.fullName` from Google |
| `mako_id` (str) | **Hybrid** | Derive from email prefix OR use derivation rule as fallback |
| `manager_email` (str\|None) | **Google** | `relations[type='manager'].value` |
| `manager_id` (int\|None) | **Hybrid** | Take `manager_email` from Google → Run SQL Bridge again |
| `is_manager` (bool) | **Google** | Check if user has direct reports in Google Directory |
| `is_investment_staff` (bool) | **Google** | Derived from `department` or `job_title` (e.g., "Trading", "Technology") |
| `company` (str\|None) | **SQL** | From `oracle_employee.company` |
| `cost_centre` (str\|None) | **SQL** | From `oracle_employee.cost_centre` |
| `roles` (list[str]) | **SQL** | From `pa_dealing.employee_role` table |

**New Field for Auditing:**
| Field | Source | Purpose |
|-------|--------|---------|
| `google_uid` (str) | **Google** | Immutable UUID from Google (`id` field) - for permanent audit trail |

---

## 4. google_uid: Production-Ready Auditing

### Why Add google_uid?

**The Problem:** Database IDs can change. If Oracle re-indexes and Alex's `employee_id` changes from `742` to `899`, audit trails break.

**The Solution:** Store Google's immutable UUID alongside the database ID.

### Schema Changes

**`pad_request` Table**
- **Add:** `google_uid` (VARCHAR, indexed)
- **Why:** Permanent identity for trade submitter. Even if `employee_id` changes, we know exactly who submitted.

**`pad_approval` Table**
- **Add:** `approver_google_uid` (VARCHAR, indexed)
- **Why:** Compliance audit requires proof of who approved. Google UUID backed by IT-enforced 2FA is stronger than just an integer ID.

**`audit_log` Table**
- **Add:** `google_uid` (VARCHAR, indexed)
- **Why:** Makes audit log "System Independent" - can be exported and verified by IT without needing access to `bo_airflow` database.

---

## 5. Implementation Workflow

### Phase 1: Identity Service Scaffolding ✅
- [x] Google Client with Service Account + Domain-Wide Delegation
- [x] Verify connectivity with standalone script
- [x] Basic `get_user_info()` and `is_manager()` methods

### Phase 2: The Hybrid Provider
- [x] Implement SQL Bridge query (email → employee_id)
- [x] Implement fallback strategy (mako_id derivation)
- [x] Create `GoogleIdentityProvider` implementing `IdentityProvider` interface
- [x] Merge Google metadata + SQL anchors → `IdentityInfo`
- [x] Add caching (TTL 1 hour)

### Phase 3: Schema Migration
- [x] Add `google_uid` column to `pad_request`
- [x] Add `approver_google_uid` column to `pad_approval`
- [x] Add `google_uid` column to `audit_log`
- [x] Create Alembic migration

### Phase 4: Application Integration
- [x] Refactor `auth.py` to use `GoogleIdentityProvider`
- [x] Refactor `handlers.py` to use new provider
- [x] Refactor `pad_service.py` to use new provider
- [x] Update request/approval/audit writes to stamp `google_uid`

### Phase 5: Testing
- [x] Create mock fixtures for `GoogleClient`
- [x] Update unit tests with Google mocks
- [x] Test fallback strategy (email fails → mako_id succeeds)
- [x] Run E2E tests with mocked Google responses
- [x] Manual verification: Resolve known user and check manager link

---

## 6. Security & Configuration

**Environment Variables:**
- `GOOGLE_ADMIN_SA_EMAIL` - Service account email
- `GOOGLE_ADMIN_DELEGATED_SUBJECT` - Admin user to impersonate
- `GOOGLE_ADMIN_QUOTA_PROJECT` - GCP project for quota tracking

**Google API Scopes:**
- `https://www.googleapis.com/auth/admin.directory.user.readonly`

**Security Rules:**
- ✅ Credentials from environment only
- ❌ Never commit private keys
- ✅ Use Domain-Wide Delegation (no user consent required)
- ✅ Implement caching to respect rate limits

---

## 7. Success Criteria

### Functional Requirements
1. ✅ Application can look up `alex.agombar@mako.com` and get:
   - Full name from Google
   - Manager email from Google
   - `employee_id` from SQL Bridge
   - `manager_id` from SQL Bridge (via manager's email)

2. ✅ Application correctly identifies:
   - `alex.agombar@mako.com` as a Manager (has direct reports in Google)
   - `luis.deburnay-bastos@mako.com` as a Non-Manager (no direct reports)

3. ✅ Fallback strategy works:
   - If email match fails, derive `mako_id` from name
   - Successfully resolve user via `oracle_employee.mako_id`

4. ✅ Audit trail includes `google_uid`:
   - New requests stamp Google UUID
   - Approvals record approver's Google UUID
   - Audit log entries include actor's Google UUID

### Quality Requirements
- ✅ Unit tests pass with mocked Google responses
- ✅ E2E tests verify approval routing uses Google-sourced manager hierarchy
- ✅ Code coverage >80%
- ✅ No database schema changes in `bo_airflow` (read-only constraint respected)

---

## 8. Out of Scope (Future Tracks)

**Not included in this track:**
- ❌ `last_trade_date` - Will be fetched from `oracle_position` in a separate data-sync track
- ❌ Removing `PostgresIdentityProvider` entirely - Keep as fallback for local-only dev
- ❌ Position/portfolio data synchronization

---

## 9. Risk Mitigation

**Risk: Email mismatch breaks the bridge**
- **Mitigation:** Fallback to mako_id derivation
- **Monitoring:** Log warnings when fallback is used

**Risk: User exists in Google but not in database**
- **Mitigation:** Log warning, return partial `IdentityInfo` (no `employee_id`)
- **Future:** Consider auto-provisioning workflow

**Risk: Manager's email can't be resolved to manager_id**
- **Mitigation:** Set `manager_id = None`, log warning
- **Impact:** Approval routing degrades gracefully (skip manager approval)

**Risk: Google API rate limits**
- **Mitigation:** 1-hour TTL cache, exponential backoff
- **Monitoring:** Track cache hit rate

---

## 10. Data Flow Diagram

```
┌─────────────────┐
│ User Request    │
│ (email)         │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────┐
│ GoogleIdentityProvider      │
├─────────────────────────────┤
│ 1. Fetch from Google API    │◄──── Google Workspace
│    - full_name              │
│    - manager_email          │
│    - department             │
│    - is_manager             │
│    - google_uid             │
│                             │
│ 2. SQL Bridge (email)       │◄──── bo_airflow.contact
│    ├─ Success → employee_id │
│    └─ Fail → Fallback:      │
│       Derive mako_id        │◄──── bo_airflow.oracle_employee
│       └─ employee_id         │
│                             │
│ 3. Manager Bridge           │
│    manager_email → SQL      │◄──── bo_airflow.contact (again)
│    → manager_id             │
│                             │
│ 4. Roles Lookup             │◄──── pa_dealing.employee_role
│                             │
│ 5. Merge → IdentityInfo     │
└────────┬────────────────────┘
         │
         ▼
┌─────────────────────────────┐
│ Application Logic           │
│ (auth, handlers, services)  │
└─────────────────────────────┘
```

---

## 11. Example: Full Resolution Flow

**Input:** `alex.agombar@mako.com` submits a trade

**Step 1: Google Fetch**
```python
google_profile = {
    "id": "abc123-def456-ghi789",  # google_uid
    "primaryEmail": "alex.agombar@mako.com",
    "name": {"fullName": "Alex Agombar", "givenName": "Alex", "familyName": "Agombar"},
    "organizations": [{"title": "Senior Analyst", "department": "Technology"}],
    "relations": [{"type": "manager", "value": "david.rolfe@mako.com"}]
}
```

**Step 2: SQL Bridge (Email)**
```sql
-- Returns employee_id = 742
```

**Step 3: Manager Bridge**
```sql
-- david.rolfe@mako.com → employee_id = 680
```

**Step 4: Merge**
```python
IdentityInfo(
    employee_id=742,
    email="alex.agombar@mako.com",
    full_name="Alex Agombar",
    mako_id="aagombar",
    manager_id=680,
    manager_email="david.rolfe@mako.com",
    is_manager=True,  # Has direct reports in Google
    company="MEU",  # From SQL
    roles=["smf16"],  # From pa_dealing.employee_role
    google_uid="abc123-def456-ghi789"  # For audit trail
)
```

**Step 5: Stamp google_uid in pad_request**
```sql
INSERT INTO pad_request (employee_id, google_uid, ...)
VALUES (742, 'abc123-def456-ghi789', ...)
```

---

## 12. Summary

**What we're building:**
- ✅ Hybrid provider that merges Google + SQL
- ✅ Robust bridge with fallback strategy
- ✅ Production-ready auditing with immutable UUIDs
- ✅ No changes to read-only reference tables

**What we're NOT doing:**
- ❌ Replacing the database entirely
- ❌ Modifying `bo_airflow` schema
- ❌ Handling position/trading data (future track)

**Constraint:** We respect the schema separation and read-only nature of `bo_airflow`.
