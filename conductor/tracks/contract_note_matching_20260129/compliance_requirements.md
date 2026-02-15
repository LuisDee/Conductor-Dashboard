# Compliance Requirements for Contract Note & Activity Statement Matching

## Overview
System needs to handle two types of documents:
1. **Contract Notes** - Individual trade confirmations
2. **Activity Statements** - Monthly statements showing positions and trades

## Key Information from Compliance Team (Joana Filipova)

### Document Sources
1. **Interactive Brokers (Primary Source)**
   - Automatic statements sent to compliance email
   - Trade notifications - sent immediately after trade execution
   - Monthly statements - show current positions + trades during month
   - **Always includes full user name**
   - Standardized format

2. **Other Brokers**
   - HSBC and others - manual submission
   - May have limited identifying information (e.g., "MR B Smith")

### Document Types & Identification

#### Contract Notes / Trade Notifications
- Sent immediately after trade execution
- Contains specific trade details
- Example: `/Users/luisdeburnay/Desktop/contract_note_1.pdf`
- May only have partial name (e.g., "MR B Basini")

#### Activity Statements
- Monthly consolidated statements
- Example: `/Users/luisdeburnay/Desktop/ActivityStatement.202512.pdf`
- Contains:
  - Account overview
  - **"Trades" section** - if present, indicates transactions occurred
  - If "Trades" section missing = no new transactions (just positions)
- Usually has full user name

### User Matching Strategy

#### Reliable Matching Scenarios
1. **Email-based matching**:
   - If received from user's @mako.com email → automatic match
   - Email sender identification is most reliable

2. **Manual upload via portal**:
   - User already authenticated → automatic match
   - Network connection identifies user

3. **Interactive Brokers statements**:
   - Full name available → high confidence match

#### Edge Cases Requiring Manual Review
1. Broker sends PDF directly (no user email)
2. Only partial name in PDF (e.g., "MR B Basini")
3. Multiple potential matches in database

### Proposed Manual Matching UI
- Split-screen interface:
  - Left: Unmatched PDF viewer
  - Right: List of pending trades
  - Drag-and-drop functionality for manual matching
- Notification system for unmatched documents

### Important Fields to Extract

#### From Activity Statements:
- User full name
- Account overview
- **Trades section** (critical - determines if transaction occurred)
- Positions/holdings

#### From Contract Notes:
- User name (may be partial)
- Trade details:
  - Instrument
  - Quantity
  - Price
  - Trade date
  - Settlement date

### Business Rules
1. Statements with "Trades" section = trade confirmation required
2. Statements without "Trades" section = position statement only (no action)
3. Interactive Brokers formats are standardized and reliable
4. Edge cases should fail gracefully to manual review queue

### Testing Considerations
- Cannot test GCS/cloud functions locally
- Need descriptive test names for Gemini debugging
- Tests should clearly indicate expected vs actual behavior
- Focus on matching logic that can be tested in isolation