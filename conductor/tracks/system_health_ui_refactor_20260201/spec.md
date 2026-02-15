# Track Spec: System Health UI Refactor & Design Alignment

## 1. Context & Problem Statement
The current implementation of the outbox monitoring page (recently renamed to "Notification Health") suffers from severe design debt:
- **Design Inconsistency**: Uses deep blue colors (#162A4F) for blocks and tables that do not exist anywhere else in the application.
- **Branding Mismatch**: Header text is white and styles do not follow the Montserrat font and spacing rules established in the MAKO Design System.
- **Poor Scalability**: The entire page is dedicated to a single table (Outbox), whereas it should be a general "System Health" page capable of hosting multiple panels (e.g., Database status, Slack connectivity, Worker health).
- **Naming Confusion**: The path and label need to be reverted to "System Health" but with proper internal organization.

## 2. Objectives
- **Align with MAKO Design System**: Standardize colors, typography, and card styles to match the rest of the application.
- **Restructure for Scalability**: Transform the page into a multi-panel dashboard where "Notifications" is a primary subsection.
- **Revert Naming**: Path: `/admin/system-health`, Label: `System Health`.
- **Improve Layout**: Use standard MAKO cards and tables instead of the custom deep blue styling.

## 3. Requirements
- **Card Styling**: Replace `#162A4F` backgrounds with standard MAKO white cards with navy-based shadows.
- **Header Alignment**: Header text must match the Montserrat/Indigo style of other pages.
- **Subsections**: Group the existing outbox table under a "Notifications" heading.
- **Access Control**: Ensure the page is visible to Compliance users while retaining the `/admin/` path for organizational clarity.

## 4. Design Constraints
- MUST use the `mako-style-guide` skill.
- MUST NOT use generic Tailwind blue/red colors for primary UI elements.
- Cards must use `rounded-2xl` and standard padding from the design system.
