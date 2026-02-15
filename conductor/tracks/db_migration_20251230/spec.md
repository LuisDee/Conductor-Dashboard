# Specification: Multi-Environment Database Migration

## Overview
This track focuses on migrating the PA Dealing system from a single local database to a multi-environment architecture supporting Development, QA/Pre-prod (hosted on `uk02vddb004.uk.makoglobal.com`), and Production. A key component is migrating application-specific tables (the `pa_dealing` schema/entities) from the existing source while maintaining compatibility with reference data already present in the target environments.

## Functional Requirements
1.  **Multi-Environment Support:**
    *   The application must support distinct configurations for Local Docker, QA, and Production.
    *   Configuration must be managed via environment-specific files (e.g., `.env.dev`, `.env.qa`).
2.  **Database Migration Process:**
    *   Identify all tables belonging to the `pa_dealing` application core (Requests, Approvals, Executions, Breaches, Audit Logs).
    *   Develop a migration strategy to move this data from the source (Oracle/Legacy Postgres) to the new target environments.
    *   Implement logic to handle table existence:
        *   If a table exists and matches the required structure, use it.
        *   If a table exists but is outdated, perform an upgrade/migration.
        *   If a table is missing, create it.
3.  **Reference Data Compatibility:**
    *   The system must continue to use mirrored tables like `oracle_position`, `oracle_employee`, and `oracle_bloomberg` as they exist in the target environments without modification.

## Non-Functional Requirements
1.  **Security:** Ensure Production and QA credentials are never hardcoded and are managed securely via `.env` files.
2.  **Data Integrity:** The migration must preserve relationships between application data and the existing Oracle reference data.
3.  **Stability:** The local development environment (Docker) must remain fully functional using its local Postgres container.

## Acceptance Criteria
1.  The application successfully connects to the QA database on `uk02vddb004` when configured with `.env.qa`.
2.  Application tables (`pad_request`, `pad_approval`, etc.) are correctly created/populated in the QA environment.
3.  The dashboard correctly displays data queried from the remote QA database.
4.  All existing tests pass in the local Docker environment.

## Out of Scope
*   Migration of non-`pa_dealing` tables (e.g., firm-wide position history or raw Oracle mirrors) which are assumed to be managed by other systems.
*   Setup of the actual database servers or network connectivity (assumed to be pre-existing).
