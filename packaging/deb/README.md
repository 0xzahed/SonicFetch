# Debian package

Build a local `.deb` from the current source tree on Debian or Ubuntu:

```sh
packaging/deb/build.sh
```

The script builds the `sonicfetch` CLI in release mode and writes `sonicfetch_<version>_amd64.deb` to `dist/`. It requires Rust/Cargo, a C toolchain, `pkg-config`, `libsqlite3-dev`, and `dpkg-deb`. The resulting package depends on `libsqlite3-0` and `ca-certificates`.

Install and run:

```sh
sudo apt install ./dist/sonicfetch_*.deb
sonicfetch --help
```

This is an unsigned local package build. Releases should be built from a tagged source archive with a committed lockfile, signed by the release owner, and verified on the supported Ubuntu baseline before publication. The CLI package does not install a desktop launcher because the GTK application is not implemented yet.
