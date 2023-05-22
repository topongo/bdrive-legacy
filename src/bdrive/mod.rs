mod upload;
mod hash;

pub use upload::{UploadOptions, UploadError};

use std::future::join;
use crate::conf::{Configs, PathsConf};
use crate::db::Database;
use crate::ssh::SSHClient;

#[derive(Debug)]
pub struct BDrive {
    db: Database,
    ssh: SSHClient,
}

impl BDrive {
    pub async fn new(cfg: Configs) -> mongodb::error::Result<Self> {
        print!("connecting to mongodb and ssh server...");
        let t_db = cfg.mongodb.to_db();
        let t_ssh = cfg.ssh.connect(cfg.paths);

        let (db, ssh) = join!(t_db, t_ssh).await;
        println!(" Done!");

        Ok(Self {
            db: db?,
            ssh: ssh?,
        })
    }
}
