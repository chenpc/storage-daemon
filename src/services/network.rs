use libnexus::nexus_service;

pub struct Network;

/// Manage network interfaces and addressing.
#[nexus_service]
impl Network {
    /// List all network interfaces.
    #[command]
    async fn list(&self) -> libnexus::Result<String> {
        Ok("eth0, eth1, lo".to_string())
    }

    /// Show info for a network interface.
    #[command]
    async fn info(
        &self,
        #[arg(hint = "interface", doc = "Network interface to inspect", complete = "network.list")]
        iface: String,
    ) -> libnexus::Result<String> {
        Ok(format!("Interface '{}': ip=10.0.0.1, mask=255.255.255.0, state=UP", iface))
    }

    /// Set IP address and subnet mask on an interface.
    #[command]
    async fn setip(
        &self,
        #[arg(hint = "interface", doc = "Network interface to configure", complete = "network.list")]
        iface: String,
        #[arg(hint = "ip", doc = "IP address (e.g. 10.0.0.1)")]
        ip: String,
        #[arg(hint = "mask", doc = "Subnet mask (e.g. 255.255.255.0)")]
        mask: String,
    ) -> libnexus::Result<String> {
        Ok(format!("Set {}/{} on interface '{}'", ip, mask, iface))
    }
}
