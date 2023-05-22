mod file;

pub use file::RemoteFile;

use mongodb::{Collection, Database as MongoDb, IndexModel};
use mongodb::bson::doc;
use mongodb::options::IndexOptions;
use mongodb::error::Error;
use crate::fs::state::{Diff, Remote, Sync};
use crate::fs::{File, FileSuccess, SyncState, ToRemoteFile, Split, Upload};

#[derive(Debug)]
pub struct Database {
    #[allow(dead_code)]
    db: MongoDb,
    files: Collection<RemoteFile>
}

impl Database {
    pub async fn connect(db: MongoDb) -> Result<Self, Error> {
        let files = db.collection::<RemoteFile>("files");
        files.create_index(
            IndexModel::builder()
                .options(IndexOptions::builder()
                    .unique(true)
                    .build())
                .keys(doc! {"path": 1})
                .build(),
            None
        ).await?;
        Ok(Self { db, files })
    }

    pub async fn get_file_path(&self, path: impl ToString) -> Result<Option<File<Remote>>, Error> {
        Ok(self.files.find_one(doc! {"path": path.to_string()}, None).await?.map(|f| f.to_local()))
    }

    pub async fn get_file_file<'a>(&self, file: impl Upload + 'a) -> Result<FileSuccess<SyncState, (), Box<dyn Upload + 'a>>, (Error, impl Upload)> {
        match self.get_file_path(&file.path()).await {
            Ok(r) => Ok(match r {
                Some(f) => FileSuccess::Yes(file.attach_remote(f)),
                None => FileSuccess::No((), Box::new(file))
            }),
            Err(e) => Err((e, file))
        }
    }

    /// Check if remote path exists
    pub async fn exists(&mut self, path: impl ToString) -> mongodb::error::Result<bool> {
        self.files.find(doc! {"path": path.to_string()}, None).await.map(|r| r.deserialize_current().is_ok())
    }

    /// Sync a File<Diff> with database, sets remote hash to local hash
    pub async fn push(&mut self, f: File<Diff>) -> FileSuccess<File<Sync>, mongodb::error::Error, File<Diff>> {
        let (local, remote) = f.split();
        println!("updating {:?} to {:?} in db...", local, remote);
        let rfile = local.to_remote_file();
        match self.files.update_one(
            doc! {"path": rfile.path},
            doc! {"$set": {
                "size": rfile.size as u32,
                "hash": rfile.hash
            }},
            None
        ).await {
            Ok(_) => FileSuccess::Yes(local.to_sync()),
            Err(e) => FileSuccess::No(e, match local.attach_remote(remote) {
                SyncState::Sync(_) => panic!(),
                SyncState::Diff(d) => d
            })
        }
    }

    /// Add a `dyn Upload` to db, convert it to `File<Sync>` on success
    pub async fn create<'a>(&mut self, f: Box<dyn Upload + 'a>) -> FileSuccess<File<Sync>, mongodb::error::Error, Box<dyn Upload + 'a>> {
        let rfile = f.to_remote_file();
        
        match self.files.insert_one(
            rfile,
            None
        ).await {
            Ok(_) => FileSuccess::Yes(f.upcast()),
            Err(e) => FileSuccess::No(e, f)
        }
    }
}
