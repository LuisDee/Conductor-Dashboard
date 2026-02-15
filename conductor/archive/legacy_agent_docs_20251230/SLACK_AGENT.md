**Role:**
You are a Principal Software Architect and Senior Python Engineer. You are tasked with building a "Middleware Bridge" service that connects a Slack App to Google's Gemini Enterprise Agent API.

**Objective:**
Create a production-ready FastAPI application that acts as a bidirectional relay. It must receive messages from Slack users, resolve their identity, maintain conversation state with Gemini, and handle complex interactive workflows for compliance approvals.

**Tech Stack:**
*   **Language:** Python 3.11+
*   **Web Framework:** FastAPI (for handling Slack Webhooks)
*   **Slack SDK:** `slack_bolt` (async preferred) or `slack_sdk`
*   **AI SDK:** `google-cloud-aiplatform` (Vertex AI Agent Engine)
*   **Deployment:** Dockerized, stateless service.

**1. Configuration & Security Requirements (The "Manifest")**
First, generate a `manifest.json` snippet that I can paste into the Slack Developer Portal. It must include:
*   **Bot Scopes:** `chat:write` (sending messages), `im:history` (reading DMs), `users:read` (resolving emails), `users:read.email` (accessing emails).
*   **Events Subscription:** Subscribe to `message.im` (Direct Messages).
*   **Interactivity:** Enable Interactivity pointing to `/slack/events` (for button clicks).

**2. Application Logic Requirements**

**A. Authentication & Event Processing**
*   Implement a middleware/dependency to verify the `X-Slack-Signature` on every request.
*   **User Resolution (Critical):**
    *   When an event `message` is received, extract the `user_id`.
    *   Call `client.users_info(user=user_id)` to fetch the user's profile.
    *   Extract `user.profile.email`. **If the email is missing or not from `@mako.com`, ignore the message.**
    *   *Performance Requirement:* Implement an in-memory TTL cache (e.g., `cachetools`) for UserID->Email lookups to avoid rate limits.

**B. Session Management (State)**
*   Gemini Agents require a persistent `session_id` to maintain context.
*   **Mapping Logic:** Create a mapping key: `f"slack_session:{channel_id}:{thread_ts}"`.
*   Check Redis (or an in-memory dictionary if prototyping) for an existing Gemini Session ID.
*   If none exists, generate a new UUID.
*   *Edge Case:* If the user posts in a new thread, it must be a *new* Gemini session.

**C. Context Injection**
*   Do not send the raw user text to Gemini. Wrap it in a context block.
*   **Format:**
    ```text
    [System Context]
    User Email: stephen.kemp@mako.com
    User Name: Stephen Kemp
    Platform: Slack
    [End Context]

    [User Message]
    {original_user_message}
    ```

**D. The "Approval Loop" (Complex Interaction)**
You need to expose a specific REST endpoint on this Bridge that the **Gemini Agent** can call as a "Tool" when it needs human approval.

*   **Endpoint:** `POST /internal/request-approval`
*   **Payload:**
    ```json
    {
      "requester_email": "stephen.kemp@mako.com",
      "instrument": "AAPL",
      "risk_level": "MEDIUM",
      "rationale": "Volatile stock...",
      "session_id": "..."
    }
    ```
*   **Logic:**
    1.  Format a **Slack Block Kit** message. Use a `section` for details and an `actions` block with two buttons:
        *   Button 1: `style="primary"`, text="Approve", value=`approve:{requester_email}:{instrument}`
        *   Button 2: `style="danger"`, text="Reject", value=`reject:{requester_email}:{instrument}`
    2.  Post this message to the **Private Compliance Channel** (Load ID from `COMPLIANCE_CHANNEL_ID` env var).
    3.  Return `200 OK` to Gemini immediately (do not wait for the human).

**E. Handling the Button Click (`block_actions`)**
*   Listen for the payload from Slack when a button is clicked.
*   **Verification:** Ensure the user clicking is NOT the requester (prevent self-approval).
*   **Update:** Immediately update the message in the Compliance Channel to remove the buttons and append: "✅ Approved by [Approver Name]" or "❌ Rejected".
*   **Notification:** Send a DM to the *Requester* informing them of the result.

**3. Deliverables**
1.  `requirements.txt`
2.  `main.py` (The FastAPI app containing the Event loop, the Shim endpoint, and the Logic).
3.  `README.md` containing the Slack App Manifest and environment variable list.
