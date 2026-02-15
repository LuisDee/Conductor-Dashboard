# Specification: Environment Configuration & Code Robustness Remediation

## Goal
Fix critical regressions in environment variable handling, model properties, and service synchronization to ensure stable authentication and database schema perception across the entire application stack.

## Problem Statement
The recent migration to the `ldeburna` schema revealed several structural weaknesses:
1.  **Logic Bug:** The `@property` decorator was missing from `is_development` in `Settings`, causing potentially unreliable environment detection.
2.  **Configuration Drift:** `docker-compose.yml` uses `${VAR:-default}` logic which causes containers to silently revert to old defaults (like `bo_airflow`) if the shell environment variable is lost, regardless of what is in the `.env` file.
3.  **Inconsistent Mapping:** New variables like `STATE_SCHEMA` were not mapped across all services in the compose file.
4.  **Application Bugs:** A crash in the `GraphEmailPoller` was discovered during verification due to a schema mismatch in the `MessageInfo` object.

## Objectives
1.  **Restore Code Integrity:** Fix the `Settings` properties and ensure authentication logic uses them robustly.
2.  **Enforce Configuration Sovereignty:** Standardize how Docker Compose loads environment variables to prevent silent reversions to old schemas.
3.  **Synchronize Stack:** Ensure all 9 services use the exact same schema and environment configuration.
4.  **Fix Discovered Bugs:** Address the `AttributeError` in the email poller to restore full functionality.

## Deliverables
- [ ] Fixed `src/pa_dealing/config/settings.py` with correct decorators.
- [ ] Standardized `docker/docker-compose.yml` environment mappings.
- [ ] Fixed `src/pa_dealing/services/graph_email_poller.py` attribute error.
- [ ] Verification report showing correct schema (`ldeburna`) and auth status (`ok`) in the API.
