# Feature Specification: Investment Tracker CLI

**Feature Branch**: `001-create-investment-tracker`  
**Created**: 2026-03-14  
**Status**: Draft  
**Input**: User description: "Create an investment tracker that allows to track my investments (deposits, ETFS, money, etc). It should be a CLI interface app, that is portable and its storage as well, opting for either JSON or SQLite. Rust should be the language of choice."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add and View Investments (Priority: P1)

As a user, I want to add my investments (deposits, ETFs, stocks, etc.) and view them in a list so I can track my portfolio.

**Why this priority**: This is the core functionality - without adding and viewing investments, the tracker has no value.

**Independent Test**: Can be fully tested by adding various investment types and verifying they appear in the list.

**Acceptance Scenarios**:

1. **Given** I have no investments, **When** I add a deposit investment, **Then** it should appear in my investment list
2. **Given** I have existing investments, **When** I add an ETF investment, **Then** it should be added to my existing list
3. **Given** I have investments, **When** I view the investment list, **Then** I should see all my investments with their details

---

### User Story 2 - Update and Delete Investments (Priority: P2)

As a user, I want to update investment details or delete investments so I can keep my portfolio accurate.

**Why this priority**: Important for maintaining accurate records, but secondary to the core add/view functionality.

**Independent Test**: Can be tested by adding investments, then updating/deleting them and verifying the changes.

**Acceptance Scenarios**:

1. **Given** I have an investment, **When** I update its value, **Then** the investment should show the updated value
2. **Given** I have an investment, **When** I delete it, **Then** it should no longer appear in my list
3. **Given** I try to delete a non-existent investment, **Then** I should get an appropriate error message

---

### User Story 3 - Portfolio Summary and Analytics (Priority: P3)

As a user, I want to see a summary of my portfolio including total value, gains/losses, and allocation by type so I can understand my investment performance.

**Why this priority**: Valuable analytics but not essential for basic tracking functionality.

**Independent Test**: Can be tested by adding investments with different values and verifying the calculations.

**Acceptance Scenarios**:

1. **Given** I have multiple investments, **When** I view portfolio summary, **Then** I should see total portfolio value
2. **Given** I have investments with different types, **When** I view portfolio summary, **Then** I should see allocation by investment type
3. **Given** I have investments with purchase and current values, **When** I view portfolio summary, **Then** I should see overall gains/losses

### Edge Cases

- What happens when trying to add an investment with invalid data (negative values, empty names)?
- How does the system handle concurrent access to the same data file?
- What happens when the storage file is corrupted or missing?
- How are decimal values and currency formatting handled?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow users to add investments with name, type, purchase value, current value, and date
- **FR-002**: System MUST allow users to view a list of all their investments  
- **FR-003**: Users MUST be able to update existing investment details
- **FR-004**: Users MUST be able to delete investments
- **FR-005**: System MUST provide a portfolio summary showing total value and allocation
- **FR-006**: System MUST persist data between sessions using either JSON or SQLite
- **FR-007**: System MUST be a command-line interface application
- **FR-008**: System MUST be portable (work across different operating systems)
- **FR-009**: System MUST be implemented in Rust

### Key Entities *(include if feature involves data)*

- **Investment**: Represents a single investment with attributes: id, name, investment_type (deposit, ETF, stock, etc.), purchase_value, current_value, purchase_date, notes
- **Portfolio**: Collection of investments belonging to a user, with calculated metrics: total_value, total_gain_loss, allocation_by_type

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can add, view, update, and delete investments successfully in under 1 minute per operation
- **SC-002**: Portfolio summary calculations are accurate within 0.01% of manual calculations
- **SC-003**: Data persistence works correctly - investments added in one session are available in subsequent sessions
- **SC-004**: Application starts and responds to commands within 2 seconds on a typical development machine
