# path-git-format

A CLI tool to format path(s) with git information. Displays git branch and
status information alongside directory paths for seamless integration with tools
like zoxide and fzf.

Always reference these instructions first and fallback to search or bash
commands only when you encounter unexpected information that does not match the
info here.

## git commit

Follow git conventional commits, use message format `type(scope): message`

## Working Effectively

- Bootstrap, build, and test the repository:

  - Rust toolchain is pre-installed: use `cargo 1.80+`, `rustc 1.80+`
  - `cargo check` -- verifies compilation. Takes ~5 seconds. Set timeout to 60+
    seconds.
  - `cargo test` -- runs all unit tests. Takes ~10 seconds. Set timeout to 120+
    seconds.
  - `cargo build --release` -- creates optimized binary. Takes ~15 seconds. Set
    timeout to 300+ seconds.
  - `cargo clippy -- -D warnings` -- linting check. Takes ~3 seconds. Set
    timeout to 60+ seconds.
  - `cargo fmt` -- formats code. Takes <1 second. Set timeout to 30+ seconds.

- Run the CLI tool:

  - ALWAYS build first with `cargo build --release`
  - Binary location: `./target/release/path-git-format`
  - Usage:
    `printf "$PWD" | ./target/release/path-git-format --format "{path}: {branch}"`
  - Usage:
    `printf "/path1\n/path2" | ./target/release/path-git-format --format "{path} ({branch})"`

- Build with nix:

  - `nix develop` -- enters development shell
  - `make build` -- builds debug binary
  - `make release` -- builds optimized release binary

- Dependencies:
  - Key dependencies: `clap`, `tokio`, `tokio-stream` (async I/O)
  - See `Cargo.toml` for complete dependency list
  - Nix development environment configured in `flake.nix`

## Validation

- Always manually validate any new code changes by running through complete
  scenarios.
- ALWAYS run through at least one complete end-to-end scenario after making
  changes:

  1. Build the application: `cargo build --release`
  2. Test with single path:
     `printf "/tmp" | ./target/release/path-git-format --format "{path}"`
  3. Test with git repository:
     `cd /path/to/git/repo && printf "." | ./target/release/path-git-format --format "{path}: {branch}"`
  4. Test with multiple paths:
     `printf "/tmp\n/var\n$(pwd)" | ./target/release/path-git-format --format "{path}"`
  5. Test with nix build: `nix develop` -> `make release`

- Test scenarios to validate after changes:

  - **Single path test**: Test with single directory path
  - **Git repo test**: Verify branch information is correctly displayed in git
    repos
  - **Multiple paths test**: Test with newline-separated paths via stdin
  - **Non-git path test**: Test with paths outside git repositories
  - **Format options test**: Test various format string options

- Always run `cargo fmt` and `cargo clippy -- -D warnings` before you are done
  or the CI (.github/workflows/ci.yml) will fail.

## Release Process

- Releases are automated via `release-please-action` and `build.yml` workflow
- Releases are triggered on version tags (v0.1.4, v0.2.0, etc.)
- Release artifacts include pre-built macOS binaries (both x86_64 and aarch64)
- Built using nix for reproducible builds
- For manual release: push a git tag with format `v<version>`
