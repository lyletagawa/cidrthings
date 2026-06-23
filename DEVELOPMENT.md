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
  lib.rs          # library — Cidr types, minimal_supernet(), summarize_contiguous(), parsing
  main.rs         # CLI binary, gated behind `cli` feature
  bin/
    web.rs        # axum web server, gated behind `web` feature
tests/
  cli.rs          # integration tests that spawn the compiled CLI binary
contrib/
  cidrthings.sh   # portable bash implementation (IPv4 only)
  cidrthings.py   # Python 3 implementation using ipaddress stdlib
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
cargo run --bin cidrthings -- 10.1.0.0/24 10.2.0.0/24
cargo run --bin cidrthings -- --summarize 10.0.0.0/24 10.0.1.0/24 192.168.0.0/24
printf '10.1.0.0/24\n10.2.0.0/24\n' | cargo run --bin cidrthings

# Run web server (default port 3000, override with $PORT)
cargo run --features web --bin cidrthings-web

# Test
cargo test --no-default-features --lib   # pure library unit tests
cargo test --no-default-features --doc   # pure library doctests
cargo test                               # lib + CLI integration + doctests
cargo test --features web                # all of the above + 13 web handler tests

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
git tag v0.2.0
git push origin v0.2.0
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
version = "0.2.0"
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
- `summarize_contiguous(cidrs: &[Cidr]) -> Result<Vec<Cidr>, SupernetError>`

**`minimal_supernet` algorithm** — for each input block compute its network address
(first address) and broadcast address (last address). XOR the global minimum start with
the global maximum end. The number of leading zero bits in that XOR is the supernet
prefix length. Apply that prefix as a mask to the minimum start to get the network
address.

**`summarize_contiguous` algorithm** — sort blocks by network address. Sweep through:
if the next block's network address ≤ current group's max broadcast + 1 (adjacent or
overlapping), extend the group; otherwise close the group and start a new one. Apply
`minimal_supernet` to each group.

Key implementation details:
- IPv4 uses `u32` arithmetic; IPv6 uses `u128`
- `FromStr` for bare IPs (no `/`) defaults to `/32` (IPv4) or `/128` (IPv6)
- Host bits are always masked off on construction
- `MixedFamilies` error if input contains both IPv4 and IPv6

### 4. Implement the CLI (`src/main.rs`)

```rust
use cidrthings::{minimal_supernet, summarize_contiguous, Cidr};
use clap::Parser;
use std::io::{self, BufRead, IsTerminal};

#[derive(Parser)]
#[command(about = "...", version)]
struct Args {
    #[arg(value_name = "CIDRs")]
    cidrs: Vec<String>,

    #[arg(short, long)]
    summarize: bool,
}
```

- `cidrs` is not `required` — if empty, read from stdin
- Read stdin when `!io::stdin().is_terminal()`: split each line on `[' ', '\t', ',']`, skip empty tokens
- Merge stdin tokens with positional args before parsing
- `-s` / `--summarize`: call `summarize_contiguous` and print one supernet per line

### 5. Implement the web server (`src/bin/web.rs`)

Routes:
- `GET /` with no query → serve HTML UI (textarea + summarize checkbox)
- `GET /?cidrs=a,b,c` → supernet as plain text
- `GET /?cidrs=a,b,c&summarize=true` → one supernet per contiguous span, newline-separated
- `POST /` with body → supernet as plain text
- `POST /?summarize=true` → per-span output

Body and `cidrs` query param accept newline, comma, space, and tab as delimiters.

Always set `Content-Type: text/plain; charset=utf-8` on text responses explicitly —
axum does not do this automatically for `(StatusCode, String)` responses.

Extract the router into `fn router() -> Router` so it can be called in tests without
binding a port.

### 6. Add tests

**Library unit tests** in `src/lib.rs` under `#[cfg(test)]` (18 tests) — cover:
- Single block returns itself
- Two disjoint blocks
- Contained block (result is the larger)
- Host routes, IPv6, mixed families error, empty error, bare IP parsing
- `summarize_contiguous`: two spans, overlapping stays one span, single block, three
  separate spans, IPv6, empty error, mixed families error

**CLI integration tests** in `tests/cli.rs` (16 tests) — use `env!("CARGO_BIN_EXE_cidrthings")`
to get the path to the compiled binary. Two helpers:
- `run(args)` — uses `Stdio::null()` for deterministic stdin behaviour
- `run_with_stdin(args, input)` — pipes input via `Stdio::piped()`

Cover: basic supernet, `--summarize`, `-s`, stdin (newline, comma), stdin merged with
args, stdin with `--summarize`, mixed families, invalid CIDR, `--version`.

**Web handler tests** in `src/bin/web.rs` under `#[cfg(test)]` (13 tests) — use
`tower::ServiceExt::oneshot` to call the router in-process without binding a port.
Use `http_body_util::BodyExt::collect` to read the response body.

Cover: GET query, HTML, newline/comma/space delimiters, bare IP, `?summarize=true` on
GET and POST, invalid CIDR, empty body, mixed families, `Content-Type` header.

**Doctests** on `Cidr`, `minimal_supernet`, and `summarize_contiguous` with `# Examples`
blocks (3 doctests).

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

### 9. Add contrib scripts (`contrib/`)

**`contrib/cidrthings.sh`** — pure bash, IPv4 only. Key implementation notes:
- Validate octet range (0–255) explicitly after the shape regex — the regex
  `^([0-9]{1,3}\.){3}[0-9]{1,3}$` accepts `999.0.0.0`; a follow-up loop checks each
  octet with `(( octet > 255 ))`
- IPv6 detected via `*:*` glob; bare IPs default to `/32`
- Bash 64-bit arithmetic handles IPv4 (32-bit) natively; IPv6 requires 128-bit and is
  not supported
- `(( expr ))` returns exit code 1 when expr is 0 — with `set -euo pipefail`, always
  use `if (( expr )); then` rather than bare `(( expr )) && ...`

**`contrib/cidrthings.py`** — Python 3, IPv4 + IPv6. Uses `ipaddress.ip_network` from
stdlib. Wrap the parse call in `try/except ValueError` for clean error messages instead
of a Python traceback.

### 10. Add `LICENSE` and `README.md`

`LICENSE`: standard MIT text with year and author name.

`README.md`: include install instructions for both feature sets, CLI examples
(including bare IP, `--summarize`, and stdin), web server curl examples for GET, POST,
and `?summarize=true`, library usage showing both `minimal_supernet` and
`summarize_contiguous` with a valid `fn main()` wrapper, a `contrib/` section, a
`## License` section, and a CI badge:

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
| `(( expr ))` with `set -e` exits when expr is 0 | Use `if (( expr )); then` in bash |
| Bash octet regex accepts values > 255 | Loop over octets and check `(( octet > 255 ))` |
| `ip_network()` raises `ValueError` on bad input | Wrap in `try/except ValueError` in Python |
| `io::stdin().is_terminal()` requires Rust 1.70+ | `IsTerminal` was stabilised in 1.70; no crate needed |
