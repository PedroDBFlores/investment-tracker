# Investment Tracker Project Plan

## Overview
Create a CLI investment tracker in Rust with portable storage (JSON/SQLite).

## Phase 1: Core Structure
- [ ] Set up Rust project with Cargo
- [ ] Create basic CLI interface with clap
- [ ] Implement storage layer (JSON first, SQLite later)
- [ ] Define investment data models

## Phase 2: Core Features
- [ ] Add investment entry (deposits, ETFs, stocks)
- [ ] List all investments
- [ ] View investment details
- [ ] Update investment records
- [ ] Delete investments

## Phase 3: Advanced Features
- [ ] Portfolio valuation
- [ ] Performance tracking
- [ ] Dividend tracking
- [ ] Export/import functionality
- [ ] Basic analytics

## Phase 4: Polish
- [ ] Error handling and validation
- [ ] Help documentation
- [ ] Configuration system
- [ ] Testing suite

## Development Approach
- **TDD**: Test-Driven Development throughout all phases
- Write tests before implementation for all features
- Minimum 80% test coverage requirement
- Integration tests for critical flows
- Property-based testing for data validation

## Storage Format
Primary: JSON files (portable, human-readable)
Alternative: SQLite (for larger datasets)

## Tech Stack
- Rust (primary language)
- clap (CLI parsing)
- serde (JSON serialization)
- rusqlite (SQLite support)
- anyhow/thiserror (error handling)