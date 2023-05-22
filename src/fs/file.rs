use std::fmt::Debug;
use std::path::{Path, PathBuf};
use crate::db::RemoteFile;
use super::state::*;

pub trait ToRemoteFile {
    fn to_remote_file(&self) -> RemoteFile;
}

pub trait LocalFile {
    fn path(&self) -> String;

    fn size(&self) -> u64 {
        // assume the file exists and is readable
        Path::new(&self.path()).metadata().unwrap().len()
    }
}

pub trait Upload: LocalFile + Debug + ToRemoteFile {
    fn local_identity(&self) -> Identity;
    fn attach_remote(self, f: File<Remote>) -> SyncState;
    fn downcast(&self) -> File<LocalHashed> {
        File {
            path: self.path(),
            state: LocalHashed {
                local: self.local_identity()
            }
        }
    }
    fn upcast(&self) -> File<Sync> {
        File {
            path: self.path(),
            state: Sync { id: self.local_identity() }
        }
    }
}

pub type BoxedUpload = Box<dyn Upload>;

pub trait Split {
    fn split(self) -> (File<LocalHashed>, File<Remote>);
}

#[derive(Debug)]
pub struct File<S> {
    pub path: String,
    state: S,
}

impl<S> File<S> {
    pub fn new(path: String, state: S) -> Self {
        Self {
            path,
            state
        }
    }
}

impl<S> LocalFile for File<S> {
    fn path(&self) -> String {
        self.path.clone()
    }
}

// macro_rules! define_file_status {
//     ($($i:ident),*) => {
//         pub enum FileStatus {
//             $($i(File<$i>)),*
//         }
//     };
// }
//
// define_file_status!(Local, LocalHashed, Remote, Sync);

impl File<Local> {
    pub fn hash(self) -> Result<File<LocalHashed>, (std::io::Error, File<Local>)> {
        Ok(File {
            state: LocalHashed::new(match Identity::try_from(self.path.to_string()) {
                Ok(v) => v,
                Err(e) => return Err((e, self))
            }),
            path: self.path
        })
    }
}

impl File<LocalHashed> {
    // create-public, we shouldn't assume the remote is equal to local.
    pub(crate) fn upcast(self) -> File<Sync> {
        File {
            path: self.path,
            state: Sync { id: self.state.local }
        }
    }
}

impl ToRemoteFile for File<LocalHashed> {
    fn to_remote_file(&self) -> RemoteFile {
        RemoteFile::new(self.path.clone(), self.state.local.hash(), self.state.local.size())
    }
}

impl ToRemoteFile for File<Remote> {
    fn to_remote_file(&self) -> RemoteFile {
        RemoteFile::new(self.path.clone(), self.state.remote.hash(), self.state.remote.size())
    }
}

impl ToRemoteFile for File<Sync> {
    fn to_remote_file(&self) -> RemoteFile {
        RemoteFile::new(self.path.to_string(), self.state.id.hash(), self.state.id.size())
    }
}

impl From<RemoteFile> for File<Remote> {
    fn from(value: RemoteFile) -> Self {
        Self {
            path: value.path,
            state: Remote { remote: Identity::new(value.hash, value.size) }
        }
    }
}

impl From<String> for File<Local> {
    fn from(value: String) -> Self {
        Self {
            path: value,
            state: Local {}
        }
    }
}

impl From<&str> for File<Local> {
    fn from(value: &str) -> Self {
        Self {
            path: value.to_string(),
            state: Local {}
        }
    }
}

impl From<PathBuf> for File<Local> {
    fn from(value: PathBuf) -> Self {
        Self {
            path: value.to_str().unwrap().to_string(),
            state: Local {}
        }
    }
}

impl PartialEq for File<LocalHashed> {
    fn eq(&self, other: &Self) -> bool {
        self.state.eq(&other.state)
    }
}

impl Upload for File<LocalHashed> {
    fn local_identity(&self) -> Identity {
        self.state.local.clone()
    }

    fn attach_remote(self, f: File<Remote>) -> SyncState {
        assert_eq!(f.path, self.path);

        if self.state.local == f.state.remote {
            SyncState::Sync(File { state: Sync { id: self.state.local }, path: self.path })
        } else {
            SyncState::Diff(File { path: self.path, state: Diff {
                local: self.state.local,
                remote: f.state.remote
            } })
        }
    }
}

impl ToRemoteFile for File<Diff> {
    fn to_remote_file(&self) -> RemoteFile {
        let id = self.local_identity();
        RemoteFile::new(self.path(), id.hash(), id.size())
    }
}

impl Upload for File<Diff> {
    fn local_identity(&self) -> Identity {
        self.state.local.clone()
    }

    /// Overwrite Diff's remote identity with new one.
    fn attach_remote(self, f: File<Remote>) -> SyncState {
        assert_eq!(f.path, self.path);

        if self.state.local == f.state.remote {
            SyncState::Sync(File { state: Sync { id: self.state.local }, path: self.path })
        } else {
            SyncState::Diff(File { path: self.path, state: Diff {
                local: self.state.local,
                remote: f.state.remote
            } })
        }
    }
}

impl Split for File<Diff> {
    fn split(self) -> (File<LocalHashed>, File<Remote>) {
        (File {
            path: self.path.clone(),
            state: LocalHashed { local: self.state.local }
        },
         File {
             path: self.path,
             state: Remote { remote: self.state.remote }
         })
    }
}

impl Split for File<Sync> {
    fn split(self) -> (File<LocalHashed>, File<Remote>) {
        (File {
            path: self.path.clone(),
            state: LocalHashed { local: self.state.id.clone() }
        },
         File {
             path: self.path,
             state: Remote { remote: self.state.id }
         })
    }
}

#[derive(Debug)]
pub enum SyncState {
    Sync(File<Sync>),
    Diff(File<Diff>)
}
