# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-06-29

### Changed
- Web: error response bodies now use the `error: ` prefix consistently with the CLI. Supernet errors (`Empty`, `MixedFamilies`) and parse errors previously rendered without a prefix (e.g. `error parsing "x": ...`); they now read `error: ...`. Parse-error offending tokens remain debug-quoted (`{s:?}`) for safe escaping of control characters in HTTP responses.

### Added
- Tests: `ParseError` variants (`PrefixTooLong`, `InvalidPrefix`, `InvalidAddress`) and their `Display` output are now directly asserted; CLI/web only exercised them via exit codes/status before
- Tests: edge-prefix coverage — full address space collapsing to `/0`, single `/32` host routes, and the `broadcast()` extremes (`/0`, `/32`) for both IPv4 and IPv6
- Tests: web `GET` error and bare-IP coverage to match the existing `POST` tests
- Tests: CLI and web error messages now pinned to exact strings rather than "nonempty"/status-only checks

### Fixed
- CLI: `--cidrs` argument help now correctly states that stdin is read when not a terminal and merged with positional arguments (was documented as "if omitted")
- Docs: `Ipv6Cidr::broadcast()` clarifies that IPv6 has no broadcast concept; it returns the highest address in the range

## [0.2.0] - 2026-06-22

### Added
- Library: `summarize_contiguous()` — splits blocks into contiguous runs, returns one supernet per run
- CLI: `-s` / `--summarize` flag to print one supernet per contiguous run
- CLI: reads CIDR blocks from stdin when piped (merged with any positional arguments); accepts newline, comma, and space delimiters
- Web: `?summarize=true` query parameter on `GET` and `POST` for per-run output
- Web: accepts space- and tab-delimited blocks in addition to newline and comma
- `contrib/cidrthings.sh` — portable bash implementation (IPv4 only, no dependencies)
- `contrib/cidrthings.py` — Python 3 implementation using `ipaddress` stdlib (IPv4 + IPv6)

## [0.1.0] - 2026-06-22

### Added
- Library: `Cidr`, `Ipv4Cidr`, `Ipv6Cidr` types with `FromStr` and `Display`
- Library: `minimal_supernet()` — returns the smallest CIDR enclosing all inputs
- IPv4 and IPv6 support; mixing families in one call is an error
- Bare IP addresses (no prefix) treated as `/32` (IPv4) or `/128` (IPv6)
- CLI binary (`cidrthings`) with `--version` and `--help`
- Web server binary (`cidrthings-web`) with `GET /?cidrs=` and `POST /` endpoints
- Browser UI served at `GET /`
- CI: test, lint, and doc jobs via GitHub Actions
- Multi-arch release pipeline: x86_64/aarch64 Linux, x86_64/aarch64 macOS, x86_64 Windows

[0.2.1]: https://github.com/lyletagawa/cidrthings/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/lyletagawa/cidrthings/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/lyletagawa/cidrthings/releases/tag/v0.1.0
