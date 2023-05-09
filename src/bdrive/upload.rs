use std::fmt::Debug;
use super::BDrive;
use crate::fs::{Upload, File, state::*};
use ssh2::Error as SSHError;

impl BDrive {
    /// This function tries upload a file to the remote server via ssh.
    /// If it succeeds then it tries to updates the remote database with the changes.
    /// If it fails the remote file is deleted and an UploadError is returned.
    pub async fn uplaod(&mut self, file: impl Upload + Debug, options: Option<UploadOptions>) -> Result<File<Sync>, UploadError> {
        let options = if let Some(o) = options { o } else { UploadOptions::default() };
        println!("local file before searching {:?}", file);
        // println!("search result: {:?}", self.get_file_file(file).await);

        // match self.db.get_file_file(file).await {
        //
        // }

        todo!()
    }
}

pub struct UploadOptions {
    pub overwrite: bool
}

pub struct UploadOptionsBuilder {
    inner: UploadOptions
}

impl UploadOptions {
    pub fn builder() -> UploadOptionsBuilder {
        UploadOptionsBuilder { inner: Self::default() }
    }

    pub fn default() -> Self {
        Self { overwrite: false }
    }
}

impl UploadOptionsBuilder {
    pub fn overwrite(mut self, overwrite: bool) -> UploadOptionsBuilder {
        self.inner.overwrite = overwrite;
        self
    }

    pub fn build(self) -> UploadOptions {
        self.inner
    }
}

#[derive(Debug)]
pub enum UploadError {
    OverwriteError(File<LocalHashed>, File<Remote>),
    NetworkError(File<LocalHashed>, /* todo */),
    MongoDBError(File<LocalHashed>, mongodb::error::Error)
}

