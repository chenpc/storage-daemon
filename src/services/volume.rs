use libnexus::nexus_service;
use libnexus::NamedMap;
use std::fs;
use std::process::Command;

const SHARES_DIR: &str = "/etc/samba/shares";

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

/// Ensure the shares directory exists.
fn ensure_shares_dir() -> anyhow::Result<()> {
    fs::create_dir_all(SHARES_DIR)?;
    Ok(())
}

fn reload_samba() {
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

        // Ensure the mount directory exists
        fs::create_dir_all(&mountpoint)?;

        // Create Samba share configuration
        let share_conf = format!(
            "[{name}]\n   path = {mountpoint}\n   browseable = yes\n   guest ok = yes\n   guest only = no\n   read only = no\n   writable = yes\n   create mask = 0664\n   directory mask = 0775\n"
        );
        ensure_shares_dir()?;
        fs::write(format!("{}/{}.conf", SHARES_DIR, name), share_conf)?;
        reload_samba();

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
        let conf_path = format!("{}/{}.conf", SHARES_DIR, share_name);
        let _ = fs::remove_file(&conf_path);
        reload_samba();

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
