# Security test plan

No security regression tests are implemented here yet. Prioritize URL/redirect validation, output-path and symlink races, resume validator mismatches, cross-process worker leases, and sensitive URL redaction.

Required cases are listed in [the security threat model](../../docs/security-threat-model.md).
