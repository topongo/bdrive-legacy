use crate::fs::LocalFile;
use super::{File, SyncState, state::*, Upload};

#[derive(Debug)]
pub enum FileSuccess<T, S> {
    Yes(T),
    No(S)
}

// impl FileSuccess<SyncState, LocalHashed> {
//     pub fn from(value: Option<File<Remote>>, original: impl Upload) -> Self {
//
//     }
// }
