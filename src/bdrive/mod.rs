mod upload;
mod hash;
mod paths;

pub use upload::{UploadOptions, UploadError};

use std::future::join;
use std::path::PathBuf;
use crate::conf::{Configs, PathError, PathsConf};
use crate::db::Database;
use crate::ssh::{SSHClient, SSHPathBindings};

#[derive(Debug)]
pub struct BDrive {
    db: Database,
    ssh: SSHClient,
    pub paths: PathsConf,
    path_key: PathBuf
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

        let mut bd = Self {
            db: db?,
            ssh: ssh?,
            paths: cfg.paths,
            path_key: PathBuf::from(curdir)
        };

        bd.bind_ssh_paths();

        Ok(bd)
    }

    pub fn bind_paths(&mut self) {
        let bindings = SSHPathBindings {
            is_canonical: Box::new(|r: &str| self.paths.is_canonical(r)),
            absolute: Box::new(|r: &str| self.paths.absolute(r)),
            canonical: Box::new(|r: &str| self.paths.canonical(r)),
            to_remote: Box::new(|r: &str| self.paths.to_remote(r))
        };

        self.ssh.bind_paths(bindings);
    }
}
