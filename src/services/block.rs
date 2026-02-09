use libnexus::nexus_service;

pub struct Block;

/// Query and inspect block devices.
#[nexus_service]
impl Block {
    /// List all block devices.
    #[command]
    async fn list(&self) -> libnexus::Result<String> {
        Ok("sda, sdb, sdc, nvme0n1".to_string())
    }

    /// Show info for a block device.
    #[command]
    async fn info(&self, #[arg(doc = "Device path to inspect", complete = "block.list")] device: String) -> libnexus::Result<String> {
        Ok(format!("Block device '{}': size=500G, type=SSD", device))
    }
}
