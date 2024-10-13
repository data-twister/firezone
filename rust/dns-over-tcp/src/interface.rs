use std::time::Instant;

use smoltcp::{
    iface::{Config, Interface, Route},
    wire::{HardwareAddress, Ipv4Address, Ipv4Cidr, Ipv6Address, Ipv6Cidr},
};

use crate::stub_device::InMemoryDevice;

const IP4_ADDR: Ipv4Address = Ipv4Address::new(127, 0, 0, 1);
const IP6_ADDR: Ipv6Address = Ipv6Address::new(0, 0, 0, 0, 0, 0, 0, 1);

/// Creates a smoltcp [`Interface`].
///
/// smoltcp's abstractions allow to directly plug it in a TUN device.
/// As a result, it has all the features you'd expect from a network interface:
/// - Setting IP addresses
/// - Defining routes
///
/// In our implementation, we don't want to use any of that.
/// Our device is entirely backed by in-memory buffers and we and selectively feed IP packets to it.
/// Therefore, we configure it to:
/// - Accept any packet
/// - Define dummy IPs (localhost for IPv4 and IPv6)
/// - Define catch-all routes (0.0.0.0/0) that routes all traffic to the interface
pub fn create_interface(device: &mut InMemoryDevice, now: Instant) -> Interface {
    let mut interface = Interface::new(Config::new(HardwareAddress::Ip), device, now.into());
    // Accept packets with any destination IP, not just our interface.
    interface.set_any_ip(true);

    // Set our interface IPs. These are just dummies and don't show up anywhere!
    interface.update_ip_addrs(|ips| {
        ips.push(Ipv4Cidr::new(IP4_ADDR, 32).into()).unwrap();
        ips.push(Ipv6Cidr::new(IP6_ADDR, 128).into()).unwrap();
    });

    // Configure catch-all routes, meaning all packets given to `smoltcp` will be routed to our interface.
    interface.routes_mut().update(|routes| {
        routes.push(Route::new_ipv4_gateway(IP4_ADDR)).unwrap();
        routes.push(Route::new_ipv6_gateway(IP6_ADDR)).unwrap();
    });

    interface
}