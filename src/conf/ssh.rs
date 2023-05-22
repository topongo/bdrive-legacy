use crate::conf::{PathsConf, SSHConfig};
use crate::ssh::SSHClient;

impl SSHConfig {
    pub async fn connect(self) -> std::io::Result<SSHClient> {
        let mut ssh_client = SSHClient::new();
        ssh_client.connect(
            self.username,
            self.port,
            self.host
        ).await?;
        Ok(ssh_client)
    }
}