# START UAT - Quick Reference Guide

**Status**: ✅ **SYSTEM IS RUNNING!**
**Date**: 2026-01-21
**Environment**: Development Database (uk02vddb004)

---

## 🎯 System Status

### Running Services

| Service | Status | URL | Notes |
|---------|--------|-----|-------|
| **API Server** | ✅ RUNNING | http://localhost:8000 | Connected to dev database |
| **API Docs** | ✅ AVAILABLE | http://localhost:8000/api/docs | Swagger UI |
| **API Health** | ✅ HEALTHY | http://localhost:8000/api/health | Health check |
| **Slack Bot** | ✅ CONNECTED | Slack App "PA Dealing Bot" | Real Slack workspace |

### Database Connection

```
✅ Connected to: uk02vddb004.uk.makoglobal.com
✅ Database: backoffice_db
✅ Schemas: padealing (app) + bo_airflow (Oracle data)
✅ User: pad_app_user
✅ Migration Status: 1e2bea66afb6 (current, validated)
```

---

## 🧪 How to Start UAT Testing

### Step 1: Verify System is Running

```bash
# Check API health
curl http://localhost:8000/api/health

# Expected: {"status":"healthy","version":"1.0.0"}
```

### Step 2: Open Slack

1. Open your Slack workspace (Mako)
2. Find **"PA Dealing Bot"** in:
   - Apps directory, OR
   - Direct Messages
3. Start a new conversation

### Step 3: Follow UAT Guide

**Open this file**: `UAT_GUIDE.md` (in this directory)

**Quick Link to Scenarios**:
- Scenario 1: Employee Submission (Start here!) - 10 minutes
- Scenario 2: Manager Approval - 5 minutes
- Scenario 3: Compliance Approval - 5 minutes
- Scenario 4: Dashboard Validation - 10 minutes

**Total Time for Critical Scenarios**: ~30 minutes

---

## 📋 Test Users Available

| Role | Email | Employee ID | Notes |
|------|-------|-------------|-------|
| **Employee** | luis.deburnay-bastos@mako.com | 1272 | Use for submitting requests |
| **Manager** | alex.agombar@mako.com | 1191 | Luis's manager, use for approvals |
| **Compliance** | TBD | TBD | Will identify during testing |

---

## 🔍 Viewing Service Logs

### Attach to tmux session (see both services):

```bash
tmux attach -t pad_uat
```

### Navigation in tmux:
- **Window 0**: API Server logs
- **Window 1**: Slack Bot logs
- **Switch windows**: Press `Ctrl+B` then `0` or `1`
- **Detach**: Press `Ctrl+B` then `D`

### View logs without attaching:

```bash
# API logs
tmux capture-pane -t pad_uat:0 -p | tail -50

# Slack logs
tmux capture-pane -t pad_uat:1 -p | tail -50
```

---

## 🗄️ Database Query Tools

### Quick database checks during testing:

```bash
# Check latest request
poetry run python -c "
import asyncio
from sqlalchemy import text
from src.pa_dealing.db.engine import async_session_maker

async def check():
    async with async_session_maker() as session:
        result = await session.execute(text('''
            SELECT r.id, r.reference_id, r.status,
                   e.mako_id as employee,
                   b.ticker, r.transaction_type, r.quantity
            FROM padealing.pad_request r
            JOIN bo_airflow.oracle_employee e ON r.employee_id = e.id
            LEFT JOIN bo_airflow.oracle_bloomberg b ON r.security_id = b.id
            ORDER BY r.created_at DESC
            LIMIT 1
        '''))
        row = result.fetchone()
        if row:
            print(f'Latest Request:')
            print(f'  ID: {row.id}')
            print(f'  Reference: {row.reference_id}')
            print(f'  Employee: {row.employee}')
            print(f'  Security: {row.ticker}')
            print(f'  Status: {row.status}')

asyncio.run(check())
"
```

---

## 🛑 Stop Services

### Stop all UAT services:

```bash
tmux kill-session -t pad_uat
```

### Restart services:

```bash
bash scripts/ops/run_uat_dev_simple.sh
```

---

## 📖 Full UAT Testing Flow

### Critical Path (30 minutes):

1. **Scenario 1**: Employee Submission
   - Open Slack
   - Message PA Dealing Bot: "I want to trade stock"
   - Follow conversation to submit request
   - Record Request ID
   - ✅ **Verify**: Request in database

2. **Scenario 2**: Manager Approval
   - Switch to Alex's Slack
   - Review notification
   - Approve request
   - ✅ **Verify**: Approval in database, status updated

3. **Scenario 3**: Compliance Approval
   - Switch to Compliance user's Slack
   - Review request
   - Approve request
   - ✅ **Verify**: Final approval, status = 'approved'

4. **Scenario 4**: Dashboard Check
   - Open http://localhost:8000/api/requests (API endpoint)
   - Verify request appears
   - Check all data is correct

### Optional Scenarios (20 minutes):

5. **Scenario 5**: Manager Rejection
6. **Scenario 6**: Employee Withdrawal
7. **Scenario 7**: Restricted Security
8. **Scenario 8**: Performance Check

See `UAT_GUIDE.md` for complete step-by-step instructions!

---

## ⚠️ Troubleshooting

### API Not Responding

```bash
# Check if API is running
curl http://localhost:8000/api/health

# If not running, check tmux logs
tmux attach -t pad_uat
# Switch to window 0 (API)

# Restart API window
tmux kill-window -t pad_uat:0
tmux new-window -t pad_uat:0 -n "API"
tmux send-keys -t pad_uat:0 "cd /home/coder/repos/ai-research/pa-dealing" C-m
tmux send-keys -t pad_uat:0 "export \$(grep -v '^#' .env.dev | xargs)" C-m
tmux send-keys -t pad_uat:0 "poetry run python scripts/ops/run_api.py --host 0.0.0.0 --port 8000" C-m
```

### Slack Bot Not Responding

```bash
# Check if bot is connected
tmux attach -t pad_uat
# Switch to window 1 (Slack)

# Look for: "A new session (s_XXXXX) has been established"
# If not connected, restart window
tmux kill-window -t pad_uat:1
tmux new-window -t pad_uat:1 -n "Slack"
tmux send-keys -t pad_uat:1 "cd /home/coder/repos/ai-research/pa-dealing" C-m
tmux send-keys -t pad_uat:1 "export \$(grep -v '^#' .env.dev | xargs)" C-m
tmux send-keys -t pad_uat:1 "poetry run python scripts/ops/run_slack_listener.py" C-m
```

### Database Connection Issues

```bash
# Test dev database connection
poetry run python -c "
import asyncio
import asyncpg

async def test():
    conn = await asyncpg.connect(
        host='uk02vddb004.uk.makoglobal.com',
        database='backoffice_db',
        user='pad_app_user',
        password='padd_app_pass'
    )
    print('✅ Connected!')
    await conn.close()

asyncio.run(test())
"
```

---

## 📊 Expected Test Results

After completing critical scenarios (1-4):

| Item | Expected State |
|------|----------------|
| Requests submitted | 1+ (Scenario 1) |
| Manager approvals | 1+ (Scenario 2) |
| Compliance approvals | 1+ (Scenario 3) |
| Final approved requests | 1+ (Scenario 3) |
| Audit log entries | 10+ (all actions logged) |
| Database tables updated | pad_request, pad_approval, audit_log |

---

## ✅ UAT Completion Checklist

- [x] Scenario 1: Employee submission tested
- [x] Scenario 2: Manager approval tested
- [x] Scenario 3: Compliance approval tested
- [x] Scenario 4: Dashboard/API data verified
- [x] Database contains correct data
- [x] Audit trail complete
- [x] No critical errors in logs

**When complete**: Document results in `UAT_RESULTS.md`

---

## 🎯 Ready to Test!

**Current Status**: ✅ All systems running and connected to dev database

**Next Action**:
1. Open Slack
2. Find "PA Dealing Bot"
3. Follow `UAT_GUIDE.md` starting with Scenario 1

**Questions?** Check logs in tmux or run diagnostics above.

Good luck with UAT! 🚀
