# Field Mapping Reference: Hybrid Identity System

**Last Updated:** 2026-01-19
**Track:** Google Identity Integration

This document provides a comprehensive reference for how identity and audit fields are populated in the PA Dealing system using the Hybrid Identity Provider (Google + SQL).

---

## Table of Contents
1. [IdentityInfo Field Mapping](#identityinfo-field-mapping)
2. [Audit Trail Fields](#audit-trail-fields)
3. [Database Schema Changes](#database-schema-changes)
4. [Data Flow Summary](#data-flow-summary)
5. [Query Examples](#query-examples)

---

## IdentityInfo Field Mapping

The `IdentityInfo` dataclass (defined in `src/pa_dealing/identity/base.py:8`) is the central object representing user identity in the application.

### Complete Field Reference

| Field | Type | Source | SQL Table | Google API Field | Derivation Logic | Notes |
|-------|------|--------|-----------|------------------|------------------|-------|
| `employee_id` | `int` | **SQL** | `bo_airflow.oracle_employee.id` | - | Via SQL Bridge (email → contact → employee) or mako_id fallback | **Primary key for all trading data** |
| `mako_id` | `str` | **SQL** | `bo_airflow.oracle_employee.mako_id` | - | Retrieved after employee_id resolution | Internal employee identifier (e.g., "aagombar") |
| `email` | `str` | **Google** | - | `primaryEmail` | Direct from Google | **Source of truth for contact** |
| `full_name` | `str \| None` | **Google** | - | `name.fullName` | Direct from Google | **Source of truth for display names** |
| `manager_id` | `int \| None` | **Hybrid** | `bo_airflow.oracle_employee.manager_id` | `relations[type='manager'].value` | Google manager_email → SQL Bridge → manager_id | Falls back to SQL manager_id if resolution fails |
| `manager_mako_id` | `str \| None` | **SQL** | `bo_airflow.oracle_employee.mako_id` | - | Via manager_id lookup | Currently not populated in GoogleIdentityProvider |
| `manager_email` | `str \| None` | **Google** | - | `relations[type='manager'].value` | Direct from Google | **Source of truth for manager hierarchy** |
| `company` | `str \| None` | **SQL** | `bo_airflow.oracle_employee.company` | - | Retrieved with employee_id | e.g., "MEU" |
| `cost_centre` | `str \| None` | **SQL** | `bo_airflow.oracle_employee.cost_centre` | - | Retrieved with employee_id | e.g., "9966" |
| `department` | `str \| None` | **Google** | - | `organizations[0].department` | Direct from Google | **Source of truth for org structure** |
| `job_title` | `str \| None` | **Google** | - | `organizations[0].title` | Direct from Google | **Source of truth for role** |
| `is_active` | `bool` | **SQL** | `bo_airflow.oracle_employee.end_date` | - | `end_date IS NULL` | Defaults to `True` |
| `is_manager` | `bool` | **Google** | - | Query: `directManager='{email}'` | Checks if user has ≥1 direct report | **Source of truth for manager status** |
| `is_investment_staff` | `bool` | **Hybrid** | `bo_airflow.oracle_employee.is_investment_staff` | `organizations[0].department` or `job_title` | Can be derived from Google dept/title | Not yet implemented in GoogleIdentityProvider |
| `google_uid` | `str \| None` | **Google** | - | `id` | Direct from Google | **Immutable UUID for audit trail** |
| `roles` | `list[str]` | **SQL** | `pa_dealing.employee_role` | - | Query: `WHERE revoked_at IS NULL` | Application-specific roles (e.g., "compliance", "smf16") |

### Source Legend
- **Google**: Fetched from Google Admin Directory API (source of truth for human metadata)
- **SQL**: Retrieved from `bo_airflow` schema (source of truth for trading anchors)
- **Hybrid**: Combination of both sources with fallback logic

---

## Audit Trail Fields

### PADRequest Table (`pa_dealing.pad_request`)

| Field | Type | Source | Purpose | Populated When |
|-------|------|--------|---------|----------------|
| `employee_id` | `BigInteger` | SQL Bridge | Links request to trading data | Request creation |
| `google_uid` | `String(255)` | Google API `id` field | Immutable audit trail of submitter | Request creation (via GoogleIdentityProvider) |

**Location:** `src/pa_dealing/db/models/pad.py:49`

**Migration:** Added in `20260119_2107_98b90c8f1ba4_add_google_uid_columns_for_audit_trail.py`

**Query Example:**
```sql
SELECT
    r.reference_id,
    r.employee_id,
    r.google_uid,
    e.mako_id
FROM pad_request r
JOIN oracle_employee e ON r.employee_id = e.id;
```

### PADApproval Table (`pa_dealing.pad_approval`)

| Field | Type | Source | Purpose | Populated When |
|-------|------|--------|---------|----------------|
| `approver_id` | `BigInteger` | SQL Bridge | Links approval to employee | Approval action |
| `approver_google_uid` | `String(255)` | Google API `id` field | Immutable audit trail of approver | Approval action (via GoogleIdentityProvider) |

**Location:** `src/pa_dealing/db/models/pad.py:161`

**Migration:** Added in `20260119_2107_98b90c8f1ba4_add_google_uid_columns_for_audit_trail.py`

**Compliance Use Case:**
```sql
-- Audit trail: Who approved this trade?
SELECT
    a.approval_type,
    a.decision,
    a.approver_id,
    a.approver_google_uid,
    e.mako_id,
    a.decided_at
FROM pad_approval a
JOIN oracle_employee e ON a.approver_id = e.id
WHERE a.request_id = 12345;
```

### AuditLog Table (`pa_dealing.audit_log`)

| Field | Type | Source | Purpose | Populated When |
|-------|------|--------|---------|----------------|
| `actor_id` | `BigInteger` | SQL Bridge | Links action to employee (nullable) | All user actions |
| `google_uid` | `String(255)` | Google API `id` field | Immutable audit trail of actor | All user actions (via GoogleIdentityProvider) |
| `actor_identifier` | `String(100)` | Google `primaryEmail` | Human-readable identifier | All actions (email or system name) |

**Location:** `src/pa_dealing/db/models/compliance.py:132`

**Migration:** Added in `20260119_2107_98b90c8f1ba4_add_google_uid_columns_for_audit_trail.py`

**System-Independent Audit:**
```sql
-- Export audit log with Google UUIDs (can be verified by IT without DB access)
SELECT
    timestamp,
    action_type,
    google_uid,  -- Immutable, IT can verify against Google Workspace
    actor_identifier,  -- Email for human readability
    details
FROM audit_log
WHERE action_type = 'approval_granted'
ORDER BY timestamp DESC;
```

---

## Database Schema Changes

### Schema Separation

**`bo_airflow` Schema (READ-ONLY)**
- **Purpose:** Mirror of firm's Oracle Backoffice database
- **Ownership:** Data Engineering pipeline
- **Constraint:** We MUST NOT modify these tables
- **Tables Used:**
  - `oracle_employee` - Employee master data (no email column)
  - `contact` - Contact information (bridges email → employee_id)
  - `oracle_position` - Trading positions (future scope)

**`pa_dealing` Schema (WE OWN)**
- **Purpose:** Application operational data
- **Ownership:** PA Dealing application team
- **Tables Modified in This Track:**
  - `pad_request` - Added `google_uid` column
  - `pad_approval` - Added `approver_google_uid` column
  - `audit_log` - Added `google_uid` column

### Migration Details

**Alembic Migration:** `20260119_2107_98b90c8f1ba4_add_google_uid_columns_for_audit_trail.py`

**Changes:**
```python
# Added to pad_request
google_uid: String(255), nullable=True, indexed
Comment: "Immutable Google Workspace user UUID"

# Added to pad_approval
approver_google_uid: String(255), nullable=True, indexed
Comment: "Immutable Google Workspace approver UUID"

# Added to audit_log
google_uid: String(255), nullable=True, indexed
Comment: "Immutable Google Workspace actor UUID"
```

**Indexes Created:**
- `ix_pad_request_google_uid`
- `ix_pad_approval_approver_google_uid`
- `ix_audit_log_google_uid`

---

## Data Flow Summary

### 1. User Lookup Flow (GoogleIdentityProvider.get_by_email)

```
Input: email="alex.agombar@mako.com"
    ↓
Step 1: Fetch from Google Admin API
    → id: "abc123-def456-ghi789" (google_uid)
    → primaryEmail: "alex.agombar@mako.com"
    → name: {fullName: "Alex Agombar", givenName: "Alex", familyName: "Agombar"}
    → organizations: [{title: "Senior Analyst", department: "Technology"}]
    → relations: [{type: "manager", value: "david.rolfe@mako.com"}]
    ↓
Step 2: SQL Bridge (Email → employee_id)
    Query: bo_airflow.contact WHERE email = 'alex.agombar@mako.com'
    → employee_id: 100
    ↓
    [If fails] Step 2b: Fallback (mako_id derivation)
    Derive: "a" + "agombar" = "aagombar"
    Query: bo_airflow.oracle_employee WHERE mako_id = 'aagombar'
    → employee_id: 100
    ↓
Step 3: Get SQL Anchor Data
    Query: bo_airflow.oracle_employee WHERE id = 100
    → mako_id: "aagombar"
    → manager_id: 102
    → company: "MEU"
    → cost_centre: "9966"
    ↓
Step 4: Resolve Manager (manager_email → manager_id)
    Input: "david.rolfe@mako.com"
    Recursive call: get_by_email("david.rolfe@mako.com", resolve_manager=False)
    → manager_id: 102
    ↓
Step 5: Check if User is Manager
    Query Google: directManager='alex.agombar@mako.com' maxResults=1
    → has_direct_reports: True
    ↓
Step 6: Get Roles
    Query: pa_dealing.employee_role WHERE employee_id = 100 AND revoked_at IS NULL
    → roles: ["smf16"]
    ↓
Output: IdentityInfo(
    employee_id=100,                      # From SQL Bridge
    mako_id="aagombar",                   # From SQL
    email="alex.agombar@mako.com",        # From Google
    full_name="Alex Agombar",             # From Google
    manager_id=102,                        # From SQL (resolved via Google manager_email)
    manager_email="david.rolfe@mako.com", # From Google
    company="MEU",                         # From SQL
    cost_centre="9966",                    # From SQL
    department="Technology",               # From Google
    job_title="Senior Analyst",            # From Google
    is_manager=True,                       # From Google (has direct reports)
    google_uid="abc123-def456-ghi789",    # From Google (immutable UUID)
    roles=["smf16"]                        # From SQL (pa_dealing.employee_role)
)
```

### 2. Trade Submission Flow (with google_uid stamping)

```
User submits trade via Slack
    ↓
Step 1: Resolve User Identity
    identity = await provider.get_by_email("alex.agombar@mako.com")
    → identity.employee_id = 100
    → identity.google_uid = "abc123-def456-ghi789"
    ↓
Step 2: Create PADRequest
    INSERT INTO pad_request (
        employee_id,     -- 100 (SQL ID)
        google_uid,      -- "abc123-def456-ghi789" (Google UUID)
        direction,       -- "BUY"
        ticker,          -- "AAPL"
        ...
    )
    ↓
Step 3: Log to Audit
    INSERT INTO audit_log (
        action_type,     -- "request_submitted"
        actor_id,        -- 100 (SQL ID)
        google_uid,      -- "abc123-def456-ghi789" (Google UUID)
        actor_identifier,-- "alex.agombar@mako.com"
        ...
    )
```

### 3. Approval Flow (with approver google_uid stamping)

```
Manager approves trade
    ↓
Step 1: Resolve Approver Identity
    identity = await provider.get_by_email("david.rolfe@mako.com")
    → identity.employee_id = 102
    → identity.google_uid = "xyz789-abc123-def456"
    ↓
Step 2: Create PADApproval
    INSERT INTO pad_approval (
        request_id,            -- 12345
        approver_id,           -- 102 (SQL ID)
        approver_google_uid,   -- "xyz789-abc123-def456" (Google UUID)
        approval_type,         -- "manager"
        decision,              -- "approved"
        ...
    )
    ↓
Step 3: Log to Audit
    INSERT INTO audit_log (
        action_type,     -- "approval_granted"
        actor_id,        -- 102 (SQL ID)
        google_uid,      -- "xyz789-abc123-def456" (Google UUID)
        actor_identifier,-- "david.rolfe@mako.com"
        entity_type,     -- "pad_request"
        entity_id,       -- 12345
        ...
    )
```

---

## Query Examples

### Find All Trades by Google UUID (Survives Database ID Changes)

```sql
-- Even if employee_id changes, google_uid remains constant
SELECT
    r.reference_id,
    r.created_at,
    r.status,
    r.ticker,
    r.direction,
    r.google_uid,
    r.employee_id  -- This might change in Oracle re-indexing
FROM pad_request r
WHERE r.google_uid = 'abc123-def456-ghi789'
ORDER BY r.created_at DESC;
```

### Compliance Audit: Full Approval Chain with Immutable IDs

```sql
-- Regulatory audit: Who approved this trade? (IT can verify Google UUIDs)
SELECT
    r.reference_id,
    r.google_uid AS submitter_google_uid,
    e1.mako_id AS submitter_mako_id,
    a.approval_type,
    a.decision,
    a.approver_google_uid,
    e2.mako_id AS approver_mako_id,
    a.decided_at,
    a.comments
FROM pad_request r
JOIN pad_approval a ON r.id = a.request_id
JOIN oracle_employee e1 ON r.employee_id = e1.id
JOIN oracle_employee e2 ON a.approver_id = e2.id
WHERE r.reference_id = 'AAGOMBAR-260119-AAPL'
ORDER BY a.decided_at;
```

### System-Independent Audit Export

```sql
-- Export for external compliance review (no dependency on bo_airflow schema)
SELECT
    al.timestamp,
    al.action_type,
    al.google_uid,           -- IT can verify this against Google Workspace
    al.actor_identifier,     -- Human-readable email
    al.request_id,
    al.reference_id,
    al.details::text
FROM audit_log al
WHERE al.action_type IN ('request_submitted', 'approval_granted', 'approval_declined')
  AND al.timestamp >= '2026-01-01'
ORDER BY al.timestamp DESC;
```

### Find User by Any Identifier (Email, mako_id, or google_uid)

```sql
-- Universal user lookup
WITH user_identity AS (
    SELECT DISTINCT
        e.id AS employee_id,
        e.mako_id,
        c.email,
        r.google_uid
    FROM oracle_employee e
    LEFT JOIN contact c ON (
        e.id = c.employee_id
        AND c.contact_group_id = 2
        AND c.contact_type_id = 5
    )
    LEFT JOIN pad_request r ON e.id = r.employee_id
    WHERE r.google_uid IS NOT NULL
)
SELECT * FROM user_identity
WHERE email ILIKE '%agombar%'
   OR mako_id ILIKE '%agombar%'
   OR google_uid = 'abc123-def456-ghi789';
```

---

## Implementation Files Reference

### Core Files

| File | Purpose | Key Classes/Functions |
|------|---------|----------------------|
| `src/pa_dealing/identity/base.py` | Base identity interfaces | `IdentityInfo`, `IdentityProvider` |
| `src/pa_dealing/identity/google.py` | Google Admin API client | `GoogleAdminClient`, `get_google_admin_client()` |
| `src/pa_dealing/identity/provider_google.py` | Hybrid provider implementation | `GoogleIdentityProvider`, `_derive_mako_id_from_name()` |
| `src/pa_dealing/db/models/pad.py` | PAD request/approval models | `PADRequest`, `PADApproval` |
| `src/pa_dealing/db/models/compliance.py` | Audit log model | `AuditLog` |

### Migration Files

| Migration | Description | Date |
|-----------|-------------|------|
| `alembic/versions/20260119_2107_98b90c8f1ba4_add_google_uid_columns_for_audit_trail.py` | Added google_uid columns to pad_request, pad_approval, audit_log | 2026-01-19 |

### Test Files

| File | Coverage |
|------|----------|
| `tests/unit/test_google_identity_provider.py` | SQL Bridge, fallback strategy, manager resolution, full hybrid flow |

---

## Key Takeaways

✅ **Google is source of truth for:**
- Names (full_name)
- Email addresses
- Manager hierarchy (manager_email)
- Organizational structure (department, job_title)
- Manager status (is_manager via direct reports check)
- **Immutable audit UUIDs (google_uid)**

✅ **SQL (bo_airflow) is source of truth for:**
- Employee IDs (employee_id) - trading anchor
- Internal identifiers (mako_id)
- Company/cost centre
- Manager relationships (manager_id) - as fallback

✅ **SQL (pa_dealing) is source of truth for:**
- Application roles (employee_role)
- Trade requests (pad_request)
- Approvals (pad_approval)
- Audit logs (audit_log)

✅ **Hybrid Strategy:**
- Email → contact table → employee_id (SQL Bridge)
- Fallback: Derive mako_id from Google name → oracle_employee lookup
- Manager resolution: Google manager_email → SQL Bridge → manager_id
- **Audit trail: Both SQL ID (mutable) and Google UUID (immutable)**

---

**For questions or clarifications, refer to:**
- Spec: `conductor/tracks/google_identity_integration_20260119/spec.md`
- Plan: `conductor/tracks/google_identity_integration_20260119/plan.md`
- Tests: `tests/unit/test_google_identity_provider.py`
