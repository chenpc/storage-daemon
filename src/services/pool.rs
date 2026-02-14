use libnexus::nexus_service;
use libnexus::NamedMap;
use std::process::Command;

#[derive(serde::Serialize)]
struct PoolInfo {
    size: String,
    alloc: String,
    free: String,
    frag: String,
    cap: String,
    dedup: String,
    health: String,
}

pub struct Pool;

/// Manage storage pools.
#[nexus_service]
impl Pool {
    /// Create a new storage pool.
    #[command]
    async fn create(
        &self,
        #[arg(doc = "Name for the new pool")] name: String,
        #[arg(doc = "RAID type", hint = "<raid>", complete = ["mirror", "raidz", "raidz2", "raidz3"])] raid_type: String,
        #[arg(doc = "Devices to use", hint = "<dev>...", complete = "block.list")] devices: Vec<String>,
    ) -> anyhow::Result<String> {
        let valid = ["mirror", "raidz", "raidz2", "raidz3"];
        if !valid.contains(&raid_type.as_str()) {
            anyhow::bail!("invalid raid type '{}' (expected: {})", raid_type, valid.join(", "));
        }

        let min_devs = match raid_type.as_str() {
            "raidz3" => 4,
            "raidz2" => 3,
            _ => 2,
        };
        if devices.len() < min_devs {
            anyhow::bail!(
                "{} requires at least {} devices, got {}",
                raid_type, min_devs, devices.len()
            );
        }

        let dev_paths: Vec<String> = devices.iter().map(|d| format!("/dev/{}", d)).collect();
        let output = Command::new("zpool")
            .arg("create")
            .arg("-f")
            .arg(&name)
            .arg(&raid_type)
            .args(&dev_paths)
            .output()?;
        if !output.status.success() {
            anyhow::bail!("zpool create failed: {}", String::from_utf8_lossy(&output.stderr).trim());
        }
        Ok(format!("Pool '{}' created", name))
    }

    /// Destroy a storage pool.
    #[command]
    async fn destroy(
        &self,
        #[arg(doc = "Pool to destroy", complete = "pool.list")] name: String,
    ) -> anyhow::Result<String> {
        let output = Command::new("zpool")
            .arg("destroy")
            .arg(&name)
            .output()?;
        if !output.status.success() {
            anyhow::bail!("zpool destroy failed: {}", String::from_utf8_lossy(&output.stderr).trim());
        }
        Ok(format!("Pool '{}' destroyed", name))
    }

    /// List all storage pools.
    #[command]
    async fn list(&self) -> anyhow::Result<NamedMap<PoolInfo>> {
        let output = Command::new("zpool")
            .args(["list", "-H"])
            .output()?;
        if !output.status.success() {
            anyhow::bail!("zpool list failed: {}", String::from_utf8_lossy(&output.stderr).trim());
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let pools: NamedMap<PoolInfo> = stdout
            .lines()
            .filter_map(|line| {
                let cols: Vec<&str> = line.split('\t').collect();
                if cols.len() < 8 { return None; }
                Some((cols[0].to_string(), PoolInfo {
                    size: cols[1].to_string(),
                    alloc: cols[2].to_string(),
                    free: cols[3].to_string(),
                    frag: cols[4].to_string(),
                    cap: cols[5].to_string(),
                    dedup: cols[6].to_string(),
                    health: cols[7].to_string(),
                }))
            })
            .collect();
        Ok(pools)
    }
}
