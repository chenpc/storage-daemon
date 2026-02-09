use libnexus::nexus_service;

pub struct Pool;

/// Manage storage pools.
#[nexus_service]
impl Pool {
    /// Create a new storage pool.
    #[command]
    async fn create(&self, #[arg(doc = "Name for the new pool")] name: String) -> libnexus::Result<String> {
        Ok(format!("Pool '{}' created", name))
    }

    /// Destroy a storage pool.
    #[command]
    async fn destroy(&self, #[arg(doc = "Pool to destroy", complete = "pool.create")] name: String) -> libnexus::Result<String> {
        Ok(format!("Pool '{}' destroyed", name))
    }
}
