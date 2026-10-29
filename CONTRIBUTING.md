# Contributing to SonicFetch

Thanks for contributing.

SonicFetch is a security-focused download manager. Reliability and safe defaults are prioritized over feature volume.

## Development flow

1. Keep changes scoped to one milestone or concern.
2. Add or update tests for behavior changes.
3. Run quality gates before proposing changes:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Local setup

Install Rust and required Linux dependencies (Ubuntu/Debian):

```bash
sudo apt-get update
sudo apt-get install -y curl pkg-config libsqlite3-dev libgtk-4-dev libadwaita-1-dev
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
```

## Branch and PR guidance

1. Create a topic branch.
2. Keep commits focused and readable.
3. Include tests for new behavior and bug fixes.
4. Update docs when architecture or behavior changes.
5. Open a PR with a clear summary and test evidence.

## Security and safety requirements

- Do not add code that executes downloaded files.
- Do not disable TLS verification in download paths.
- Do not log credentials or signed URL tokens.
- Validate all filesystem paths and URL inputs.
- Do not add hidden telemetry or covert data collection.

## Documentation

Any significant behavior or architectural change should update:
- docs/architecture.md
- docs/architecture-decisions.md
- docs/testing-strategy.md
- docs/security-threat-model.md

## Commit guidance

- Use clear, action-oriented commit messages.
- Prefer small, reviewable diffs.

## Code of conduct

Please follow CODE_OF_CONDUCT.md for community interactions.
