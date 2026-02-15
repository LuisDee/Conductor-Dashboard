# Track Plan: UX/GUI Overhaul

## Phase 1: Foundation & Navigation (COMPLETED)
- [x] Task: Setup Shared UI Components (Buttons, Badges, Cards)
    - [x] Subtask: Create atomic design components in `src/components/ui/` using Tailwind.
    - [x] Subtask: Implement `StatusBadge` with strict Red/Amber/Green variants.
    - [x] Subtask: Create `Sidebar` component with navigation links.
    - [x] Subtask: Write Playwright tests for component rendering.
- [x] Task: Implement Main Layout
    - [x] Subtask: Update `App.tsx` to use the new Sidebar + Main Content layout.
    - [x] Subtask: Ensure responsive behavior (collapsible sidebar on mobile).
    - [x] Subtask: Verify layout with Playwright.
- [x] Task: Global Styling Standardization
    - [x] Subtask: Unified `.label` and `.input` classes in `index.css`.
    - [x] Subtask: Implement `SearchableSelect` for consistent employee/entity selection.

## Phase 2: Compliance Dashboard Data Views (COMPLETED)
- [x] Task: Redesign Requests Table
    - [x] Subtask: Create standardized 7-column filter grid across all data pages.
    - [x] Subtask: Implement linked filtering (Employee selection narrows Request list).
    - [x] Subtask: Prettify Audit Log "Event Insight" with compact key-value pills.
    - [x] Subtask: Connect to API using React Query for live data.
- [x] Task: Build Detailed Request View
    - [x] Subtask: Create overhauled `RequestDetail` page with high-density "Trade Details" card.
    - [x] Subtask: Implement split view: "Request Data" (Left) vs "Employment/Security Metadata" (Right).
    - [x] Subtask: Add "Action Bar" logic for Manual Chase and Approvals.

## Phase 3: Slack Block Kit Migration (COMPLETED)
- [x] Task: Update Request Submission Flow
    - [x] Subtask: Replace text conversation with a `views.open` Modal form.
    - [x] Subtask: Update backend handler to process Modal submission.
- [x] Task: Update Notification Messages
    - [x] Subtask: Redesign "New Request" notification using Block Kit (Sections + Actions).
    - [x] Subtask: Add dynamic status emojis (🟢/🔴) to message updates.
    - [x] Subtask: Test Slack rendering with mock payload.

## Phase 4: Final Polish & Verification (COMPLETED)
- [x] Task: End-to-End User Journey Test
    - [x] Subtask: Simulate full flow: Employee submits (Slack Modal) -> Manager approves (Slack Block) -> Compliance views (Dashboard).
    - [x] Subtask: Verify all UI states and transitions.
- [x] Milestone: Infrastructure & Quality
    - [x] Refactor `AIDecisionService` to `ComplianceDecisionService` for clarity.
    - [x] Implement deterministic DB seeding for "Zero-Config" test stability.
    - [x] Integrated `Ruff` for automated linting and formatting.
