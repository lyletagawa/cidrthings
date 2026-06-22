#!/usr/bin/env python3
import sys
from ipaddress import ip_network

if len(sys.argv) < 2:
    sys.exit("usage: cidrthings.py <cidr>...")

nets = [ip_network(a, strict=False) for a in sys.argv[1:]]
if len({n.version for n in nets}) > 1:
    sys.exit("error: cannot mix IPv4 and IPv6")

lo = min(int(n.network_address) for n in nets)
hi = max(int(n.broadcast_address) for n in nets)
bits = 32 if nets[0].version == 4 else 128
print(ip_network((lo, bits - (lo ^ hi).bit_length()), strict=False))
