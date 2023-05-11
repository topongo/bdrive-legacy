mod file;
mod inode;
mod file_success;

pub mod state;
pub use file::{File, ToRemoteFile, SyncState, Upload, Split, LocalFile};
pub use file_success::FileSuccess;