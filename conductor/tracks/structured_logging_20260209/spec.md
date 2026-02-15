# Structured Logging Migration — Specification

## Objective

Migrate the entire codebase from stdlib `logging` with f-string messages to unified `structlog` with structured JSON output in production, human-readable console output in development, correlation ID propagation, and OTel-readiness.

## Problem Statement

- 56 files using stdlib `logging.getLogger(__name__)` with 307+ f-string log calls in `src/`
- No correlation ID propagation across requests
- No consistent output format (plain text mixed across services)
- Two separate logging configurations (stdlib `basicConfig` for API, structlog for orchestrator)
- Logs unsearchable due to variable data embedded in message strings

## Requirements

1. **Unified structlog** across all services and scripts
2. **JSON output in production**, human-readable console in development
3. **Correlation ID middleware** for FastAPI — generates or echoes `X-Correlation-ID`
4. **Correlation ID propagation** on outbound HTTP calls (httpx, aiohttp)
5. **OTel trace bridge** — optional processor that adds `trace_id`/`span_id` when OpenTelemetry is installed
6. **stdlib integration** via `ProcessorFormatter` so third-party logs also output structured JSON
7. **Noisy logger suppression** — uvicorn.access, sqlalchemy.engine, httpx, httpcore, slack_sdk at WARNING
8. **Standard field names** — `request_id`, `user_id`, `username`, `instrument`, `duration_ms`, `error`, `breach_id`, `correlation_id`, `trace_id`, `span_id`
9. **Comprehensive tests** — pytest for logging package, Playwright for correlation ID E2E

## Out of Scope

- `scripts/debug/` and `scripts/db/` (developer utilities)
- OpenTelemetry SDK installation (just the bridge processor)
- Log aggregation infrastructure
- Changes to existing `AuditLogger` system (already uses structlog correctly)
