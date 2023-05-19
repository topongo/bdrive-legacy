use std::fmt::{Debug, Formatter};
use super::BDrive;
use crate::fs::{Upload, File, state::*, FileSuccess, SyncState, Split, LocalFile};
use ssh2::Error as SSHError;

impl BDrive {
    /// This function tries upload a file to the remote server via ssh.
    /// If it succeeds then it tries to updates the remote database with the changes.
    /// If it fails the remote file is deleted and an UploadError is returned.
    pub async fn upload(&mut self, file: impl Upload + Debug + Sized, options: Option<UploadOptions>) -> Result<File<Sync>, UploadError> {
        let options = if let Some(o) = options { o } else { UploadOptions::default() };
        println!("local file before searching {:?}", file);
        // println!("search result: {:?}", self.get_file_file(file).await);

        match self.db.get_file_file(file).await {
            Ok(r) => match r {
                FileSuccess::Yes(st) => {
                    match st {
                        SyncState::Sync(f) => {
                            println!("file is online and synced");
                            Ok(f)
                        },
                        SyncState::Diff(f) => {
                            println!("file is uploaded but not in sync");
                            if options.overwrite {
                                println!("overwriting remote file");
                                if let Err(e) = self.ssh.write(f.path(), f.size()) {
                                    return Err(UploadError::NetworkError(f.downcast(), e))
                                }
                                // upload ok, update database.
                                // todo: if fails we need to clean remote files
                                match self.db.push(f).await {
                                    FileSuccess::Yes(f) => Ok(f),
                                    FileSuccess::No(e, o) => Err(UploadError::MongoDBError(self.clean_storage(o).await, e))
                                }
                            } else {
                                let (local, remote) = f.split();
                                Err(UploadError::OverwriteError(local, remote))
                            }
                        }
                    }
                }
                FileSuccess::No((), o) => {
                    if let Err(e) = self.ssh.write(o.path(), o.size()) {
                        return Err(UploadError::NetworkError(o.downcast(), e))
                    }
                    // todo: check if remote file is ok
                    todo!()
                }
            }
            Err((e, f)) => Err(UploadError::MongoDBError(f.downcast(), e))
        }
    }

    async fn clean_storage(&self, f: File<Diff>) -> File<LocalHashed> {
        // todo: actually we should check if the file existed, if so, restore the original 
        match self.ssh.delete(f.path()) {
            Ok(_) => f.downcast(),
            Err(e) => {
                // todo: if this fails add an entry to database, when possible clean dangling files
                f.downcast();
                todo!()
            }
        }
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
    NetworkError(File<LocalHashed>, ssh2::Error),
    MongoDBError(File<LocalHashed>, mongodb::error::Error),
    Culo(Box<dyn Upload>)
}

