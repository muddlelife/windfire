use std::net::Ipv4Addr;
use std::str::FromStr;

pub fn is_cidr(s: &str) -> bool {
    let (ip, prefix) = match s.split_once('/') {
        Some(pair) => pair,
        None => return false,
    };

    let ip_ok = Ipv4Addr::from_str(ip).is_ok();
    let prefix_ok = prefix.parse::<u8>().map(|p| p <= 32).unwrap_or(false);

    ip_ok && prefix_ok
}

pub fn expand_cidr(cidr: &str) -> Vec<String> {
    let (base_ip, prefix) = cidr.split_once('/').unwrap();
    let prefix: u32 = prefix.parse().unwrap();

    let base_ip: Ipv4Addr = base_ip.parse().unwrap();
    let base_u32 = u32::from(base_ip);

    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };

    let network = base_u32 & mask;
    let broadcast = network | !mask;

    (network..=broadcast)
        .map(|ip| Ipv4Addr::from(ip).to_string())
        .collect()
}
