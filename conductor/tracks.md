# Project Tracks

This file tracks all major tracks for the project. Each track has its own detailed plan in its respective folder.

## [x] Track: Dashboard UI Overhaul & Role-Based Views ✅ COMPLETE
*Link: [./conductor/tracks/dashboard_overhaul_20260206/](./conductor/tracks/dashboard_overhaul_20260206/)*
**Priority**: High
**Tags**: frontend, ui, dashboard, role-based-access, activity-feed, trends
**Status**: Completed (2026-02-06)
**Branch**: feat/dashboard-overhaul

Major UI/UX overhaul of the PA Dealing Dashboard. Restructures the sidebar based on user roles, implements an "ALL / MY" data filter for compliance users, replaces Quick Actions with an Activity Feed, and adds trend indicators to summary cards.

**Key Deliverables**:
- ✅ Role-based sidebar sections ("MY PA DEALING" vs "OPERATIONS")
- ✅ "ALL / MY" data filter toggle for compliance users
- ✅ Real-time Dashboard Activity Feed (last 10 actions)
- ✅ Summary card trend indicators (e.g., "↑3 from yesterday")
- ✅ Removal of Quick Actions section
- ✅ Updated backend support for activity tracking and trend calculation

---

## [x] Track: PAD Search (Conflict View)
*Link: [./conductor/tracks/pad_search_conflict_view_20260202/](./conductor/tracks/pad_search_conflict_view_20260202/)*
**Priority**: High
**Tags**: frontend, backend, compliance, search, conflict-detection
**Status**: Completed
**Branch**: DSS-4074

Replicate the legacy "Conflict Search" functionality - a side-by-side view of Mako institutional trading vs Employee PA trades for identifying potential conflicts of interest.

**Key Deliverables**:
- ✅ PAD Search as new top-level navigation item
- ✅ Split-view layout with resizable divider
- ✅ Left Panel: Mako Trading (3-tier symbol lookup from Bloomberg → MapInstSymbol → Product, data from ProductUsage)
- ✅ Right Panel: PA Account Trading (approved `pad_request` records with employee/division joins)
- ✅ Unified search bar with debounced input
- ✅ 30-day "Risk Zone" highlighting (bold gold text for recent trades)
- ✅ Parallel API calls for both panels
- ✅ Comprehensive Unit & Integration Tests (Verified)
- ✅ Logic Verification against legacy requirements
- ✅ Bug fixes and UI refinements

**Data Sources**:
- `oracle_bloomberg`, `oracle_map_inst_symbol`, `oracle_product` (symbol lookup)
- `oracle_product_usage` (Mako trading data)
- `pad_request`, `oracle_employee`, `oracle_division` (PA trades)

---

## [x] Track: MAKO Design System Implementation ✅ COMPLETE
*Link: [./conductor/tracks/mako_colour_palette_20260126/](./conductor/tracks/mako_colour_palette_20260126/)*
**Priority**: High
**Tags**: frontend, ui, design-system, branding
**Status**: Complete (2026-01-29)

Transform the entire PA Dealing Dashboard to use MAKO's official corporate design system including color palette, typography, spacing, and component styling.

**Key Deliverables**:
- MAKO color palette integration (Navy #0E1E3F, Blue #5471DF, Gold #B28C54)
- Montserrat font family implementation
- Mako logo in sidebar with "PA DEALING" and "Compliance Suite" branding
- Update all 11+ pages (Dashboard, Approvals, Requests, Breaches, etc.)
- Replace Tailwind generic colors with MAKO design tokens
- Navy-based shadows (not black)
- Semantic status colors (Gold for warnings, not red)
- 220px sidebar, precise spacing/radius values

**Reference**: MAKO_UI_SYSTEM_PROMPT.md
**Timeline**: 5-6 days

---

## [x] Track: GCS PDF Polling & Ingestion System ✅ COMPLETE
*Link: [./conductor/tracks/gcs_pdf_ingestion_20260129/](./conductor/tracks/gcs_pdf_ingestion_20260129/)*
**Priority**: High
**Tags**: gcs, pdf, ingestion, automation, contract-notes
**Status**: Complete (2026-01-29)

Production-ready system for automated polling of PDFs from Google Cloud Storage, processing through AI parser, and storing results with full end-to-end traceability.

**Key Deliverables**:
- ✅ `gcs_document`, `parsed_trade`, `document_processing_log` tables with Alembic migration
- ✅ GCS client with blob operations (list, move, archive, signed URLs)
- ✅ PDF poller service with deduplication via `gcs_generation`
- ✅ Docker Compose integration (`pdf-poller` service)
- ✅ API endpoints: document list, stats, signed URLs, retry
- ✅ Health check extended to include GCS connectivity
- ✅ Unit tests (poller, GCS client) + integration tests (13 tests)
- ✅ Tested with real contract notes in dev bucket

**Configuration**:
- Bucket: `cmek-encrypted-bucket-europe-west2-roe18`
- Prefixes: `contract_notes/{incoming,processing,archive,failed}/`
- ADC for dev, Workload Identity for prod (no code changes needed)

**Next**: User matching moved to separate track (`contract_note_matching_20260129`)

---

## [x] Track: Contract Note User Matching & Verification ✅ COMPLETE
*Link: [./conductor/tracks/contract_note_matching_20260129/](./conductor/tracks/contract_note_matching_20260129/)*
**Priority**: High
**Tags**: pdf-extraction, identity-matching, compliance, contract-notes, instructor, activity-statements
**Status**: Complete (2026-02-02)
**Dependencies**: `gcs_pdf_ingestion_20260129` (COMPLETE)

Match incoming Contract Notes AND Activity Statements to employees with approved PAD requests. Supports Interactive Brokers monthly statements with "Trades" section detection.

**Key Deliverables**:
- Instructor + LiteLLM integration for self-healing extraction with `max_retries=3`
- Enhanced ExtractedTradeData schema with rich field descriptions
- Document type classification (CONTRACT_NOTE vs ACTIVITY_STATEMENT vs OTHER)
- Two-pass extraction for Activity Statements (section identification → targeted extraction)
- Fuzzy name matching against candidate pool (approved requests only)
- Confidence-based routing (HIGH → auto, MEDIUM → audit, LOW → manual review)
- All identifier extraction (ticker, ISIN, SEDOL, CUSIP, Bloomberg)

**Implementation Phases**:
- Phase 0: Add instructor, refactor DocumentAgent
- Phase 1: Enhanced schema with validators
- Phase 2: Document classification & two-pass extraction
- Phase 3: Alembic migration for matching fields
- Phase 4: User matching (email → name → manual)
- Phase 5: Confidence routing & review triggers
- Phase 6: Comprehensive tests

---

## [x] Track: Dev Database Migration & Validation ✅ COMPLETE
*Link: [./conductor/tracks/dev_db_migration_validation_20260120/](./conductor/tracks/dev_db_migration_validation_20260120/)*
**Status**: Complete (2026-01-24)

---

## [x] Track: Authorization Failure UI Indicator ✅ COMPLETE
*Link: [./conductor/tracks/auth_failure_ui_20260124/](./conductor/tracks/auth_failure_ui_20260124/)*
**Status**: Complete (2026-01-24)

---

## [x] Track: Multi-Environment Database Migration ✅ COMPLETE
*Link: [./conductor/tracks/db_migration_20251230/](./conductor/tracks/db_migration_20251230/)*
**Status**: Complete (2026-01-24)

---

## [ ] Track: Legacy PAD Field Review & Integration
*Link: [./conductor/tracks/legacy_field_review_20260120/](./conductor/tracks/legacy_field_review_20260120/)*
**Dependencies**: Multi-Environment Database Migration

---

## [x] Track: Google Identity Integration ✅ COMPLETE
*Link: [./conductor/tracks/google_identity_integration_20260119/](./conductor/tracks/google_identity_integration_20260119/)*
**Status**: Complete (2026-01-25)

---

## [x] Track: Advanced Instrument Validation (3-Tier Lookup) ✅ COMPLETE
*Link: [./conductor/tracks/advanced_instrument_validation_20260115/](./conductor/tracks/advanced_instrument_validation_20260115/)*
**Status**: Complete (2026-01-25)

---

## [x] Track: Firm Trading Conflict Detection & Enrichment ✅ COMPLETE
*Link: [./conductor/tracks/firm_trading_conflict_detection_20260120/](./conductor/tracks/firm_trading_conflict_detection_20260120/)*
**Priority**: High
**Tags**: conflict-detection, product-usage, risk-engine, oracle
**Status**: Complete (2026-02-03)

Implement automated conflict-of-interest detection by analyzing Mako's firm trading activity against employee personal trading requests. Enriches the risk assessment process with firm trading context and enables side-by-side comparison in the dashboard.

**Key Deliverables**:
- ✅ Integrated `ProductUsage`, `OraclePortfolioGroup`, and `OraclePosition` data sources
- ✅ Implemented 3-tier instrument resolution waterfall
- ✅ Conflict detection logic in repository layer with full desk name resolution
- ✅ Automated conflict flagging in Orchestrator and Advisory System
- ✅ Side-by-side "Conflict Search" view in Dashboard
- ✅ Integration tests for position lookup and conflict risk calculation

---

## [x] Track: Local Database Schema Consolidation ✅ COMPLETE
*Link: [./conductor/tracks/schema_consolidation_20260121/](./conductor/tracks/schema_consolidation_20260121/)*
**Status**: Complete (2026-01-21)

---

## [x] Track: Schema Validation Against Dev Database ✅ COMPLETE
*Link: [./conductor/tracks/schema_validation_dev_20260121/](./conductor/tracks/schema_validation_dev_20260121/)*
**Status**: Complete (2026-01-24)

---

## [ ] Track: Slack UI Dashboard Links & Cleanup
*Link: [./conductor/tracks/slack_ui_dashboard_links_20260125/](./conductor/tracks/slack_ui_dashboard_links_20260125/)*
**Priority**: High
**Tags**: slack, ui, dashboard, tdd, playwright, auth, identity, notification
**Status**: Complete (2026-02-12) (Phases 5-7 outstanding)

---

## [x] Track: Chatbot Text Response Handling ✅ COMPLETE
*Link: [./conductor/tracks/chatbot_text_responses_20260125/](./conductor/tracks/chatbot_text_responses_20260125/)*
**Priority**: Medium
**Tags**: chatbot, adk, ux, text-responses
**Status**: Complete (2026-02-03)

Enable deterministic text parsing for yes/no questions in the chatbot (Derivative/Leveraged/Confirmation), allowing users to type responses naturally instead of only clicking buttons.

**Key Deliverables**:
- ✅ Deterministic `parse_yes_no_response` helper with multi-variant support (yes, y, yep, nope, etc.)
- ✅ Interception logic in `process_message` for all compliance questions
- ✅ Integration of Derivative + Non-Leveraged "Are you sure?" confirmation flow for text
- ✅ Enhanced derivative justification capture with flow continuation
- ✅ Robust unit tests covering all yes/no text permutations and edge cases
- ✅ Ambiguous response handling with helpful user guidance

---

## [x] Track: Instrument Matching Overhaul & Consistency Fix ✅ COMPLETE
*Link: [./conductor/tracks/instrument_matching_overhaul_20260129/](./conductor/tracks/instrument_matching_overhaul_20260129/)*
**Priority**: High
**Tags**: instrument-matching, consistency, scoring, disambiguation, isin-passthrough
**Status**: Complete (2026-01-29)

## [x] Track: Confluence Integration for Restricted Instruments List ✅ COMPLETE
*Link: [./conductor/tracks/confluence_restricted_list_20260129/](./conductor/tracks/confluence_restricted_list_20260129/)*
**Priority**: High
**Tags**: confluence, integration, restricted-list, compliance, sync
**Status**: Complete (2026-01-30)

---

## [x] Track: Authoritative Identity & Sequential Guardrail Refactor ✅ COMPLETE
*Link: [./conductor/tracks/authoritative_identity_refactor_20260130/](./conductor/tracks/authoritative_identity_refactor_20260130/)*
**Priority**: Critical
**Tags**: architecture, security, bugfix, backend, tdd
**Status**: Complete (2026-02-01)

Unify the system's identity resolution and risk assessment logic. Transition from an ambiguous ticker-based model to an inst_symbol anchor (internal) and isin (regulatory) model. Implement a Sequential Pipeline where policy violations discovered by the Advisory System act as a physical circuit breaker for status transitions.

**Key Deliverables**:
- Alembic migration to add inst_symbol and drop ticker
- Tiered identity resolution (Bloomberg -> Mappings -> Product)
- Categorical Risk Scorer implementation (1 High / 2 Medium rules)
- Orchestrator veto gate for Advisory System violations
- Comprehensive TDD coverage

---

## [x] Track: Slack Transactional Outbox Standardization ✅ COMPLETE
*Link: [./conductor/tracks/slack_outbox_standardization_20260201/](./conductor/tracks/slack_outbox_standardization_20260201/)*
**Priority**: High
**Tags**: architecture, slack, reliability, outbox, refactoring
**Status**: Complete (2026-02-01)

Standardize all Slack notifications to use the Transactional Outbox pattern consistently across the codebase. Refactor SlackAgent and MonitoringService to eliminate direct "fire-and-forget" API calls and ensure guaranteed delivery for all compliance alerts.

**Key Deliverables**:
- ✅ Refactored SlackAgent with AsyncSession support (@requires_session)
- ✅ Outbox-integrated MonitoringService alerts
- ✅ Atomic outbox population in SlackSocketHandler
- ✅ Verified worker health and retry logic
- ✅ Created slack-outbox architectural skill
- ✅ Implemented 3-tier Manager Resolution Fallback flow
- ✅ Schema contract tests for DB-to-Pydantic consistency

---

## [x] Track: Restricted Instruments UI & Sync Controls ✅ COMPLETE
*Link: [./conductor/tracks/restricted_instruments_ui_20260202/](./conductor/tracks/restricted_instruments_ui_20260202/)*
**Priority**: Medium
**Tags**: frontend, ui, restricted-list, confluence, sync
**Status**: Complete (2026-02-02)

Enhance the dashboard to display the restricted instruments list and provide controls for sync management. Add a restricted instruments table to the Mako Conflicts page, a "Sync Now" button, and configurable sync frequency in Settings.

**Key Deliverables**:
- ✅ Backend: GET /dashboard/restricted-instruments
- ✅ Backend: POST /api/config/restricted-list-sync
- ✅ Backend: PUT /api/config/restricted-list-sync-interval
- ✅ Backend: Dynamic scheduler reloading
- ✅ Frontend: SyncStatusCard component
- ✅ Frontend: RestrictedInstrumentsSection component
- ✅ Frontend: MakoConflicts integration
- ✅ Frontend: Settings page sync controls
- ✅ Integration tests for all new endpoints

---

## [x] Track: Align Confluence Restricted List to inst_symbol ✅ COMPLETE
*Link: [./conductor/tracks/align_confluence_restricted_list_to_inst_symbol_20260203/](./conductor/tracks/align_confluence_restricted_list_to_inst_symbol_20260203/)*
**Priority**: High
**Tags**: confluence, integration, restricted-list, refactor
**Status**: Complete (2026-02-03)
**Branch**: fix/confluence-restricted-list-alignment

Align the restricted instruments sync workflow with the `inst_symbol` authoritative identity anchor across Confluence and the codebase.

**Key Deliverables**:
- ✅ Rename `ticker` to `inst_symbol` in Confluence page table headers
- ✅ Update `ConfluenceClient` parsing logic with fallback support
- ✅ Align `RestrictedListSyncService` with new keys
- ✅ Refactor unit and integration tests to use `inst_symbol`
- ✅ Update `ModelFactory` to match current DB schema
- ✅ Verified sync end-to-end (5 instruments processed)

---

---

## [x] Track: Contract Note Trap Verification ✅ COMPLETE
*Link: [./conductor/tracks/contract_note_trap_verification_20260202/](./conductor/tracks/contract_note_trap_verification_20260202/)*
**Priority**: High
**Tags**: pdf-extraction, testing, edge-cases, document-agent, router
**Status**: Complete (2026-02-02)

Verify the robustness of the DocumentAgent and ExtractionRouter against 5 known financial document "traps" (Holdings Mirror, Entity Ambiguity, Pence vs Pounds, Date Ambiguity, and Cancelled Trades) using real PDF assets.

**Key Deliverables**:
- ✅ Automated integration tests for the 5 specific PDF edge cases
- ✅ Enhanced `ExtractedTradeData` schema with cancellation flags and fingerprinting
- ✅ Refined extraction prompts for currency and date normalization
- ✅ Verified confidence-based routing for ambiguous and cancelled trades
- ✅ Ingestion-first poller refactor for database persistence



## [x] Track: Advanced Slack Integration ✅ COMPLETE
*Link: [./conductor/tracks/advanced_slack_integration_20251223/](./conductor/tracks/advanced_slack_integration_20251223/)*
**Priority**: Medium
**Tags**: implementation
**Status**: Complete

Advanced Slack integration features including App Home, Slash Commands, and a full Slack Mock Server for E2E testing.

---

## [x] Track: Chatbot Architecture Hardening ✅ COMPLETE
*Link: [./conductor/tracks/chatbot_architecture_hardening_20251231/](./conductor/tracks/chatbot_architecture_hardening_20251231/)*
**Priority**: Medium
**Tags**: implementation
**Status**: Complete

Refactor Slack Chatbot to use server-side session state and code-driven UI, eliminating hallucinations.

---

## [~] Track: Chatbot Robustness Hardening
*Link: [./conductor/tracks/chatbot_robustness_hardening_20260129/](./conductor/tracks/chatbot_robustness_hardening_20260129/)*
**Priority**: High
**Tags**: chatbot, robustness, tdd, conversation-flow
**Status**: Complete (2026-02-12)

---

## [x] Track: Chatbot Tool Refactor - Split Monolithic update_draft ✅ COMPLETE
*Link: [./conductor/tracks/chatbot_tool_refactor_20260128/](./conductor/tracks/chatbot_tool_refactor_20260128/)*
**Priority**: Medium
**Tags**: chatbot, adk, tools, refactor
**Status**: Complete

---

## [x] Track: Compliance Workflow Enhancements ✅ COMPLETE
*Link: [./conductor/tracks/compliance_enhancements_20260127/](./conductor/tracks/compliance_enhancements_20260127/)*
**Priority**: High
**Tags**: compliance, auto-approval, smf16
**Status**: Not_started

---

## [x] Track: Contract Note Ingestion Pipeline ✅ COMPLETE
*Link: [./conductor/tracks/contract_note_ingestion_20260127/](./conductor/tracks/contract_note_ingestion_20260127/)*
**Priority**: High
**Tags**: contract-note, upload, compliance, storage
**Status**: Complete

Streamline contract note ingestion pipeline: multi-upload history, abstracted file storage, detailed compliance mismatch messaging, and visual hierarchy redesign

---

## [x] Track: Conversational Session Hardening ✅ COMPLETE
*Link: [./conductor/tracks/conversational_session_hardening_20260104/](./conductor/tracks/conversational_session_hardening_20260104/)*
**Priority**: Medium
**Tags**: implementation
**Status**: Complete

---

## [x] Track: E2E Testing Overhaul ✅ COMPLETE
*Link: [./conductor/tracks/e2e_testing_overhaul_20251224/](./conductor/tracks/e2e_testing_overhaul_20251224/)*
**Priority**: Medium
**Tags**: implementation
**Status**: Complete

Critical analysis and overhaul of E2E testing infrastructure across Backend, Slack Mock, and Compliance Dashboard.

---

## [x] Track: Detailed End to End Testing and Initial UAT ✅ COMPLETE
*Link: [./conductor/tracks/e2e_uat_audit_20251229/](./conductor/tracks/e2e_uat_audit_20251229/)*
**Priority**: High
**Tags**: e2e, uat, testing
**Status**: Complete

Detailed end to end testing and initial UAT

---

## [x] Track: Easy Test Fixes ✅ COMPLETE
*Link: [./conductor/tracks/easy_test_fixes_20260121/](./conductor/tracks/easy_test_fixes_20260121/)*
**Priority**: Medium
**Tags**: tests, bugfix
**Status**: Complete

---

## [x] Track: entra integration_20251230 ✅ COMPLETE
*Link: [./conductor/tracks/entra_integration_20251230/](./conductor/tracks/entra_integration_20251230/)*
**Priority**: Low
**Tags**: backlog
**Status**: Not Moving Forward (Superseded by Google Identity Integration)

Integration with Azure Entra ID and Microsoft Graph for dynamic identity discovery and reporting hierarchy. This track is deprecated as the system has successfully moved to Google Workspace API for all identity and manager resolution needs.

---

## [x] Track: Generic Bug Fixes (Batch 1) ✅ COMPLETE
*Link: [./conductor/tracks/generic_bugfixes_20260127/](./conductor/tracks/generic_bugfixes_20260127/)*
**Priority**: High
**Tags**: bugs, risk-scoring, derivatives
**Status**: Complete

Batch of 6 bug fixes: score_pad_request syntax error, expired approval breach, Rights derivative classification, derivative+non-leveraged confirmation, table padding, currency dropdown

---

## [x] Track: Google Email Source of Truth ✅ COMPLETE
*Link: [./conductor/tracks/google_email_source_of_truth_20260124/](./conductor/tracks/google_email_source_of_truth_20260124/)*
**Priority**: High
**Tags**: identity, google, email
**Status**: Complete

---

## [x] Track: LiteLLM Proxy Migration ✅ COMPLETE
*Link: [./conductor/tracks/litellm_proxy_migration_20260106/](./conductor/tracks/litellm_proxy_migration_20260106/)*
**Priority**: High
**Tags**: litellm, proxy, migration
**Status**: Complete

---

## [ ] Track: Interactive Manual UAT Walkthrough
*Link: [./conductor/tracks/manual_uat_walkthrough_20251231/](./conductor/tracks/manual_uat_walkthrough_20251231/)*
**Priority**: Low
**Tags**: uat, testing
**Status**: Not_started

Interactive walkthrough of MANUAL_TESTING_SCRIPT.md to verify end-to-end functionality in Real Slack Mode.

---

## [x] Track: Slack Notification Bugs & Dashboard Count Fix ✅ COMPLETE
*Link: [./conductor/tracks/notification_bugs_20260126/](./conductor/tracks/notification_bugs_20260126/)*
**Priority**: High
**Tags**: slack, notifications, bugfix, dashboard
**Status**: Complete

---

## [x] Track: Notification Reliability & Silent Failure Prevention ✅ COMPLETE
*Link: [./conductor/tracks/notification_reliability_20260128/](./conductor/tracks/notification_reliability_20260128/)*
**Priority**: Critical
**Tags**: reliability, notifications, slack, outbox-pattern
**Status**: Complete

---

## [ ] Track: PAD Policy Version Monitoring
*Link: [./conductor/tracks/pad_policy_version_monitoring_20260129/](./conductor/tracks/pad_policy_version_monitoring_20260129/)*
**Priority**: Medium
**Tags**: confluence, compliance, dashboard, monitoring
**Status**: Planning

---

## [x] Track: Risk Fixes & Derivative Justification ✅ COMPLETE
*Link: [./conductor/tracks/risk_fixes_derivative_justification_20260127/](./conductor/tracks/risk_fixes_derivative_justification_20260127/)*
**Priority**: High
**Tags**: risk-scoring, derivatives, chatbot, slack
**Status**: Complete

Fix risk assessment bugs (derivative/leveraged auto-approval, question skipping), add derivative justification field, update scoring thresholds, add holding period risk factor

---

## [x] Track: Risk Scoring Overhaul & Oracle Position Enrichment ✅ COMPLETE
*Link: [./conductor/tracks/risk_scoring_overhaul_20260126/](./conductor/tracks/risk_scoring_overhaul_20260126/)*
**Priority**: High
**Tags**: risk-scoring, oracle, compliance
**Status**: Not_started

Risk Scoring Overhaul & Oracle Position Enrichment - Simplify scoring to 6 factors, add advisory warnings, Oracle DB integration for Mako position data

---

## [x] Track: Security Confirmation UX & Position Lookup ✅ COMPLETE
*Link: [./conductor/tracks/security_confirmation_ux_and_position_lookup_20260122/](./conductor/tracks/security_confirmation_ux_and_position_lookup_20260122/)*
**Priority**: High
**Tags**: ux, slack, position-lookup
**Status**: Complete

Fix UAT issues: database error blocking trade submission, improve symbol extraction, simplify confirmation UX, add position lookup & conflict detection, fix dashboard startup

---

## [x] Track: Spec Compliance Gaps ✅ COMPLETE
*Link: [./conductor/tracks/spec_compliance_gaps_20251224/](./conductor/tracks/spec_compliance_gaps_20251224/)*
**Priority**: High
**Tags**: compliance, spec
**Status**: Complete

---

## [x] Track: Stale Test Fixes ✅ COMPLETE
*Link: [./conductor/tracks/stale_test_fixes_20260128/](./conductor/tracks/stale_test_fixes_20260128/)*
**Priority**: High
**Tags**: tests, maintenance, slack-ui
**Status**: Complete

---

## [x] Track: Project Structure Cleanup & Documentation Overhaul ✅ COMPLETE
*Link: [./conductor/tracks/structure_overhaul_20251230/](./conductor/tracks/structure_overhaul_20251230/)*
**Priority**: Medium
**Tags**: documentation, structure
**Status**: Complete

Deep dive into project structure, cleanup, reorganization, and documentation overhaul.

---

## [x] Track: Submission Notification Fix ✅ COMPLETE
*Link: [./conductor/tracks/submission_notification_fix_20260126/](./conductor/tracks/submission_notification_fix_20260126/)*
**Priority**: High
**Tags**: slack, notifications, bugfix
**Status**: Complete

---

## [x] Track: System Health UI Refactor ✅ COMPLETE
*Link: [./conductor/tracks/system_health_ui_refactor_20260201/](./conductor/tracks/system_health_ui_refactor_20260201/)*
**Priority**: Medium
**Status**: Complete (2026-02-02)

Standardize the System Health page using the MAKO Design System. Refactor the layout to group operational metrics and infrastructure status panels.

**Key Deliverables**:
- ✅ MAKO Navy/Gold/Blue palette integration
- ✅ Montserrat typography and weighted headers
- ✅ Grouped stats cards for Outbox and Health
- ✅ Infrastructure status placeholder panels
- ✅ Standardized Table.tsx integration

---

## [ ] Track: Bug Fixes
*Link: [./conductor/tracks/test_failures_20260126/](./conductor/tracks/test_failures_20260126/)*
**Priority**: Medium
**Tags**: testing, bugfix, cleanup
**Status**: Complete (2026-02-12)

---

## [x] Track: UX GUI Overhaul ✅ COMPLETE
*Link: [./conductor/tracks/ux_gui_overhaul_20251223/](./conductor/tracks/ux_gui_overhaul_20251223/)*
**Priority**: Medium
**Tags**: implementation
**Status**: Complete

UX/GUI Overhaul: User-Centric Workflow Optimization

---

## [x] Track: UI Overhaul & Bug Fixes ✅ COMPLETE
*Link: [./conductor/tracks/ui_overhaul_bug_fixes_20260203/](./conductor/tracks/ui_overhaul_bug_fixes_20260203/)*
**Priority**: High
**Tags**: bugfix, frontend, backend, compliance, search
**Status**: Complete (2026-02-03)
**Branch**: DSS-4074

UI bugs, search logic issues, and UX improvements discovered during exploratory testing. 5 phases: PAD Search status fix + soft-delete, PAD Search UI tightening + dynamic lookback, Holding Periods overhaul, Mako Conflicts count/position/type fixes + PAD Search filter system, Restricted Instruments standalone page (Confluence deprecated).

**Key Deliverables**:
- ✅ PAD Search: status filter fix (approved+executed), soft-delete exclusion, dynamic conflict window (mako_lookback_days)
- ✅ Global theme: 4px border radius, compact padding across all pages and components
- ✅ Holding Periods: dynamic period, removed stat blocks, split columns
- ✅ Mako Conflicts: fixed count query, Long/Short position display, dynamic conflict types (parallel/opposite/restricted)
- ✅ PAD Search: double-click filter system with URL param sync and filter chips
- ✅ Restricted Instruments: deprecated Confluence sync, new standalone CRUD page with audit trail
- ✅ Regression tests: 59 backend unit tests + 28 Playwright UI tests

---

## [x] Track: API & Frontend Performance Optimization ✅ COMPLETE
*Link: [./conductor/tracks/performance_optimization_20260203/](./conductor/tracks/performance_optimization_20260203/)*
**Priority**: High
**Tags**: performance, api, frontend, caching, sql-optimization
**Status**: Complete (2026-02-03)

Address severe dashboard performance issues (5-10s page loads) by optimizing the authentication flow, eliminating N+1 queries, parallelizing frontend fetches, and adding missing database indexes.

**Key Deliverables**:
- ✅ Implement Auth Caching (Google API & Manager status)
- ✅ Optimize Identity Resolution (Tiered fallback matching)
- ✅ Fix N+1 queries in `/documents` and summary endpoints
- ✅ Frontend Optimization (Unify auth keys, parallelize waterfalls, prefetching)
- ✅ Database Indexing & Connection Pooling
- ✅ Verify targets: /auth/me <200ms, Dashboard Load <1s

---
*Link: [./conductor/tracks/minimal_context_test_execution_20260203/](./conductor/tracks/minimal_context_test_execution_20260203/)*
**Priority**: High
**Tags**: testing, tooling, token-efficiency
**Status**: Complete (2026-02-03)

Implement a test runner that redirects output to log files and returns only minimal failure context to prevent agent context bloat and optimize token usage.

**Key Deliverables**:
- ✅ Synchronous test runner with failure extraction (`scripts/test-runner.sh`)
- ✅ Background test execution and status polling (`scripts/test-bg.sh`, `scripts/test-status.sh`)
- ✅ Integrated Gemini Skill `test-runner`
- ✅ Specialized extraction logic for Pytest, Playwright, and Linting
- ✅ Full logging at `/tmp/agent_tests/`
- ✅ Updated `GEMINI.md` with critical test mandates

---
*Link: [./conductor/tracks/instrument_identity_refactor_20260203/](./conductor/tracks/instrument_identity_refactor_20260203/)*
**Priority**: High
**Tags**: refactor, database, api-design, architectural-consistency
**Status**: Complete (2026-02-03)

Rename and restructure the instrument lookup system to follow the Pythonic 'Search vs Resolve' pattern. Distinguish between human-facing fuzzy searches (returning ranked lists) and system-facing authoritative resolution (returning single records).

**Key Deliverables**:
- ✅ Rename `lookup_instrument` to `_search_instruments` (private)
- ✅ Implement `search_instruments` (public wrapper)
- ✅ Rename `lookup_instrument_comprehensive` to `resolve_instrument_identity`
- ✅ Update all call sites (Agents, Services, API, UI)
- ✅ Update `tests/test_instrument_lookup.py`
- ✅ Document as a Gemini Skill and update `docs/tooling/instrument-lookup.md`

---

## [x] Track: Dashboard Summary Latency Fix ✅ COMPLETE
*Link: [./conductor/tracks/dashboard_summary_latency_20260204/](./conductor/tracks/dashboard_summary_latency_20260204/)*
**Priority**: High
**Tags**: performance, backend, caching, sql-optimization
**Status**: Complete (2026-02-04)

Resolved dashboard performance issues by implementing parallel execution for summary counts, adding expression indexes for symbol lookups, and introducing a tiered caching strategy (Identity cache + Summary cache).

**Key Deliverables**:
- ✅ Parallel execution of summary queries via `asyncio.gather`
- ✅ Expression indexes on `oracle_bloomberg` and `oracle_position`
- ✅ Tiered caching for Google Identity and Dashboard Summary counts
- ✅ Verified 50% reduction in dashboard load times
- ✅ 870+ tests passing post-refactor

---

## [x] Track: Fix PA Search Division Lookup ✅ COMPLETE
*Link: [./conductor/tracks/pad_search_division_fix_20260204/](./conductor/tracks/pad_search_division_fix_20260204/)*
**Priority**: High
**Tags**: bugfix, backend, pad-search, data-integrity, cross-panel-filter
**Status**: Complete (2026-02-05)
**Branch**: DSS-4074

Fix PA Search to use the correct division lookup chain (`oracle_employee.division_id → oracle_department.name`)
instead of the incorrect `COALESCE(boffin_group, cost_centre)` which returns semicolon-delimited group strings.
Also added cross-panel filtering for Symbol and Description columns.

**Key Deliverables**:
- ✅ Division lookup via `oracle_department.name` with fallback chain
- ✅ Cross-panel Symbol/Description filters (clicking in one panel dims both panels)

---

## [x] Track: Email Ingestion via Microsoft Graph Polling ✅ COMPLETE
*Link: [./conductor/tracks/email_ingestion_graph_webhook_20260204/](./conductor/tracks/email_ingestion_graph_webhook_20260204/)*
**Priority**: High
**Tags**: email, graph-api, polling, contract-notes, pdf-ingestion
**Status**: Complete (2026-02-09)
**Branch**: DSS-4074

Implemented real-time contract note ingestion via Microsoft Graph API. Pivoted from Webhooks to a more robust Polling strategy to ensure reliability and bypass public URL connectivity constraints.

**Key Deliverables**:
- ✅ `GraphEmailPoller` for deterministic message retrieval
- ✅ Graph API client with delta-sync and lookback support
- ✅ Exactly-once processing via `email_ingestion_state` tracking
- ✅ Direct PDF extraction via `DocumentAgent` + `ExtractionRouter`
- ✅ GCS archival with hive-partitioned storage paths
- ✅ Comprehensive TDD suite (218 tests passing)
- ✅ Standardized logging and monitoring integration

**Architecture**: Active polling worker (primary) with delta-link state persistence. Webhook infrastructure remains in code but is deprecated in favor of polling.

---

## [x] Track: Fuzzy Instrument Matching for Typo Detection ✅ COMPLETE
*Link: [./conductor/tracks/fuzzy_instrument_matching_20260205/](./conductor/tracks/fuzzy_instrument_matching_20260205/)*
**Priority**: High
**Tags**: compliance, instrument-lookup, fuzzy-matching, chatbot, rapidfuzz
**Status**: Complete (2026-02-05)
**Branch**: DSS-4074

Add fuzzy matching fallback to the 3-tier instrument lookup system to catch typos and prevent compliance violations from false "not found" responses.

**Problem**: User types "Vodafon3" → LIKE query returns nothing → System says "not found" → Auto-approval could trigger for a security that IS traded → Compliance risk.

**Solution**: In-memory fuzzy cache using `rapidfuzz` (already in dependencies):
- Load 15k instruments at startup in `pad_api` and `pad_slack` containers
- Fuzzy fallback only triggers when existing 3-tier DB lookup returns zero results
- 24-hour TTL with stale-while-revalidate refresh pattern
- No Redis needed (~10MB memory per container is acceptable)

**Key Deliverables**:
- ✅ `src/pa_dealing/instruments/fuzzy_cache.py` module (326 lines)
- ✅ Repository integration with `match_type` field ("exact", "fuzzy", "verified_not_found")
- ✅ Startup integration for pad_api and pad_slack containers
- ✅ TDD test suite with 32 tests (88% coverage)
- ✅ Documentation updates (tooling docs + Gemini skill)

**Success Criteria**:
- ✅ "Vodafon3" → finds "Vodafone" as fuzzy match
- ✅ "APPL" → finds "AAPL" as fuzzy match
- ✅ Exact matches still work via DB (no regression)
- ✅ Cache loads <5s, fuzzy search <50ms

---

## [x] Track: Mock PDF Generator for Parser Testing ✅ COMPLETE
*Link: [./conductor/tracks/mock_pdf_generator_20260205/](./conductor/tracks/mock_pdf_generator_20260205/)*
**Priority**: High
**Tags**: testing, pdf-generation, tooling, parser-validation, standalone
**Status**: Complete (2026-02-05)
**Branch**: DSS-4074

Standalone CLI tool to generate realistic broker PDFs (Activity Statements, Contract Notes, Trade Confirmations) from user-specified JSON trade data. For end-to-end testing of the document extraction pipeline.

**Key Deliverables**:
- ✅ Jinja2 + WeasyPrint template engine
- ✅ 5 broker skins: IB Activity, IB Confirmation, UK Contract Note, Fidelity, Indian
- ✅ Fee calculation engine (US/UK/India jurisdiction rules)
- ✅ CLI: `python -m tools.mock_pdf_generator.generate --input trades.json --skin ib_activity`
- ✅ Ground truth JSON output for parser validation
- ✅ 38 unit tests, 4 sample JSON files, README documentation

---

## [x] Track: External Instrument Resolution Layer ✅ COMPLETE
*Link: [./conductor/tracks/external_instrument_resolution_20260206/](./conductor/tracks/external_instrument_resolution_20260206/)*
**Priority**: High
**Tags**: backend, compliance, instrument-lookup, external-api, tdd
**Status**: Completed (2026-02-06)
**Branch**: feat/external-instrument-resolution

Add an external instrument resolution layer (EODHD/OpenFIGI) to the PA Dealing instrument lookup pipeline to catch securities that fail internal matching, preventing false auto-approvals.

**Key Deliverables**:
- ✅ `ExternalInstrumentResolver` abstraction & `ResolvedInstrument` model
- ✅ Phase 1: EODHD Resolver implementation
- ✅ Phase 2: OpenFIGI Resolver implementation (stubbed/ready)
- ✅ Repository integration (Tier 0 lookup + exact match)
- ✅ 4-Outcome routing logic (Insider Check, Auto-Approve, Clarification, Flagged Review)
- ✅ Outcome 4 Chatbot UX (Clarification prompt + Proceed-with-flag)
- ✅ Audit logging for external lookups
- ✅ TDD suite with 8 unit/integration tests

---

## [x] Track: PDF Variance Engine for Parser Robustness Testing ✅ COMPLETE
*Link: [./conductor/tracks/pdf_variance_engine_20260205/](./conductor/tracks/pdf_variance_engine_20260205/)*
**Priority**: High
**Tags**: testing, pdf-generation, variance, parser-robustness, faker
**Status**: Complete (2026-02-05)
**Branch**: DSS-4074
**Dependencies**: mock_pdf_generator_20260205

Implemented variance injection to the Mock PDF Generator to produce visually different PDFs from identical trade data. This ensures the AI parser can reliably recognize trades sections regardless of formatting, date styles, or text casing.

**Key Deliverables**:
- ✅ `VarianceConfig` with seeded deterministic generation
- ✅ Custom Jinja2 filters for date/number/currency/text formatting variance
- ✅ Faker integration for realistic broker context (disclaimers, addresses)
- ✅ CLI support for `--variance` and `--seed` flags
- ✅ Ground truth JSON enrichment with variance metadata
- ✅ Integration tests verifying 5 unique variants of the same trade data

---

## [x] Track: PDF Ingestion History Dashboard ✅ COMPLETE
*Link: [./conductor/tracks/pdf_ingestion_dashboard_20260205/](./conductor/tracks/pdf_ingestion_dashboard_20260205/)*
**Priority**: Medium
**Tags**: dashboard, monitoring, pdf-ingestion, analytics, extraction-pipeline
**Status**: Complete (2026-02-06)
**Branch**: DSS-4074

New dashboard page showing all PDFs ingested into the system with full visibility into extraction results, confidence scores, matched trades, and pipeline performance analytics.

**Key Deliverables**:
- ✅ Document list view with filters (source, status, date range, search)
- ✅ Document detail modal with extracted trades and raw LLM output
- ✅ Inline PDF viewer (signed GCS URLs with dev proxy fallback)
- ✅ Confidence score display (clickable badges with 60/40 breakdown)
- ✅ Trade matching status (matched/unmatched/manual_review)
- ✅ Processing timeline (event log visualization)
- ✅ Pipeline performance stats (throughput, success rate, avg processing time)
- ✅ Token usage tracking (prompt/completion metrics from Gemini 3 Flash)
- ✅ Field coverage analytics (% of docs with ticker, ISIN, etc.)
## [x] Track: Modernize Trade Upload Pipeline (GCS Native) ✅ COMPLETE
*Link: [./conductor/tracks/modernize_trade_upload_pipeline_20260206/](./conductor/tracks/modernize_trade_upload_pipeline_20260206/)*
**Priority**: High
**Tags**: refactor, slack, gcs, dashboard
**Status**: Complete (2026-02-06)
**Branch**: DSS-4074

Unify UI and Slack trade confirmation flows to use Google Cloud Storage, ensuring Trade History 'View' icons work consistently.

**Key Deliverables**:
- ✅ Refactored Slack handler to use `process_trade_document` (GCS-native)
- ✅ Updated Repository to store and retrieve `gcs_document_id`
- ✅ Fixed UI upload pipe to correctly pass GCS ID to execution records
- ✅ Removed legacy `data/contract_notes` local disk storage and volume mounts
- ✅ Verified "View" icons appear and function for new uploads

---

## [~] Track: Generic Bug Fixes and Tweaks
*Link: [./conductor/tracks/generic_bugfixes_and_tweaks_20260206/](./conductor/tracks/generic_bugfixes_and_tweaks_20260206/)*
**Priority**: Medium
**Tags**: bugfix, maintenance, iterative
**Status**: Complete (2026-02-12)
**Branch**: DSS-4074

Iterative track for addressing small bugs, UI tweaks, and system hygiene as they arise.

**Initial Items**:
- ✅ Enhanced logging visibility (pad-main-logs.py)
- ✅ Unignored scripts/ directory
- ⏳ Hide "SMF16 Required: None" from Slack summary (Pending Discussion) ✅ COMPLETE

---

## [x] Track: Web Search Fallback for Price Discovery ✅ COMPLETE
*Link: [./conductor/tracks/web_search_price_discovery_fallback_20260211/](./conductor/tracks/web_search_price_discovery_fallback_20260211/)*
**Priority**: Medium
**Tags**: chatbot, market-data, web-search, adk, gemini-3-flash, price-discovery
**Status**: Not_started

Implement a secondary price discovery mechanism using Google ADK and Gemini 3 Flash to find market prices via web search when the primary EODHD API fails.

---
## [x] Track: Price Resolution Strategy ✅ COMPLETE
*Link: [./conductor/tracks/price_resolution_strategy_20260208/](./conductor/tracks/price_resolution_strategy_20260208/)*
**Priority**: Low
**Tags**: chatbot, market-data, ux, future
**Status**: Planning

Future handling of security market prices to prevent AI hallucinations and unintentional overwriting of user-provided value estimates.

---

## [x] Track: Structured Logging Migration ✅ COMPLETE
*Link: [./conductor/tracks/structured_logging_20260209/](./conductor/tracks/structured_logging_20260209/)*
**Priority**: High
**Tags**: logging, observability, structlog, correlation-id, middleware, otel
**Status**: Complete (2026-02-09)
**Branch**: DSS-4074

Migrate entire codebase from stdlib logging to structlog with unified JSON output in production and human-readable console output in development. Includes correlation ID middleware for FastAPI and propagation to outbound HTTP calls.

**Key Deliverables**:
- ✅ Unified structlog across all services and scripts
- ✅ Correlation ID middleware and propagation (X-Correlation-ID)
- ✅ Standardized field names (user_id, instrument, duration_ms, etc.)
- ✅ Support for JSON (prod) and Console (dev) formats
- ✅ Comprehensive unit and E2E test suites (pytest + Playwright)

---

## [x] Track: Operations Dashboard Redesign ✅ COMPLETE
*Link: [./conductor/tracks/dashboard_redesign_20260209/](./conductor/tracks/dashboard_redesign_20260209/)*
**Priority**: High
**Tags**: frontend, backend, dashboard, audit-events, role-based-access, sparklines, compliance
**Status**: Complete (2026-02-09)
**Branch**: DSS-4074

Full redesign of the Operations Dashboard with three-tier role-aware views (standard, manager, compliance), new `audit_events` database table, inline audit event writes in 8 action handlers, sparkline charts, and two compliance-only bottom panels (Recent Activity + Request Statistics).

**Completed Phases:**
- ✅ Phase 1: Database Layer — AuditEvent model, migration, 5 repository functions
- ✅ Phase 2: Audit Event Writes — 8 instrumented action handlers (same-transaction safety)
- ✅ Phase 3: API Endpoints — 3-tier scoping, sparklines, request statistics, enhanced activity
- ✅ Phase 4: Frontend Redesign — 5 new components, Dashboard.tsx rewrite, TypeScript types, API client
- ✅ Phase 5: Test User "Tibi Eris" — dev switcher, seed data, sample requests
- ✅ Phase 6: Test Suites — 21 pytest + 24 pytest + 27 Playwright tests written
- ✅ Phase 7: Run tests in live environment & patch bugs

---

---

## [x] Track: Universal Instrument Validation & Interface Sync ✅ COMPLETE
*Link: [./conductor/tracks/universal_instrument_validation_20260209/](./conductor/tracks/universal_instrument_validation_20260209/)*
**Priority**: High
**Tags**: chatbot, web-ui, validation, synchronization, compliance, identifiers
**Status**: Complete (2026-02-09)
**Branch**: DSS-4074

Enforce a "one-of-four" identifier constraint (ISIN, SEDOL, Bloomberg, Ticker) across both Chatbot and Web UI, while synchronizing "soft" logic like justification coaching to ensure minimal drift between submission methods. Also implemented manual SMF16 escalation gate.

**Key Deliverables**:
- ✅ Shared `ValidationService` for cross-platform consistency.
- ✅ Enhanced Chatbot extraction for ISIN/SEDOL/Bloomberg/Ticker.
- ✅ Unified "one-of-four" identifier requirement in API and Chatbot.
- ✅ Shared justification coaching logic.
- ✅ Manual SMF16 escalation gate (Compliance vet → Manual SMF16 routing).
- ✅ Real-time coaching in Web UI.
- ✅ 1312 unit tests passed.

## [ ] Track: SMF16 Approval Bug Investigation
*Link: [./conductor/tracks/smf16_approval_bug_investigation_20260209/](./conductor/tracks/smf16_approval_bug_investigation_20260209/)*
**Priority**: High
**Tags**: bug, compliance, smf16, approval-workflow
**Status**: Complete (2026-02-12)

Investigate why an SMF16-authorized admin cannot approve a "pending SMF16" request (LDEBURNA-260209-gd-488b).

## [x] Track: High-Integrity Audit Logging System ✅ COMPLETE
*Link: [./conductor/tracks/audit_refinement_20260210/](./conductor/tracks/audit_refinement_20260210/)*
**Priority**: High
**Tags**: compliance, audit-logging, trace-id, snapshoting, immutability, tdd
**Status**: Completed (2026-02-10)

Upgrade the audit logging system to meet high-integrity financial compliance standards. Implements strict linkage between business events (Trade Reference ID) and technical execution (Trace ID), point-in-time rules snapshots, and comprehensive AI rationale logging.

**Key Deliverables**:
- Unified Audit Schema with trace_id and trade_reference_id
- Automated trace_id propagation across sync/async events
- Point-in-time snapshots of risk thresholds and rules at decision time
- Enhanced AI decision logging with model_input_hash and rationale
- Immutable append-only audit storage logic
- Redesigned Dashboard Timeline with exception highlighting and drill-down

## [ ] Track: Rules Engine UI Refactor
*Link: [./conductor/tracks/rules_engine_ui_refactor_20260211/](./conductor/tracks/rules_engine_ui_refactor_20260211/)*
**Priority**: High
**Tags**: frontend, refactor, rules-engine, compliance
**Status**: Complete (2026-02-12)

Refactor Rules Engine UI to simplify risk factors, standardize severity options, and improve audit logging.

## [x] Track: Reference Schema Parameterization & ldeburna Migration ✅ COMPLETE
*Link: [./conductor/tracks/reference_schema_parameterization_20260212/](./conductor/tracks/reference_schema_parameterization_20260212/)*
**Priority**: High
**Tags**: backend, database, refactor, architecture, migration
**Status**: Complete (2026-02-12)

Decouple the application from the hardcoded 'bo_airflow' schema and enable seamless migration to the 'ldeburna' reference schema via configuration. Implements a hybrid FK migration strategy for safety.

**Key Deliverables**:
- ✅ Dynamic schema mapping in SQLAlchemy models (SchemaMixin)
- ✅ Parameterized raw SQL fragments and joins
- ✅ Hybrid FK Migration (NOT VALID + VALIDATE)
- ✅ Column parity audit utility
- ✅ repoint Dev environment to ldeburna

## [x] Track: Migrate Email Ingestion State to ldeburna ✅ COMPLETE
*Link: [./conductor/tracks/migrate_email_state_ldeburna_20260212/](./conductor/tracks/migrate_email_state_ldeburna_20260212/)*
**Priority**: Critical
**Tags**: backend, migration, database, decoupling
**Status**: Complete (2026-02-12)

Migrate 'email_ingestion_state' to 'ldeburna' to achieve 100% decoupling from 'bo_airflow'.

---

- [ ] **Track: Fix Missing Audit Columns (Trace ID)**
*Link: [./conductor/tracks/fix_audit_columns_20260212/](./conductor/tracks/fix_audit_columns_20260212/)*

## [x] Track: PostgreSQL Connection Hygiene Remediation ✅ COMPLETE
*Link: [./conductor/tracks/postgres_connection_hygiene_20260212/](./conductor/tracks/postgres_connection_hygiene_20260212/)*
**Priority**: High
**Tags**: backend, database, performance, reliability
**Status**: Complete (2026-02-12)

Remediate connection leaks and improve database observability by implementing proper shutdown handling, test isolation, and connection tagging.

**Key Deliverables**:
- ✅ Configurable pool settings (size=3, overflow=2)
- ✅ Explicit engine disposal on shutdown
- ✅ Test suite isolation (await dispose)
- ✅ Application name tagging

## [ ] Track: Fix Graph Poller Attribute Error
*Link: [./conductor/tracks/fix_graph_poller_attribute_error_20260212/](./conductor/tracks/fix_graph_poller_attribute_error_20260212/)*
**Priority**: High
**Tags**: bugfix, email-ingestion
**Status**: Complete (2026-02-12)

Fix AttributeError in email poller preventing message processing.

## [x] Track: Fix Graph Poller Attribute Error ✅ COMPLETE
*Link: [./conductor/tracks/fix_graph_poller_attribute_error_20260212/](./conductor/tracks/fix_graph_poller_attribute_error_20260212/)*
**Priority**: High
**Tags**: bugfix, email-ingestion
**Status**: Complete (2026-02-12)

Fixed 'MessageInfo' attribute error. Discovered second error in AuditLogger.

## [x] Track: Fix Graph Poller Audit Error ✅ COMPLETE
*Link: [./conductor/tracks/fix_graph_poller_audit_error_20260212/](./conductor/tracks/fix_graph_poller_audit_error_20260212/)*
**Priority**: High
**Tags**: bugfix, email-ingestion, audit
**Status**: Complete (2026-02-12)

Fix AttributeError in AuditLogger call within the email poller.

## [x] Track: Environment Configuration & Code Robustness Remediation ✅ COMPLETE
*Link: [./conductor/tracks/env_config_remediation_20260212/](./conductor/tracks/env_config_remediation_20260212/)*
**Priority**: High
**Tags**: backend, configuration, bugfix, decoupling
**Status**: Planned

Fix regressions in environment variable handling and model properties to ensure stable schema and auth behavior.

---

- [x] **Track: Centralised OTel Log Collection System** - COMPLETE
*Link: [./tracks/otel_collector_20260212/](./tracks/otel_collector_20260212/)*
**Branch**: `feat/otel-log-collection`
**Completed**: 2026-02-12 - OTel Collector Contrib service added to docker-compose, filelog receiver, structlog JSON parsing, audit log filtering, date-rotated file export.

---

## Autopsy Remediation Tracks (2026-02-12)

> Derived from `.autopsy/REVIEW_REPORT.md` and `.autopsy/ARCHITECTURE_REPORT.md`.
> All findings verified against actual codebase code by investigation agents.

### Wave 1: Production Blockers (CRITICAL)

## [x] Track: Security & Authentication Hardening ✅ COMPLETE
*Link: [./conductor/tracks/security_auth_hardening_20260212/](./conductor/tracks/security_auth_hardening_20260212/)*
**Priority**: Critical
**Tags**: security, authentication, production-blocker
**Status**: Complete
**Branch**: `fix/surgical-data-integrity` (Finding #5), `fix/wave2-safe-fixes` (Findings #1, #4), `fix/security-auth-hardening-remaining` (Findings #2, #3)
**Completed**: 2026-02-17

Fix CRITICAL security findings: ~~dev auth bypass in production~~, ~~CORS wildcard with credentials~~, ~~IAP JWT not verified~~, ~~terminated employees can authenticate~~, ~~hardcoded API token~~.
All 5/5 findings resolved. Finding #2: CORS wildcard replaced with configurable allowlist (`cors_allowed_origins`). Finding #3: IAP JWT verification via `X-Goog-IAP-JWT-Assertion` with signature validation, issuer check, and graceful fallback. 29 security tests added.

---

## [-] Track: Critical Data Integrity Bug Fixes - IN PROGRESS (6/8 bugs)
*Link: [./conductor/tracks/critical_data_integrity_bugs_20260212/](./conductor/tracks/critical_data_integrity_bugs_20260212/)*
**Priority**: Critical
**Tags**: bugs, data-integrity, compliance
**Status**: In Progress
**Branch**: `fix/surgical-data-integrity`, `fix/wave2-safe-fixes`

Fix 8 verified logic bugs: ~~PDF double-decode corruption~~, ~~breach auto-resolution (not vs not_())~~, FX rate fallback to 1.0, HIGH risk misrouting (reclassified to investigation), ~~orphan recovery timeout~~, ~~range slider validation~~, ~~audit ActionTypes~~, boolean select types.
**Done**: Bugs #1 (double decode removed), #3 (breach not_()), #5 (orphan timeout cutoff), #6 (range slider clamping), #7/#8 (audit ActionTypes). **Remaining**: Bugs #2 (FX rate), #4 (approval routing - reclassified to investigation).

---

## [x] Track: Datetime & Timezone Standardization - COMPLETE
*Link: [./conductor/tracks/datetime_timezone_standardization_20260212/](./conductor/tracks/datetime_timezone_standardization_20260212/)*
**Priority**: Critical
**Tags**: datetime, timezone, deprecation, compliance
**Status**: Complete
**Branch**: `fix/mechanical-datetime-imports`
**Completed**: 2026-02-12

Replaced all 31 `datetime.utcnow()` calls across 13 files with `datetime.now(UTC)`. Added DTZ ruff lint rules to pyproject.toml. Zero violations remaining.

---

## [x] Track: Import Path Standardization - COMPLETE
*Link: [./conductor/tracks/import_path_standardization_20260212/](./conductor/tracks/import_path_standardization_20260212/)*
**Priority**: Critical
**Tags**: imports, code-quality, reliability
**Status**: Complete
**Branch**: `fix/mechanical-datetime-imports`
**Completed**: 2026-02-12

Standardized all 128 `from src.pa_dealing` imports to `from pa_dealing` across 34 files. Removed `src/__init__.py`. Added TID251 banned-api ruff rule. Zero violations remaining.

---

## [-] Track: Rules Engine Cache Invalidation & Concurrency Fix - IN PROGRESS
*Link: [./conductor/tracks/rules_engine_cache_and_concurrency_20260212/](./conductor/tracks/rules_engine_cache_and_concurrency_20260212/)*
**Priority**: Critical
**Tags**: rules-engine, cache, concurrency
**Status**: In Progress
**Branch**: `fix/wave2-safe-fixes` (cache invalidation)
**Depends on**: rules_engine_ui_refactor_20260211

~~Invalidate PADRuleRegistry cache on write operations.~~ Remove singleton risk scorer race condition.
**Done**: Cache invalidation after update_rule() and toggle_rule(). **Remaining**: Singleton risk scorer race condition.

---

### Wave 2: Reliability & Performance (HIGH)

## [-] Track: Error Handling & Resilience Hardening - IN PROGRESS
*Link: [./conductor/tracks/error_handling_resilience_20260212/](./conductor/tracks/error_handling_resilience_20260212/)*
**Priority**: High
**Tags**: error-handling, resilience, reliability
**Status**: In Progress
**Branch**: `fix/wave2-safe-fixes` (asyncio.gather fix)

Fix ~~asyncio.gather error handling~~, API error handler types, credential leaking in logs, unprotected flush() calls, and silent failure patterns.
**Done**: asyncio.gather return_exceptions=True in dashboard summary. **Remaining**: API error types, credential leaking, flush() calls, silent failures.

---

## [ ] Track: Performance & Async Optimization
*Link: [./conductor/tracks/performance_async_optimization_20260212/](./conductor/tracks/performance_async_optimization_20260212/)*
**Priority**: High
**Tags**: performance, async, scalability
**Status**: Not Started

Replace per-request HTTP clients with shared AsyncClient. Wrap GCS operations with asyncio.to_thread(). Optimize N+1 queries and add pagination.

---

### Wave 3: Architecture (HIGH, after Wave 1 & 2)

## [ ] Track: God Module Decomposition
*Link: [./conductor/tracks/god_module_decomposition_20260212/](./conductor/tracks/god_module_decomposition_20260212/)*
**Priority**: High
**Tags**: refactor, maintainability, architecture
**Status**: Not Started
**Depends on**: critical_data_integrity_bugs_20260212, error_handling_resilience_20260212

Split 5 god modules (>2,300 LOC each) into focused modules <800 LOC. Extract business logic from Slack handlers to service layer.
