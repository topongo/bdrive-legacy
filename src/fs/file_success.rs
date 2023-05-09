use super::{File, SyncState, state::*, Upload};

#[derive(Debug)]
pub enum FileSuccess<T, S> {
    Yes(T),
    No(File<S>)
}

impl FileSuccess<SyncState, LocalHashed> {
    pub fn from(value: Option<File<Remote>>, original: File<LocalHashed>) -> Self {
        match value {
            Some(f) => FileSuccess::Yes(original.attach_remote(f)),
            None => FileSuccess::No(original, )
        }
    }
}
