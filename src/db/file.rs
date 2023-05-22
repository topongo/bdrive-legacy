use serde::{Serialize, Deserialize};
use crate::fs::File;
use crate::fs::state::Remote;

#[derive(Serialize, Deserialize)]
pub struct Local;
#[derive(Serialize, Deserialize)]
pub struct Sync;

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteFile {
    pub path: String,
    pub hash: String,
    pub size: u64,
}

impl RemoteFile {
    pub fn new(path: String, hash: String, size: u64) -> Self {
        Self { path, hash, size }
    }

    // pub fn push(self, c: &mut Collection<Self>) {
    //     // c.insert_one(&self, None)
    // }

    pub fn to_local(self) -> File<Remote> {
        File::<Remote>::from(self)
    }
}
