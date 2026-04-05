use libnexus::nexus_service;
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
struct CpuInfo {
    usage_percent: f64,
    load_1m: f64,
    load_5m: f64,
    load_15m: f64,
    cores: usize,
}

#[derive(Serialize)]
struct MemoryInfo {
    total_mb: u64,
    used_mb: u64,
    free_mb: u64,
    available_mb: u64,
    usage_percent: f64,
}

#[derive(Serialize)]
struct NetworkInterface {
    rx_bytes: u64,
    tx_bytes: u64,
    rx_packets: u64,
    tx_packets: u64,
}

#[derive(Serialize)]
struct DiskInfo {
    total_mb: u64,
    used_mb: u64,
    free_mb: u64,
    usage_percent: f64,
    mountpoint: String,
}

#[derive(Serialize)]
struct SystemStatus {
    hostname: String,
    uptime_seconds: u64,
    cpu: CpuInfo,
    memory: MemoryInfo,
    network: std::collections::BTreeMap<String, NetworkInterface>,
    disks: Vec<DiskInfo>,
}

fn read_hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn read_uptime() -> u64 {
    fs::read_to_string("/proc/uptime")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64
}

fn read_cpu() -> CpuInfo {
    let loadavg = fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let parts: Vec<&str> = loadavg.split_whitespace().collect();
    let load_1m = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let load_5m = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let load_15m = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0);

    // Count CPU cores
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let cores = cpuinfo.lines().filter(|l| l.starts_with("processor")).count().max(1);

    // CPU usage from /proc/stat (snapshot — shows since boot)
    let stat = fs::read_to_string("/proc/stat").unwrap_or_default();
    let usage_percent = stat
        .lines()
        .find(|l| l.starts_with("cpu "))
        .map(|l| {
            let vals: Vec<u64> = l.split_whitespace().skip(1).filter_map(|s| s.parse().ok()).collect();
            if vals.len() >= 4 {
                let total: u64 = vals.iter().sum();
                let idle = vals[3];
                if total > 0 {
                    ((total - idle) as f64 / total as f64 * 100.0 * 10.0).round() / 10.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);

    CpuInfo { usage_percent, load_1m, load_5m, load_15m, cores }
}

fn read_memory() -> MemoryInfo {
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total = 0u64;
    let mut free = 0u64;
    let mut available = 0u64;
    let mut buffers = 0u64;
    let mut cached = 0u64;

    for line in meminfo.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 { continue; }
        let val: u64 = parts[1].parse().unwrap_or(0); // in kB
        match parts[0] {
            "MemTotal:" => total = val,
            "MemFree:" => free = val,
            "MemAvailable:" => available = val,
            "Buffers:" => buffers = val,
            "Cached:" => cached = val,
            _ => {}
        }
    }

    // If MemAvailable not present, approximate
    if available == 0 { available = free + buffers + cached; }
    let used = total.saturating_sub(available);
    let usage_percent = if total > 0 {
        (used as f64 / total as f64 * 100.0 * 10.0).round() / 10.0
    } else {
        0.0
    };

    MemoryInfo {
        total_mb: total / 1024,
        used_mb: used / 1024,
        free_mb: free / 1024,
        available_mb: available / 1024,
        usage_percent,
    }
}

fn read_network() -> std::collections::BTreeMap<String, NetworkInterface> {
    let net_dev = fs::read_to_string("/proc/net/dev").unwrap_or_default();
    let mut result = std::collections::BTreeMap::new();

    for line in net_dev.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 11 { continue; }
        let name = parts[0].trim_end_matches(':').to_string();
        if name == "lo" { continue; } // skip loopback
        result.insert(name, NetworkInterface {
            rx_bytes: parts[1].parse().unwrap_or(0),
            rx_packets: parts[2].parse().unwrap_or(0),
            tx_bytes: parts[9].parse().unwrap_or(0),
            tx_packets: parts[10].parse().unwrap_or(0),
        });
    }
    result
}

fn read_disks() -> Vec<DiskInfo> {
    let output = std::process::Command::new("df")
        .args(["-m", "--output=target,size,used,avail,pcent"])
        .output();
    let stdout = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => return vec![],
    };

    stdout
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 { return None; }
            let mountpoint = parts[0].to_string();
            // Only show real filesystems
            if !mountpoint.starts_with('/') { return None; }
            Some(DiskInfo {
                mountpoint,
                total_mb: parts[1].parse().unwrap_or(0),
                used_mb: parts[2].parse().unwrap_or(0),
                free_mb: parts[3].parse().unwrap_or(0),
                usage_percent: parts[4].trim_end_matches('%').parse().unwrap_or(0.0),
            })
        })
        .collect()
}

pub struct System;

/// System status and information.
#[nexus_service]
impl System {
    /// Get full system status (CPU, memory, network, disk).
    #[command]
    async fn status(&self) -> anyhow::Result<SystemStatus> {
        Ok(SystemStatus {
            hostname: read_hostname(),
            uptime_seconds: read_uptime(),
            cpu: read_cpu(),
            memory: read_memory(),
            network: read_network(),
            disks: read_disks(),
        })
    }

    /// Get current hostname.
    #[command]
    async fn hostname(&self) -> anyhow::Result<String> {
        Ok(read_hostname())
    }

    /// Set hostname.
    #[command]
    async fn set_hostname(
        &self,
        #[arg(doc = "New hostname")] name: String,
    ) -> anyhow::Result<String> {
        // Set runtime hostname
        let output = std::process::Command::new("hostname").arg(&name).output()?;
        if !output.status.success() {
            anyhow::bail!("hostname command failed: {}", String::from_utf8_lossy(&output.stderr).trim());
        }
        // Persist
        fs::write("/etc/hostname", format!("{}\n", name))?;
        Ok(format!("Hostname set to '{}'", name))
    }
}
