mod file;
mod inode;
mod file_success;

pub mod state;
pub use file::{File, ToRemoteFile, SyncState, Upload, Split};
pub use file_success::FileSuccess;