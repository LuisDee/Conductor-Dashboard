# Track Specification: UX/GUI Overhaul

## 1. Goal
Revolutionize the user experience for PA Dealing by implementing a modern, professional, and data-dense interface. This track focuses on redesigning the React Compliance Dashboard and the Slack Bot interactions to provide a seamless "single pane of glass" experience.

## 2. Scope
- **Compliance Dashboard:**
    - Implement persistent sidebar navigation (Dashboard, Requests, Reports, Settings).
    - Redesign "Requests" and "Breaches" tables for high data density.
    - Implement "Traffic Light" (Red/Amber/Green) risk visualization system.
    - Create detailed "Request View" with clear separation of data, risk status, and actions.
- **Slack Bot:**
    - Migrate all text-based interactions to Slack Block Kit.
    - Implement visual status badges (🟢, 🔴, ⏳) in messages.
    - Create interactive modals for request submission.

## 3. User Stories
- **As a Compliance Officer,** I want a sidebar navigation so I can quickly switch between views without losing context.
- **As a Compliance Officer,** I want to see risk levels (Red/Green) instantly in lists so I can prioritize high-risk items.
- **As an Employee,** I want to fill out a structured form in Slack (not just chat) so I know exactly what information is required.
- **As a Manager,** I want to approve/reject requests with a single click in Slack.

## 4. Technical Requirements
- **Frontend:** React, Tailwind CSS, Lucide React (Icons).
- **Slack:** Python `slack_bolt`, Block Kit Builder.
- **State Management:** React Query (TanStack Query) for real-time data fetching.
- **Testing:** Playwright for visual regression testing of new UI components.

## 5. Design Guidelines (from Product Guidelines)
- **Visual Style:** Professional, Trustworthy, Minimalist.
- **Density:** Compact tables, minimal whitespace in data views.
- **Feedback:** Clear error messages with "Contact Support" actions.
