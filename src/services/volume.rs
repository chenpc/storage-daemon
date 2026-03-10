use libnexus::nexus_service;
use libnexus::NamedMap;
use std::fs;
use std::process::Command;

const SMB_CONF: &str = "/etc/samba/smb.conf";

#[derive(serde::Serialize)]
struct VolumeInfo {
    used: String,
    avail: String,
    refer: String,
    mountpoint: String,
}

fn zfs(args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("zfs").args(args).output()?;
    if !output.status.success() {
        anyhow::bail!("zfs {}: {}", args[0], String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Add a Samba share section directly to smb.conf if not already present.
fn add_samba_share(name: &str, path: &str) -> anyhow::Result<()> {
    let conf = fs::read_to_string(SMB_CONF).unwrap_or_default();
    let section_header = format!("[{}]", name);

    // Skip if share already exists
    if conf.contains(&section_header) {
        return Ok(());
    }

    // Append share section
    let share_section = format!(
        "\n[{}]\n   path = {}\n   browseable = yes\n   guest ok = yes\n   guest only = no\n   read only = no\n   writable = yes\n   create mask = 0664\n   directory mask = 0775\n",
        name, path
    );

    fs::write(SMB_CONF, conf + &share_section)?;
    reload_samba();
    Ok(())
}

/// Remove a Samba share section from smb.conf.
fn remove_samba_share(name: &str) -> anyhow::Result<()> {
    let conf = fs::read_to_string(SMB_CONF).unwrap_or_default();
    let section_header = format!("[{}]", name);

    let mut result = String::new();
    let mut skip_section = false;

    for line in conf.lines() {
        if line.trim().starts_with('[') {
            skip_section = line.trim() == section_header;
        }
        if !skip_section {
            result.push_str(line);
            result.push('\n');
        }
    }

    fs::write(SMB_CONF, result)?;
    reload_samba();
    Ok(())
}

/// Send SIGHUP to smbd to reload configuration.
fn reload_samba() {
    // Try to read PID from standard location
    if let Ok(pid_str) = fs::read_to_string("/run/samba/smbd.pid") {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            let _ = Command::new("kill").args(["-HUP", &pid.to_string()]).output();
            return;
        }
    }

    // Fallback: use smbcontrol
    let _ = Command::new("smbcontrol").args(["all", "reload-config"]).status();
}

pub struct Volume;

/// Manage ZFS volumes with Samba shares.
#[nexus_service]
impl Volume {
    /// Create a new ZFS dataset and share it over Samba.
    #[command]
    async fn create(
        &self,
        #[arg(hint = "volume name", doc = "Name for the new volume")] name: String,
        #[arg(hint = "pool", doc = "Pool to create the volume in", complete = "pool.list")] pool: String,
    ) -> anyhow::Result<String> {
        let dataset = format!("{}/{}", pool, name);

        // Set mountpoint to /mnt/pools/{pool}/{name}
        let mountpoint = format!("/mnt/pools/{}/{}", pool, name);
        zfs(&["create", "-o", &format!("mountpoint={}", mountpoint), &dataset])?;

        // Set directory permissions
        Command::new("chmod").args(["755", &mountpoint]).output()?;

        // Add Samba share
        add_samba_share(&name, &mountpoint)?;

        Ok(format!("Volume '{}' created on pool '{}' and shared via Samba", name, pool))
    }

    /// Delete a ZFS volume and remove its Samba share.
    #[command]
    async fn delete(
        &self,
        #[arg(hint = "pool/volume", doc = "Dataset to delete (pool/name)", complete = "volume.list")] dataset: String,
    ) -> anyhow::Result<String> {
        zfs(&["destroy", &dataset])?;

        // Share name is the last component of the dataset path.
        let share_name = dataset.rsplit('/').next().unwrap_or(&dataset);
        let _ = remove_samba_share(share_name);

        Ok(format!("Volume '{}' deleted", dataset))
    }

    /// List all ZFS volumes.
    #[command]
    async fn list(&self) -> anyhow::Result<NamedMap<VolumeInfo>> {
        let out = zfs(&["list", "-H", "-t", "filesystem", "-o", "name,used,avail,refer,mountpoint"])?;
        let vols = out
            .lines()
            .filter_map(|line| {
                let cols: Vec<&str> = line.split('\t').collect();
                if cols.len() < 5 { return None; }
                let name = cols[0];
                // Skip top-level pool datasets (no slash).
                if !name.contains('/') { return None; }
                Some((name.to_string(), VolumeInfo {
                    used: cols[1].to_string(),
                    avail: cols[2].to_string(),
                    refer: cols[3].to_string(),
                    mountpoint: cols[4].to_string(),
                }))
            })
            .collect();
        Ok(vols)
    }
}
