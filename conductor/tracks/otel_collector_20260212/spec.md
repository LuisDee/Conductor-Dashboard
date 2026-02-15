# Specification: Centralised OTel Log Collection System

## Overview
Implement a centralised observability pipeline using the OpenTelemetry Collector (Contrib) to aggregate, process, and export structured logs from all PA Dealing core services. This system will bridge the gap between ephemeral Docker container logs and long-term storage/analysis in BigQuery/GCS, while preserving critical technical and business correlation context.

## Functional Requirements
- **Automated Collection:** Automatically discover and tail Docker JSON logs from `/var/lib/docker/containers/` for all core services (`api`, `chatbot`, `pollers`, `workers`).
- **Strict Separation:** Ensure no audit logs (which are routed directly to PostgreSQL) are captured or duplicated in this pipeline.
- **Context Preservation:**
    - Map `structlog` levels to standard OTel `SeverityText` and `SeverityNumber`.
    - Extract and preserve `trace_id` in the OTel log record.
    - Retain `correlation_id`, `request_id`, and `trade_reference_id` as searchable attributes.
- **Resource Attribution:** Use the `resourcedetection` processor to attach container labels and `OTEL_SERVICE_NAME` to every log record.
- **Dual Export:**
    - **Live Debugging:** Export all logs to the Collector's `stdout`.
    - **Persistence:** Export logs to JSON line files partitioned by date and service name.

## Non-Functional Requirements
- **Partitioning Strategy:** Files must be stored at `/padealing/log/otel/YYYY-MM-DD/service_name.json`.
- **Rotation Policy:** Implement time-based rotation at midnight (UTC) to facilitate clean daily uploads to GCS/BigQuery.
- **Integrity:** Ensure the pipeline is resilient to container restarts and collector failures (using file checkpoints).

## Acceptance Criteria
- [ ] OTel Collector (Contrib) service is added to `docker-compose.yml`.
- [ ] Application services are instrumented with `OTEL_SERVICE_NAME` environment variables.
- [ ] Collector successfully parses `structlog` JSON fields into OTel attributes.
- [ ] Logs appearing in `docker logs <service>` are visible in the OTel Collector container output.
- [ ] Log files are created in the specified directory structure: `/padealing/log/otel/2026-02-12/pa-dealing-api.json`.
- [ ] `trace_id` and business IDs are present in the final file export.

## Out of Scope
- Modification of the existing SQLAlchemy-based Audit Logging system.
- Direct implementation of GCS upload scripts (this track focuses on local file generation).
- Collection of logs from third-party infra (Nginx, Postgres) unless they emit compatible JSON.
