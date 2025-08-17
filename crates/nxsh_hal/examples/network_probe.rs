use nxsh_hal::platform::Platform;

fn main() -> anyhow::Result<()> {
    let platform = Platform::current();
    println!("Testing Network Interface Detection...");

    let interfaces = platform.get_network_interfaces();
    if interfaces.is_empty() {
        println!("❌ No network interfaces found");
    } else {
        println!("✅ Found {} network interfaces:", interfaces.len());
        for interface in &interfaces {
            println!("  Interface: {}", interface.name);
            println!("    MAC Address: {}", interface.mac_address.as_deref().unwrap_or("Unknown"));
            println!("    MTU: {}", interface.mtu.unwrap_or(0));
            println!("    Addresses: {:?}", interface.ip_addresses);
            if let Some(stats) = &interface.statistics {
                println!("    RX bytes: {}, TX bytes: {}", stats.rx_bytes, stats.tx_bytes);
                println!("    RX packets: {}, TX packets: {}", stats.rx_packets, stats.tx_packets);
            }
            println!();
        }
    }

    Ok(())
}
