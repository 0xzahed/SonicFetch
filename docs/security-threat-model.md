# Security Threat Model

Date: 2026-10-30
Method: Threat-centric model for local desktop download manager.

## 1. Assets

- Local filesystem integrity.
- User-selected destination paths.
- Downloaded artifact integrity.
- Queue metadata (SQLite DB).
- Application availability and stability.

## 2. Trust Boundaries

Untrusted inputs:
- URLs, redirects, response headers, payload bytes.
- Optional imported queue files.

Trusted components:
- SonicFetch binaries and signed release artifacts.
- Local SQLite DB after integrity checks.

Conditional trust:
- Browser integration module (future optional) only with explicit consent.

## 3. Primary Threats and Mitigations

Path traversal / unsafe write targets:
- Mitigation: canonicalize destination, reject path escapes, enforce user-approved root, sanitize filename.

Symlink/file replacement race:
- Mitigation: use safe temp directory, create new temp files with restrictive permissions, verify target path before atomic rename, avoid following unexpected symlinks.

Corrupt or mixed-version resumes:
- Mitigation: persist and compare ETag/Last-Modified and expected length; restart safely when validators mismatch.

TLS downgrade / invalid cert acceptance:
- Mitigation: rely on default TLS certificate verification; no insecure bypass toggle for initial release.

Resource exhaustion (memory, disk, workers):
- Mitigation: enforce bounded concurrency, bounded retries, disk-space checks, chunked stream writes.

Information leakage in logs:
- Mitigation: redact credentials and signed query parameters; structured typed logging fields.

Archive/executable auto-run risks:
- Mitigation: never execute downloaded files; no post-download auto-launch.

Local privilege misuse via IPC (future):
- Mitigation: avoid IPC in initial release; if introduced, use strict message schema validation and least-privilege endpoints.

## 4. Residual Risks

- User may choose unsafe destinations outside recommended paths.
- Network adversaries may still deny service (availability impact).
- Third-party server behavior can invalidate resume assumptions; handled through safe fallback/restart.

## 5. Security Testing Requirements

- URL parser fuzzing for malformed input.
- Path validation tests including traversal and symlink edge cases.
- Retry/backoff abuse and worker bound tests.
- Corrupt metadata recovery tests.
- Log redaction tests for sensitive query/header-like input.

## 6. Explicit Non-Goals

- No analysis or execution of crack/patch artifacts.
- No credential harvesting or covert browser activity capture.
- No insecure TLS mode.
