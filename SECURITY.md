# Security Policy

## Supported development stage

SonicFetch is currently pre-release and under active development.

Security fixes are applied on the main branch first.

## Reporting a vulnerability

Please report security issues privately and do not open a public issue with exploit details.

Recommended report content:
- Affected component and version/commit
- Reproduction steps
- Expected vs observed behavior
- Potential impact
- Any suggested mitigation

## Response expectations

Best effort targets:
- Initial acknowledgement: within 7 days
- Triage decision: as soon as practical
- Fix timeline: depends on severity and complexity

## Scope priorities

Highest priority areas include:
- Path traversal and unsafe filesystem writes
- TLS/certificate validation behavior
- Resume/segment integrity mismatches
- Credential/token leakage in logs or storage
- Resource exhaustion and denial-of-service vectors

## Coordinated disclosure

Please allow time for a patch before public disclosure.

## Non-goals

- No support for activation bypasses, cracks, or unauthorized binary patching
- No execution of untrusted downloaded artifacts by default
