# Track Brief: Performance & Async Optimization

**Goal**: Eliminate per-request HTTP overhead, unblock async event loop from GCS, and optimize database queries.

**Source**: `.autopsy/ARCHITECTURE_REPORT.md` + `.autopsy/REVIEW_REPORT.md` - verified against code.

## Scope
3 areas: (1) Shared httpx.AsyncClient for 5 files. (2) asyncio.to_thread() wrappers for GCS operations. (3) Query optimization (LIMIT, SQL-side filtering).

## Key Files
- `services/price_discovery/eodhd_provider.py`, `instruments/external_resolver.py`, `identity/google.py` (HTTP clients)
- `services/gcs_client.py` (sync GCS), `services/pdf_poller.py` (caller)
- `db/repository.py` (N+1 queries, unbounded results)

## Effort Estimate
M (1-2 weeks) - shared client setup + GCS wrapper + query optimization
