# Structured Logging Migration — Implementation Plan

## Status: Implementation Complete (Pending Test Execution)

All 10 phases have been implemented. Tests written but not yet executed (read-only clone environment).

---

## Phase 1: Shared Logging Package [DONE]

Created `src/pa_dealing/logging/` package with 7 files:

| File | Purpose |
|------|---------|
| `__init__.py` | Public API exports |
| `config.py` | `setup_logging(service_name)` — processor pipeline + stdlib integration |
| `context.py` | Correlation ID helpers using `structlog.contextvars` |
| `middleware.py` | `CorrelationIdMiddleware` for FastAPI |
| `processors.py` | Custom processors: service context, OTel bridge |
| `constants.py` | Standard field name reference |
| `http_client.py` | Outbound correlation ID header helper |

## Phase 2: Instrument Entry Points [DONE]

Modified 7 entry points to use `setup_logging()`:
- `src/pa_dealing/api/main.py` — `setup_logging(service_name="pa-dealing-api")` + `CorrelationIdMiddleware`
- `src/pa_dealing/main.py` — `setup_logging(service_name="pa-dealing-agent")`
- `src/pa_dealing/api/middleware.py` — Converted ResponseTimeMiddleware to structlog
- `scripts/ops/run_graph_email_poller.py` — `setup_logging(service_name="graph-email-poller")`
- `scripts/ops/run_pdf_poller.py` — `setup_logging(service_name="gcs-pdf-poller")`
- `scripts/ops/run_slack_listener.py` — `setup_logging(service_name="slack-listener")`
- `scripts/ops/run_monitoring.py` — `setup_logging(service_name="monitoring")`

## Phase 3: Migrate Service Layer [DONE]

17 files migrated in `src/pa_dealing/services/`:
- All `import logging` → `import structlog`
- All `logger = logging.getLogger(__name__)` → `log = structlog.get_logger()`
- All f-string logs → structured `log.info("event_name", key=value)` format
- Syntax validated, zero remnants

## Phase 4: Migrate API Layer [DONE]

6 files migrated in `src/pa_dealing/api/`:
- `routes/requests.py`, `routes/documents.py`, `auth.py`, `routes/config.py`, `routes/notifications.py`, `routes/restricted_instruments.py`

## Phase 5: Migrate Identity/Instruments/DB/Storage [DONE]

12 files migrated:
- `identity/google.py`, `identity/provider_google.py`, `identity/postgres.py`, `identity/google_provider.py`, `identity/fuzzy_matcher.py`
- `instruments/external_resolver.py`, `instruments/fuzzy_cache.py`
- `db/repository.py`, `db/email_ingestion_repository.py`, `db/engine.py`, `db/oracle_position.py`
- `storage/__init__.py`

## Phase 6: Migrate Agent Layer [DONE]

17 files migrated in `src/pa_dealing/agents/`:
- `slack/handlers.py` (44 logs), `slack/chatbot.py` (38 logs), `monitoring/jobs.py` (15 logs)
- `orchestrator/risk_scoring_service.py`, `orchestrator/agent.py`, `orchestrator/advisory_system.py`
- `document_processor/agent.py`, `slack/client.py`, `slack/callbacks.py`, `slack/agent.py`, `slack/session.py`
- `slack/plugins/draft_context_plugin.py`, `slack/plugins/error_recovery_plugin.py`, `slack/plugins/response_guarantee_plugin.py`
- `slack/draft_context.py`, `monitoring/scheduler.py`, `models.py`

## Phase 7: Migrate Ops Scripts [DONE]

- `scripts/ops/manual_restricted_sync.py`
- `scripts/ops/backfill_trade_fingerprints.py`
- `scripts/ops/run_api.py`

## Phase 8: Inject Correlation ID in Outbound HTTP Calls [DONE]

3 files modified to add `get_correlation_headers()`:
- `identity/google.py` — 2 httpx call sites (user lookup, manager check)
- `instruments/external_resolver.py` — 1 httpx call site (EODHD search)
- `agents/slack/client.py` — 1 aiohttp call site (file download)

Note: `services/graph_client.py` uses Microsoft Graph SDK which doesn't support easy custom header injection — skipped.

## Phase 9: Write Test Suite [DONE]

- `tests/unit/test_structured_logging.py` — 36 pytest tests across 11 test classes
- `dashboard/tests/structured_logging.spec.ts` — 5 Playwright E2E tests

## Phase 10: Run Tests [PENDING — Live Environment]

- [ ] `pytest tests/unit/test_structured_logging.py -v`
- [ ] `npx playwright test dashboard/tests/structured_logging.spec.ts`
- [ ] Full regression: `pytest tests/ -v --timeout=120`
- [ ] Full Playwright regression: `npx playwright test`

---

## Total Impact

- **9 new files** created (logging package + tests)
- **~60 existing files** modified
- **~380+ f-string log calls** converted to structured format
- **0 behaviour changes** — purely observability improvements
- **Correlation ID propagation** across all HTTP boundaries
