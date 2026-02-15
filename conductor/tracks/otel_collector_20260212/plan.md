# Implementation Plan: Centralised OTel Log Collection System

This plan outlines the steps to implement a centralized logging pipeline using the OpenTelemetry Collector to aggregate Docker container logs, preserve compliance context, and export them for long-term storage.

## Phase 1: Infrastructure & Foundation
Establish the base container environment and configuration structure.

- [ ] Task: Create OTel configuration directory at `config/otel/`
- [ ] Task: Add `otel-collector` service to `docker/docker-compose.yml` using `otelcol-contrib` image
- [ ] Task: Configure Docker volume mounts for the collector:
    - Mount `/var/lib/docker/containers/` (read-only) to access logs
    - Mount `/var/run/docker.sock` for metadata detection
    - Mount `./config/otel/otel-collector.yaml` for configuration
    - Mount `/padealing/log/otel/` for persistent output
- [ ] Task: Conductor - User Manual Verification 'Infrastructure & Foundation' (Protocol in workflow.md)

## Phase 2: Collection & Context Parsing (TDD)
Configure the collector to ingest and transform `structlog` JSON lines.

- [ ] Task: Write a verification script to validate log parsing logic using a sample `structlog` JSON line
- [ ] Task: Implement `filelog` receiver with JSON parsing and `trace_id` extraction
- [ ] Task: Implement `resourcedetection` and `docker_observer` processors for container metadata
- [ ] Task: Implement `transform` processor for severity mapping (e.g., `info` -> `INFO`)
- [ ] Task: Implement attribute preservation for `correlation_id`, `request_id`, and `trade_reference_id`
- [ ] Task: Conductor - User Manual Verification 'Collection & Context Parsing' (Protocol in workflow.md)

## Phase 3: Exporting & Partitioning
Configure the persistence layer with date-based partitioning.

- [ ] Task: Create test case to verify file partitioning logic (`/YYYY-MM-DD/service.json`)
- [ ] Task: Configure `file` exporter with dynamic path: `/padealing/log/otel/%Y-%m-%d/%{service.name}.json`
- [ ] Task: Configure midnight (UTC) rotation strategy in the file exporter
- [ ] Task: Configure `debug` (stdout) exporter for live visibility during development
- [ ] Task: Conductor - User Manual Verification 'Exporting & Partitioning' (Protocol in workflow.md)

## Phase 4: Service Instrumentation & Integration
Apply configuration to the application services.

- [ ] Task: Add `OTEL_SERVICE_NAME` environment variable to all core services in `docker/docker-compose.yml`
- [ ] Task: Ensure all services have appropriate Docker labels for the collector to identify them
- [ ] Task: Perform end-to-end verification:
    - Trigger a trade request via Slack
    - Verify JSON log appears in Collector stdout
    - Verify partitioned file is created in `/padealing/log/otel/` with correct attributes
- [ ] Task: Conductor - User Manual Verification 'Service Instrumentation & Integration' (Protocol in workflow.md)

## Phase 5: Finalisation
- [ ] Task: Audit the generated JSON files to ensure NO audit records from PostgreSQL were leaked into the files
- [ ] Task: Document the OTel pipeline in `docs/tooling/otel-collector.md`
- [ ] Task: Conductor - User Manual Verification 'Finalisation' (Protocol in workflow.md)
