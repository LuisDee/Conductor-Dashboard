# Spec: API & Frontend Performance Optimization

## Problem Statement

The PA Dealing dashboard pages are extremely slow to load, with `/api/auth/me` taking **5+ seconds** on initial page load. This severely impacts user experience and productivity for compliance officers reviewing requests.

### Observed Behavior
1. User navigates to any dashboard page
2. Blank screen displayed for 3-5 seconds
3. `/api/auth/me` endpoint takes 1-5 seconds alone
4. Additional API calls cause waterfall loading pattern
5. Total time-to-interactive: 5-10 seconds

### Target Performance
- `/api/auth/me`: <200ms (currently 1-5 seconds)
- Dashboard initial load: <1 second (currently 5-10 seconds)
- Page navigation: <500ms (currently 2-3 seconds)

---

## Root Cause Analysis

### Category 1: Authentication Flow Bottlenecks (CRITICAL)

**Location**: `src/pa_dealing/identity/provider_google.py`, `src/pa_dealing/api/auth.py`

#### Issue 1.1: Duplicate Google API Calls (600-1400ms wasted)

```
Current flow for /api/auth/me:
├─ get_user_info(user@email.com)      [API #1: 100-300ms]
├─ find_best_match() - fuzzy scoring  [200-500ms]
├─ get_by_email(manager@email.com)    [RECURSIVE]
│  ├─ get_user_info(manager@email)    [API #2: 100-300ms]
│  └─ find_best_match()               [200-500ms]
├─ is_manager(user@email.com)         [API #3: 100-300ms]
└─ is_manager(user@email.com) AGAIN   [API #4: 100-300ms] (in auth.py!)
```

**Problems Identified**:
1. `is_manager()` called 2-3 times per authentication (in `get_by_email` AND `get_current_user`)
2. Manager resolution triggers recursive `get_by_email()` with full Google API + fuzzy matching
3. Existing user cache (`google.py:_user_cache`) is NEVER USED - direct calls bypass it
4. `is_manager()` has NO cache at all

#### Issue 1.2: Inefficient 3-Tier Fuzzy Matching (200-500ms)

```python
# fuzzy_matcher.py - Current flow
1. Fetch ALL employees by surname initial (100-500 candidates)
2. Run 6 scoring metrics on EACH candidate (JaroWinkler, token_set_ratio, etc.)
3. Sort all candidates in Python
4. Return best match
```

**Problem**: 90% of users will match on exact email lookup. Fuzzy matching should be a fallback, not the primary path.

**Recommended Order**:
1. **Tier 1**: Exact email match (5ms) - SHOULD BE FIRST
2. **Tier 2**: Exact mako_id match (5ms)
3. **Tier 3**: Fuzzy name matching (200-500ms) - FALLBACK ONLY

#### Issue 1.3: Unnecessary Manager Resolution

The `/api/auth/me` endpoint only needs `is_manager: boolean`, but currently:
- Fetches full manager identity via recursive `get_by_email(manager_email)`
- Triggers separate Google API call for manager
- Triggers fuzzy matching for manager

**Fix**: Skip manager resolution entirely for auth - `is_manager` comes from Google API result already.

---

### Category 2: API Route Bottlenecks (HIGH)

**Location**: `src/pa_dealing/api/routes/`

#### Issue 2.1: N+1 Query in `/documents` Endpoint

**File**: `documents.py:169-191`

```python
# CURRENT: 1 + N queries (20 documents = 21 queries)
documents = result.scalars().all()
for doc in documents:
    trade_count_result = await session.execute(
        select(func.count()).where(ParsedTrade.document_id == doc.id)
    )
```

**Fix**: Single grouped query with LEFT JOIN

#### Issue 2.2: Sequential Stats Counts (8 queries → 1)

**File**: `documents.py:422-448`

```python
# CURRENT: 8 separate COUNT queries
for status in [DocumentStatus.PENDING, DocumentStatus.PROCESSING, ...]:
    result = await session.execute(
        select(func.count()).where(GCSDocument.status == status)
    )
```

**Fix**: Single GROUP BY query

#### Issue 2.3: Dashboard Summary Fetches Full Lists for Counts

**File**: `dashboard.py:224-244`

```python
# CURRENT: Fetches 100+ items just to count them
pending = await service.get_pending_approvals(...)
breaches = await service.get_active_breaches()
return {"pending_approvals": len(pending)}  # Only needs count!
```

**Fix**: Create dedicated count-only service methods

#### Issue 2.4: Duplicate Identity Provider Lookups

Multiple `get_session()` + `identity.get_by_email()` calls within single endpoint.

---

### Category 3: Frontend Bottlenecks (HIGH)

**Location**: `dashboard/src/`

#### Issue 3.1: Waterfall Auth + Data Fetching

```
Current page load sequence:
1. ProtectedRoute: fetch /auth/me [BLOCKING]
   └─ Wait...
2. Page component mounts
3. Page: fetch /auth/me AGAIN (different query key!)
   └─ Wait...
4. Page: fetch /audit/employees
   └─ Wait...
5. Page: fetch main data
   └─ Finally visible
```

**Total**: 4 sequential network waterfalls instead of 2-3 parallel calls.

#### Issue 3.2: Duplicate Employee List Fetches

6 pages independently fetch the same employee list:
- PendingApprovals, Breaches, ExecutionTracking, HoldingPeriods, MakoConflicts, AuditLog

Each navigation triggers a new fetch instead of sharing cache.

#### Issue 3.3: Inconsistent Query Keys for Auth

```typescript
// ProtectedRoute.tsx
queryKey: ['current-user', getDevUserEmail()]

// Dashboard.tsx
queryKey: ['current-user', getDevUserEmail()]  // Same

// RequestDetail.tsx
queryKey: ['me']  // DIFFERENT - cache miss!
```

#### Issue 3.4: Dev Mode Cache Invalidation

```typescript
staleTime: isDevMode() ? 0 : 30000  // Always stale in dev!

const handleSwitchUser = () => {
  queryClient.invalidateQueries()  // Invalidate EVERYTHING
}
```

#### Issue 3.5: No Suspense/Lazy Loading

All pages eagerly imported and bundled. No code splitting.

---

### Category 4: Database Bottlenecks (MEDIUM)

**Location**: `src/pa_dealing/db/models/`

#### Issue 4.1: Missing Indexes

| Table | Missing Index | Impact |
|-------|---------------|--------|
| `notification_outbox` | `(status, next_attempt_at)` | Full table scan on retry processing |
| `pad_request` | `(created_at DESC)` | Slow dashboard sorting |
| `pad_request` | `(deleted_at)` partial | Slow soft-delete filtering |
| `parsed_trade` | `(match_status, request_id)` composite | Slow status filtering |

#### Issue 4.2: N+1 Relationship Loading

- `PADApproval.approver` not eager-loaded
- `ParsedTrade.documents` many-to-many triggers separate queries
- `PADBreach.employee` and `.resolved_by` not eager-loaded

#### Issue 4.3: JSONB Column Bloat

`parsed_trade.raw_extracted_data` can be 100KB+ per record but is loaded with default queries.

#### Issue 4.4: Connection Pool Missing Recycle

No `pool_recycle` configured - stale connections may cause issues.

---

## Acceptance Criteria

### AC1: Auth Endpoint Performance (<200ms)
- [ ] `/api/auth/me` responds in <200ms for cached users
- [ ] `/api/auth/me` responds in <500ms for uncached users (first login)
- [ ] Google API calls batched or cached (max 1 call per auth)
- [ ] No duplicate `is_manager()` calls
- [ ] Manager resolution skipped for auth endpoint

### AC2: Dashboard Load Performance (<1s)
- [ ] Dashboard visible with data in <1 second
- [ ] Summary counts use dedicated count queries
- [ ] N+1 queries eliminated from document list
- [ ] Stats endpoints consolidated to single grouped query

### AC3: Frontend Optimization
- [ ] Auth fetched once, shared across all components
- [ ] Employee list prefetched at app root
- [ ] Parallel data fetching where dependencies allow
- [ ] Consistent query keys for all auth queries
- [ ] Lazy loading for non-critical pages

### AC4: Database Optimization
- [ ] All missing indexes added
- [ ] Eager loading configured for common relationships
- [ ] Large JSONB columns excluded from default selections
- [ ] Connection pool configured with `pool_recycle=3600`

### AC5: Measurable Improvement
- [ ] Lighthouse performance score >80 (currently ~50)
- [ ] API average response time <200ms (currently ~1000ms)
- [ ] Time-to-interactive <2 seconds (currently ~5-10 seconds)

---

## Technical Approach

### Phase 1: Auth Caching (Highest Impact)

1. **Use existing `lookup_user()` cache** instead of direct `get_user_info()`
2. **Add `is_manager()` cache** with 1-hour TTL
3. **Skip manager resolution** in auth flow - only need `is_manager` boolean
4. **Batch Google API calls** if possible (get_user_info + is_manager in one)

**Expected savings**: 400-1000ms per request

### Phase 2: Optimize Identity Resolution

1. **Reorder 3-tier lookup**:
   - Tier 1: Exact email match (5ms)
   - Tier 2: Exact mako_id match (5ms)
   - Tier 3: Fuzzy matching (200-500ms) - FALLBACK ONLY
2. **Cache fuzzy match results** (1-hour TTL)
3. **Short-circuit** on first match

**Expected savings**: 200-400ms for 90% of requests

### Phase 3: Fix N+1 Queries

1. **Documents endpoint**: Join + GROUP BY for trade counts
2. **Stats endpoints**: Single GROUP BY query
3. **Dashboard summary**: Count-only service methods
4. **Add missing indexes**: status, created_at, match_status

**Expected savings**: 100-300ms per list endpoint

### Phase 4: Frontend Optimization

1. **Unify auth query key** to `['current-user']`
2. **Prefetch employees** at app root level
3. **Parallelize** auth + employees + main data queries
4. **Lazy load** non-critical pages (Reports, AuditLog, Config)
5. **Fix dev mode** staleTime (use 30000 in dev too)

**Expected savings**: 500-2000ms on page load

### Phase 5: Database Optimization

1. **Add indexes**:
   ```sql
   CREATE INDEX ix_notification_pending ON notification_outbox(status, next_attempt_at);
   CREATE INDEX ix_pad_request_created ON pad_request(created_at DESC);
   CREATE INDEX ix_parsed_trade_status ON parsed_trade(match_status, request_id);
   ```
2. **Add `selectinload()`** for PADApproval.approver, PADBreach.employee
3. **Exclude `raw_extracted_data`** from default ParsedTrade selection
4. **Configure `pool_recycle=3600`** in engine settings

---

## Quick Wins (Implement First)

| Fix | Impact | Effort | Location |
|-----|--------|--------|----------|
| Remove duplicate `is_manager()` call in auth.py | 100-300ms | 5 min | `auth.py:129` |
| Use existing `lookup_user()` cache | 100-300ms | 15 min | `provider_google.py:225` |
| Add `is_manager()` cache | 100-300ms | 30 min | `google.py` |
| Unify auth query key to `['current-user']` | Cache hits | 15 min | `RequestDetail.tsx:*` |
| Fix dev mode staleTime | Cache hits | 5 min | All pages with `isDevMode()` |
| Skip manager resolution in auth | 200-600ms | 30 min | `provider_google.py:294-309` |

**Total quick wins**: 600-1800ms improvement with ~2 hours work

---

## Out of Scope

- Redis/Memcached distributed caching (future enhancement)
- CDN for static assets (infrastructure change)
- Server-side rendering (architecture change)
- WebSocket real-time updates (separate feature)

---

## Test Plan

1. **Benchmark before/after**: Record `/api/auth/me` response times
2. **Lighthouse audit**: Run before and after each phase
3. **Load test**: 50 concurrent users hitting dashboard
4. **Regression tests**: All existing tests must pass
5. **Manual UAT**: Navigate through all dashboard pages, verify responsiveness

---

## References

### FastAPI Performance
- [FastAPI Dependency Caching](https://fastapi.tiangolo.com/advanced/advanced-dependencies/) - `use_cache=False` for per-request behavior
- [FastAPI Background Tasks](https://fastapi.tiangolo.com/tutorial/background-tasks/) - Resource isolation patterns
- [lru_cache for Settings](https://fastapi.tiangolo.com/advanced/settings/) - Single instantiation pattern

### TanStack Query Performance
- [Prefetching](https://tanstack.com/query/latest/docs/framework/react/guides/prefetching) - `queryClient.prefetchQuery()` patterns
- [Parallel Queries](https://tanstack.com/query/latest/docs/framework/react/guides/parallel-queries) - Concurrent data fetching
- [Query Deduplication](https://tanstack.com/query/latest/docs/framework/react/guides/caching) - Automatic request deduplication

### SQLAlchemy Performance
- [selectinload](https://docs.sqlalchemy.org/en/20/orm/queryguide/relationships.html#selectin-eager-loading) - Batch relationship loading
- [joinedload](https://docs.sqlalchemy.org/en/20/orm/queryguide/relationships.html#joined-eager-loading) - Single-query relationship loading
- [Async Session Best Practices](https://docs.sqlalchemy.org/en/20/orm/extensions/asyncio.html) - Avoiding implicit IO
