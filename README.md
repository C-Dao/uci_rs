# uci_rs

[![Test Status](https://github.com/c-dao/uci_rs/workflows/Tests/badge.svg?event=push)](https://github.com/C-Dao/uci_rs/actions)
[![Crate](https://img.shields.io/crates/v/uci_rs.svg)](https://crates.io/crates/uci_rs)
[![Documents](https://docs.rs/uci_rs/badge.svg)](https://docs.rs/uci_rs)

An openwrt's UCI (Unified Configuration Interface) parser and serializer.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
uci_rs = "0.1.0"
```

Example:

```rust
use uci_rs::{load_config, Uci, UciCommand};

/// file_path: /etc/config/network
///
/// config interface 'lan'
///         option type 'bridge'
///         option ifname 'eth0.1'
///         option proto 'static'
///         option netmask '255.255.255.0'
///         option ip6assign '60'
///         option ipaddr '192.168.1.1'
///
/// config interface 'wan'
///         option ifname 'eth0.2'
///         option proto 'dhcp'  

fn main(){
  let uci_network = load_config("network", "/etc/config")?;
  assert_eq!(uci.get_package(), "network");
  assert_eq!(uci.get_section("wan"), Ok(("interface", "wan" )));
  assert_eq!(uci.get_option("wan", "ifname"), Ok(("ifname", ["eth0.2"])));
  assert_eq!(uci.get_option("lan", "proto"), Ok(("proto", ["static"])));
}
```

## Documentation

[Docs.rs](https://docs.rs/uci_rs)

## License

uci_rs is distributed under the [LICENSE-MIT](LICENSE) .
