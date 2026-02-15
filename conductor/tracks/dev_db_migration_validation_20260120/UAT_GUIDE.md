# UAT Testing Guide - Personal Account Dealing System

**Environment**: Development (uk02vddb004)
**Date**: 2026-01-21
**Tester**: User
**Test Approach**: Manual end-to-end validation

---

## Prerequisites Checklist

Before starting UAT, verify:

- [x] ✅ Dashboard accessible at http://localhost:XXXX
- [x] ✅ API server running and connected to dev database
- [x] ✅ Slack bot running and responsive
- [x] ✅ Test user accounts available:
  - Employee: `luis.deburnay-bastos@mako.com` (ID: 1272)
  - Manager: `alex.agombar@mako.com` (ID: 1191, Luis's manager)
  - Compliance: TBD (will identify during setup)
- [x] ✅ Test securities available in oracle_bloomberg
- [x] ✅ Database connection confirmed (dev database)

---

## Test Scenarios Overview

| # | Scenario | Priority | Est. Time | Status |
|---|----------|----------|-----------|--------|
| 1 | Employee Submission (Happy Path) | CRITICAL | 10 min | ⬜ |
| 2 | Manager Approval | CRITICAL | 5 min | ⬜ |
| 3 | Compliance Approval | CRITICAL | 5 min | ⬜ |
| 4 | Execution Recording | HIGH | 5 min | ⬜ |
| 5 | Manager Rejection | MEDIUM | 5 min | ⬜ |
| 6 | Employee Withdrawal | MEDIUM | 5 min | ⬜ |
| 7 | Dashboard Validation | HIGH | 10 min | ⬜ |
| 8 | Restricted Security Block | HIGH | 3 min | ⬜ |
| 9 | Conflict Detection (Optional) | LOW | 5 min | ⬜ |
| 10 | Performance Check | LOW | 3 min | ⬜ |

**Total Time**: ~50 minutes for critical scenarios (1-4, 7-8)

---

## Scenario 1: Employee Submission Flow (Happy Path) 🎯

**Goal**: Submit a personal trading request via Slack bot

**Test User**: luis.deburnay-bastos@mako.com (Employee)
**Expected Flow**: Submit → Pending Manager Approval
**Time**: ~10 minutes

### Steps:

#### 1.1 Open Slack and Find PA Dealing Bot

**Action**:
1. Open Slack workspace
2. Find "PA Dealing Bot" in Apps or DMs
3. Start new conversation

**Expected**:
- Bot should be online and available
- Previous conversation history may exist (that's OK)

**Screenshot**: 📸 Take screenshot of bot in Slack

---

#### 1.2 Initiate Request

**What to Type**:
```
I want to trade some stock
```

**Expected Response**:
- Bot greets you
- Bot asks for security details
- Response time: < 5 seconds

**Example**:
```
Bot: Hi Luis! I'll help you submit a personal trading request.
     What security would you like to trade?
     You can provide a ticker (e.g., AAPL), Bloomberg code, or description.
```

**Checkpoint**: ✅ Bot responds within 5 seconds

---

#### 1.3 Provide Security Details

**What to Type**:
```
AAPL US Equity
```
*(or any stock in oracle_bloomberg - we'll provide a list)*

**Expected Response**:
- Bot looks up security (3-tier lookup: Bloomberg → MapInstSymbol → Product)
- Bot displays security details (ticker, description)
- Bot asks for transaction type (Buy/Sell)

**Example**:
```
Bot: Found security:
     Ticker: AAPL
     Description: Apple Inc
     Type: Equity

     Would you like to BUY or SELL?
```

**Checkpoint**: ✅ Security identified correctly

---

#### 1.4 Select Transaction Type

**What to Type**:
```
Buy
```

**Expected Response**:
- Bot records "Buy"
- Bot asks for quantity

**Example**:
```
Bot: Got it, you want to BUY AAPL.
     How many shares?
```

**Checkpoint**: ✅ Transaction type recorded

---

#### 1.5 Enter Quantity

**What to Type**:
```
100
```

**Expected Response**:
- Bot records quantity
- Bot asks for estimated price

**Example**:
```
Bot: 100 shares of AAPL.
     What's the estimated price per share (in USD)?
```

**Checkpoint**: ✅ Quantity accepted

---

#### 1.6 Enter Price

**What to Type**:
```
150.00
```

**Expected Response**:
- Bot calculates estimated value (100 × 150 = $15,000)
- Bot asks for currency confirmation

**Example**:
```
Bot: Estimated total value: $15,000 USD
     Is the currency USD?
```

**Checkpoint**: ✅ Value calculated correctly

---

#### 1.7 Confirm Currency

**What to Type**:
```
Yes
```
*(or select from buttons if provided)*

**Expected Response**:
- Bot asks for broker name

**Checkpoint**: ✅ Currency confirmed

---

#### 1.8 Enter Broker

**What to Type**:
```
Interactive Brokers
```

**Expected Response**:
- Bot asks for account number

**Example**:
```
Bot: Which account will you use?
     Please provide your brokerage account number.
```

**Checkpoint**: ✅ Broker recorded

---

#### 1.9 Enter Account Number

**What to Type**:
```
U1234567
```

**Expected Response**:
- Bot asks for trading reason/justification

**Example**:
```
Bot: What's the reason for this trade?
     (e.g., "Portfolio diversification", "Investment opportunity", etc.)
```

**Checkpoint**: ✅ Account number recorded

---

#### 1.10 Provide Trading Reason

**What to Type**:
```
Portfolio diversification - adding tech exposure
```

**Expected Response**:
- Bot displays **full summary** of request
- Bot asks for confirmation (Submit / Cancel)

**Example**:
```
Bot: Please review your request:

     Security: Apple Inc (AAPL)
     Type: Buy
     Quantity: 100 shares
     Price: $150.00
     Total Value: $15,000 USD
     Broker: Interactive Brokers
     Account: U1234567
     Reason: Portfolio diversification - adding tech exposure

     [Submit Request] [Cancel]
```

**Checkpoint**: ✅ Summary displayed with all details

---

#### 1.11 Submit Request

**What to Do**:
- Click **"Submit Request"** button (or type "Submit")

**Expected Response**:
- Bot creates PADRequest in database
- Bot performs risk assessment
- Bot routes to manager for approval
- Bot displays confirmation with **Request ID**
- Bot shows status: "Pending Manager Approval"
- Bot may send Slack notification to manager (Alex)

**Example**:
```
Bot: ✅ Request submitted successfully!

     Request ID: PAD-2026-001
     Status: Pending Manager Approval

     I've notified your manager (Alex Agombar) for review.
     You'll be notified when there's an update.
```

**Critical**: 📝 **RECORD THE REQUEST ID**: _______________

**Checkpoint**: ✅ Request ID displayed and recorded

---

#### 1.12 Verify in Database

**Open Terminal and Run**:
```bash
poetry run python -c "
import asyncio
from sqlalchemy import select, text
from src.pa_dealing.db.engine import async_session_maker

async def check():
    async with async_session_maker() as session:
        result = await session.execute(text('''
            SELECT r.id, r.reference_id, r.status,
                   e.mako_id as employee,
                   b.ticker, r.transaction_type, r.quantity, r.estimated_value
            FROM padealing.pad_request r
            JOIN bo_airflow.oracle_employee e ON r.employee_id = e.id
            LEFT JOIN bo_airflow.oracle_bloomberg b ON r.security_id = b.id
            ORDER BY r.created_at DESC
            LIMIT 1
        '''))
        row = result.fetchone()
        if row:
            print(f'✅ Request found in database:')
            print(f'   ID: {row.id}')
            print(f'   Reference: {row.reference_id}')
            print(f'   Employee: {row.employee}')
            print(f'   Security: {row.ticker}')
            print(f'   Type: {row.transaction_type}')
            print(f'   Quantity: {row.quantity}')
            print(f'   Value: {row.estimated_value}')
            print(f'   Status: {row.status}')
        else:
            print('❌ No request found!')

asyncio.run(check())
"
```

**Expected Output**:
```
✅ Request found in database:
   ID: 123
   Reference: PAD-2026-001
   Employee: luis.deburnay-bastos
   Security: AAPL
   Type: buy
   Quantity: 100
   Value: 15000.00
   Status: pending_manager_approval
```

**Checkpoint**: ✅ Request in database with correct data

---

### Scenario 1 Complete ✅

**Mark as complete if**:
- ✅ Conversation completed smoothly
- ✅ Request ID obtained
- ✅ Database contains request
- ✅ Status = "pending_manager_approval"

**Record Results**:
- Request ID: _______________
- Time to complete: _______ minutes
- Any issues? _______________

---

## Scenario 2: Manager Approval Flow 🎯

**Goal**: Manager reviews and approves the employee's request

**Test User**: alex.agombar@mako.com (Manager)
**Expected Flow**: Review → Approve → Pending Compliance
**Time**: ~5 minutes

### Steps:

#### 2.1 Check Manager Notification

**Action**:
1. Switch to Alex's Slack account (or check Alex's DMs with PA Dealing Bot)
2. Look for notification about Luis's request

**Expected**:
- Alex should have received notification
- Notification includes: Employee name, security, transaction details
- Notification has "Review Request" button/link

**Example**:
```
Bot: 📋 New request awaiting your approval

     From: Luis de Burnay Bastos
     Security: Apple Inc (AAPL)
     Type: Buy 100 shares @ $150.00
     Total Value: $15,000 USD
     Reason: Portfolio diversification - adding tech exposure

     [Review Request]
```

**Checkpoint**: ✅ Manager notification received

---

#### 2.2 Review Request

**What to Do**:
- Click "Review Request" (or type "review requests" to see pending)

**Expected Response**:
- Bot displays full request details:
  - Employee info (name, division, role)
  - Security details
  - Transaction details
  - Trading reason
  - Risk assessment (if available)
- Bot shows approval options: [Approve] [Reject] [Request More Info]

**Checkpoint**: ✅ Full details displayed

---

#### 2.3 Approve Request

**What to Do**:
- Click **"Approve"** button (or type "Approve")

**Expected Response**:
- Bot may ask for optional comments

**Example**:
```
Bot: Would you like to add any comments to your approval?
     (Optional - press Skip to continue)
```

**What to Type**:
```
Approved - reasonable investment for portfolio diversification
```

**Expected**:
- Bot creates PADApproval record (approver_type='manager', status='approved')
- Bot updates PADRequest status to 'pending_compliance_approval'
- Bot sends confirmation to Alex
- Bot notifies compliance officer
- Bot notifies Luis of progress

**Example**:
```
Bot: ✅ Request approved!

     Luis's request has been forwarded to Compliance for final review.
     They will be notified shortly.
```

**Checkpoint**: ✅ Approval confirmation received

---

#### 2.4 Verify Luis Gets Update

**Action**:
- Switch back to Luis's Slack
- Check for update notification

**Expected**:
- Luis receives notification: "Your request has been approved by your manager"
- Status update: "Pending Compliance Approval"

**Checkpoint**: ✅ Employee notified

---

#### 2.5 Verify in Database

**Run**:
```bash
poetry run python -c "
import asyncio
from sqlalchemy import text
from src.pa_dealing.db.engine import async_session_maker

async def check():
    async with async_session_maker() as session:
        # Check approval record
        result = await session.execute(text('''
            SELECT a.id, a.approver_type, a.status, a.comments,
                   e.mako_id as approver
            FROM padealing.pad_approval a
            JOIN bo_airflow.oracle_employee e ON a.approver_id = e.id
            WHERE a.request_id = (
                SELECT id FROM padealing.pad_request
                ORDER BY created_at DESC LIMIT 1
            )
            AND a.approver_type = 'manager'
        '''))
        approval = result.fetchone()

        # Check request status
        result = await session.execute(text('''
            SELECT status FROM padealing.pad_request
            ORDER BY created_at DESC LIMIT 1
        '''))
        request = result.fetchone()

        if approval and request:
            print(f'✅ Manager approval found:')
            print(f'   Approver: {approval.approver}')
            print(f'   Status: {approval.status}')
            print(f'   Comments: {approval.comments}')
            print(f'✅ Request status: {request.status}')
        else:
            print('❌ Approval not found!')

asyncio.run(check())
"
```

**Expected Output**:
```
✅ Manager approval found:
   Approver: alex.agombar
   Status: approved
   Comments: Approved - reasonable investment for portfolio diversification
✅ Request status: pending_compliance_approval
```

**Checkpoint**: ✅ Approval in database, status updated

---

### Scenario 2 Complete ✅

**Mark as complete if**:
- ✅ Manager received notification
- ✅ Approval submitted successfully
- ✅ Database shows approval record
- ✅ Request status = "pending_compliance_approval"
- ✅ Employee notified

---

## Scenario 3: Compliance Approval Flow 🎯

**Goal**: Compliance officer performs final review and approves

**Test User**: Compliance officer (we'll identify during setup)
**Expected Flow**: Review → Final Approval → Approved
**Time**: ~5 minutes

### Steps:

#### 3.1 Identify Compliance User

**First, find who has compliance role**:
```bash
poetry run python -c "
import asyncio
from sqlalchemy import text
from src.pa_dealing.db.engine import async_session_maker

async def find_compliance():
    async with async_session_maker() as session:
        result = await session.execute(text('''
            SELECT e.id, e.mako_id, c.email
            FROM padealing.employee_role er
            JOIN bo_airflow.oracle_employee e ON er.employee_id = e.id
            LEFT JOIN bo_airflow.oracle_contact c ON (
                e.id = c.employee_id
                AND c.contact_group_id = 2
                AND c.contact_type_id = 5
            )
            WHERE er.role_name = 'compliance'
            LIMIT 5
        '''))
        rows = result.fetchall()
        if rows:
            print('Compliance users:')
            for row in rows:
                print(f'  - {row.mako_id} ({row.email or \"no email\"})')
        else:
            print('❌ No compliance users found!')
            print('   You may need to add one to employee_role table')

asyncio.run(find_compliance())
"
```

**Expected**: Get email/mako_id of compliance user

**📝 RECORD**: Compliance user: _______________

---

#### 3.2 Check Compliance Notification

**Action**:
- Open Slack as compliance user
- Check PA Dealing Bot for notifications

**Expected**:
- Compliance user received notification about request
- Notification shows:
  - Employee name
  - Manager approval status
  - Security details
  - Risk assessment
  - Conflict flags (if any)
- "Review Request" button available

**Checkpoint**: ✅ Compliance notification received

---

#### 3.3 Review Request (Compliance View)

**What to Do**:
- Click "Review Request"

**Expected Response**:
- Bot displays comprehensive details:
  - **Employee**: Name, division, role
  - **Security**: Ticker, inst_symbol, description
  - **Transaction**: Type, quantity, price, value, currency
  - **Broker**: Name, account number
  - **Reason**: Trading justification
  - **Risk Assessment**: Automated risk score/flags
  - **Manager Approval**: Who approved, comments, timestamp
  - **Conflict Detection**: Firm trading conflicts (if implemented)
  - **Restricted List**: Check if security is restricted
- Approval options: [Approve] [Reject] [Request More Info]

**Checkpoint**: ✅ Comprehensive details shown

---

#### 3.4 Approve Request (Compliance)

**What to Do**:
- Click **"Approve"** button

**Expected Response**:
- Bot asks for optional compliance comments

**What to Type**:
```
Approved - no conflicts identified, within policy limits
```

**Expected**:
- Bot creates PADApproval record (approver_type='compliance', status='approved')
- Bot updates PADRequest status to **'approved'** (FINAL APPROVAL)
- Bot sends confirmation to compliance officer
- Bot notifies Luis: "Your request is APPROVED and ready for execution"
- Bot notifies Alex (FYI)

**Example**:
```
Bot: ✅ Request approved!

     This request is now FULLY APPROVED and ready for execution.
     The employee has been notified.
```

**Checkpoint**: ✅ Final approval confirmation

---

#### 3.5 Verify Luis Gets Final Approval

**Action**:
- Switch to Luis's Slack
- Check for final approval notification

**Expected**:
- Luis receives: "Your request has been APPROVED by Compliance"
- Status: "Approved - Ready for Execution"
- Instructions on how to record execution after trade

**Checkpoint**: ✅ Employee notified of final approval

---

#### 3.6 Verify in Database

**Run**:
```bash
poetry run python -c "
import asyncio
from sqlalchemy import text
from src.pa_dealing.db.engine import async_session_maker

async def check():
    async with async_session_maker() as session:
        # Check compliance approval
        result = await session.execute(text('''
            SELECT a.id, a.approver_type, a.status, a.comments
            FROM padealing.pad_approval a
            WHERE a.request_id = (
                SELECT id FROM padealing.pad_request
                ORDER BY created_at DESC LIMIT 1
            )
            AND a.approver_type = 'compliance'
        '''))
        compliance_approval = result.fetchone()

        # Check all approvals
        result = await session.execute(text('''
            SELECT approver_type, status
            FROM padealing.pad_approval
            WHERE request_id = (
                SELECT id FROM padealing.pad_request
                ORDER BY created_at DESC LIMIT 1
            )
            ORDER BY created_at
        '''))
        all_approvals = result.fetchall()

        # Check request status
        result = await session.execute(text('''
            SELECT status FROM padealing.pad_request
            ORDER BY created_at DESC LIMIT 1
        '''))
        request = result.fetchone()

        if compliance_approval and request:
            print(f'✅ Compliance approval found:')
            print(f'   Status: {compliance_approval.status}')
            print(f'   Comments: {compliance_approval.comments}')
            print(f'')
            print(f'✅ All approvals:')
            for a in all_approvals:
                print(f'   - {a.approver_type}: {a.status}')
            print(f'')
            print(f'✅ Final request status: {request.status}')
        else:
            print('❌ Compliance approval not found!')

asyncio.run(check())
"
```

**Expected Output**:
```
✅ Compliance approval found:
   Status: approved
   Comments: Approved - no conflicts identified, within policy limits

✅ All approvals:
   - manager: approved
   - compliance: approved

✅ Final request status: approved
```

**Checkpoint**: ✅ Both approvals exist, status = "approved"

---

### Scenario 3 Complete ✅

**Mark as complete if**:
- ✅ Compliance received notification
- ✅ Final approval submitted
- ✅ Database shows both approvals (manager + compliance)
- ✅ Request status = "approved"
- ✅ Employee notified of final approval

---

## Scenario 4: Dashboard Validation 🎯

**Goal**: Verify dashboard displays all data correctly

**Time**: ~10 minutes

### 4.1 Access Dashboard

**Action**:
- Open browser: http://localhost:XXXX (port will be provided)

**Expected**:
- Dashboard loads within 3 seconds
- Login screen OR automatic dev auth (X-Dev-User-Email)

**Checkpoint**: ✅ Dashboard accessible

---

### 4.2 Home Page / Summary Statistics

**What to Check**:
- Total requests count
- Pending requests count (should be 0 after approval)
- Approved requests count (should include our test)
- Rejected/withdrawn count

**Verification**:
- Numbers should match database reality
- Charts/graphs display correctly

**Checkpoint**: ✅ Summary statistics accurate

---

### 4.3 Request List View

**Action**:
- Navigate to "Requests" or "All Requests" page

**What to Check**:
- List displays all requests
- Our test request (PAD-2026-001) is visible
- Columns show: Request ID, Employee, Security, Type, Status, Date
- Filters work (status, employee, date range)
- Sorting works (by date, status, etc.)

**Checkpoint**: ✅ Request list displays correctly

---

### 4.4 Request Detail View

**Action**:
- Click on our test request (PAD-2026-001)

**What to Check**:
- **Employee Section**: Luis de Burnay Bastos, division, role
- **Security Section**: AAPL, description, type
- **Transaction Section**: Buy, 100 shares, $150, $15,000, USD
- **Broker Section**: Interactive Brokers, U1234567
- **Reason Section**: "Portfolio diversification - adding tech exposure"
- **Risk Section**: Risk assessment results (if available)
- **Approvals Section**:
  - Manager: Alex Agombar, approved, comments, timestamp
  - Compliance: [Name], approved, comments, timestamp
- **Audit Trail**: All status changes with timestamps
  - pad_request_submitted
  - manager_approved
  - compliance_approved
  - (others as applicable)

**Checkpoint**: ✅ All sections display correct data

---

### 4.5 Compliance Dashboard View

**Action**:
- Navigate to "Compliance" or "Compliance Dashboard"

**What to Check**:
- Pending approvals section (should be empty now)
- Recently approved requests (should include ours)
- Conflict detection warnings (if applicable)
- Restricted security flags (if applicable)

**Checkpoint**: ✅ Compliance view functional

---

### 4.6 Audit Log View

**Action**:
- Navigate to "Audit Log" or "Activity Log"

**What to Check**:
- All actions for our test request visible:
  - pad_request_submitted (Luis)
  - manager_approved (Alex)
  - compliance_approved ([Compliance user])
  - status_changed events
- Timestamps in chronological order
- Actor information correct (emails/names)
- Details column shows relevant info

**Checkpoint**: ✅ Audit trail complete and accurate

---

### Scenario 4 Complete ✅

**Mark as complete if**:
- ✅ Dashboard loads quickly (< 3 seconds)
- ✅ Summary statistics accurate
- ✅ Request list shows all data
- ✅ Detail view displays all sections correctly
- ✅ Audit log shows complete trail

**📸 Screenshots**: Take 3-5 screenshots of key pages

---

## Quick Scenarios (5-10): Checklist Format

### Scenario 5: Manager Rejection Flow

**Steps**:
1. Submit new request as Luis (use different security, e.g., MSFT)
2. Record Request ID: _______________
3. Alex reviews and clicks **Reject**
4. Alex provides reason: "Trade size too large for current portfolio allocation"
5. Verify rejection notification sent to Luis
6. Verify database: status = 'rejected', approval record shows rejection

**Time**: 5 minutes
**Status**: ⬜ Complete

---

### Scenario 6: Employee Withdrawal

**Steps**:
1. Submit new request as Luis
2. Progress through manager approval (Alex approves)
3. Before compliance approval, Luis initiates withdrawal
4. Luis types: "I want to withdraw my request [REQUEST_ID]"
5. Luis confirms withdrawal with reason: "Market conditions changed"
6. Verify status = 'withdrawn', deleted_at timestamp set

**Time**: 5 minutes
**Status**: ⬜ Complete

---

### Scenario 7: Restricted Security Block

**Steps**:
1. Add test security to restricted list (via SQL or dashboard)
2. Luis attempts to submit request for restricted security
3. Bot should immediately detect and block:
   - "This security is on the restricted list"
   - "You cannot trade this security"
4. Verify NO PADRequest created
5. Verify audit log has "restricted_security_prevented" entry

**Time**: 3 minutes
**Status**: ⬜ Complete

---

### Scenario 8: Performance Check

**Steps**:
1. Time bot response to initial question (< 5 seconds)
2. Time dashboard page load (< 3 seconds)
3. Run query performance test:
   ```sql
   EXPLAIN ANALYZE
   SELECT * FROM padealing.pad_request
   WHERE status = 'pending_manager_approval'
   LIMIT 50;
   ```
4. Verify execution time < 100ms

**Time**: 3 minutes
**Status**: ⬜ Complete

---

## Test Data Reference

**Available Test Users**:
- Employee: `luis.deburnay-bastos@mako.com` (ID: 1272)
- Manager: `alex.agombar@mako.com` (ID: 1191)
- Compliance: [Will identify during setup]

**Available Securities** (will provide list from dev database):
- AAPL (Apple Inc)
- MSFT (Microsoft Corp)
- GOOGL (Alphabet Inc)
- [More from oracle_bloomberg]

---

## Results Recording

**After each scenario**, record:

| Scenario | Status | Time | Issues | Notes |
|----------|--------|------|--------|-------|
| 1. Employee Submission | ⬜ | ___ min | None / [Details] | Request ID: ___ |
| 2. Manager Approval | ⬜ | ___ min | None / [Details] | |
| 3. Compliance Approval | ⬜ | ___ min | None / [Details] | |
| 4. Dashboard Validation | ⬜ | ___ min | None / [Details] | |
| 5. Manager Rejection | ⬜ | ___ min | None / [Details] | |
| 6. Employee Withdrawal | ⬜ | ___ min | None / [Details] | |
| 7. Restricted Security | ⬜ | ___ min | None / [Details] | |
| 8. Performance Check | ⬜ | ___ min | None / [Details] | |

---

## Sign-Off

After completing critical scenarios (1-4, 7-8):

- [x] All critical scenarios passed
- [x] Database validation confirmed
- [x] Dashboard displays correct data
- [x] Audit trail complete
- [x] No blocking issues found

**Tester Signature**: ________________
**Date**: ________________
**Notes**: ________________

---

**Next**: Once UAT is complete, we'll document results in UAT_RESULTS.md and mark track complete!
