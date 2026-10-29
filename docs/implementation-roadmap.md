# Implementation Roadmap

Date: 2026-10-30
Status: Milestone plan aligned with documented architecture.

## Milestone 1: Foundation and workspace bootstrap

Deliverables:
- Cargo workspace scaffolding.
- Core crate interfaces and typed errors.
- Storage crate with initial migrations.
- CI-local commands documented.

Exit criteria:
- Workspace builds.
- Basic tests pass.

## Milestone 2: Minimal transfer engine

Deliverables:
- HTTP/HTTPS single-stream downloads.
- Redirect handling.
- Progress events.
- Cancellation and retries.
- Safe temp writes and finalization.

Exit criteria:
- Deterministic success/failure integration tests pass.

## Milestone 3: Persistent queue and lifecycle

Deliverables:
- Add/pause/resume/cancel/retry/remove APIs.
- SQLite-backed queue and history.
- Startup recovery of unfinished tasks.

Exit criteria:
- Recovery and transition tests pass.

## Milestone 4: Resumable downloads with validators

Deliverables:
- Range request resume flow when valid.
- ETag/Last-Modified persistence and revalidation.
- Safe restart on validator mismatch.

Exit criteria:
- Resume validity and mismatch tests pass.

## Milestone 5: Segmented transfers

Deliverables:
- Range capability discovery.
- Non-overlapping segment planning and workers.
- Segment response validation.
- Fallback logic.

Exit criteria:
- Segment correctness and fallback tests pass.

## Milestone 6: GTK4/libadwaita desktop UI

Deliverables:
- Main list and add-download dialog.
- Pause/resume/cancel actions.
- Status filtering and details view.
- Basic notifications and settings.

Exit criteria:
- App launches and controls function against live engine.

## Milestone 7: Hardening and security testing

Deliverables:
- Expanded security/race/fault tests.
- Improved diagnostics and redaction.
- Resource limit tuning and documentation.

Exit criteria:
- Security regression suite passes.

## Milestone 8: Packaging and release process

Deliverables:
- Debian packaging instructions and scripts.
- Flatpak evaluation document and constraints.
- Release checklist and reproducibility notes.

Exit criteria:
- Documented build/package process reproducibly completes on Ubuntu baseline.

## Immediate next implementation scope

- Begin Milestone 1 after docs are reviewed.
- Keep each crate compiling at each commit step.
- Run fmt/clippy/test gates after each milestone increment.
