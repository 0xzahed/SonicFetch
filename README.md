# SonicFetch

SonicFetch is an open-source Linux download manager focused on reliability, security, and maintainability.

## Project status

SonicFetch is in active early development.

Currently available in this repository:
- Architecture and threat-model documentation
- Rust workspace scaffold with core, engine, storage, and desktop crates
- Initial download state-machine model and tests

Current roadmap and milestones are documented in:
- docs/implementation-roadmap.md

## Goals

- Native Linux desktop experience (GTK4/libadwaita)
- Robust HTTP/HTTPS downloads with safe pause/resume
- Persistent queue and crash recovery
- Secure defaults and deterministic automated testing

## Repository layout

- crates/sonicfetch-core: domain model, states, and typed errors
- crates/sonicfetch-engine: transfer orchestration and queue logic
- crates/sonicfetch-storage: SQLite persistence and migrations
- crates/sonicfetch-desktop: desktop UI shell (phased)
- docs: architecture, security, testing, and roadmap
- migrations: SQL schema migrations
- tests: integration, fixtures, and security test assets

## Development setup (Ubuntu/Debian)

Install build dependencies:

```bash
sudo apt-get update
sudo apt-get install -y curl pkg-config libsqlite3-dev libgtk-4-dev libadwaita-1-dev
```

Install Rust toolchain:

```bash
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

## Build and test

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Security principles

- Never execute downloaded files automatically
- Keep TLS verification enabled
- Validate URLs and output paths
- Redact sensitive URL components in logs
- Avoid storing credentials/tokens in persistence or logs

See also:
- docs/security-threat-model.md
- SECURITY.md

## Contributing

Contributions are welcome.

Before submitting changes:
1. Read CONTRIBUTING.md
2. Keep changes scoped and test-backed
3. Run formatting, lint, and tests locally

## License

SonicFetch is licensed under the MIT License. See LICENSE.
