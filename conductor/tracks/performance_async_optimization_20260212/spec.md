# Spec: Performance & Async Optimization

## Problem Statement
Per-request HTTP client creation, synchronous GCS operations blocking the async event loop, and N+1 query patterns add unnecessary latency and limit throughput.

## Source
- `.autopsy/ARCHITECTURE_REPORT.md` - Section 3: Per-request HTTP clients, Section 4: Performance quality attribute
- `.autopsy/REVIEW_REPORT.md` - HIGH: Performance Issues (38 findings)

## Findings (Verified Against Code)

### 1. Per-Request HTTP Clients (HIGH)
5 locations create new `httpx.AsyncClient()` per request:
| File | Line | Context |
|------|------|---------|
| `services/price_discovery/eodhd_provider.py` | 111 | EODHD API price lookups |
| `logging/http_client.py` | 17 | Correlation header example |
| `instruments/external_resolver.py` | 65 | EODHD instrument lookup |
| `identity/google.py` | 109 | Google Directory API |
| `identity/google.py` | 159 | Google manager status check |

- **Impact:** 50-200ms added latency per call (TCP + TLS handshake). No connection reuse.
- **Fix:** Create shared `httpx.AsyncClient` with connection pooling, initialized in FastAPI lifespan

### 2. GCS Blocking Async Event Loop (HIGH)
- **File:** `services/gcs_client.py` - ALL methods are synchronous `def` (lines 92-332)
- **Called from:** `services/pdf_poller.py` (async context, lines 111-340)
- **Operations:** `list_blobs()`, `blob.reload()`, `blob.download_as_bytes()`, `rename_blob()`
- **Impact:** Event loop blocked during PDF processing. Large batches can block for seconds.
- **Fix:** Use `asyncio.to_thread()` wrapper for GCS calls, or migrate to async GCS library

### 3. N+1 Queries and Unbounded Results (HIGH)
- **File:** `db/repository.py`
- `get_all_requests()` (lines 274-309): Accesses related objects in list comprehension. Has `selectinload` but may still trigger lazy loads.
- Holding period check (lines 630-675): Fetches all trades, filters in Python. No LIMIT.
- Mako conflict check (lines 425-454): Fetches all positions, uses only first 5. Should be `LIMIT 5` in query.
- **Impact:** Unnecessary data transfer, client-side filtering of large result sets, potential lock contention.

## Requirements
1. Create shared `httpx.AsyncClient` factory with connection pooling
2. Initialize in FastAPI lifespan, close on shutdown
3. Replace per-request client creation in 5 files
4. Wrap synchronous GCS calls with `asyncio.to_thread()`
5. Add LIMIT to unbounded queries (Mako conflict: LIMIT 5)
6. Move client-side filtering to SQL WHERE clauses where possible
7. Verify `selectinload` patterns are sufficient (no lazy load warnings)

## Acceptance Criteria
- [ ] Single shared `httpx.AsyncClient` used for all external HTTP calls
- [ ] GCS operations do not block the async event loop
- [ ] No unbounded queries without LIMIT/pagination
- [ ] Client-side filtering moved to SQL where feasible
- [ ] All existing tests pass
- [ ] Performance improvement measurable (latency reduction on external API calls)
