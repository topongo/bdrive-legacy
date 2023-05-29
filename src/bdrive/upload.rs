use std::fmt::Debug;
use super::BDrive;
use crate::fs::{Upload, File, state::*, FileSuccess, SyncState, Split, LocalFile};
use crate::ssh::SSHError;

impl BDrive {
    /// This function tries upload a file to the remote server via ssh.
    /// If it succeeds then it tries to updates the remote database with the changes.
    /// If it fails the remote file is deleted and an UploadError is returned.
    pub async fn upload<'a>(&mut self, file: impl Upload + Debug + Sized + 'a, options: Option<UploadOptions>) -> Result<File<Sync>, UploadError> {
        let options = if let Some(o) = options { o } else { UploadOptions::default() };
        // println!("local file before searching {:?}", file);
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
                                if let Err(e) = self.ssh.write(&self.paths, f.path(), f.size()) {
                                    return Err(UploadError::SSHError(f.downcast(), e))
                                }
                                // upload ok, update database.
                                print!("file uploaded, trying to update database...");
                                match self.db.push(f).await {
                                    FileSuccess::Yes(f) => { println!("Ok, done"); Ok(f) },
                                    FileSuccess::No(e, o) => Err(UploadError::MongoDBError(self.clean_storage(o).await, e))
                                }
                            } else if options.confirm {
                                Err(UploadError::ConfirmationNeeded(f))
                            } else {
                                let (local, remote) = f.split();
                                Err(UploadError::OverwriteError(local, remote))
                            }
                        }
                    }
                }
                FileSuccess::No((), o) => {
                    println!("cannot find file remotely, creating new one.");
                    if let Err(e) = self.ssh.write(&self.paths, o.path(), o.size()) {
                        Err(UploadError::SSHError(o.downcast(), e))
                    } else {
                        println!("upload success, creating file in db.");
                        match self.db.create(o).await {
                            FileSuccess::Yes(s) => Ok(s),
                            FileSuccess::No(e, o) => Err(UploadError::MongoDBError(o.downcast(), e))
                        }
                    }
                }
            }
            Err((e, f)) => Err(UploadError::MongoDBError(f.downcast(), e))
        }
    }

    async fn clean_storage(&self, f: File<Diff>) -> File<LocalHashed> {
        // todo: actually we should check if the file existed, if so, restore the original
        println!("deleting {}, since it cannot be added to database.", f.path);
        match self.ssh.delete(f.path()) {
            Ok(_) => f.downcast(),
            Err(_e) => {
                // todo: if this fails add an entry to database, when possible clean dangling files
                f.downcast();
                todo!()
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct UploadOptions {
    pub overwrite: bool,
    pub confirm: bool
}

pub struct UploadOptionsBuilder {
    inner: UploadOptions
}

impl UploadOptions {
    pub fn builder() -> UploadOptionsBuilder {
        UploadOptionsBuilder { inner: Self::default() }
    }
}

impl UploadOptionsBuilder {
    pub fn overwrite(mut self, overwrite: bool) -> UploadOptionsBuilder {
        self.inner.overwrite = overwrite;
        self
    }

    pub fn confirm(mut self, confirm: bool) -> UploadOptionsBuilder {
        self.inner.confirm = confirm;
        self
    }

    pub fn build(self) -> UploadOptions {
        self.inner
    }
}

#[derive(Debug)]
pub enum UploadError {
    OverwriteError(File<LocalHashed>, File<Remote>),
    SSHError(File<LocalHashed>, SSHError),
    MongoDBError(File<LocalHashed>, mongodb::error::Error),
    ConfirmationNeeded(File<Diff>)
}

