# cidrthings

[![CI](https://github.com/lyletagawa/cidrthings/actions/workflows/ci.yml/badge.svg)](https://github.com/lyletagawa/cidrthings/actions/workflows/ci.yml)

Computes the minimal supernet that contains a set of CIDR blocks.

Given any mix of IPv4 or IPv6 networks, returns the smallest single CIDR block that encloses all of them.

## Install

```
cargo install --path .                 # CLI only
cargo install --path . --features web  # CLI + web server
```

## CLI

```
cidrthings <cidr>...
```

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

## Web server

```
cargo run --features web --bin cidrthings-web
```

Starts an HTTP server on port 3000 (override with `$PORT`).

- `GET /` — browser UI with a textarea for entering blocks
- `GET /?cidrs=10.1.0.0/24,10.2.0.0/24` — supernet as plain text
- `POST /` — newline- or comma-separated blocks in the body, supernet as plain text

```
$ curl -s 'http://localhost:3000/?cidrs=10.1.0.0/24,10.2.0.0/24'
10.0.0.0/14

$ printf '10.1.0.0/24\n10.2.0.0/24' | curl -s -X POST http://localhost:3000/ --data-binary @-
10.0.0.0/14
```

## Library

```toml
[dependencies]
cidrthings = { path = "." }
```

```rust
use cidrthings::{minimal_supernet, Cidr};

fn main() {
    let blocks: Vec<Cidr> = ["10.1.0.0/24", "10.2.0.0/24"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

    println!("{}", minimal_supernet(&blocks).unwrap()); // 10.0.0.0/14
}
```

## How it works

For each input block, compute its network address (first address) and broadcast address (last address). XOR the overall minimum start with the maximum end to find where they diverge — the number of leading zero bits in that XOR is the supernet prefix length. The supernet network address is the minimum start masked to that prefix.

IPv4 and IPv6 are supported; mixing them in a single call is an error.

## License

MIT — see [LICENSE](LICENSE).
