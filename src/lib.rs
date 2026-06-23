use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// An IPv4 or IPv6 CIDR block, e.g. `10.0.0.0/8` or `2001:db8::/32`.
///
/// Bare IP addresses (no `/` prefix) parse as `/32` (IPv4) or `/128` (IPv6).
///
/// # Examples
///
/// ```
/// use cidrthings::Cidr;
///
/// let c: Cidr = "10.0.0.0/8".parse().unwrap();
/// assert_eq!(c.to_string(), "10.0.0.0/8");
///
/// // bare IP becomes a host route
/// let h: Cidr = "192.168.1.1".parse().unwrap();
/// assert_eq!(h.to_string(), "192.168.1.1/32");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cidr {
    V4(Ipv4Cidr),
    V6(Ipv6Cidr),
}

/// An IPv4 network in CIDR notation. Host bits are always zeroed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Cidr {
    /// Network address (host bits zeroed).
    pub network: Ipv4Addr,
    /// Number of leading network bits (0–32).
    pub prefix_len: u8,
}

/// An IPv6 network in CIDR notation. Host bits are always zeroed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv6Cidr {
    /// Network address (host bits zeroed).
    pub network: Ipv6Addr,
    /// Number of leading network bits (0–128).
    pub prefix_len: u8,
}

/// Error returned when parsing a CIDR block from a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The IP address portion is not valid.
    InvalidAddress(String),
    /// The prefix length is not a valid integer.
    InvalidPrefix(String),
    /// The prefix length exceeds the maximum for this address family.
    PrefixTooLong { prefix: u8, max: u8 },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidAddress(s) => write!(f, "invalid address: {s}"),
            ParseError::InvalidPrefix(s) => write!(f, "invalid prefix length: {s}"),
            ParseError::PrefixTooLong { prefix, max } => {
                write!(
                    f,
                    "prefix /{prefix} exceeds maximum /{max} for this address family"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl Ipv4Cidr {
    /// Construct an [`Ipv4Cidr`], masking off any host bits in `addr`.
    pub fn new(addr: Ipv4Addr, prefix_len: u8) -> Result<Self, ParseError> {
        if prefix_len > 32 {
            return Err(ParseError::PrefixTooLong {
                prefix: prefix_len,
                max: 32,
            });
        }
        let network = apply_mask_v4(addr, prefix_len);
        Ok(Self {
            network,
            prefix_len,
        })
    }

    /// The last address in the block (all host bits set).
    pub fn broadcast(self) -> Ipv4Addr {
        let n = u32::from(self.network);
        let host_bits = 32 - self.prefix_len;
        let broadcast = if host_bits == 32 {
            !0u32
        } else {
            n | ((1u32 << host_bits) - 1)
        };
        Ipv4Addr::from(broadcast)
    }
}

impl Ipv6Cidr {
    /// Construct an [`Ipv6Cidr`], masking off any host bits in `addr`.
    pub fn new(addr: Ipv6Addr, prefix_len: u8) -> Result<Self, ParseError> {
        if prefix_len > 128 {
            return Err(ParseError::PrefixTooLong {
                prefix: prefix_len,
                max: 128,
            });
        }
        let network = apply_mask_v6(addr, prefix_len);
        Ok(Self {
            network,
            prefix_len,
        })
    }

    /// The last address in the block (all host bits set).
    pub fn broadcast(self) -> Ipv6Addr {
        let n = u128::from(self.network);
        let host_bits = 128 - self.prefix_len;
        let broadcast = if host_bits == 128 {
            !0u128
        } else {
            n | ((1u128 << host_bits) - 1)
        };
        Ipv6Addr::from(broadcast)
    }
}

fn apply_mask_v4(addr: Ipv4Addr, prefix_len: u8) -> Ipv4Addr {
    let n = u32::from(addr);
    let mask = if prefix_len == 0 {
        0u32
    } else {
        !0u32 << (32 - prefix_len)
    };
    Ipv4Addr::from(n & mask)
}

fn apply_mask_v6(addr: Ipv6Addr, prefix_len: u8) -> Ipv6Addr {
    let n = u128::from(addr);
    let mask = if prefix_len == 0 {
        0u128
    } else {
        !0u128 << (128 - prefix_len)
    };
    Ipv6Addr::from(n & mask)
}

impl FromStr for Ipv4Cidr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            Some((addr_str, prefix_str)) => {
                let addr = addr_str
                    .parse::<Ipv4Addr>()
                    .map_err(|_| ParseError::InvalidAddress(addr_str.to_owned()))?;
                let prefix_len = prefix_str
                    .parse::<u8>()
                    .map_err(|_| ParseError::InvalidPrefix(prefix_str.to_owned()))?;
                Self::new(addr, prefix_len)
            }
            None => {
                let addr = s
                    .parse::<Ipv4Addr>()
                    .map_err(|_| ParseError::InvalidAddress(s.to_owned()))?;
                Self::new(addr, 32)
            }
        }
    }
}

impl FromStr for Ipv6Cidr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            Some((addr_str, prefix_str)) => {
                let addr = addr_str
                    .parse::<Ipv6Addr>()
                    .map_err(|_| ParseError::InvalidAddress(addr_str.to_owned()))?;
                let prefix_len = prefix_str
                    .parse::<u8>()
                    .map_err(|_| ParseError::InvalidPrefix(prefix_str.to_owned()))?;
                Self::new(addr, prefix_len)
            }
            None => {
                let addr = s
                    .parse::<Ipv6Addr>()
                    .map_err(|_| ParseError::InvalidAddress(s.to_owned()))?;
                Self::new(addr, 128)
            }
        }
    }
}

impl FromStr for Cidr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addr_str = s.split_once('/').map(|(a, _)| a).unwrap_or(s);
        if addr_str.contains(':') {
            Ok(Cidr::V6(s.parse()?))
        } else {
            Ok(Cidr::V4(s.parse()?))
        }
    }
}

impl fmt::Display for Ipv4Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.network, self.prefix_len)
    }
}

impl fmt::Display for Ipv6Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.network, self.prefix_len)
    }
}

impl fmt::Display for Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cidr::V4(c) => c.fmt(f),
            Cidr::V6(c) => c.fmt(f),
        }
    }
}

/// Error returned by [`minimal_supernet`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupernetError {
    /// No CIDR blocks were provided.
    Empty,
    /// The input contained both IPv4 and IPv6 blocks.
    MixedFamilies,
}

impl fmt::Display for SupernetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SupernetError::Empty => write!(f, "no CIDR blocks provided"),
            SupernetError::MixedFamilies => {
                write!(f, "cannot compute supernet across IPv4 and IPv6 blocks")
            }
        }
    }
}

impl std::error::Error for SupernetError {}

/// Summarizes each contiguous span of `cidrs` as a single supernet.
///
/// Blocks are sorted by network address. Adjacent or overlapping blocks are
/// collected into a span; a gap in address space starts a new span. Each
/// span is then reduced with [`minimal_supernet`].
///
/// Returns [`SupernetError::Empty`] if `cidrs` is empty, or
/// [`SupernetError::MixedFamilies`] if it contains both IPv4 and IPv6 blocks.
///
/// # Examples
///
/// ```
/// use cidrthings::{summarize_contiguous, Cidr};
///
/// let blocks: Vec<Cidr> = [
///     "10.0.0.0/24", "10.0.1.0/24",   // contiguous → one group
///     "192.168.0.0/24", "192.168.1.0/24", // contiguous → another group
/// ]
/// .iter()
/// .map(|s| s.parse().unwrap())
/// .collect();
///
/// let groups = summarize_contiguous(&blocks).unwrap();
/// assert_eq!(groups[0].to_string(), "10.0.0.0/23");
/// assert_eq!(groups[1].to_string(), "192.168.0.0/23");
/// ```
pub fn summarize_contiguous(cidrs: &[Cidr]) -> Result<Vec<Cidr>, SupernetError> {
    if cidrs.is_empty() {
        return Err(SupernetError::Empty);
    }
    let has_v4 = cidrs.iter().any(|c| matches!(c, Cidr::V4(_)));
    let has_v6 = cidrs.iter().any(|c| matches!(c, Cidr::V6(_)));
    if has_v4 && has_v6 {
        return Err(SupernetError::MixedFamilies);
    }
    if has_v4 {
        let mut v4: Vec<Ipv4Cidr> = cidrs
            .iter()
            .map(|c| match c {
                Cidr::V4(x) => *x,
                _ => unreachable!(),
            })
            .collect();
        v4.sort_by_key(|c| u32::from(c.network));
        Ok(group_v4(&v4).into_iter().map(Cidr::V4).collect())
    } else {
        let mut v6: Vec<Ipv6Cidr> = cidrs
            .iter()
            .map(|c| match c {
                Cidr::V6(x) => *x,
                _ => unreachable!(),
            })
            .collect();
        v6.sort_by_key(|c| u128::from(c.network));
        Ok(group_v6(&v6).into_iter().map(Cidr::V6).collect())
    }
}

fn group_v4(sorted: &[Ipv4Cidr]) -> Vec<Ipv4Cidr> {
    let mut result = Vec::new();
    let mut group: Vec<Ipv4Cidr> = Vec::new();
    let mut max_end: u32 = 0;

    for &cidr in sorted {
        let net = u32::from(cidr.network);
        let bcast = u32::from(cidr.broadcast());
        if group.is_empty() || net <= max_end.saturating_add(1) {
            group.push(cidr);
            max_end = max_end.max(bcast);
        } else {
            result.push(minimal_supernet_v4(&group));
            group.clear();
            group.push(cidr);
            max_end = bcast;
        }
    }
    if !group.is_empty() {
        result.push(minimal_supernet_v4(&group));
    }
    result
}

fn group_v6(sorted: &[Ipv6Cidr]) -> Vec<Ipv6Cidr> {
    let mut result = Vec::new();
    let mut group: Vec<Ipv6Cidr> = Vec::new();
    let mut max_end: u128 = 0;

    for &cidr in sorted {
        let net = u128::from(cidr.network);
        let bcast = u128::from(cidr.broadcast());
        if group.is_empty() || net <= max_end.saturating_add(1) {
            group.push(cidr);
            max_end = max_end.max(bcast);
        } else {
            result.push(minimal_supernet_v6(&group));
            group.clear();
            group.push(cidr);
            max_end = bcast;
        }
    }
    if !group.is_empty() {
        result.push(minimal_supernet_v6(&group));
    }
    result
}

/// Returns the smallest single CIDR block that contains every block in `cidrs`.
///
/// Returns [`SupernetError::Empty`] if `cidrs` is empty, or
/// [`SupernetError::MixedFamilies`] if it contains both IPv4 and IPv6 blocks.
///
/// # Examples
///
/// ```
/// use cidrthings::{minimal_supernet, Cidr};
///
/// let blocks: Vec<Cidr> = ["10.1.0.0/24", "10.2.0.0/24"]
///     .iter()
///     .map(|s| s.parse().unwrap())
///     .collect();
/// assert_eq!(minimal_supernet(&blocks).unwrap().to_string(), "10.0.0.0/14");
///
/// // a block already contained within another is a no-op
/// let blocks: Vec<Cidr> = ["10.0.0.0/8", "10.1.0.0/24"]
///     .iter()
///     .map(|s| s.parse().unwrap())
///     .collect();
/// assert_eq!(minimal_supernet(&blocks).unwrap().to_string(), "10.0.0.0/8");
/// ```
pub fn minimal_supernet(cidrs: &[Cidr]) -> Result<Cidr, SupernetError> {
    if cidrs.is_empty() {
        return Err(SupernetError::Empty);
    }
    let has_v4 = cidrs.iter().any(|c| matches!(c, Cidr::V4(_)));
    let has_v6 = cidrs.iter().any(|c| matches!(c, Cidr::V6(_)));
    if has_v4 && has_v6 {
        return Err(SupernetError::MixedFamilies);
    }
    if has_v4 {
        let v4: Vec<Ipv4Cidr> = cidrs
            .iter()
            .map(|c| match c {
                Cidr::V4(x) => *x,
                _ => unreachable!(),
            })
            .collect();
        Ok(Cidr::V4(minimal_supernet_v4(&v4)))
    } else {
        let v6: Vec<Ipv6Cidr> = cidrs
            .iter()
            .map(|c| match c {
                Cidr::V6(x) => *x,
                _ => unreachable!(),
            })
            .collect();
        Ok(Cidr::V6(minimal_supernet_v6(&v6)))
    }
}

fn minimal_supernet_v4(cidrs: &[Ipv4Cidr]) -> Ipv4Cidr {
    let min_start = cidrs.iter().map(|c| u32::from(c.network)).min().unwrap();
    let max_end = cidrs
        .iter()
        .map(|c| u32::from(c.broadcast()))
        .max()
        .unwrap();
    let diff = min_start ^ max_end;
    let prefix_len = if diff == 0 {
        32
    } else {
        diff.leading_zeros() as u8
    };
    let mask = if prefix_len == 0 {
        0u32
    } else {
        !0u32 << (32 - prefix_len)
    };
    let network = Ipv4Addr::from(min_start & mask);
    Ipv4Cidr {
        network,
        prefix_len,
    }
}

fn minimal_supernet_v6(cidrs: &[Ipv6Cidr]) -> Ipv6Cidr {
    let min_start = cidrs.iter().map(|c| u128::from(c.network)).min().unwrap();
    let max_end = cidrs
        .iter()
        .map(|c| u128::from(c.broadcast()))
        .max()
        .unwrap();
    let diff = min_start ^ max_end;
    let prefix_len = if diff == 0 {
        128
    } else {
        diff.leading_zeros() as u8
    };
    let mask = if prefix_len == 0 {
        0u128
    } else {
        !0u128 << (128 - prefix_len)
    };
    let network = Ipv6Addr::from(min_start & mask);
    Ipv6Cidr {
        network,
        prefix_len,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cidr(s: &str) -> Cidr {
        s.parse().unwrap()
    }

    #[test]
    fn single_block_is_itself() {
        let result = minimal_supernet(&[cidr("10.1.0.0/24")]).unwrap();
        assert_eq!(result.to_string(), "10.1.0.0/24");
    }

    #[test]
    fn two_disjoint_24s() {
        let result = minimal_supernet(&[cidr("10.1.0.0/24"), cidr("10.2.0.0/24")]).unwrap();
        assert_eq!(result.to_string(), "10.0.0.0/14");
    }

    #[test]
    fn contained_block_uses_larger() {
        let result = minimal_supernet(&[cidr("10.0.0.0/8"), cidr("10.1.0.0/24")]).unwrap();
        assert_eq!(result.to_string(), "10.0.0.0/8");
    }

    #[test]
    fn host_routes() {
        let result = minimal_supernet(&[cidr("192.168.1.0/32"), cidr("192.168.1.1/32")]).unwrap();
        assert_eq!(result.to_string(), "192.168.1.0/31");
    }

    #[test]
    fn ipv6_basic() {
        let result = minimal_supernet(&[cidr("2001:db8::/32"), cidr("2001:db9::/32")]).unwrap();
        assert_eq!(result.to_string(), "2001:db8::/31");
    }

    #[test]
    fn mixed_families_error() {
        let result = minimal_supernet(&[cidr("10.0.0.0/8"), cidr("2001:db8::/32")]);
        assert_eq!(result, Err(SupernetError::MixedFamilies));
    }

    #[test]
    fn empty_error() {
        assert_eq!(minimal_supernet(&[]), Err(SupernetError::Empty));
    }

    #[test]
    fn three_blocks() {
        let result = minimal_supernet(&[
            cidr("192.168.0.0/24"),
            cidr("192.168.1.0/24"),
            cidr("192.168.2.0/24"),
        ])
        .unwrap();
        assert_eq!(result.to_string(), "192.168.0.0/22");
    }

    #[test]
    fn host_bits_ignored_in_input() {
        let a: Ipv4Cidr = "10.1.0.1/24".parse().unwrap();
        assert_eq!(a.network, "10.1.0.0".parse::<Ipv4Addr>().unwrap());
    }

    #[test]
    fn bare_ipv4_is_host_route() {
        let result = minimal_supernet(&[cidr("10.0.0.1"), cidr("10.0.0.2")]).unwrap();
        assert_eq!(result.to_string(), "10.0.0.0/30");
    }

    #[test]
    fn bare_ipv6_is_host_route() {
        let c: Cidr = "::1".parse().unwrap();
        assert_eq!(c.to_string(), "::1/128");
    }

    #[test]
    fn summarize_contiguous_two_spans() {
        let result = summarize_contiguous(&[
            cidr("10.0.0.0/24"),
            cidr("10.0.1.0/24"),
            cidr("192.168.0.0/24"),
            cidr("192.168.1.0/24"),
        ])
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].to_string(), "10.0.0.0/23");
        assert_eq!(result[1].to_string(), "192.168.0.0/23");
    }

    #[test]
    fn summarize_contiguous_overlapping_is_one_span() {
        let result = summarize_contiguous(&[cidr("10.0.0.0/8"), cidr("10.1.0.0/24")]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_string(), "10.0.0.0/8");
    }

    #[test]
    fn summarize_contiguous_single_block() {
        let result = summarize_contiguous(&[cidr("10.1.0.0/24")]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_string(), "10.1.0.0/24");
    }

    #[test]
    fn summarize_contiguous_three_separate_spans() {
        let result = summarize_contiguous(&[
            cidr("10.0.0.0/24"),
            cidr("172.16.0.0/16"),
            cidr("192.168.0.0/24"),
        ])
        .unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].to_string(), "10.0.0.0/24");
        assert_eq!(result[1].to_string(), "172.16.0.0/16");
        assert_eq!(result[2].to_string(), "192.168.0.0/24");
    }

    #[test]
    fn summarize_contiguous_ipv6() {
        let result = summarize_contiguous(&[cidr("2001:db8::/32"), cidr("2001:dba::/32")]).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn summarize_contiguous_empty_error() {
        assert_eq!(summarize_contiguous(&[]), Err(SupernetError::Empty));
    }

    #[test]
    fn summarize_contiguous_mixed_families_error() {
        assert_eq!(
            summarize_contiguous(&[cidr("10.0.0.0/8"), cidr("2001:db8::/32")]),
            Err(SupernetError::MixedFamilies)
        );
    }
}
