# Testing Strategy

Date: 2026-10-30
Scope: Deterministic test architecture before full feature implementation.

## 1. Test Layers

Unit tests:
- State machine transitions.
- Retry policy/backoff calculations.
- URL/path validators.
- Segment planning and merge accounting.

Integration tests:
- Engine + storage round trips.
- Pause/resume with persisted metadata.
- Recovery from simulated restart.

Security tests:
- Traversal/symlink defenses.
- Log redaction.
- Invalid TLS/redirect edge behavior assertions.

UI tests (phase-gated):
- Basic launch smoke tests.
- Core action wiring (add/pause/resume/cancel).

## 2. Deterministic Local Fixture Server

Capabilities required:
- Successful download (small/large fixtures)
- Redirect chains
- No-range support
- Range support
- 403, 404, 416, 429, 500
- Timeouts and interrupted responses
- Malformed Content-Range
- Changed ETag across resume attempt
- Incomplete transfer simulation

## 3. Critical Scenarios

- Single-stream download success.
- Segmented download success with non-overlapping ranges.
- Fallback from segmented to single stream.
- Retry exhaustion into Failed.
- Pause/resume preserving bytes and validators.
- Crash/restart recovery from Downloading and Verifying states.
- Final output byte-for-byte fixture match.

## 4. Tooling

- cargo test for unit/integration tests.
- tokio::test for async workflows.
- tempdir/tempfile for isolated filesystem tests.
- Optional: insta snapshots for structured diagnostics outputs.

## 5. Quality Gates

Per milestone gate:
- cargo fmt --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test --all

## 6. Flakiness Controls

- No external internet dependencies in tests.
- Deterministic fixture content and seeded timing where needed.
- Explicit timeout budgets with diagnostic failure messages.

## 7. Evidence Policy

- Do not claim any feature is complete without passing tests for happy and failure paths.
