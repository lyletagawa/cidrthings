# cidrthings

[![CI](https://github.com/lyletagawa/cidrthings/actions/workflows/ci.yml/badge.svg)](https://github.com/lyletagawa/cidrthings/actions/workflows/ci.yml)

Computes the minimal supernet that contains a set of CIDR blocks.

Given any mix of IPv4 or IPv6 networks, returns the smallest single CIDR block that encloses all of them. Pass `--summarize` to instead return one supernet per contiguous run of blocks.

## Install

```
cargo install --path .                 # CLI only
cargo install --path . --features web  # CLI + web server
```

## CLI

```
cidrthings [-s] [<cidr>...]
```

CIDR blocks may be given as arguments or piped via stdin — one per line, or comma- or space-separated.

```
$ cidrthings 10.1.0.0/24 10.2.0.0/24
10.0.0.0/14

$ cidrthings 10.0.0.0/8 10.1.0.0/24
10.0.0.0/8

$ cidrthings 192.168.1.0/32 192.168.1.1/32
192.168.1.0/31

$ cidrthings 2001:db8::/32 2001:db9::/32
2001:db8::/31

$ cidrthings 10.0.0.1 10.0.0.2
10.0.0.0/30
```

Bare IP addresses (no prefix) are treated as `/32` (IPv4) or `/128` (IPv6).

### --summarize

`-s` / `--summarize` groups blocks into contiguous runs and prints one supernet per run:

```
$ cidrthings --summarize 10.0.0.0/24 10.0.1.0/24 192.168.0.0/24 192.168.1.0/24
10.0.0.0/23
192.168.0.0/23
```

### Stdin

```
$ printf '10.1.0.0/24\n10.2.0.0/24\n' | cidrthings
10.0.0.0/14

$ printf '10.0.0.0/24\n10.0.1.0/24\n192.168.0.0/24\n' | cidrthings --summarize
10.0.0.0/23
192.168.0.0/24
```

## Web server

```
cargo run --features web --bin cidrthings-web
```

Starts an HTTP server on port 3000 (override with `$PORT`).

| Endpoint | Description |
|----------|-------------|
| `GET /` | Browser UI with textarea and summarize checkbox |
| `GET /?cidrs=...` | Supernet as plain text |
| `GET /?cidrs=...&summarize=true` | One supernet per contiguous run, newline-separated |
| `POST /` | Blocks in body (newline, comma, or space separated) |
| `POST /?summarize=true` | Same, per-run output |

```
$ curl -s 'http://localhost:3000/?cidrs=10.1.0.0/24,10.2.0.0/24'
10.0.0.0/14

$ printf '10.1.0.0/24\n10.2.0.0/24' | curl -s -X POST http://localhost:3000/ --data-binary @-
10.0.0.0/14

$ curl -s 'http://localhost:3000/?cidrs=10.0.0.0/24,10.0.1.0/24,192.168.0.0/24&summarize=true'
10.0.0.0/23
192.168.0.0/24
```

## Library

```toml
[dependencies]
cidrthings = { path = "." }
```

```rust
use cidrthings::{summarize_contiguous, minimal_supernet, Cidr};

fn main() {
    let blocks: Vec<Cidr> = ["10.1.0.0/24", "10.2.0.0/24"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

    println!("{}", minimal_supernet(&blocks).unwrap()); // 10.0.0.0/14

    let blocks: Vec<Cidr> = ["10.0.0.0/24", "10.0.1.0/24", "192.168.0.0/24"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

    for s in summarize_contiguous(&blocks).unwrap() {
        println!("{s}"); // 10.0.0.0/23, then 192.168.0.0/24
    }
}
```

## Contrib

Portable single-file implementations in [`contrib/`](contrib/):

- [`cidrthings.sh`](contrib/cidrthings.sh) — bash, IPv4 only, no dependencies
- [`cidrthings.py`](contrib/cidrthings.py) — Python 3, IPv4 and IPv6, uses `ipaddress` from stdlib

## How it works

For each input block, compute its network address (first address) and broadcast address (last address). XOR the overall minimum start with the maximum end to find where they diverge — the number of leading zero bits in that XOR is the supernet prefix length. The supernet network address is the minimum start masked to that prefix.

For `--summarize`, blocks are sorted by network address and split into runs wherever a gap exists between one block's broadcast and the next block's network. Each run is then summarized independently.

IPv4 and IPv6 are supported; mixing them in a single call is an error.

## License

MIT — see [LICENSE](LICENSE).
