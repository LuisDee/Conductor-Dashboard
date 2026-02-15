# PA Dealing E2E Test Scenarios

This document describes each end-to-end test scenario in `tests/test_e2e_full_pipeline.py`, explaining what functionality is being tested and the expected outcomes.

## Overview

The E2E test suite exercises the complete PA Dealing workflow through the HTTP API, persisting data to the database for audit trail verification. Unlike unit tests (which use transactional rollback), these tests leave data in place so you can manually inspect the results.

**Run the tests:**
```bash
pytest tests/test_e2e_full_pipeline.py -v -s
```

**Prerequisites:**
- Docker stack running (`docker compose -f docker/docker-compose.yml up -d`)
- Database seeded (automatically done by test fixture, or manually via `python scripts/seed_dev_database.py`)

---

## Test 01: Low Risk Auto-Approve

**File:** `test_01_low_risk_auto_approve`

**Purpose:** Verify that low-value, low-risk trades are automatically approved without requiring manual intervention.

### Steps

1. **Submit Request**
   - User: `skemp@mako.com`
   - Security: Tesco PLC (TSCO)
   - Direction: Buy
   - Value: £500 (below £10k medium threshold)
   - Insider info confirmed: Yes

2. **Expected Outcome**
   - Status: `auto_approved` or `approved`
   - Risk Level: `LOW`
   - No manual approval required

### What This Tests

- Risk classification correctly identifies low-value trades
- Auto-approval feature is working when enabled
- Audit log records submission with `pad_request_submitted`

### Audit Trail

| Action | Actor | Details |
|--------|-------|---------|
| `pad_request_submitted` | skemp@mako.com | value=500, buysell=B |

---

## Test 02: Medium Risk Manager Approval

**File:** `test_02_medium_risk_manager_approval`

**Purpose:** Verify that medium-value trades require manager approval before proceeding to compliance.

### Steps

1. **Submit Request**
   - User: `skemp@mako.com`
   - Security: Barclays PLC (BARC)
   - Direction: Buy
   - Value: £25,000 (between £10k-£50k thresholds)
   - Insider info confirmed: Yes

2. **Initial Status**
   - Status: `pending_manager`
   - Risk Level: `MEDIUM`

3. **Manager Approval**
   - Approver: `swilliam@mako.com` (skemp's manager)
   - Action: Approve with comments

4. **Expected Outcome**
   - Status changes to: `pending_compliance`
   - Approval recorded in `pad_approval` table

### What This Tests

- Risk classification correctly identifies medium-value trades
- Manager relationship lookup works (swilliam is skemp's manager)
- Approval workflow advances correctly
- Only the correct manager can approve

### Audit Trail

| Action | Actor | Details |
|--------|-------|---------|
| `pad_request_submitted` | skemp@mako.com | value=25000, buysell=B |
| `pad_request_viewed` | system | Before approval check |
| `pad_request_manager_approved` | swilliam@mako.com | approved=true |

---

## Test 03: High Risk SMF16 Escalation

**File:** `test_03_high_risk_smf16_escalation`

**Purpose:** Verify that high-value trades go through the full approval chain: Manager → Compliance → SMF16.

### Steps

1. **Submit Request**
   - User: `skemp@mako.com`
   - Security: Amazon.com Inc (AMZN)
   - Direction: Buy
   - Value: £75,000 (above £50k high threshold)
   - Insider info confirmed: Yes

2. **Initial Status**
   - Status: `pending_manager`
   - Risk Level: `HIGH` (or `MEDIUM` depending on other factors)

3. **Approval Chain**

   a. **Manager Approval**
   - Approver: `swilliam@mako.com`
   - Result: Status → `pending_compliance`

   b. **Compliance Approval**
   - Approver: `jsmith@mako.com` (has compliance role)
   - Result: Status → `pending_smf16` or `approved`

   c. **SMF16 Approval** (if required)
   - Approver: `skemp@mako.com` (has smf16 role)
   - Result: Status → `approved`

### What This Tests

- High-value trade detection
- Full sequential approval workflow
- Role-based authorization (compliance role, SMF16 role)
- Each approval stage is recorded separately

### Audit Trail

| Action | Actor | Details |
|--------|-------|---------|
| `pad_request_submitted` | skemp@mako.com | value=75000, buysell=B |
| `pad_request_manager_approved` | swilliam@mako.com | - |
| `pad_request_compliance_approved` | jsmith@mako.com | - |
| `pad_request_smf16_approved` | skemp@mako.com | (if escalated) |

---

## Test 04: Restricted Security Blocked

**File:** `test_04_restricted_security_blocked`

**Purpose:** Verify that trades on restricted securities are blocked at submission.

### Steps

1. **Submit Request**
   - User: `skemp@mako.com`
   - Security: NVIDIA Corporation (NVDA) - **on restricted list**
   - Direction: Buy
   - Value: £5,000

2. **Expected Outcome**
   - HTTP Status: `400 Bad Request`
   - Error: "Security is on the RESTRICTED LIST - trading prohibited"
   - No request created in database

### What This Tests

- Restricted security list lookup
- Pre-submission validation blocks prohibited trades
- Appropriate error message returned
- Blocked attempt still logged for audit

### Audit Trail

| Action | Actor | Details |
|--------|-------|---------|
| `pad_request_submitted` | skemp@mako.com | action_status=blocked |

### Seed Data Dependency

NVDA is added to `restricted_security` table by the seed script with reason: "Inside information - pending acquisition announcement"

---

## Test 05: Mako Conflict Detection

**File:** `test_05_mako_conflict_detected`

**Purpose:** Verify that trades in securities where Mako has recent positions are flagged as conflicts.

### Steps

1. **Submit Request**
   - User: `skemp@mako.com`
   - Security: Apple Inc (AAPL) - **Mako has position**
   - Direction: Buy
   - Value: £15,000
   - Bloomberg: "AAPL US Equity"

2. **Expected Outcome**
   - Request created successfully
   - Conflict flag set (check `has_conflict` field)
   - May be escalated due to conflict

### What This Tests

- Mako position lookup (90-day lookback)
- Conflict detection logic
- Risk elevation due to conflicts
- Conflict details included in compliance assessment

### Audit Trail

| Action | Actor | Details |
|--------|-------|---------|
| `pad_request_submitted` | skemp@mako.com | security=Apple Inc |

### Seed Data Dependency

AAPL is added to `oracle_position` table with:
- inst_symbol: AAPL
- portfolio: GLOBAL_TECH
- position_size: 150,000
- last_trade_date: 15 days ago

---

## Test 06: Holding Period Violation

**File:** `test_06_holding_period_violation`

**Purpose:** Verify that selling a security within the 30-day holding period is detected.

### Steps

1. **Submit Buy Request**
   - User: `skemp@mako.com`
   - Security: Shell PLC (SHEL)
   - Direction: Buy
   - Value: £4,000

2. **Submit Sell Request** (immediately after)
   - User: `skemp@mako.com`
   - Security: Shell PLC (SHEL) - **same security**
   - Direction: Sell
   - Value: £2,000

3. **Expected Outcome**
   - Both requests created
   - Sell request may have holding period warning in assessment
   - System detects recent buy of same security

### What This Tests

- Holding period check (30-day rule)
- Same-security detection for employee
- Warning/flag generation for early sells

### Audit Trail

| Action | Actor | Details |
|--------|-------|---------|
| `pad_request_submitted` | skemp@mako.com | SHEL, Buy |
| `pad_request_submitted` | skemp@mako.com | SHEL, Sell |

### Note

In this test, the sell happens in the same test run as the buy, so the holding period logic should detect a very recent buy. In production, this would catch sells within 30 days of a buy.

---

## Test 07: Execution Recording

**File:** `test_07_execution_with_contract_note`

**Purpose:** Verify that trade executions can be recorded after approval.

### Steps

1. **Submit Request**
   - User: `skemp@mako.com`
   - Security: Lloyds Banking Group (LLOY)
   - Direction: Buy
   - Quantity: 1,000
   - Value: £500

2. **Approve if Needed**
   - If status is `pending_manager`, approve as `swilliam@mako.com`

3. **Record Execution**
   - Endpoint: `POST /api/requests/{id}/execution`
   - Data:
     ```json
     {
       "price": 0.52,
       "quantity": 1000,
       "broker_reference": "E2E-TEST-001"
     }
     ```

4. **Expected Outcome**
   - Status changes to: `executed`
   - Execution details stored in `pad_execution` table

### What This Tests

- Execution recording endpoint
- Status transition to `executed`
- Execution price and quantity storage
- Broker reference tracking

### Audit Trail

| Action | Actor | Details |
|--------|-------|---------|
| `pad_request_submitted` | skemp@mako.com | LLOY, qty=1000 |
| `pad_request_manager_approved` | swilliam@mako.com | (if needed) |
| `pad_execution_recorded` | skemp@mako.com | price=0.52 |

---

## Test 08: Prohibited Derivative Blocked

**File:** `test_08_prohibited_derivative_blocked`

**Purpose:** Verify that derivative products (futures, options) are blocked as prohibited instruments.

### Steps

1. **Submit Request**
   - User: `skemp@mako.com`
   - Security: "Oil Futures Contract"
   - Ticker: CL1
   - Bloomberg: "CL1 Comdty" (Commodity type)
   - Direction: Buy
   - Value: £50,000
   - `is_derivative: true`

2. **Expected Outcome**
   - HTTP Status: `400 Bad Request`
   - Error indicates prohibited product type
   - No request created

### What This Tests

- Product type detection (derivatives, futures, options)
- Bloomberg instrument type parsing ("Comdty" = commodity)
- `is_derivative` flag handling
- Pre-submission blocking of prohibited products

### Audit Trail

| Action | Actor | Details |
|--------|-------|---------|
| `pad_request_submitted` | skemp@mako.com | action_status=blocked |

---

## Test 09: Related Party Trade

**File:** `test_09_related_party_trade`

**Purpose:** Verify that trades on behalf of connected persons (spouse, parent, etc.) are flagged and handled appropriately.

### Steps

1. **Submit Request**
   - User: `skemp@mako.com`
   - Security: Diageo PLC (DGE)
   - Direction: Buy
   - Value: £5,000
   - `is_related_party: true`
   - `relation: "Spouse"`

2. **Expected Outcome**
   - Request created successfully
   - Related party flag is set
   - May have elevated risk due to related party status

### What This Tests

- Related party flag handling
- Relation type storage (Spouse, Parent, Child, etc.)
- Risk adjustment for related party trades
- Compliance assessment includes related party info

### Audit Trail

| Action | Actor | Details |
|--------|-------|---------|
| `pad_request_submitted` | skemp@mako.com | is_related_party=true |

---

## Test 10: Manager Decline Flow

**File:** `test_10_manager_decline_flow`

**Purpose:** Verify that managers can decline requests and the decline is properly recorded.

### Steps

1. **Submit Request**
   - User: `skemp@mako.com`
   - Security: "Test Decline Security"
   - Direction: Buy
   - Value: £15,000

2. **Manager Decline**
   - Actor: `swilliam@mako.com` (skemp's manager)
   - Endpoint: `POST /api/requests/{id}/decline`
   - Data:
     ```json
     {
       "approval_type": "manager",
       "reason": "E2E test: Declining for test purposes"
     }
     ```

3. **Expected Outcome**
   - Status changes to: `declined`
   - Decline recorded in `pad_approval` table with decision="declined"

### What This Tests

- Decline endpoint functionality
- Status transition to `declined`
- Decline reason/comments storage
- Manager authorization for decline

### Audit Trail

| Action | Actor | Details |
|--------|-------|---------|
| `pad_request_submitted` | skemp@mako.com | TEST |
| `pad_request_viewed` | system | - |
| `pad_request_manager_declined` | swilliam@mako.com | reason provided |

---

## Verification Tests

### Test 11: Audit Log Verification

**File:** `test_verify_audit_log_entries`

**Purpose:** Summarize and verify audit log entries after all tests run.

**Output:**
```
AUDIT LOG SUMMARY
Total entries: 22
Action type breakdown:
  pad_request_compliance_approved: 1
  pad_request_manager_approved: 2
  pad_request_manager_declined: 1
  pad_request_submitted: 12
  pad_request_viewed: 6
```

### Test 12: Request Status Summary

**File:** `test_verify_request_statuses`

**Purpose:** Summarize request statuses and risk levels after all tests.

**Output:**
```
REQUEST STATUS SUMMARY
Total requests: 9
Status breakdown:
  approved: 4
  declined: 1
  executed: 1
  pending_compliance: 1
  pending_manager: 2
```

---

## Database Tables Used

| Table | Purpose |
|-------|---------|
| `pad_request` | Core request data |
| `pad_approval` | Individual approval records |
| `pad_execution` | Trade execution details |
| `audit_log` | Complete audit trail |
| `oracle_employee` | Employee data with manager relationships |
| `employee_role` | Role assignments (compliance, smf16) |
| `oracle_position` | Mako trading positions for conflict check |
| `restricted_security` | Restricted securities list |
| `compliance_config` | Thresholds and settings |

---

## Seed Data Requirements

The tests require the following seed data (created by `scripts/seed_dev_database.py`):

### Employees
| mako_id | email | manager | roles |
|---------|-------|---------|-------|
| pjohn | pjohn@mako.com | - | admin |
| swilliam | swilliam@mako.com | pjohn | - |
| jsmith | jsmith@mako.com | pjohn | compliance |
| skemp | skemp@mako.com | swilliam | smf16 |
| cdavis | cdavis@mako.com | jsmith | compliance |

### Mako Positions (for conflict detection)
| inst_symbol | portfolio | position_size | last_trade_date |
|-------------|-----------|---------------|-----------------|
| AAPL | GLOBAL_TECH | 150,000 | 15 days ago |
| MSFT | GLOBAL_TECH | 75,000 | 30 days ago |
| NVDA | GLOBAL_TECH | 50,000 | 10 days ago |

### Restricted Securities
| ticker | reason |
|--------|--------|
| NVDA | Inside information - pending acquisition |
| XYZ | Regulatory restriction |

### Compliance Config
| key | value |
|-----|-------|
| medium_value_threshold | 10000 |
| high_value_threshold | 50000 |
| auto_approve_enabled | true |
| holding_period_days | 30 |
| mako_lookback_days | 90 |

---

## Running Individual Tests

```bash
# Run all E2E tests
pytest tests/test_e2e_full_pipeline.py -v -s

# Run a specific test
pytest tests/test_e2e_full_pipeline.py::TestE2EFullPipeline::test_01_low_risk_auto_approve -v -s

# Run just the verification tests
pytest tests/test_e2e_full_pipeline.py::TestE2EAuditVerification -v -s
```

## Inspecting Results

After running tests, inspect the database:

```bash
# View all requests
docker exec pad_db psql -U pad -d pa_dealing -c \
  "SELECT id, reference_id, status, risk_level FROM pad_request ORDER BY id"

# View audit log
docker exec pad_db psql -U pad -d pa_dealing -c \
  "SELECT action_type, action_status, actor_identifier, entity_id FROM audit_log ORDER BY id DESC LIMIT 30"

# View approvals
docker exec pad_db psql -U pad -d pa_dealing -c \
  "SELECT request_id, approval_type, decision, comments FROM pad_approval"
```
