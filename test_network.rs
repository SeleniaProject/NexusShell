use std::io::Write;
use nxsh_hal::platform::Platform;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let platform = Platform::new()?;
    
    println!("Testing Network Interface Detection...");
    
    // Get network interfaces
    let interfaces = platform.get_network_interfaces().await?;
    
    if interfaces.is_empty() {
        println!("❌ No network interfaces found");
    } else {
        println!("✅ Found {} network interfaces:", interfaces.len());
        for interface in &interfaces {
            println!("  Interface: {}", interface.name);
            println!("    MAC Address: {}", interface.mac_address.as_deref().unwrap_or("Unknown"));
            println!("    MTU: {}", interface.mtu.unwrap_or(0));
            println!("    Addresses: {:?}", interface.addresses);
            if let Some(stats) = &interface.statistics {
                println!("    RX bytes: {}, TX bytes: {}", stats.bytes_received, stats.bytes_sent);
                println!("    RX packets: {}, TX packets: {}", stats.packets_received, stats.packets_sent);
            }
            println!();
        }
    }
    
    Ok(())
}
