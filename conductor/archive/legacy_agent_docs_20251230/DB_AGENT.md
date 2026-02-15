**Role:**
You are a Senior Backend Engineer and Database Administrator. You are tasked with building a "Data Shim" API. This API will serve as a secure, functional interface between an AI Agent and our internal SQL database.

**Objective:**
Create a Python FastAPI service that exposes specific business logic as REST endpoints. The AI Agent will call these endpoints as "Tools". The service must manage SQL connections via SQLAlchemy and provide an `openapi.json` spec for easy integration.

**Database Schema (Mock Definitions):**
Please use `SQLAlchemy` to define these models. Assume the DB is PostgreSQL.
1.  **`BackofficePositions`** (Table: `BACKOFFICE.POSITIONS`)
    *   `id` (PK)
    *   `employee_email` (String, Index)
    *   `ticker_symbol` (String)
    *   `quantity` (Integer)
    *   `last_updated` (DateTime)
2.  **`BackofficePADRequests`** (Table: `BACKOFFICE.PERSONAL_ACCOUNT_DEALING`)
    *   `request_id` (UUID, PK)
    *   `employee_email` (String, Index)
    *   `ticker_symbol` (String)
    *   `direction` (Enum: BUY, SELL)
    *   `risk_score` (Enum: LOW, MEDIUM, HIGH)
    *   `status` (Enum: PENDING, APPROVED, REJECTED, EXECUTED)
    *   `created_at` (DateTime, default=now)

**Functional Endpoints (The "Tools"):**

**1. Tool: `get_current_position`**
*   **Endpoint:** `GET /positions`
*   **Query Params:** `email` (str), `ticker` (str)
*   **Logic:** Return the sum of `quantity` for that email/ticker. Return `0` if no record exists.
*   **Response:** `{"email": "...", "ticker": "...", "current_holding": 500}`

**2. Tool: `check_30_day_conflicts`**
*   **Endpoint:** `GET /compliance/conflicts`
*   **Query Params:** `ticker` (str)
*   **Logic:**
    *   Query `BackofficePADRequests`.
    *   Filter where `ticker_symbol` == `ticker`.
    *   Filter where `status` == 'APPROVED' or 'EXECUTED'.
    *   Filter where `created_at` >= (Current Time - 30 Days).
*   **Response:**
    ```json
    {
      "conflict_found": true,
      "recent_trades": [
        {"date": "2023-10-01", "direction": "BUY"}
      ]
    }
    ```

**3. Tool: `submit_pad_request`**
*   **Endpoint:** `POST /requests/submit`
*   **Body Schema (Pydantic):**
    *   `employee_email`: EmailStr
    *   `ticker`: str (uppercase)
    *   `direction`: Enum (BUY/SELL)
    *   `quantity`: int (> 0)
    *   `risk_classification`: Enum (LOW/MED/HIGH)
    *   `rationale`: str (optional)
*   **Logic:** Insert a new record into `BackofficePADRequests` with status `PENDING`.
*   **Response:** `{"request_id": "uuid-...", "status": "PENDING"}`

**4. Tool: `update_request_status`**
*   **Endpoint:** `PATCH /requests/{request_id}/status`
*   **Body Schema:**
    *   `status`: Enum (APPROVED, REJECTED)
    *   `approver_notes`: str
*   **Logic:** Update the status of the row.
*   **Response:** `{"request_id": "...", "new_status": "APPROVED"}`

**Key Requirements:**
1.  **OpenAPI Auto-Generation:** The app must be configured to generate a clean `openapi.json` at `/openapi.json`. Ensure every endpoint has a `summary` and `description` as these are used by the AI Agent to understand what the tool does.
2.  **Validation:** Use Pydantic V2 for all input validation. If the Agent sends a lowercase ticker, convert it to uppercase automatically.
3.  **Error Handling:** If a record isn't found, return 404. If a database error occurs, return 500 with a clean JSON error message, not a stack trace.
4.  **Database Connection:** Use an async database driver (`asyncpg`) and manage the session lifecycle properly (dependency injection).

**Deliverables:**
*   `models.py`: SQLAlchemy definitions.
*   `schemas.py`: Pydantic models.
*   `main.py`: The application logic.
*   `database.py`: Connection string handling.
