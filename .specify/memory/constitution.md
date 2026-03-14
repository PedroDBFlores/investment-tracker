<!--
Sync Impact Report
- Version change: none -> 1.0.0
- Modified principles: n/a (new constitution) — 12 principles added
- Added sections: TDD Policy, Performance & UX Requirements, Adoption Plan
- Removed sections: none
- Templates requiring updates: 
  - .specify/templates/plan-template.md ✅ updated
  - .specify/templates/spec-template.md ✅ updated
  - .specify/templates/tasks-template.md ✅ updated
  - .specify/templates/commands/ ⚠ pending (directory not present)
- Follow-up TODOs:
  - RATIFICATION_DATE must be set by project maintainers
-->

# Investment Tracker Constitution

## Core Principles

### 1. Test-Driven Development (TDD) is the default
Description: Development MUST follow the red-green-refactor cycle: write failing tests before implementation, make them pass, then refactor.
Rationale: TDD produces verifiable requirements, reduces regressions, and enforces design driven by observable behavior.
Enforcement: CI must fail PRs that lack tests for new behavioral code; PR template requires test files and a test run summary. Automated check: PR lint script verifies that for every added/modified non-test source file there is a corresponding new/modified test file.
Examples: Create failing unit or integration tests in tests/unit or tests/integration before adding feature code; include a short before/after test log in PR description.

### 2. Code Quality: Clear, Small, and Reviewable
Description: Changes MUST be small, scoped to a single intent, and pass automated linters and static analysis.
Rationale: Small, focused PRs are easier to review, revert, and reason about; automated checks catch common issues early.
Enforcement: Enforce formatting (e.g., Prettier/black) and lint rules (ESLint/Pylint/clang-tidy) in CI; block merges if linter or static-analysis reports exceed configured severity thresholds.
Examples: Run `npm run lint` or `flake8` locally; use autofix where available and include lint report in CI artifacts.

### 3. Mandatory Test Coverage Floor (measurable)
Description: New code must increase or maintain repository-level coverage; each module must have a minimum coverage (unit + integration) of 80% for lines and 90% for critical modules.
Rationale: Coverage targets ensure tests exercise behavior and guard against regressions in high-risk areas.
Enforcement: CI collects coverage and fails if global or module coverage drops below thresholds. Coverage reports uploaded as CI artifacts and displayed on PR.
Examples: Use coverage tooling (coverage.py/nyc/istanbul) and add badge to README; for critical modules add extra integration tests to reach 90%.

### 4. Testing Types & Boundaries
Description: Use unit tests for pure logic, integration tests for component interactions, and end-to-end (E2E) tests for user workflows; every user-story PR must include at least unit and where applicable integration tests.
Rationale: Different test types catch different classes of bugs; TDD-first approach must apply across types to be effective.
Enforcement: CI pipeline stages: unit → integration → e2e (optional for small changes) with test selection based on changed files. PRs touching APIs or UX require integration or e2e tests respectively.
Examples: Add unit tests under tests/unit; integration tests under tests/integration using test doubles; use Playwright/Cypress for critical UI flows.

### 5. UX Consistency: Component and Interaction Guidelines
Description: UX patterns (spacing, typography, colors, interaction states) MUST follow the project's design tokens and component library; deviations require documented rationale and visual review.
Rationale: Consistent UX reduces user confusion and maintenance cost when building new screens or flows.
Enforcement: Visual regression tests (Percy or Chromatic) and a design-review checklist in PRs for UI changes; Storybook stories required for new/changed components.
Examples: Add Storybook story for each new component; include before/after screenshots in PR and run visual diff CI job.

### 6. Accessibility by Default
Description: All public UI MUST meet WCAG 2.1 AA standards for contrast, semantics, and keyboard navigation; accessibility checks are part of CI.
Rationale: Inclusive design broadens user base and reduces legal and support risks.
Enforcement: Run axe-core or pa11y in CI; block merges on high-severity violations; require manual a11y verification for complex widgets.
Examples: Include role and aria attributes in components; add automated a11y checks to Storybook CI jobs.

### 7. Performance Budget and Measurement
Description: Define measurable performance budgets (response times, bundle sizes, memory usage). New features MUST not violate budgets without justification and a mitigation plan.
Rationale: Performance degradation harms user experience and increases operational cost.
Enforcement: CI performance checks (Lighthouse, custom scripts) that fail on budget breaches for PRs labeled perf-impacting; performance regressions require a profiling artifact in PR.
Examples: Require Lighthouse CI for web frontend PRs; run `node scripts/check-bundle-size.js` to ensure bundle < 250KB gzipped.

### 8. Observability and Error Handling
Description: Code MUST include structured logging, metrics, and meaningful error messages; errors MUST be handled with graceful fallbacks where appropriate.
Rationale: Observability enables reliable debugging and faster incident resolution.
Enforcement: Lint or static checks for logging usage patterns; CI verifies presence of telemetry hooks in services (smoke tests/assertions); PR must reference related metric/log dashboards if adding instrumentation.
Examples: Use structured JSON logs, instrument latency counters, and add a smoke test that asserts telemetry events on integration environments.

### 9. Dependency and API Stability
Description: Public APIs and data contracts MUST be versioned; breaking changes require a documented migration path and approval from maintainer group.
Rationale: Predictable versioning prevents downstream breakages and supports safe evolution.
Enforcement: Require changelog entry and semantic versioning bump for breaking changes; CI checks for public API diffs (e.g., openapi-diff or public API diff tools) and block merges until approved.
Examples: Add `/openapi.yaml` diff job to CI; include migration guide in PR when changing schemas.

### 10. Security and Secrets Policy
Description: Secrets MUST not be committed; code MUST validate inputs and follow secure defaults (principle of least privilege).
Rationale: Security-first reduces risk of data leaks and exploitation.
Enforcement: Git hooks/CI secret scanning (e.g., truffleHog, GitGuardian) and SAST in CI; block merges on high-confidence leaks or critical SAST findings.
Examples: Use environment variables for credentials; add dependency security scan job (`npm audit`/`snyk`) in CI.

### 11. Observed Simplicity and YAGNI
Description: Prefer simple, explicit solutions; avoid premature abstractions—introduce complexity only when justified and documented.
Rationale: Simpler codebases are easier to maintain and review; YAGNI reduces long-term technical debt.
Enforcement: PR must include a short justification for new abstractions; architecture review required for changes impacting >2 modules.
Examples: Prefer clear functions over generic frameworks; document when a generalization is postponed.

### 12. Release and Rollback Discipline
Description: Release processes MUST include automated checks and a tested rollback plan; production changes require a monitored rollout (canary/gradual) for high-risk releases.
Rationale: Controlled releases reduce blast radius and make rollbacks fast and predictable.
Enforcement: CI must run pre-release smoke tests and execute post-deploy health checks; require a rollback playbook linked in release notes for any production change.
Examples: Use feature flags with gradual traffic, add health-check endpoint tests in deployment pipeline.

## TDD Policy

TDD Workflow Steps:
1. Red: Author writes one or more failing tests describing the desired behavior (unit, integration, or e2e). Tests MUST be included in the feature branch before implementation commits.
2. Green: Implement the minimal code to make tests pass. Keep the change minimal and focused on the test behavior.
3. Refactor: Clean up code while keeping tests passing; improve design, remove duplication, and add comments where it aids clarity.

Required PR Artifacts:
- Failing test run evidence (CI screenshot or paste) in the PR description OR test history demonstrating the red state prior to implementation.
- Tests included in tests/unit, tests/integration, or tests/e2e with clear names and assertions.
- Small, focused commits that separate test addition from implementation where practical.

Definition of Done (DoD) for tasks:
- All new behavior has at least one passing test that would have failed before implementation.
- No tests are skipped or marked xfail to meet DoD; flaky tests must be fixed or quarantined with a remediation plan.
- Relevant documentation (README, docstring, Storybook) updated when behavior or component API changes.

Minimum Test Types and Coverage Expectations:
- Unit tests: mandatory for logic-heavy code; aim for 80% lines coverage per module.
- Integration tests: required for API contracts, persistence, cross-service flows; no hard coverage floor but must cover critical paths.
- E2E tests: required for end-user critical journeys (checkout, onboarding); limited to essential flows to avoid brittleness.

Recommended Tools & CI Integration:
- Unit: pytest (Python), Jest (JS/TS) with coverage tooling (coverage.py/nyc).
- Integration: pytest with fixtures, Docker-based test environments, or Testcontainers where helpful.
- E2E: Playwright or Cypress, run in separate CI stage with failure visibility.
- CI gates: run unit → coverage check → integration → e2e (if labeled). Merge blocked on failing gates.

Handling Flaky Tests and Ownership:
- Flaky tests must be triaged within 48 hours of detection. Flaky test policy: quarantine only with an associated bug/issue and a 2-week remediation SLT.
- Each test suite must have an owner (team or individual) listed in TEST_OWNERS file. Owners respond to failing flaky test alerts and maintain stability.

## Performance & UX Requirements

Targets (example, adapt to project context):
- API latency: p95 < 200ms for user-facing endpoints under normal load; 99th percentile under load testing documented.
- Web perceived load: Time to First Contentful Paint (FCP) < 1.2s on 3G simulated mobile; Time to Interactive (TTI) < 3s.
- Bundle size: initial JS bundle gzipped < 250KB; incremental PRs limited to +50KB without justification.
- Memory: per-process RSS < 200MB for backend services under typical load profile.

How to measure:
- CI performance stage: Lighthouse CI for front-end metrics; bespoke scripts for API p95/p99 using k6/loadtest against a staging environment; bundle-size check script (e.g., webpack-bundle-analyzer or rollup-plugin-filesize).
- Continuous monitoring: SLOs and alerts defined in production monitoring (Datadog/Prometheus + Grafana). Automated regression detection alerts on breaches.

CI Gates and Monitoring Rules:
- PR-level gates: run Lighthouse CI (frontend) and bundle size check; fail PR if budgets exceeded.
- Pre-merge performance run: for perf-impacting PRs (label: perf), run a short k6 profile and attach summary to PR; fail if p95 regresses >20% vs baseline.
- Post-deploy: exports of real-user metrics (RUM) and alerting on SLO breaches; incident playbooks linked in monitoring dashboards.

## Adoption Plan (3-5 steps)

1. Documentation & Templates (Week 0): Publish this constitution in .specify/memory/constitution.md, update PR and issue templates, and add TEST_OWNERS and Performance baseline artifacts to repo.
2. CI Enforcement (Week 1): Add CI jobs for linting, coverage, test gating, visual diff, Lighthouse CI, and bundle-size checks. Enable blocking merge for critical gates.
3. Training & Ramp (Weeks 1-3): Run two 90-minute workshops on TDD, test writing, and performance checks; pair-programming sessions for first three TDD PRs.
4. Enforcement Cadence (Weeks 3-8): Maintain a weekly audit (automated) for uncovered rule violations; rotate a constitution steward to review exceptions and approve migration plans.
5. Retro & Iteration (Month 2): Run a retrospective on adoption, update constitution rules where practical friction exists, and bump minor/patch version for clarifications.

## Governance

Amendments:
- Minor or patch updates (clarifications, wording) MAY be approved by maintainers and recorded with a patch version increment.
- Material additions (new mandatory principle) or behavioral changes (e.g., change in test coverage floor) require a minor version bump and consensus from maintainers.
- Breaking governance changes (removing TDD mandate, changing security policy) require a major version bump and explicit approval from project owners.

Versioning and Dates:
- Version: 1.0.0 | Ratified: TODO(RATIFICATION_DATE): set the formal ratification date when accepted by maintainers | Last Amended: 2026-03-14

Compliance and Review:
- PR template will include a constitution checklist; CI will run automated gates where possible. Exceptions require documented justification and an owner.
- Quarterly compliance review: automated scans + steward review to ensure rules are followed; outstanding exceptions tracked as issues.



