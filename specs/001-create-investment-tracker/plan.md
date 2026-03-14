# Development Plan: Investment Tracker CLI

**Feature**: 001-create-investment-tracker  
**Created**: 2026-03-14  
**Status**: Planning  
**Language**: Rust

## Overview

Create a command-line investment tracker application in Rust that allows users to track various types of investments (deposits, ETFs, stocks, etc.) with portable storage using JSON or SQLite.

## Implementation Strategy

### Phase 1: Core Functionality (P1 - Add/View Investments)

**Goal**: Implement the basic CRUD operations for investments

**Tasks**:
- [ ] Set up Rust project structure with Cargo
- [ ] Implement investment data model (struct with fields: id, name, investment_type, purchase_value, current_value, purchase_date, notes)
- [ ] Implement JSON-based storage system
- [ ] Create CLI interface using clap or structopt
- [ ] Implement add investment command
- [ ] Implement list investments command
- [ ] Implement basic error handling
- [ ] Write unit tests for core functionality

**Estimated Time**: 3-5 days

**Dependencies**: None

### Phase 2: Update/Delete Operations (P2)

**Goal**: Add editing and deletion capabilities

**Tasks**:
- [ ] Implement update investment command
- [ ] Implement delete investment command
- [ ] Add input validation for all operations
- [ ] Implement proper error messages
- [ ] Add confirmation prompts for destructive actions
- [ ] Write integration tests

**Estimated Time**: 2-3 days

**Dependencies**: Phase 1 completion

### Phase 3: Portfolio Analytics (P3)

**Goal**: Add portfolio summary and analytics features

**Tasks**:
- [ ] Implement portfolio summary calculation (total value, gains/losses)
- [ ] Implement allocation by investment type
- [ ] Add summary display command
- [ ] Implement data export functionality (CSV/JSON)
- [ ] Add performance metrics over time
- [ ] Write tests for calculations

**Estimated Time**: 3-4 days

**Dependencies**: Phase 2 completion

### Phase 4: Polish and Deployment

**Goal**: Final touches and deployment preparation

**Tasks**:
- [ ] Add comprehensive error handling
- [ ] Implement data backup/restore functionality
- [ ] Add user configuration options
- [ ] Create man page/documentation
- [ ] Set up CI/CD pipeline
- [ ] Package for distribution (cargo install, homebrew, etc.)
- [ ] Final testing and bug fixing

**Estimated Time**: 2-3 days

**Dependencies**: Phase 3 completion

## Technical Decisions

### Storage Choice: JSON vs SQLite

**Decision**: Start with JSON for simplicity, with SQLite as future enhancement

**Rationale**:
- JSON is simpler to implement initially
- Better for portable single-file storage
- Easier to debug and manually edit if needed
- Can migrate to SQLite later if performance becomes an issue

### CLI Framework

**Decision**: Use clap for command-line parsing

**Rationale**:
- Most popular Rust CLI framework
- Good documentation and community support
- Supports subcommands naturally
- Active development and maintenance

### Data Model

**Decision**: Use Rust structs with serde for serialization

**Rationale**:
- Type safety with Rust's type system
- Easy serialization/deserialization with serde
- Good performance
- Integration with JSON storage

## Testing Strategy

### Unit Testing
- Test individual functions (add, update, delete, calculations)
- Test data model serialization/deserialization
- Test validation logic

### Integration Testing
- Test complete workflows (add then list, update then verify)
- Test error scenarios
- Test data persistence across sessions

### Manual Testing
- Test on different operating systems (macOS, Linux, Windows)
- Test with various input scenarios
- Test edge cases and error conditions

## Risk Assessment

### High Risk Items
- **Data corruption**: If storage format changes or errors occur during write
  - Mitigation: Implement backup functionality, validation on load
- **Cross-platform compatibility**: Different filesystem behaviors
  - Mitigation: Use std::path, test on multiple platforms

### Medium Risk Items
- **Performance with large datasets**: JSON may become slow with many investments
  - Mitigation: Start with JSON, monitor performance, migrate to SQLite if needed
- **User input validation**: Ensuring all edge cases are covered
  - Mitigation: Comprehensive testing, use established validation crates

### Low Risk Items
- **CLI interface design**: Can be iterated on based on user feedback
- **Analytics calculations**: Straightforward math operations

## Success Metrics

- All P1 user stories implemented and tested
- All P2 user stories implemented and tested  
- All P3 user stories implemented and tested
- Application works on macOS, Linux, and Windows
- Data persists correctly between sessions
- Response time under 2 seconds for all operations
- Comprehensive test coverage (>80%)

## Timeline

- **Week 1**: Phase 1 (Core functionality)
- **Week 2**: Phase 2 (Update/Delete) + Phase 3 (Analytics)
- **Week 3**: Phase 4 (Polish) + Testing
- **Week 4**: Buffer for unexpected issues, final testing

## Resources

- Rust documentation
- clap crate documentation
- serde documentation
- JSON storage examples
- CLI best practices guides