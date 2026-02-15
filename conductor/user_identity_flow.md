# User Identity Flow for `/api/auth/me`

This document outlines the end-to-end flow for retrieving authenticated user information via the `/api/auth/me` endpoint, detailing the requests, code execution path, database interactions (tables and SQL), and external API calls.

## 1. Initial Network Request

**Endpoint:** `GET /api/auth/me`
**Purpose:** To retrieve the current authenticated user's profile and authorization status for the frontend dashboard.
**Example Response Payload:**
```json
{
    "success": true,
    "message": null,
    "data": {
        "email": "luis.deburnay-bastos@mako.com",
        "employee_id": 1272,
        "mako_id": "ldeburna",
        "full_name": "Luis De Burnay-Bastos",
        "employee_uuid": "7a80d9a8-b49e-45a4-8a6d-dd3e8453b97a",
        "is_compliance": true,
        "is_admin": true,
        "is_manager": false,
        "is_smf16": true,
        "auth_status": "ok",
        "auth_message": null
    }
}
```

## 2. API Endpoint Handling (`src/pa_dealing/api/main.py`)

The `/api/auth/me` endpoint is defined in `src/pa_dealing/api/main.py`:

```python
# src/pa_dealing/api/main.py
@app.get("/api/auth/me", response_model=APIResponse, tags=["auth"])
async def get_me(user: CurrentUser = Depends(get_current_user)):
    # ...
    return APIResponse(
        success=True,
        data={
            "email": user.email,
            "employee_id": user.employee_id,
            "mako_id": user.mako_id,
            "full_name": user.full_name,
            "employee_uuid": user.employee_uuid,
            "is_compliance": user.is_compliance,
            "is_admin": user.is_admin,
            "is_manager": user.is_manager,
            "is_smf16": user.is_smf16,
            "auth_status": user.auth_status,
            "auth_message": user.auth_message,
        },
    )
```
- The `get_me` function is an asynchronous endpoint that depends on `get_current_user` to resolve the `CurrentUser` object.
- The `CurrentUser` object (which matches the structure of the desired payload) is directly returned within an `APIResponse`.

## 3. User Authentication and Identity Resolution (`src/pa_dealing/api/auth.py`)

The `get_current_user` function, defined in `src/pa_dealing/api/auth.py`, is responsible for authenticating the user and populating the `CurrentUser` object:

```python
# src/pa_dealing/api/auth.py
async def get_current_user(...):
    # ... (Authentication logic to determine user email)
    async with get_session() as session:
        identity = get_identity_provider_with_session(session)
        employee = await identity.get_by_email(email)

        if not employee:
            # Employee not found in database, return basic CurrentUser with error status
            return CurrentUser(...)
        
        is_manager = await identity._google_client.is_manager(email)
        is_admin = "admin" in employee.roles

        return CurrentUser(
            email=email,
            # ... other fields from employee and derived roles/manager status
            employee_uuid=employee.employee_uuid, # From Google externalIds (employee UUID)
            is_compliance="compliance" in employee.roles,
            is_smf16="smf16" in employee.roles,
            is_admin=is_admin,
            is_manager=is_manager,
            roles=employee.roles,
            auth_status="ok",
            auth_message=None,
        )
```

**Authentication Priority:**
1.  **IAP Headers:** Checks for `X-Goog-Authenticated-User-Email` and `X-Goog-Authenticated-User-Id` (production).
2.  **Dev Header:** Checks for `X-Dev-User-Email` (development/test).
3.  **Auto-bypass:** If in `development` environment and no dev header, defaults to a predefined email (`luis.deburnay-bastos@mako.com`). This is likely the path for the example payload.
4.  **Bypass Setting:** Checks `settings.bypass_auth` (testing).

**Identity Resolution:**
- Once an `email` is established, an `AsyncSession` is obtained (`get_session()`).
- `get_identity_provider_with_session(session)` is called to instantiate the appropriate `IdentityProvider`.
- The `identity.get_by_email(email)` method is then called to fetch detailed employee information, which forms the core of the `CurrentUser` object.
- `identity._google_client.is_manager(email)` is called to determine if the user is a manager.

## 4. Hybrid Identity Provider (`src/pa_dealing/identity/provider_google.py`)

The `get_identity_provider_with_session` function (defined in `src/pa_dealing/identity/__init__.py`) returns a `GoogleIdentityProvider` instance. This class, located in `src/pa_dealing/identity/provider_google.py`, is a hybrid provider that combines data from Google Workspace APIs and the SQL database.

The critical method here is `GoogleIdentityProvider.get_by_email(email)`:

### A. External API Call (Google Admin API)

-   `google_data = await self._google_client.get_user_info(email)`
    -   **Purpose:** Fetches comprehensive user metadata from Google Workspace.
    -   **Information Retrieved:** `full_name`, `given_name`, `family_name`, `manager_email`, `department`, `job_title`, and critically, `employee_id` which is mapped to `employee_uuid` from Google's `externalIds`. This is the source of the `employee_uuid` in the final payload.
-   `is_manager = await self._google_client.is_manager(email)`
    -   **Purpose:** Determines if the user has direct reports, signifying a manager role according to Google Workspace.

### B. Database Interactions (SQL Queries)

The `get_by_email` method performs several SQL lookups to resolve the `employee_id`, fetch trading anchor data, and retrieve roles. All queries are executed using `sqlalchemy`'s `text()` construct against a PostgreSQL database (likely named `bo_airflow` for employee data and `padealing` for application-specific data).

**Tables Referenced:**
-   `bo_airflow.oracle_employee`: Core employee information, including `id` (employee_id), `mako_id`, `manager_id`, `company`, `cost_centre`, `end_date`.
-   `padealing.employee_contact`: Links email/name information to `oracle_employee.id`, containing `email`, `forename`, `surname`.
-   `padealing.employee_role`: Stores roles assigned to specific `employee_id`s.
-   `padealing.manager_override`: (Potentially) for manual manager assignments.

**SQL Queries (Simplified/Key Fragments):**

1.  **`find_best_match` (Internal to `provider_google.py`, for `employee_id` resolution):**
    -   This is a sophisticated fuzzy matching process that attempts to link Google metadata (name, email, job title) to an `employee_id` in the database.
    -   It typically involves selecting from `bo_airflow.oracle_employee` and joining `padealing.employee_contact`.
    -   **Example Query Pattern:**
        ```sql
        SELECT e.id AS employee_id, ...
        FROM bo_airflow.oracle_employee e
        JOIN padealing.employee_contact c ON e.id = c.employee_id
        WHERE ... (fuzzy matching logic on names, emails)
        ```
    -   **Tables:** `bo_airflow.oracle_employee`, `padealing.employee_contact`

2.  **`_resolve_employee_id_by_email` (Fallback `employee_id` resolution):**
    -   `SELECT e.id FROM bo_airflow.oracle_employee e JOIN padealing.employee_contact c ON e.id = c.employee_id WHERE LOWER(c.email) = LOWER(:email)`
    -   **Purpose:** Direct lookup of `employee_id` using the email from `padealing.employee_contact`.
    -   **Tables:** `bo_airflow.oracle_employee`, `padealing.employee_contact`

3.  **`_resolve_employee_id_by_mako_id` (Fallback `employee_id` resolution):**
    -   `SELECT id FROM bo_airflow.oracle_employee WHERE LOWER(mako_id) = LOWER(:mako_id)`
    -   **Purpose:** Looks up `employee_id` using a derived Mako ID.
    -   **Table:** `bo_airflow.oracle_employee`

4.  **`_get_sql_anchor_data` (Fetching core employee attributes):**
    -   `SELECT e.mako_id, e.manager_id, e.company, e.cost_centre, e.end_date, c.email AS contact_email, c.forename, c.surname FROM bo_airflow.oracle_employee e LEFT JOIN padealing.employee_contact c ON e.id = c.employee_id WHERE e.id = :employee_id`
    -   **Purpose:** Retrieves key employee attributes linked to the `employee_id`.
    -   **Tables:** `bo_airflow.oracle_employee`, `padealing.employee_contact`

5.  **`get_roles` (Fetching user roles):**
    -   `SELECT role FROM padealing.employee_role WHERE employee_id = :employee_id AND revoked_at IS NULL`
    -   **Purpose:** Fetches all active roles for the employee (e.g., "compliance", "admin", "smf16").
    -   **Table:** `padealing.employee_role`

6.  **`_get_manager_override` (Manager resolution, if enabled):**
    -   `SELECT manager_id FROM padealing.manager_override WHERE employee_id = :employee_id AND is_active = true AND (expires_at IS NULL OR expires_at > now())`
    -   **Purpose:** Checks for manually configured manager overrides.
    -   **Table:** `padealing.manager_override`

### C. Manager Resolution (Recursive)

- If `manager_email` is provided by Google, `identity.get_by_email(manager_email, resolve_manager=False)` is recursively called to resolve the manager's `IdentityInfo` and `employee_id`. This involves the same steps (Google API call, SQL lookups) for the manager.

## 5. Final `CurrentUser` and `APIResponse` Construction

- All the collected data (from Google Admin API, and various SQL queries against `bo_airflow.oracle_employee`, `padealing.employee_contact`, `padealing.employee_role`, and potentially `padealing.manager_override`) is combined and used to instantiate the `IdentityInfo` object.
- This `IdentityInfo` object is then used to construct the `CurrentUser` object, which is passed back to `main.py`'s `get_me` function.
- `get_me` then wraps this `CurrentUser` data into an `APIResponse` object and sends it as the final network response.

---
**Summary of Data Sources:**

-   **Google Admin API:** `email`, `full_name`, `given_name`, `family_name`, `manager_email`, `department`, `job_title`, `employee_uuid` (from Google `externalIds`), `is_manager` status.
-   **`bo_airflow.oracle_employee` table:** `employee_id`, `mako_id`, `manager_id`, `company`, `cost_centre`, `end_date` (for `is_active`).
-   **`padealing.employee_contact` table:** `email`, `forename`, `surname`.
-   **`padealing.employee_role` table:** User roles (`is_compliance`, `is_admin`, `is_smf16`).
-   **`padealing.manager_override` table:** Manual manager assignments.
