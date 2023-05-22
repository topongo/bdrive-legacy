mod upload;
mod hash;
mod paths;

pub use upload::{UploadOptions, UploadError};

use std::future::join;
use std::path::PathBuf;
use crate::conf::{Configs, PathsConf};
use crate::db::Database;
use crate::ssh::SSHClient;

#[derive(Debug)]
pub struct BDrive {
    db: Database,
    // todo: remove these pub(s)
    pub ssh: SSHClient,
    pub paths: PathsConf,
    exe_path: PathBuf
}

impl BDrive {
    pub async fn new(cfg: Configs) -> mongodb::error::Result<Self> {
        let curdir = std::env::current_dir()?.canonicalize()?;
        if !curdir.starts_with(&cfg.paths.local) {
            // todo: fix this
            panic!("out of bounds: {:?}", curdir);
        }
        // assume prefix is present
        let curdir = curdir.strip_prefix(&cfg.paths.local).unwrap();
        // todo: fix this
        std::env::set_current_dir(&cfg.paths.local).unwrap();
        let t_db = cfg.mongodb.to_db();
        let t_ssh = cfg.ssh.connect();

        let (db, ssh) = join!(t_db, t_ssh).await;

        let bd = Self {
            db: db?,
            ssh: ssh?,
            paths: cfg.paths,
            exe_path: PathBuf::from(curdir)
        };

        Ok(bd)
    }
}
