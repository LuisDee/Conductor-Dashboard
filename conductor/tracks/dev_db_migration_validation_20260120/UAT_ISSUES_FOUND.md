# UAT Issues Found - 2026-01-21

**Status**: Investigation Required
**Priority**: HIGH - Blocking UAT completion

---

## Issues Discovered During UAT

### Issue 1: Slack Message Formatting Broken 🔴

**Symptom**:
Bot messages show markdown syntax instead of formatted text:
```
Would you like to **BUY** or **SELL**?
```

**Expected**:
```
Would you like to *BUY* or *SELL*?  (renders as bold in Slack)
```

**Root Cause**:
Bot is using markdown `**bold**` syntax instead of Slack's `*bold*` syntax.

**Location**: Likely in the LLM response generation or UI building

**Fix Required**: Update prompt or post-process LLM responses to use Slack formatting

---

### Issue 2: Summary Not Displayed 🔴

**Symptom**:
Bot says "here is a summary for you to review" but NO summary is shown.

**Expected**:
Block Kit UI with structured summary (security, direction, quantity, value, etc.)

**Root Cause**:
Bot is NOT calling `show_preview()` function even though it says it will.

**Evidence from logs**:
```
DEBUG: update_draft called with security_search_term=Euro-Bund Future
(No "show_preview" call found in logs)
```

**Fix Required**:
- Debug why `show_preview` isn't being called
- Check if `instructional_hint` says "DRAFT_COMPLETE"
- Verify chatbot agent logic for calling show_preview

---

### Issue 3: Security Lookup Failure 🔴

**Symptom**:
User searched for "spreads on bund" / "FGBL" / "RX1" - bot couldn't find security.

**Test Results**:
```sql
-- Searched dev database for:
FGBL ticker:     0 results ❌
RX1 ticker:      0 results ❌
Euro-Bund:       0 results ❌
German bonds:    5 results (but NOT Bund futures)
```

**Root Cause**:
Euro-Bund futures (German government bond futures) **DO NOT exist** in dev database `bo_airflow.oracle_bloomberg` table.

**Oracle Data Coverage**:
- ✅ Has: Equities (AAPL, MSFT, etc.)
- ✅ Has: Some futures (DAX futures found)
- ❌ Missing: Euro-Bund futures (FGBL, RX1)

**Implications**:
- Security lookup is working correctly (searched, found nothing)
- Database is incomplete for futures contracts
- UAT needs to use securities that EXIST in dev database

---

### Issue 4: 3-Tier Lookup Investigation Needed 🟡

**Question**: Is the 3-tier instrument lookup working?

**3-Tier Lookup** (from Advanced Instrument Validation track):
```
User Input → Bloomberg Lookup → MapInstSymbol → Product
```

**Test Searches Attempted**:
1. "FGBL" → Not found in any tier
2. "RX1 Comdty" → Not found
3. "Euro-Bund Future" → Not found

**Investigation Required**:
- [x] Check if `oracle_map_inst_symbol` table exists and has data
- [x] Check if lookup queries all 3 tiers correctly
- [x] Test with a known security (e.g., AAPL) to verify lookup works
- [x] Check logs for which tables were queried

---

## Investigation Plan

### Phase 1: Immediate Fixes (Formatting & Summary)

**1.1 Fix Slack Formatting**
- [x] Find where LLM responses are generated
- [x] Post-process to replace `**text**` with `*text*`
- [x] OR update system prompt to use Slack markdown

**1.2 Fix Summary Not Showing**
- [x] Add debug logging to `show_preview()` function
- [x] Check why `instructional_hint` isn't triggering it
- [x] Verify session state when draft is complete
- [x] Test with a complete draft

### Phase 2: Security Lookup Validation

**2.1 Test with Known Securities**
- [x] Try AAPL (known to exist)
- [x] Try MSFT (known to exist)
- [x] Try DAX (future that exists)
- [x] Verify 3-tier lookup works for these

**2.2 Investigate Database Coverage**
```sql
-- Check what securities are available
SELECT
    inst_type,
    COUNT(*) as count,
    array_agg(DISTINCT ticker ORDER BY ticker) FILTER (WHERE ticker IS NOT NULL) as sample_tickers
FROM bo_airflow.oracle_bloomberg
GROUP BY inst_type
ORDER BY count DESC
LIMIT 20;
```

**2.3 Check MapInstSymbol Table**
```sql
-- Check if mapping table exists and has data
SELECT COUNT(*) FROM bo_airflow.oracle_map_inst_symbol;

-- Sample mappings
SELECT * FROM bo_airflow.oracle_map_inst_symbol LIMIT 10;
```

### Phase 3: Update UAT Guide

**3.1 Document Available Securities**
- [x] Query dev database for available securities by type
- [x] Create list of test-friendly securities:
  - ✅ Equities: AAPL, MSFT, GOOGL
  - ✅ Futures: DAX, [others from database]
  - ❌ Avoid: Bund futures (not in dev)

**3.2 Update UAT_GUIDE.md**
- [x] Replace Bund examples with available securities
- [x] Add "Available Test Securities" section
- [x] Warn about incomplete futures coverage

---

## Quick Fixes for Immediate UAT

### Workaround 1: Use Available Securities

**Instead of**: "spreads on bund"
**Use**: "AAPL" or "Apple stock"

**Test Securities Known to Exist**:
- AAPL (Apple Inc) - Equity
- DAX (German Stock Index) - Future
- [Need to query for more]

### Workaround 2: Format Issues

**Accept for now** - formatting is cosmetic, doesn't block functionality

### Workaround 3: Summary Issue

**Workaround**: Ask user to type "show summary" or "submit" after providing all info

---

## SQL Queries for Investigation

```sql
-- 1. Check what inst_types exist
SELECT inst_type, COUNT(*)
FROM bo_airflow.oracle_bloomberg
GROUP BY inst_type
ORDER BY COUNT DESC;

-- 2. Find available futures
SELECT ticker, bloomberg, inst_symbol, description
FROM bo_airflow.oracle_bloomberg
WHERE inst_type IN ('F', 'FUT', 'FUTURE')
LIMIT 20;

-- 3. Check if MapInstSymbol table has data
SELECT COUNT(*) FROM bo_airflow.oracle_map_inst_symbol;

-- 4. Sample securities for testing
SELECT ticker, description, inst_type
FROM bo_airflow.oracle_bloomberg
WHERE ticker IS NOT NULL
AND ticker != ''
ORDER BY RANDOM()
LIMIT 20;
```

---

## Priority Actions

**CRITICAL** (Must fix for UAT):
1. ✅ Identify test securities that exist in dev database
2. ⚠️ Fix summary not displaying (blocking submission)
3. ⏳ Update UAT guide with working securities

**HIGH** (Should fix soon):
4. Fix Slack formatting (cosmetic but unprofessional)
5. Investigate 3-tier lookup completeness

**MEDIUM** (Track for later):
6. Document dev database coverage gaps
7. Create follow-up track for data enrichment

---

## Next Steps

1. **Run SQL queries** to get list of available test securities
2. **Test with AAPL** to verify lookup and submission work end-to-end
3. **Debug show_preview** issue with logging
4. **Update UAT_GUIDE.md** with correct test data
5. **Resume UAT** with working securities

---

**Created**: 2026-01-21 22:56
**Status**: Investigation in progress
**Impact**: HIGH - Blocking UAT completion
