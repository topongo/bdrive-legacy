mod upload;
mod hash;

pub use upload::{UploadOptions, UploadError};

use std::future::join;
use crate::conf::Configs;
use crate::db::Database;
use crate::ssh::SSHClient;

#[derive(Debug)]
pub struct BDrive {
    db: Database,
    pub ssh: SSHClient
}

impl BDrive {
    pub async fn new(cfg: Configs) -> mongodb::error::Result<Self> {
        let t_db = cfg.mongodb.to_db();
        let t_ssh = cfg.ssh.connect();
        let mut db: Database;
        let mut ssh: SSHClient;

        let (db, ssh) = join!(t_db, t_ssh).await;

        Ok(Self {
            db: db?,
            ssh: ssh?
        })
    }
}
