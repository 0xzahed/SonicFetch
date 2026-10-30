# Architecture Decisions

Date: 2026-10-30
Format: Architecture decision records. Decisions describe the target architecture; implementation status is called out where it differs.

## ADR-001: Rust as the implementation language

Decision:
- Use Rust for core engine, persistence, and desktop integration.

Why:
- Strong memory safety and concurrency model.
- Mature async/network/database ecosystem.
- Good fit for long-running, I/O-heavy, reliability-critical workloads.

Trade-offs:
- Higher ramp-up than dynamic languages.
- GTK Rust bindings can increase integration complexity.

## ADR-002: GTK4 + libadwaita for desktop UI

Decision:
- Build native Linux desktop UI using GTK4 and libadwaita.

Why:
- Native Linux integration, accessibility support, and desktop conventions.
- Avoids embedding web runtime overhead for initial release.

Trade-offs:
- Best fit is GNOME-oriented UX; cross-desktop polishing may require extra work.

## ADR-003: Tokio + reqwest for transfer runtime

Decision:
- Use Tokio for async runtime and reqwest for HTTP transport.

Why:
- Reliable async ecosystem and robust HTTP/TLS handling.
- Supports redirect handling, streaming, and request customization.

Trade-offs:
- Careful control of worker counts and buffering is needed to prevent resource spikes.

## ADR-004: SQLite for persistent metadata

Decision:
- Use SQLite with migration-controlled schema and explicit transactions.

Why:
- Local desktop-appropriate storage with strong transactional guarantees.
- Supports crash recovery workflows and deterministic test setup.

Trade-offs:
- Requires careful transaction boundaries to avoid lock contention under high update frequency.

## ADR-005: Layered crate architecture

Decision:
- Maintain separation between core domain, engine, storage, and UI crates.

Why:
- Keeps transfer logic testable without launching UI.
- Enables deterministic integration/security testing around engine+storage.

Trade-offs:
- Some boilerplate at crate boundaries (mapping DTOs/events/errors).

## ADR-006: State-machine enforced lifecycle

Status: Normal repository transitions validate `StateTransition`; startup recovery may repair stale rows directly by design.

Decision:
- Implement explicit lifecycle transition validation in sonicfetch-core.

Why:
- Prevents invalid state transitions and inconsistent persistence.
- Simplifies retry/cancel/recovery semantics.

Trade-offs:
- Requires disciplined transition tests and migration-safe state evolution.

## ADR-007: Security-first defaults

Decision:
- Enforce strict URL/path validation, TLS verification, redacted logs, and safe file finalize semantics.

Why:
- Desktop download managers process untrusted input at scale.
- Security bugs here commonly become high-impact local compromise vectors.

Trade-offs:
- Some user requests (unsafe paths, invalid cert workarounds) are intentionally unsupported.

## ADR-008: Deterministic in-process test infrastructure

Status: Test plan only; integration/security harnesses are not yet implemented.

Decision:
- Use a local deterministic fixture server and filesystem fault injection tests.

Why:
- Avoid flaky reliance on third-party servers.
- Makes retry/resume/range edge cases reproducible.

Trade-offs:
- Upfront investment in test harness before broad feature delivery.

## ADR-009: Independent feature parity, not implementation cloning

Decision:
- Target comparable download-manager capabilities to IDM using documented public protocols and Linux-native integration.
- Do not copy proprietary code, interface assets, branding, or claim knowledge of undocumented IDM internals.

Why:
- Feature parity answers the product goal while preserving the independent implementation and evidence boundary established by the research reports.
- Dynamic segmentation, browser integration, scheduling, queues, categories, and Site Grabber can be designed around Linux and explicit consent.

Trade-offs:
- Full parity is a multi-milestone effort and will take longer than a single-stream downloader.
- Some IDM behavior is Windows-specific or depends on protocols and browser APIs that differ on Linux.

## ADR-010: Sensitive integrations are opt-in

Decision:
- Browser capture, clipboard monitoring, site crawling, scanner launching, and update installation are disabled until explicitly enabled by the user.
- Never execute downloaded content automatically.

Why:
- These capabilities cross privacy or execution boundaries beyond ordinary HTTP transfers.
- User-visible controls and narrow permissions preserve trust without ruling out useful IDM-style features.

Trade-offs:
- First-run setup requires explicit choices for integrations that some users expect to be automatic.

## ADR-011: SQLite worker leases for the CLI prototype

Decision:
- Store the active process ID with each task and use conditional database updates to coordinate pause, cancel, resume, and recovery across CLI invocations.

Why:
- The prototype has no always-running daemon or IPC server, yet separate CLI invocations need safe queue control.
- SQLite transactions provide a small local coordination surface without adding a backend service.

Trade-offs:
- PID reuse and crashes inside a still-running process need continued hardening; a future desktop process can replace this mechanism with in-process task ownership or a narrowly scoped local IPC protocol.
