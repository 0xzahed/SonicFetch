# Architecture Decisions

Date: 2026-10-30
Format: Compact architecture decision records for initial release.

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

Decision:
- Use a local deterministic fixture server and filesystem fault injection tests.

Why:
- Avoid flaky reliance on third-party servers.
- Makes retry/resume/range edge cases reproducible.

Trade-offs:
- Upfront investment in test harness before broad feature delivery.
