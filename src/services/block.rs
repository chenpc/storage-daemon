use libnexus::nexus_service;
use libnexus::NamedMap;
use serde_json::Value;
use std::process::Command;

#[derive(serde::Serialize)]
struct BlockDevice {
    size: Option<String>,
    dtype: Option<String>,
    model: Option<String>,
    mountpoint: Option<String>,
}

impl BlockDevice {
    fn from_json(v: &Value) -> Option<(String, Self)> {
        let name = v["name"].as_str()?.to_string();
        Some((name, Self {
            size: v["size"].as_str().map(str::to_string),
            dtype: v["type"].as_str().map(str::to_string),
            model: v["model"].as_str().map(str::to_string),
            mountpoint: v["mountpoint"].as_str().map(str::to_string),
        }))
    }
}

fn run_lsblk(args: &[&str]) -> anyhow::Result<Value> {
    let output = Command::new("lsblk")
        .args(args)
        .arg("-J")
        .output()?;
    if !output.status.success() {
        anyhow::bail!("lsblk failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(serde_json::from_reader(output.stdout.as_slice())?)
}

pub struct Block;

/// Query and inspect block devices.
#[nexus_service]
impl Block {
    /// List all block devices.
    #[command]
    async fn list(&self) -> anyhow::Result<NamedMap<BlockDevice>> {
        let lsblk = run_lsblk(&["-d", "-o", "NAME,SIZE,TYPE,MODEL,MOUNTPOINT"])?;
        let devices = lsblk["blockdevices"].as_array()
            .ok_or_else(|| anyhow::anyhow!("unexpected lsblk output"))?;
        Ok(devices
            .iter()
            .filter(|d| d["type"].as_str() == Some("disk"))
            .filter(|d| d["mountpoint"].as_str() != Some("/"))
            .filter_map(BlockDevice::from_json)
            .collect())
    }

    /// Show info for a block device.
    #[command]
    async fn info(
        &self,
        #[arg(doc = "Device name to inspect", complete = "block.list")] device: String,
    ) -> anyhow::Result<BlockDevice> {
        let lsblk = run_lsblk(&["-o", "NAME,SIZE,TYPE,MODEL,MOUNTPOINT", &format!("/dev/{}", device)])?;
        let dev = lsblk["blockdevices"].as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| anyhow::anyhow!("device '{}' not found", device))?;
        BlockDevice::from_json(dev)
            .map(|(_, d)| d)
            .ok_or_else(|| anyhow::anyhow!("failed to parse device '{}'", device))
    }
}
