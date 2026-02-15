# Track Specification: MAKO Design System Implementation

## 1. Goal
Transform the PA Dealing Compliance Dashboard to match MAKO's official design system with exact color palette, typography, spacing, and component styling as defined in `MAKO_UI_SYSTEM_PROMPT.md`.

## 2. Background
The current dashboard uses generic Tailwind CSS defaults (blue #0ea5e9, slate grays). This track implements the **official MAKO design system** with:
- MAKO Navy (#0E1E3F) for sidebar and primary text
- MAKO Blue (#5471DF) for buttons and active states
- Montserrat font family
- Precise spacing, shadows, and border-radius values
- Official status color mapping (Gold for warnings, not red)

**Critical:** This is NOT a generic "make it look nice" task. Every color, font weight, spacing value, and shadow MUST match the system prompt exactly.

## 3. Scope

### Phase 1: Design Tokens & Foundation
**Files:** `tailwind.config.js`, `index.css`, new `design-tokens.css`

- [x] Replace Tailwind primary colors with MAKO colors
- [x] Import Montserrat font from Google Fonts
- [x] Define CSS custom properties for MAKO design tokens
- [x] Ban generic Tailwind colors (slate, indigo, generic blue/red)
- [x] Create shadow utility classes with navy-based shadows
- [x] Define spacing scale (8px base)

**MAKO Color Palette:**
```css
--navy:      #0E1E3F   /* Sidebar, text */
--blue:      #5471DF   /* Buttons, active states */
--gold:      #B28C54   /* Warnings, pending */
--light-blue:#DBE1F5   /* Hover states */
--gray:      #F4F4F4   /* Page background */
--success:   #2C5F2D   /* Approved, success */
--error:     #B85042   /* Critical, breaches */
--tertiary:  rgb(139, 159, 233) /* Subtitles */
```

**Typography Scale:**
```css
--font-family: 'Montserrat', sans-serif
--page-title: 800 weight, 32px
--section-title: 600 weight, 24px
--body: 400 weight, 14px
--label: 600 weight, 11px uppercase, letter-spacing 0.08em
--subtitle: 400 weight, 14px, rgb(139, 159, 233)
--stat-value: 800 weight, 42px
```

### Phase 2: Logo & Header
**Files:** `Sidebar.tsx`, new logo asset handling

- [x] Copy `Mako logo white on blue.png` to `dashboard/src/assets/`
- [x] Add logo to top of sidebar (220px width sidebar, #0E1E3F background)
- [x] Logo display:
  - Image: Mako logo (white on blue background already in asset)
  - Below logo: "PA DEALING" (Montserrat 800, white, 16px)
  - Below that: "Compliance Suite" (10px uppercase, letter-spacing 2px, rgb(139, 159, 233))
- [x] Spacing: Logo + text group has 20px padding, 24px bottom margin

**Logo Component Structure:**
```tsx
<div className="logo-container">
  <img src={makoLogo} alt="MAKO" className="logo-image" />
  <div className="logo-text-primary">PA DEALING</div>
  <div className="logo-text-subtitle">COMPLIANCE SUITE</div>
</div>
```

### Phase 3: Sidebar Navigation
**Files:** `Sidebar.tsx`

- [x] Background: #0E1E3F (navy) - EXACT hex, not approximation
- [x] Width: 220px fixed
- [x] Section headers: 10px uppercase, letter-spacing 1.5px, rgba(255,255,255,0.35)
- [x] Nav items: 14px, 500 weight, rgba(255,255,255,0.65)
- [x] Nav item hover: background rgba(84, 113, 223, 0.2)
- [x] Nav item active: background #5471DF, white text, 12px border-radius
- [x] Nav item padding: 10px 14px
- [x] Remove any Tailwind slate colors

### Phase 4: Card Components
**Files:** `Card.tsx`, all page components

- [x] Background: #FFFFFF (pure white)
- [x] Border: NONE (delete any border properties)
- [x] Shadow: `0 2px 8px rgba(14, 30, 63, 0.08)` - navy-based, not black
- [x] Border-radius: 12px (all cards)
- [x] Padding: 20-24px
- [x] Top accent bar: 4px height, semantic color (blue/gold/green)

**Stat Card Pattern:**
```css
.stat-card::before {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 4px;
  background: #5471DF; /* or semantic color */
}
```

### Phase 5: Button System
**Files:** Button components, all interactive elements

- [x] Primary: #5471DF background, white text, 8px radius
- [x] Primary hover: #4661C9 with `0 4px 16px rgba(14, 30, 63, 0.12)` shadow
- [x] Secondary: #DBE1F5 background, #0E1E3F text
- [x] Danger: #B85042 (NOT Tailwind red)
- [x] Font: 600 weight, 14px Montserrat
- [x] Padding: 8px 16px
- [x] Remove ALL Tailwind button classes

### Phase 6: Table Styling
**Files:** All table components (PendingApprovals, Breaches, etc.)

- [x] Header background: #DBE1F5 (light blue)
- [x] Header text: 600 weight, 13px uppercase, letter-spacing 0.04em
- [x] Header border-bottom: 2px solid #5471DF
- [x] Row border-bottom: 1px solid rgba(14, 30, 63, 0.08)
- [x] Row hover: background rgba(219, 225, 245, 0.4)
- [x] Cell padding: 16px

### Phase 7: Form Inputs
**Files:** All form components, RequestDetail, Settings

- [x] Border: 1px solid rgba(14, 30, 63, 0.2)
- [x] Border-radius: 8px
- [x] Padding: 12px 16px
- [x] Font: 400 weight, 14px Montserrat
- [x] Focus: border-color #5471DF, box-shadow `0 0 0 3px rgba(84, 113, 223, 0.15)`

### Phase 8: Badge Components
**Files:** Status badges across all pages

- [x] Font: 600 weight, 10px uppercase, letter-spacing 0.04em
- [x] Padding: 4px 12px
- [x] Border-radius: 9999px (pill shape)
- [x] Primary: #5471DF background, white text
- [x] Success: rgba(44,95,45,0.12) background, #2C5F2D text
- [x] Warning: rgba(178,140,84,0.12) background, #B28C54 text (GOLD not red!)
- [x] Error: rgba(184,80,66,0.12) background, #B85042 text

### Phase 9: Page Layout
**Files:** App.tsx, all page components

- [x] Sidebar: 220px, #0E1E3F
- [x] Main content: background #F4F4F4 (light gray page background)
- [x] Main padding: 32px
- [x] Page title: Montserrat 800, 32px, #0E1E3F
- [x] Subtitle: 14px, rgb(139, 159, 233) - NOT generic gray!
- [x] Card gaps: 20px
- [x] Section gaps: 32px

### Phase 10: Status Color Mapping
**Files:** All components showing status

Apply semantic colors consistently:
| Status | Color | Component Example |
|--------|-------|-------------------|
| Requires Action | Gold #B28C54 | Pending approvals badge |
| Warning | Gold #B28C54 | Risk indicators |
| Success | Green #2C5F2D | Approved badge |
| Error | Red #B85042 | Breach indicator |
| Info/Neutral | Blue #5471DF | In-progress |

**Critical:** Warnings use GOLD (#B28C54), NOT red. This is MAKO standard.

## 4. Pages to Update (ALL)

- [x] Dashboard (overview with stat cards)
- [x] PendingApprovals (table)
- [x] MyRequests (table)
- [x] RequestDetail (form, cards)
- [x] Breaches (table, cards)
- [x] ExecutionTracking (table)
- [x] HoldingPeriods (calendar view)
- [x] MakoConflicts (table)
- [x] AuditLog (table)
- [x] RiskScoringConfig (forms, cards)
- [x] AccuracyMetrics (stats, charts)

## 5. Technical Requirements

### Configuration Changes
```javascript
// tailwind.config.js - REPLACE primary/slate with MAKO colors
theme: {
  extend: {
    colors: {
      navy: '#0E1E3F',
      blue: '#5471DF',
      gold: '#B28C54',
      'light-blue': '#DBE1F5',
      'mako-gray': '#F4F4F4',
      success: '#2C5F2D',
      error: '#B85042',
    },
    fontFamily: {
      sans: ['Montserrat', 'sans-serif'],
    },
    boxShadow: {
      'mako-card': '0 2px 8px rgba(14, 30, 63, 0.08)',
      'mako-hover': '0 4px 16px rgba(14, 30, 63, 0.12)',
      'mako-modal': '0 8px 32px rgba(14, 30, 63, 0.16)',
    },
  },
},
```

### Font Import
```css
/* index.css */
@import url('https://fonts.googleapis.com/css2?family=Montserrat:wght@400;500;600;700;800&display=swap');
```

### CSS Custom Properties
```css
:root {
  --navy: #0E1E3F;
  --blue: #5471DF;
  --gold: #B28C54;
  --light-blue: #DBE1F5;
  --gray: #F4F4F4;
  --success: #2C5F2D;
  --error: #B85042;
  --tertiary: rgb(139, 159, 233);

  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --space-12: 48px;
}
```

## 6. Validation Checklist

Before marking this track complete, verify EVERY item:

- [x] Sidebar is #0E1E3F (not #1E293B or other approximation)
- [x] Active nav states use #5471DF (not #3B82F6)
- [x] Cards have NO border, only shadow
- [x] All shadows use rgba(14, 30, 63, X) not black rgba
- [x] Montserrat font loaded and applied to all text
- [x] Page titles are 800 weight
- [x] Subtitles are rgb(139, 159, 233) NOT gray
- [x] Warnings use #B28C54 gold, NOT red
- [x] Card border-radius is 12px
- [x] Button border-radius is 8px
- [x] Labels are uppercase with letter-spacing
- [x] Logo displays correctly in sidebar
- [x] "PA DEALING" text uses Montserrat 800
- [x] "COMPLIANCE SUITE" uses correct tertiary color
- [x] No Tailwind slate/indigo/generic colors remain
- [x] All interactive states (hover, focus, active) use MAKO colors
- [x] Table headers use #DBE1F5 background
- [x] Status badges use semantic MAKO colors

## 7. Testing Requirements

- [x] Visual inspection: Compare each page against MAKO_UI_SYSTEM_PROMPT.md
- [x] Color audit: Use browser dev tools to verify exact hex values
- [x] Font audit: Confirm Montserrat loads and weights are correct
- [x] Responsive test: Verify on mobile/tablet viewports
- [x] Cross-browser: Chrome, Firefox, Safari
- [x] Accessibility: Check contrast ratios (should be maintained)
- [x] Screenshot comparison: Before/after for documentation

## 8. Acceptance Criteria

- [x] Logo visible in sidebar with correct text
- [x] ALL pages use MAKO color palette exclusively
- [x] NO Tailwind generic colors remain (slate, indigo, default blue/red/green)
- [x] Montserrat font applied to all text
- [x] Typography scale matches system prompt (weights, sizes)
- [x] Cards follow exact shadow/radius specifications
- [x] Buttons use MAKO blue (#5471DF)
- [x] Status colors use semantic mapping (gold for warnings)
- [x] Navigation active states use #5471DF
- [x] All spacing uses 8px-based scale
- [x] Visual consistency across all 11+ pages
- [x] Passes validation checklist 100%

## 9. Dependencies
- MAKO logo asset: `Mako logo white on blue.png` (available in $CWD)
- Google Fonts: Montserrat family
- MAKO_UI_SYSTEM_PROMPT.md (reference document)

## 10. Out of Scope
- Slack Bot UI (limited customization in Slack)
- PDF/document generation styling
- Email template styling
- Backend logging colors
- Database schema changes

## 11. Timeline Estimate
- Phase 1 (Tokens): 0.5 day
- Phase 2 (Logo): 0.25 day
- Phase 3 (Sidebar): 0.5 day
- Phase 4-5 (Cards/Buttons): 1 day
- Phase 6-8 (Tables/Forms/Badges): 1.5 days
- Phase 9 (Layout): 0.5 day
- Phase 10 (Status colors): 0.5 day
- Testing & validation: 1 day

**Total:** 5-6 days

## 12. Success Metrics
- Zero generic Tailwind colors in production
- 100% of pages match MAKO design system
- Logo visible and correctly branded
- Stakeholder approval from Compliance team
- Brand consistency with other MAKO systems

---

**Reference Document:** `/home/coder/repos/ai-research/pa-dealing/MAKO_UI_SYSTEM_PROMPT.md`
**Logo Asset:** `/home/coder/repos/ai-research/pa-dealing/Mako logo white on blue.png`
