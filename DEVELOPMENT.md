# Development guide

Operator reference for building, testing, running, and recreating cidrthings from scratch.

---

## Prerequisites

- Rust stable toolchain (`rustup install stable`)
- For the web server: no extra tools needed
- For cross-compiling ARM Linux in CI: `cross` and Docker (handled automatically in GitHub Actions)

---

## Project structure

```
src/
  lib.rs          # library — Cidr types, minimal_supernet(), parsing
  main.rs         # CLI binary, gated behind `cli` feature
  bin/
    web.rs        # axum web server, gated behind `web` feature
tests/
  cli.rs          # integration tests that spawn the compiled CLI binary
.github/
  workflows/
    ci.yml        # test / lint / doc on every push and PR
    release.yml   # multi-arch binary releases on v* tag push
```

---

## Feature flags

```toml
default = ["cli"]          # cargo build / cargo install works out of the box
cli     = ["dep:clap"]     # CLI binary
web     = ["dep:axum", "dep:tokio", "dep:serde"]  # web server binary
```

Library users can `--no-default-features` to get zero external dependencies.

---

## Common commands

```bash
# Build
cargo build                              # CLI (default features)
cargo build --features web               # CLI + web server

# Run CLI
cargo run -- 10.1.0.0/24 10.2.0.0/24
cargo run -- 10.0.0.0/8 10.1.0.0/24 192.168.0.0/16

# Run web server (default port 3000, override with $PORT)
cargo run --features web --bin cidrthings-web

# Test
cargo test --no-default-features --lib   # pure library unit tests
cargo test --no-default-features --doc   # pure library doctests
cargo test                               # lib + CLI integration + doctests
cargo test --features web                # all of the above + 9 web handler tests

# Lint
cargo fmt
cargo fmt --check                        # CI mode — exits non-zero if changes needed
cargo clippy -- -D warnings
cargo clippy --features web -- -D warnings

# Docs
cargo doc --no-deps --open
```

---

## Cutting a release

```bash
git tag v0.1.0
git push origin v0.1.0
```

This triggers `.github/workflows/release.yml`, which builds binaries for 5 targets,
strips them, packages as `.tar.gz` / `.zip`, generates `sha256sums.txt`, and creates
a GitHub release with all artifacts attached.

---

## Recreating from scratch

### 1. Initialise the repo

```bash
cargo new cidrthings --lib
cd cidrthings
git init
echo '/target' > .gitignore
```

### 2. Configure `Cargo.toml`

Add metadata and split dependencies behind optional features so library users
don't inherit CLI or web deps:

```toml
[package]
name = "cidrthings"
version = "0.1.0"
edition = "2021"
description = "Compute the minimal supernet enclosing a set of CIDR blocks"
license = "MIT"
readme = "README.md"

[[bin]]
name = "cidrthings"
path = "src/main.rs"
required-features = ["cli"]

[[bin]]
name = "cidrthings-web"
path = "src/bin/web.rs"
required-features = ["web"]

[features]
default = ["cli"]
cli = ["dep:clap"]
web = ["dep:axum", "dep:tokio", "dep:serde"]

[dependencies]
clap   = { version = "4", features = ["derive"], optional = true }
axum   = { version = "0.7", optional = true }
tokio  = { version = "1", features = ["full"], optional = true }
serde  = { version = "1", features = ["derive"], optional = true }

[dev-dependencies]
tower          = { version = "0.5", features = ["util"] }
http-body-util = "0.1"
```

### 3. Implement the library (`src/lib.rs`)

Public surface:
- `Cidr` enum wrapping `Ipv4Cidr` and `Ipv6Cidr`
- `ParseError` and `SupernetError` error enums
- `minimal_supernet(cidrs: &[Cidr]) -> Result<Cidr, SupernetError>`

**Algorithm** — for each input block compute its network address (first address) and
broadcast address (last address). XOR the global minimum start with the global maximum
end. The number of leading zero bits in that XOR is the supernet prefix length. Apply
that prefix as a mask to the minimum start to get the network address.

Key implementation details:
- IPv4 uses `u32` arithmetic; IPv6 uses `u128`
- `FromStr` for bare IPs (no `/`) defaults to `/32` (IPv4) or `/128` (IPv6)
- Host bits are always masked off on construction
- `MixedFamilies` error if input contains both IPv4 and IPv6

### 4. Implement the CLI (`src/main.rs`)

```rust
use cidrthings::{minimal_supernet, Cidr};
use clap::Parser;

#[derive(Parser)]
#[command(about = "...", version)]
struct Args {
    #[arg(required = true)]
    cidrs: Vec<String>,
}
```

Parse each argument as `Cidr`, call `minimal_supernet`, print to stdout. Print errors
to stderr and exit 1.

### 5. Implement the web server (`src/bin/web.rs`)

Routes:
- `GET /` with no query → serve HTML UI
- `GET /?cidrs=a,b,c` → supernet as plain text
- `POST /` with newline- or comma-delimited body → supernet as plain text

Always set `Content-Type: text/plain; charset=utf-8` on text responses explicitly —
axum does not do this automatically for `(StatusCode, String)` responses.

Extract the router into `fn router() -> Router` so it can be called in tests without
binding a port.

### 6. Add tests

**Library unit tests** in `src/lib.rs` under `#[cfg(test)]` — cover:
- Single block returns itself
- Two disjoint blocks
- Contained block (result is the larger)
- Host routes
- IPv6
- Mixed families error
- Empty error
- Bare IP parsing

**CLI integration tests** in `tests/cli.rs` — use `env!("CARGO_BIN_EXE_cidrthings")`
to get the path to the compiled binary. Spawn it with `std::process::Command` and
assert stdout, stderr, and exit code.

**Web handler tests** in `src/bin/web.rs` under `#[cfg(test)]` — use
`tower::ServiceExt::oneshot` to call the router in-process without binding a port.
Use `http_body_util::BodyExt::collect` to read the response body.

**Doctests** on `Cidr` and `minimal_supernet` with `# Examples` blocks.

### 7. Add CI (`.github/workflows/ci.yml`)

Three jobs on `ubuntu-latest`, triggered on push to `main` and PRs:

```yaml
test:
  - cargo test --no-default-features --lib    # must be separate from --doc
  - cargo test --no-default-features --doc    # can't combine --lib and --doc
  - cargo test
  - cargo test --features web

lint:
  - cargo fmt --check
  - cargo clippy -- -D warnings
  - cargo clippy --features web -- -D warnings

doc:
  env: RUSTDOCFLAGS: -D warnings
  - cargo doc --no-deps
```

> **Gotcha:** `--no-default-features` + integration tests fail in a clean CI environment
> because `required-features = ["cli"]` means the CLI binary is not built, so
> `env!("CARGO_BIN_EXE_cidrthings")` points to a non-existent path. Always scope
> the no-features step to `--lib` and `--doc` only.

> **Gotcha:** `cargo test --lib --doc` is invalid — `--lib` and `--doc` cannot be
> combined in a single invocation. Use two separate `cargo test` calls.

### 8. Add release workflow (`.github/workflows/release.yml`)

Triggers on `v*` tag push. Matrix of 5 targets:

| Target | Runner | Notes |
|--------|--------|-------|
| `x86_64-unknown-linux-gnu` | ubuntu | native |
| `aarch64-unknown-linux-gnu` | ubuntu | `cross`; install `binutils-aarch64-linux-gnu` for strip |
| `x86_64-apple-darwin` | macos | native |
| `aarch64-apple-darwin` | macos | native |
| `x86_64-pc-windows-msvc` | windows | native; `.zip` not `.tar.gz` |

After all builds, a final `release` job downloads artifacts, runs
`cd artifacts && sha256sum * > sha256sums.txt`, and publishes the GitHub release
with `softprops/action-gh-release@v2`.

> **Gotcha:** `aarch64-linux-gnu-strip` is not installed by default on ubuntu runners.
> Add `sudo apt-get install -y binutils-aarch64-linux-gnu` before the package step
> for that target. Using the host `strip` on a cross-compiled ARM binary silently
> produces an invalid result.

### 9. Add `LICENSE` and `README.md`

`LICENSE`: standard MIT text with year and author name.

`README.md`: include install instructions for both feature sets, CLI examples
(including bare IP), web server curl examples for both GET and POST, library usage
with a valid `fn main()` wrapper (bare `let` at top level is not valid Rust), a
`## License` section, and a CI badge pointing to the workflow:

```markdown
[![CI](https://github.com/YOUR_USERNAME/cidrthings/actions/workflows/ci.yml/badge.svg)](...)
```

---

## Known gotchas summary

| Issue | Fix |
|-------|-----|
| `--no-default-features` integration tests fail in clean CI | Use `--lib` and `--doc` separately |
| `--lib` and `--doc` can't be combined | Two separate `cargo test` invocations |
| `aarch64-linux-gnu-strip` missing on ubuntu runners | `apt-get install binutils-aarch64-linux-gnu` |
| axum doesn't set `Content-Type: text/plain` automatically | Use `(status, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()` |
| `cargo fmt` makes large-looking diffs | Always run before committing |
| `.split(|c: char| c == '\n' \|\| c == ',')` triggers clippy | Use `.split(['\n', ','])` |
| Bare `let` in README code blocks | Wrap in `fn main()` |
