use libnexus::nexus_service;

pub struct Volume;

/// Manage storage volumes on block devices.
#[nexus_service]
impl Volume {
    /// Create a new volume on the specified disk.
    #[command]
    async fn create(
        &self,
        #[arg(hint = "volume name", doc = "Name for the new volume")] name: String,
        #[arg(hint = "device", doc = "Block device to create the volume on", complete = "block.list")] disk: String,
    ) -> libnexus::Result<String> {
        Ok(format!("Volume '{}' created on disk '{}'", name, disk))
    }

    /// Delete an existing volume.
    #[command]
    async fn delete(
        &self,
        #[arg(hint = "volume name", doc = "Volume to delete", complete = "volume.list")] name: String,
    ) -> libnexus::Result<String> {
        Ok(format!("Volume '{}' deleted", name))
    }

    /// List all volumes.
    #[command]
    async fn list(&self) -> libnexus::Result<String> {
        Ok("vol0, vol1, vol2".to_string())
    }
}
