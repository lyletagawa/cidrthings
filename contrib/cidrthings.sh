#!/usr/bin/env bash
# Compute the minimal IPv4 supernet enclosing a set of CIDR blocks.
# IPv6 is not supported (bash lacks 128-bit integers).
set -euo pipefail

usage() {
    echo "usage: cidrthings.sh <cidr>..." >&2
    echo "       bare IPs treated as /32; IPv6 not supported" >&2
    exit 1
}

ip_to_int() {
    local a b c d
    IFS='.' read -r a b c d <<< "$1"
    printf '%d\n' $(( (a << 24) | (b << 16) | (c << 8) | d ))
}

int_to_ip() {
    printf '%d.%d.%d.%d\n' \
        $(( ($1 >> 24) & 255 )) \
        $(( ($1 >> 16) & 255 )) \
        $(( ($1 >>  8) & 255 )) \
        $(( $1 & 255 ))
}

# Number of leading zero bits in a 32-bit value.
leading_zeros_32() {
    local n=$1 count=0 i
    if (( n == 0 )); then echo 32; return; fi
    for (( i=31; i>=0; i-- )); do
        if (( (n >> i) & 1 )); then break; fi
        count=$(( count + 1 ))
    done
    echo "$count"
}

# Sets globals _net and _bcast (integers) for the given CIDR.
parse_cidr() {
    local cidr=$1 addr prefix ip_int mask host_bits

    if [[ "$cidr" == *:* ]]; then
        echo "error: '$cidr': IPv6 not supported" >&2; exit 1
    fi

    if [[ "$cidr" == */* ]]; then
        addr="${cidr%%/*}"; prefix="${cidr##*/}"
    else
        addr="$cidr"; prefix=32
    fi

    if ! [[ "$addr" =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]]; then
        echo "error: '$cidr': invalid IPv4 address" >&2; exit 1
    fi

    if ! [[ "$prefix" =~ ^[0-9]+$ ]] || (( prefix > 32 )); then
        echo "error: '$cidr': invalid prefix length" >&2; exit 1
    fi

    ip_int=$(ip_to_int "$addr")

    if (( prefix == 0 )); then
        mask=0
    else
        mask=$(( (~0 << (32 - prefix)) & 0xFFFFFFFF ))
    fi

    _net=$(( ip_int & mask ))

    host_bits=$(( 32 - prefix ))
    if (( host_bits == 32 )); then
        _bcast=$(( 0xFFFFFFFF ))
    else
        _bcast=$(( _net | ((1 << host_bits) - 1) ))
    fi
}

if (( $# == 0 )); then usage; fi

min_start=-1
max_end=-1

for cidr in "$@"; do
    parse_cidr "$cidr"
    if (( min_start < 0 )) || (( _net < min_start )); then
        min_start=$_net
    fi
    if (( max_end < 0 )) || (( _bcast > max_end )); then
        max_end=$_bcast
    fi
done

diff=$(( min_start ^ max_end ))
prefix_len=$(leading_zeros_32 "$diff")

if (( prefix_len == 0 )); then
    mask=0
else
    mask=$(( (~0 << (32 - prefix_len)) & 0xFFFFFFFF ))
fi

echo "$(int_to_ip $(( min_start & mask )))/$prefix_len"
