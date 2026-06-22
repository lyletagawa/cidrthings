# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.0]: https://github.com/lyletagawa/cidrthings/releases/tag/v0.1.0
