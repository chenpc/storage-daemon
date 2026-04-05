use libnexus::nexus_service;
use libnexus::NamedMap;
use serde::Serialize;
use std::fs;
use std::process::Command;

#[derive(Serialize)]
struct InterfaceInfo {
    state: String,
    mac: String,
    ipv4: String,
    ipv6: String,
    rx_bytes: u64,
    tx_bytes: u64,
}

fn parse_interfaces() -> NamedMap<InterfaceInfo> {
    let mut result = NamedMap::new();

    // Enumerate interfaces from /sys/class/net/
    let entries = match fs::read_dir("/sys/class/net") {
        Ok(e) => e,
        Err(_) => return result,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "lo" { continue; }

        let base = format!("/sys/class/net/{}", name);
        let state = fs::read_to_string(format!("{}/operstate", base))
            .unwrap_or_default().trim().to_uppercase();
        let mac = fs::read_to_string(format!("{}/address", base))
            .unwrap_or_default().trim().to_string();

        // Parse IP addresses from `ip addr show <iface>` text output
        let mut ipv4 = String::new();
        let mut ipv6 = String::new();
        if let Ok(output) = Command::new("ip").args(["addr", "show", &name]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("inet ") && ipv4.is_empty() {
                    // inet 10.0.2.15/24 brd ...
                    ipv4 = trimmed.split_whitespace().nth(1).unwrap_or("").to_string();
                } else if trimmed.starts_with("inet6 ") && ipv6.is_empty() {
                    let addr = trimmed.split_whitespace().nth(1).unwrap_or("");
                    if !addr.starts_with("fe80") {
                        ipv6 = addr.to_string();
                    }
                }
            }
        }

        let (rx_bytes, tx_bytes) = read_iface_stats(&name);
        result.insert(name, InterfaceInfo { state, mac, ipv4, ipv6, rx_bytes, tx_bytes });
    }
    result
}

fn read_iface_stats(name: &str) -> (u64, u64) {
    let net_dev = fs::read_to_string("/proc/net/dev").unwrap_or_default();
    for line in net_dev.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }
        let iface = parts[0].trim_end_matches(':');
        if iface == name && parts.len() >= 11 {
            let rx = parts[1].parse().unwrap_or(0);
            let tx = parts[9].parse().unwrap_or(0);
            return (rx, tx);
        }
    }
    (0, 0)
}

pub struct Network;

/// Manage network interfaces and hostname.
#[nexus_service]
impl Network {
    /// List all network interfaces with their addresses and status.
    #[command]
    async fn list(&self) -> anyhow::Result<NamedMap<InterfaceInfo>> {
        Ok(parse_interfaces())
    }

    /// Show info for a specific network interface.
    #[command]
    async fn info(
        &self,
        #[arg(hint = "interface", doc = "Network interface to inspect", complete = "network.list")]
        iface: String,
    ) -> anyhow::Result<InterfaceInfo> {
        let all = parse_interfaces();
        all.into_iter()
            .find(|(name, _)| name == &iface)
            .map(|(_, info)| info)
            .ok_or_else(|| anyhow::anyhow!("interface '{}' not found", iface))
    }

    /// Set IP address on an interface (CIDR notation, e.g. 10.0.0.1/24).
    #[command]
    async fn setip(
        &self,
        #[arg(hint = "interface", doc = "Network interface to configure", complete = "network.list")]
        iface: String,
        #[arg(hint = "ip/prefix", doc = "IP address in CIDR (e.g. 10.0.0.1/24)")]
        cidr: String,
    ) -> anyhow::Result<String> {
        // Flush existing addresses and add new one
        let _ = Command::new("ip").args(["addr", "flush", "dev", &iface]).output();
        let output = Command::new("ip")
            .args(["addr", "add", &cidr, "dev", &iface])
            .output()?;
        if !output.status.success() {
            anyhow::bail!("ip addr add failed: {}", String::from_utf8_lossy(&output.stderr).trim());
        }
        Ok(format!("Set {} on interface '{}'", cidr, iface))
    }

    /// Get current hostname.
    #[command]
    async fn hostname(&self) -> anyhow::Result<String> {
        Ok(fs::read_to_string("/etc/hostname").unwrap_or_default().trim().to_string())
    }

    /// Set hostname and update mDNS/Bonjour advertisement.
    #[command]
    async fn set_hostname(
        &self,
        #[arg(doc = "New hostname")] name: String,
    ) -> anyhow::Result<String> {
        // Set runtime hostname
        let output = Command::new("hostname").arg(&name).output()?;
        if !output.status.success() {
            anyhow::bail!("hostname failed: {}", String::from_utf8_lossy(&output.stderr).trim());
        }
        // Persist
        fs::write("/etc/hostname", format!("{}\n", name))?;

        // Update Avahi/Bonjour hostname
        let _ = Command::new("avahi-set-host-name").arg(&name).output();

        Ok(format!("Hostname set to '{}'", name))
    }
}
