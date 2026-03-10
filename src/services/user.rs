use libnexus::nexus_service;
use libnexus::NamedMap;
use std::fs;
use std::process::Command;

#[derive(serde::Serialize)]
struct UserInfo {
    uid: u32,
    gid: u32,
    comment: String,
    home: String,
}

pub struct User;

/// Manage system users.
#[nexus_service]
impl User {
    /// List system users (root and regular users).
    #[command]
    async fn list(&self) -> anyhow::Result<NamedMap<UserInfo>> {
        let content = fs::read_to_string("/etc/passwd")?;
        let users: NamedMap<UserInfo> = content
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() < 6 {
                    return None;
                }
                let username = parts[0].to_string();
                let uid: u32 = parts[2].parse().ok()?;
                let gid: u32 = parts[3].parse().ok()?;
                let comment = parts[4].to_string();
                let home = parts[5].to_string();

                // Include root (uid 0) or normal users (uid >= 1000)
                if uid == 0 || uid >= 1000 {
                    Some((username, UserInfo { uid, gid, comment, home }))
                } else {
                    None
                }
            })
            .collect();
        Ok(users)
    }

    /// Create a new user.
    #[command]
    async fn create(
        &self,
        #[arg(doc = "Username for the new user")] name: String,
        #[arg(doc = "Password for the new user")] password: String,
        #[arg(doc = "Full name or comment (optional)", hint = "<comment>")] comment: String,
    ) -> anyhow::Result<String> {
        // Create user with useradd
        let mut cmd = Command::new("useradd");
        cmd.arg("-m"); // Create home directory
        if !comment.is_empty() {
            cmd.arg("-c").arg(&comment);
        }
        cmd.arg(&name);

        let output = cmd.output()?;
        if !output.status.success() {
            anyhow::bail!("useradd failed: {}", String::from_utf8_lossy(&output.stderr).trim());
        }

        // Set password using chpasswd
        let chpasswd_input = format!("{}:{}", name, password);
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("echo '{}' | chpasswd", chpasswd_input))
            .output()?;

        if !output.status.success() {
            anyhow::bail!("chpasswd failed: {}", String::from_utf8_lossy(&output.stderr).trim());
        }

        Ok(format!("User '{}' created", name))
    }

    /// Delete a user.
    #[command]
    async fn delete(
        &self,
        #[arg(doc = "Username to delete", complete = "user.list")] name: String,
    ) -> anyhow::Result<String> {
        let output = Command::new("userdel")
            .arg("-r") // Remove home directory
            .arg(&name)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("userdel failed: {}", String::from_utf8_lossy(&output.stderr).trim());
        }

        Ok(format!("User '{}' deleted", name))
    }

    /// Change user password.
    #[command]
    async fn passwd(
        &self,
        #[arg(doc = "Username", complete = "user.list")] name: String,
        #[arg(doc = "New password")] password: String,
    ) -> anyhow::Result<String> {
        let chpasswd_input = format!("{}:{}", name, password);
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("echo '{}' | chpasswd", chpasswd_input))
            .output()?;

        if !output.status.success() {
            anyhow::bail!("chpasswd failed: {}", String::from_utf8_lossy(&output.stderr).trim());
        }

        Ok(format!("Password changed for user '{}'", name))
    }
}
