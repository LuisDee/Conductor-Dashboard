# Specification: Azure Entra ID & Microsoft Graph Integration

## Overview
This track implements a dynamic identity and reporting hierarchy system using Azure Entra ID (formerly Azure AD) via the Microsoft Graph API. It replaces the legacy Oracle-based employee source while maintaining external references to position and market data. The system will dynamically discover and cache user metadata and reporting lines, cross-referencing with Slack identities via email matching.

## Functional Requirements
1.  **Entra ID Integration:**
    *   Implement authentication with Azure using the Client Credentials Flow (`msal`).
    *   Integrate with Microsoft Graph API to fetch user profiles and reporting structures (managers/direct reports).
2.  **Dynamic Identity Discovery:**
    *   Implement an "on-demand" discovery mechanism when a user first interacts with the system (Slack or Dashboard).
    *   Perform multi-source resolution:
        *   Match Slack `user_id` to email.
        *   Match email to Entra ID profile.
        *   Resolve reporting lines (Manager ID) from Entra ID.
3.  **Hybrid Caching Strategy (TTL-based):**
    *   Store discovered identity and hierarchy data in the local database (`oracle_employee` table).
    *   Implement a Time-To-Live (TTL) or "Last Refreshed" mechanism.
    *   Refresh data from source APIs if the local cache is stale (e.g., > 24 hours) or during critical workflow events (e.g., login).
4.  **Reporting Hierarchy Enforcement:**
    *   Automate PAD request routing based on the dynamically resolved manager from Entra ID.
    *   Dynamically determine "Manager" role flags for the dashboard based on `directReports` data from Graph API.

## Non-Functional Requirements
1.  **Resilience:** Handle Graph API or Slack API rate limits and transient failures gracefully with backoff/retry logic.
2.  **Performance:** Optimize API calls to prevent latency during user interaction (use asynchronous requests and efficient caching).
3.  **Security:** Ensure Tenant ID, Client ID, and Client Secret are managed via environment variables and never logged or exposed.

## Acceptance Criteria
1.  A user can log in to the dashboard, and their identity is successfully resolved via email matching against Entra ID.
2.  The system correctly identifies if a user has direct reports via Graph API and grants appropriate "Manager" access in the UI.
3.  A PAD request submitted via Slack is automatically routed to the manager defined in Entra ID.
4.  Identity data in the database is automatically refreshed after the defined TTL period.

## Out of Scope
*   Replacing non-identity tables (market data, positions, etc.).
*   Manual editing of employee reporting lines within the application (Entra ID remains the sole source of truth).
