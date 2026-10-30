<div align="center">

# ⚡ SonicFetch

### Open-source download manager for Linux
#### Rust • Tokio • SQLite • HTTP/HTTPS

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024_edition-orange.svg)](https://www.rust-lang.org/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

<p>
  SonicFetch is an independent Linux download manager prototype. Its current command-line app streams HTTP and HTTPS downloads to disk, keeps queue metadata in SQLite, and supports pausing, resuming, retrying, and cancellation.
</p>

[Quick Start](#-quick-start) •
[Features](#-features) •
[Commands](#-commands) •
[Security](#-security) •
[Roadmap](#-roadmap) •
[Contributing](#-contributing) •
[License](#-license)

</div>

---

## 🎯 Project status

SonicFetch is early-stage software. The current interface is a CLI prototype; it is not yet the planned GTK4/libadwaita desktop application and has not been build- or release-verified. Segmented downloads, concurrent queue scheduling, browser integration, and many advanced download-manager features are still planned. See the [implementation roadmap](docs/implementation-roadmap.md) for the current status.

SonicFetch is an independent implementation using documented network protocols. It does not contain or reproduce proprietary IDM internals.

## ✨ Features

- HTTP and HTTPS streaming to disk without buffering the whole file in memory.
- Persistent download records stored locally in SQLite.
- Pause, resume, retry, cancel, and remove controls.
- Resume requests use HTTP Range and saved ETag or Last-Modified validators when available.
- A changed or invalid resume response is rejected instead of combining bytes from mismatched versions.
- Safe temporary-file writes and finalization that does not overwrite an existing destination file.
- Redirect handling with HTTPS downgrade protection and normal TLS certificate verification.
- Downloaded files are never executed automatically.

### Current capability status

| Capability | Status |
| --- | --- |
| HTTP/HTTPS single-stream transfer | Prototype source present; build not verified |
| Persistent queue and recovery | Prototype source present; tests not run |
| Pause, cancel, manual retry | CLI controls implemented; not build verified |
| Validator-based resume | Basic implementation present; not build verified |
| Native GTK4/libadwaita UI | Planned |
| Segmented transfers and concurrent scheduler | Planned |
| Browser integration and IDM feature parity | Planned; see [feature plan](docs/feature-parity.md) |

## 🚀 Quick Start

### Requirements

- Linux
- Rust 1.85 or newer (edition 2024)
- A C toolchain, `pkg-config`, and SQLite development headers (`libsqlite3-dev` on Debian/Ubuntu)

Install the system build dependencies on Ubuntu or Debian:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libsqlite3-dev
```

Install a current Rust toolchain with [rustup](https://rustup.rs/), then clone and build:

```bash
git clone https://github.com/0xzahed/SonicFetch.git
cd SonicFetch
cargo build --release -p sonicfetch-desktop
```

The CLI binary is `target/release/sonicfetch`.

## 🎮 Commands

```bash
# Download into the current directory
sonicfetch add 'https://example.org/archive.tar.xz'

# Choose a destination directory
sonicfetch add 'https://example.org/archive.tar.xz' ~/Downloads

# Show the saved queue
sonicfetch list

# Control a download using its ID
sonicfetch pause <download-id>
sonicfetch resume <download-id>
sonicfetch retry <download-id>
sonicfetch cancel <download-id>
sonicfetch remove <download-id>

# Remove the queue record and its completed file
sonicfetch remove <download-id> --delete-file
```

Press **Ctrl-C** during `add`, `resume`, or `retry` to pause the active transfer. The queue database is saved at `$XDG_DATA_HOME/sonicfetch/sonicfetch.sqlite3`, or `~/.local/share/sonicfetch/sonicfetch.sqlite3` when `XDG_DATA_HOME` is unset.

## 🛡️ Security

- Only credential-free HTTP and HTTPS URLs are accepted.
- TLS verification stays enabled; HTTPS redirects cannot downgrade to HTTP.
- Signed URL parameters and credentials are redacted from transfer logs.
- Partial files are stored beside the destination and published without replacing an existing file.
- SonicFetch never runs downloaded content.
- Browser integration, when developed, will be opt-in and clearly scoped.

Read the [security threat model](docs/security-threat-model.md) for current protections and known limitations.

## 🧪 Development

Format, lint, and run the workspace tests with:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Tests use local fixtures and do not require production download servers. These commands are the project’s intended checks; check the [roadmap](docs/implementation-roadmap.md) for whether they have been run in the current environment.

The repository includes architecture notes, the [component map](docs/component-map.md), [database schema](docs/database-schema.md), [testing strategy](docs/testing-strategy.md), and [contribution guide](CONTRIBUTING.md).

## 🗺️ Roadmap

Planned milestones include:

- Verify the current workspace with a Rust toolchain and deterministic test runs.
- Build the GTK4/libadwaita native interface.
- Add bounded concurrent scheduling, priorities, and segmented transfers with safe fallback.
- Expand integrity checks, recovery coverage, and release packaging.
- Consider opt-in browser integration after the core application is stable.

The full status and acceptance criteria are in [the implementation roadmap](docs/implementation-roadmap.md).

## 🤝 Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening an issue or pull request. Include test evidence for behavior changes and update the documentation when behavior or architecture changes.

## 📄 License

SonicFetch is distributed under the [MIT License](LICENSE).

---

<div align="center">
  <sub>Made for Linux by <a href="https://github.com/0xzahed">Abu Zahed</a> and open-source contributors.</sub>
</div>
