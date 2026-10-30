# Security Threat Model

Date: 2026-10-30
Method: Threat-centric model for a local download manager. Mitigations distinguish source-present prototype behavior from planned controls.

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
- Present: canonicalize the user-selected destination directory, strip path separators and controls from URL-derived filenames, keep temporary and final paths within that directory.
- Planned: explicit file/path edge-case tests and UI preview of the final destination.

Symlink/file replacement race:
- Present: validate database paths against the canonical destination and expected temp name; open with `O_NOFOLLOW`, use `create_new` for new temp files, require a regular single-link file, compare the open handle to the path before publication, and publish through a same-directory hard link that cannot replace an existing target. Temp files use mode 0600 and file/directory metadata is synced.
- Residual: the final path check and hard-link syscall are separate operations; hostile same-user processes sharing the selected directory may still race. Keep race-focused tests in the release gate.

Corrupt or mixed-version resumes:
- Present: persist ETag/Last-Modified, send `If-Range`, validate 206 range offsets/length, discard mismatched partials, and restart from zero on retry.
- Planned: stronger local temp-file integrity checks, trusted checksum support, and automated range/validator fault tests.

TLS downgrade / invalid cert acceptance:
- Present: Reqwest rustls uses native certificate roots and normal certificate validation; redirects from HTTPS to HTTP are rejected. There is no insecure TLS toggle.

Resource exhaustion (memory, disk, workers):
- Present: stream response chunks to disk and use connect/read timeouts. One task is started per CLI invocation.
- Missing before release: global concurrency limits, rate/size quotas, retry budgets, disk-space checks, and explicit resource caps. Multiple CLI invocations can still use system resources concurrently.

Information leakage in logs:
- Present: URL credentials are rejected and log URLs redact common signed/token query fields. Persisted source URLs can still contain query values for functionality and are stored in the private application data directory.
- Planned: exhaustive redaction tests for provider-specific signature parameters and removal/redaction policies for old history records.

Archive/executable auto-run risks:
- Mitigation: never execute downloaded files; no post-download auto-launch.

Cross-process queue controls:
- Present: control requests use SQLite state updates and worker PID leases; no network listener or shell command is used.
- Residual: a reused PID can look alive; worker crashes inside a still-running process can leave a lease until timeout. Replace or strengthen leases before a long-running GUI/service release.

## 4. Residual Risks

- User may choose unsafe destinations outside recommended paths.
- Network adversaries may still deny service (availability impact).
- Third-party server behavior can invalidate resume assumptions; handled through safe fallback/restart.
- The CLI prototype has no global worker cap, disk-space preflight, or comprehensive deterministic security suite yet.

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
