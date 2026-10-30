# SonicFetch

SonicFetch is an independent Linux download manager project. The current source tree contains a CLI prototype with SQLite queue persistence and an HTTP/HTTPS transfer path; it has not been build-verified and is not yet a release or an IDM-equivalent application.

## Requirements

- Rust 1.85 or newer (edition 2024)
- SQLite development headers (`libsqlite3-dev` on Debian/Ubuntu)
- A C toolchain and `pkg-config`

The current interface is a CLI. GTK4/libadwaita desktop UI, segmented transfers, browser integration, scheduling, and other IDM-parity features remain planned work; the `sonicfetch-desktop` crate is currently the CLI application shell.

## Build

```sh
cargo build --release -p sonicfetch-desktop
```

The binary is `target/release/sonicfetch`.

## Use

```sh
# Download into the current directory
sonicfetch add 'https://example.org/archive.tar.xz'

# Choose a destination directory
sonicfetch add 'https://example.org/archive.tar.xz' ~/Downloads

# Inspect and control the persistent queue
sonicfetch list
sonicfetch pause <download-id>
sonicfetch resume <download-id>
sonicfetch retry <download-id>
sonicfetch cancel <download-id>
sonicfetch remove <download-id>
sonicfetch remove <download-id> --delete-file
```

Queue state is stored at `$XDG_DATA_HOME/sonicfetch/sonicfetch.sqlite3`, or `~/.local/share/sonicfetch/sonicfetch.sqlite3` when `XDG_DATA_HOME` is unset. `RUST_LOG=sonicfetch_engine=debug` enables diagnostic logging. Sensitive query parameters are redacted in URL logs.

## Current behavior and limits

- Only HTTP and HTTPS URLs without embedded credentials are accepted. TLS certificate verification remains enabled.
- Download data is streamed to a uniquely named temporary file in the destination directory, then published without replacing an existing file. Existing destination files are preserved.
- Pause and cancel commands work from another terminal. A SQLite process lease prevents a second worker from writing the same partial file; the control command waits for the worker to stop.
- Press Ctrl-C during `add`, `resume`, or `retry` to pause safely. Resume sends a Range request with `If-Range` only when a saved ETag or Last-Modified validator is available. If a validator or range no longer matches, SonicFetch discards the partial and asks you to retry from the beginning.
- Interrupted transfers are recovered to the queue on the next invocation. Use `resume <id>` to continue one. If the process stopped during finalization, recovery checks whether the final file was published before choosing Completed or NeedsAttention.
- Each CLI invocation handles one transfer in the foreground. A background service, concurrent queue scheduler, segmented transfers, checksum verification, and graphical UI are not implemented yet.
- Downloaded files are never executed automatically.

## Development

```sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

See [the architecture](docs/architecture.md), [security threat model](docs/security-threat-model.md), and [implementation roadmap](docs/implementation-roadmap.md).

The intended IDM-style capability set and current gaps are tracked in [the feature parity plan](docs/feature-parity.md).

## License

MIT. See [LICENSE](LICENSE).
