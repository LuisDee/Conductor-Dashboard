# Specification: Reference Schema Parameterization & ldeburna Migration

## Goal
The goal of this track is to decouple the PA Dealing application from the hardcoded `bo_airflow` schema and enable seamless migration to the `ldeburna` reference schema (or any other reference schema) via configuration.

## Objectives
1.  **Parameterize Schema References:** Remove all hardcoded `bo_airflow` strings from models, SQL fragments, and repositories.
2.  **Schema-Aware SQLAlchemy Models:** Implement a dynamic schema mapping pattern for SQLAlchemy models using the `reference_schema` setting.
3.  **Hybrid Foreign Key Strategy:** Evaluate and implement a "Hybrid" approach to foreign keys (NOT VALID + VALIDATE) to minimize migration friction when switching reference schemas.
4.  **Migration to ldeburna:** Successfully transition the Dev environment to use the `ldeburna` schema as its primary reference data source.
5.  **Schema Validation Tooling:** Create a diagnostic utility to verify column-level parity between schemas to prevent runtime errors.

## Success Criteria
- [ ] Application starts and functions normally with `REFERENCE_SCHEMA=ldeburna`.
- [ ] No hardcoded `bo_airflow` references remain in `src/`.
- [ ] Identity resolution (OracleEmployee/OracleContact) works correctly against `ldeburna`.
- [ ] Instrument lookup (OracleBloomberg) works correctly against `ldeburna`.
- [ ] Database migrations successfully repoint foreign keys from `bo_airflow` to `ldeburna`.
- [ ] CI check exists to prevent new hardcoded schema names.

## Out of Scope
- Modifying the actual data sync process (Airflow).
- Changes to the `padealing` schema structure itself (except for FK repointing).
