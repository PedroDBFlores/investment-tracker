# Project Constitution

## Code Quality Principles
1. **Readability First**: Code must be self-documenting with clear naming conventions
2. **Consistent Style**: Follow language-specific style guides (PEP 8 for Python, etc.)
3. **Modular Design**: Components should have single responsibilities and clear interfaces
4. **Documentation**: All public APIs and complex logic must have docstrings/comments
5. **No Dead Code**: Remove unused imports, functions, and variables immediately

## Testing Standards
1. **Coverage**: Minimum 80% test coverage for all production code
2. **Unit Tests**: Isolate and test individual components
3. **Integration Tests**: Verify component interactions
4. **E2E Tests**: Critical user flows must have end-to-end tests
5. **Test Naming**: `test_<function>_<scenario>_<expected_result>` format
6. **Mocking**: Use mocks sparingly, prefer real dependencies when possible

## User Experience Consistency
1. **Design System**: Follow established UI patterns and component library
2. **Error Handling**: User-friendly error messages with recovery options
3. **Loading States**: Clear feedback during async operations
4. **Accessibility**: WCAG 2.1 AA compliance minimum
5. **Responsive**: Mobile-first approach with progressive enhancement

## Performance Requirements
1. **Load Times**: < 2s for primary user flows on median mobile devices
2. **API Response**: < 500ms for 95th percentile
3. **Bundle Size**: < 1MB for initial JavaScript payload
4. **Memory**: No memory leaks in long-running sessions
5. **Optimization**: Lazy load non-critical resources

## Review Process
1. **PR Requirements**: Tests passing, documentation updated, changelog entry
2. **Approval**: Minimum 2 reviewers for all changes
3. **Feedback**: Constructive, actionable, and kind communication
4. **Merge**: Only after CI passes and all discussions resolved