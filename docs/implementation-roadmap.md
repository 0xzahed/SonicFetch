# Implementation Roadmap

Date: 2026-10-30
Status: Single-download CLI implementation is present but not build-verified; feature parity and acceptance criteria remain open.

## Implemented in this repository

- Rust workspace and typed download lifecycle model.
- SQLite queue with schema initialization, listing, progress persistence, and restart recovery.
- HTTP/HTTPS streaming to same-directory temporary files and safe no-overwrite publication.
- Cancellation, pause/resume with Range and If-Range validators, and manual retry.
- Cross-process pause/cancel via SQLite worker leases, plus interrupted-worker recovery.
- Safe final publication without replacing existing files and recovery across the finalization window.
- CLI for add/list/pause/resume/retry/cancel/remove.
- URL scheme and credential validation, output filename sanitization, and URL redaction in logs.
- Local Debian package build script and Flatpak evaluation notes.
- Unit/integration test source for lifecycle rules, SQLite recovery, redirects, resume, retry, conflicts, cancellation, and symlink rejection.
- No downloaded-file execution.

These items describe source code present in the tree. Tests have not run because Cargo is unavailable here, so this is not a release claim.

## Milestone 1: Foundation and workspace bootstrap — implemented, unverified

Deliverables:
- Cargo workspace scaffolding.
- Core crate interfaces and typed errors.
- Storage crate with initial migrations.
- CI-local commands documented.

Exit criteria:
- Workspace builds.
- Basic tests pass.

## Milestone 2: Minimal transfer engine — implemented, unverified

Deliverables:
- HTTP/HTTPS single-stream downloads.
- Redirect handling.
- Progress events.
- Cancellation and retries.
- Safe temp writes and finalization.

Exit criteria:
- Deterministic success/failure integration tests pass.

## Milestone 3: Persistent queue and lifecycle — partially implemented, unverified

Deliverables:
- Add/pause/resume/cancel/manual-retry/remove APIs and SQLite-backed task records.
- Startup recovery of unfinished tasks and finalization recovery.
- History table exists in the schema; history event writes and UI are pending.

Exit criteria:
- Recovery and transition tests pass.

## Milestone 4: Resumable downloads with validators — partially implemented, unverified

Deliverables:
- Range request resume flow when valid.
- ETag/Last-Modified persistence and revalidation.
- Safe restart on validator mismatch.

Exit criteria:
- Resume validity and mismatch tests pass.

## Milestone 5: Segmented transfers — not implemented

Deliverables:
- Range capability discovery.
- Non-overlapping segment planning and workers.
- Segment response validation.
- Fallback logic.

Exit criteria:
- Segment correctness and fallback tests pass.

## Milestone 6: GTK4/libadwaita desktop UI — not implemented

Deliverables:
- Main list and add-download dialog.
- Pause/resume/cancel actions.
- Status filtering and details view.
- Basic notifications and settings.

Exit criteria:
- App launches and controls function against live engine.

## Milestone 7: Hardening and security testing — partial source coverage only

Deliverables:
- Expanded security/race/fault tests.
- Improved diagnostics and redaction.
- Resource limit tuning and documentation.

Exit criteria:
- Security regression suite passes.

## Milestone 8: Packaging and release process — instructions/scripts present, unverified

Deliverables:
- Debian packaging instructions and scripts.
- Flatpak evaluation document and constraints.
- Release checklist and reproducibility notes.

Exit criteria:
- Documented build/package process reproducibly completes on Ubuntu baseline.

## Milestone 9: Native application integration and queue controls — not implemented

Deliverables:
- Multiple named queues, bounded concurrency, priorities, retry budgets, and scheduled start/stop.
- Categories and file-type destination rules.
- Per-download and queue speed limits/quotas.

Exit criteria:
- Queue scheduling and restart behavior pass deterministic tests and are controllable from the native UI.

## Milestone 10: Linux browser and desktop integration — not implemented

Deliverables:
- GTK4/libadwaita UI with search, filtering, history, details, settings, notifications, and theme support.
- Opt-in browser extension for supported Linux browsers, with explicit review before capturing a link.
- Download-all/media candidate selection, clipboard monitoring, and drag-and-drop behind user controls.

Exit criteria:
- Browser and desktop integration is permission-scoped, visible, tested, and off until enabled.

## Milestone 11: Protocols and advanced integrations — not implemented

Deliverables:
- FTP and evaluate MMS support, documented proxy and server authentication, and scoped Site Grabber.
- Optional scanner integration that invokes only a user-configured scanner, never the downloaded file.
- Signed update metadata and user-controlled updates.
- Translations, interface customization, and release hardening.

Exit criteria:
- Each integration has deterministic protocol/security tests, clear consent, and documented support boundaries.

## Remaining implementation scope

- Add deterministic integration and security coverage for the implemented transfer path.
- Add a long-running queue service with bounded concurrent workers and robust cancellation semantics.
- Complete GTK4/libadwaita UI, segmented transfers, checksum verification, and signed reproducible release automation.
- Deliver the IDM-style capability set in [the feature parity plan](feature-parity.md), while preserving Linux-native UX and explicit consent for privacy-sensitive integrations.
- Keep each crate compiling at each commit step.
- Run fmt/clippy/test gates after each milestone increment.
