mod file;

pub use file::RemoteFile;

use mongodb::{Collection, Database as MongoDb, IndexModel};
use mongodb::bson::doc;
use mongodb::options::IndexOptions;
use mongodb::error::Error;
use crate::bdrive::{UploadError, UploadOptions};
use crate::fs::state::{Diff, LocalHashed, Remote, Sync};
use crate::fs::{File, FileSuccess, SyncState, ToRemoteFile, Split, LocalFile, Upload};

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

    pub async fn upload(&mut self, file: File<LocalHashed>, options: Option<UploadOptions>) -> Result<File<Sync>, UploadError> {
        // async fn upload_wrapper(
        //     se: &mut Database,
        //     file: impl Upload,
        //     hint_existing: Option<bool>
        // ) -> Result<File<Sync>, UploadError>
        // {
        //     match se.upload_file(file).await {
        //         FileSuccess::Yes(s) => match se.set_hash(s.to_remote_file(), hint_existing).await {
        //             Ok(_) => Ok(s),
        //             Err(e) => {
        //                 // TODO
        //                 // upload has failed, so discard the operation
        //                 // and delete file on remote server
        //                 // ====
        //
        //                 let (local, _) = s.split();
        //                 Err(UploadError::MongoDBError(local, e))
        //             }
        //         }
        //         FileSuccess::No(e, o) => Err(UploadError::NetworkError(e))
        //     }
        // }
        //
        // let options = if let Some(o) = options { o } else { UploadOptions::default() };
        // println!("local file before searching {:?}", file);
        // // println!("search result: {:?}", self.get_file_file(file).await);
        //
        // match self.get_file_file(file).await {
        //     Ok(r) => match r {
        //         FileSuccess::Yes(st) => {
        //             match st {
        //                 SyncState::Sync(f) => {
        //                     println!("file is online and synced");
        //                     Ok(f)
        //                 },
        //                 SyncState::Diff(f) => {
        //                     println!("file is uploaded but not in sync");
        //                     if options.overwrite {
        //                         println!("overwriting remote file");
        //                         let (local, _) = f.split();
        //                         upload_wrapper(
        //                             self,
        //                             local,
        //                             Some(true)
        //                         ).await
        //                     } else {
        //                         let (local, remote) = f.split();
        //                         Err(UploadError::OverwriteError(local, remote))
        //                     }
        //                 }
        //             }
        //         }
        //         // FileSuccess::No(_, o) => {
        //         //     upload_wrapper(
        //         //         self,
        //         //         o,
        //         //         Some(false)
        //         //     ).await
        //         // }
        //     }
        //     Err((e, f)) => Err(UploadError::MongoDBError(f, e))
        // }
        todo!()
    }

    // async fn upload_file(&self, f: File<LocalHashed>) -> FileSuccess<File<Sync>, LocalHashed> {
    //     use std::thread::sleep;
    //     use std::time::Duration;
    //
    //     // TODO
    //     // upload to server, simulate by sleeping
    //     // ===
    //
    //     sleep(Duration::from_secs(2));
    //
    //     FileSuccess::Yes(f.to_sync())
    // }

    // async fn create_file(&mut self, f: &RemoteFile) -> mongodb::error::Result<InsertOneResult> {
    //     Ok(self.files.insert_one(f, None).await?)
    // }

    pub async fn exists(&mut self, path: impl ToString) -> mongodb::error::Result<bool> {
        self.files.find(doc! {"path": path.to_string()}, None).await.map(|r| r.deserialize_current().is_ok())
    }

    async fn set_hash(&mut self, f: RemoteFile, hint_existing: Option<bool>) -> mongodb::error::Result<()> {
        match hint_existing {
            Some(ex) => {
                if ex {
                    self.files.update_one(doc! {"path": f.path}, doc! {"$set" : {"hash": f.hash}}, None).await.map(|_| ())
                } else {
                    self.files.insert_one(f, None).await.map(|_| ())
                }
            }
            None => {
                if self.exists(&f.path).await? {
                    self.files.update_one(doc! {"path": f.path}, doc! {"$set": {"hash": f.hash}}, None).await.map(|_| ())
                } else {
                    self.files.insert_one(f, None).await.map(|_| ())
                }
            }
        }
    }

    /// Sync a File<Diff> with database, sets remote hash to local hash
    pub async fn push(&mut self, f: File<Diff>) -> FileSuccess<File<Sync>, mongodb::error::Error, File<Diff>> {
        let (local, remote) = f.split();
        let rfile = local.to_remote_file();
        match self.files.update_one(
            doc! {"path": rfile.path},
            doc! {"$set": {
                "size": rfile.size,
                "hash": rfile.hash
            }}, None).await {
            Ok(_) => FileSuccess::Yes(local.to_sync()),
            Err(e) => FileSuccess::No(e, match local.attach_remote(remote) {
                SyncState::Sync(_) => panic!(),
                SyncState::Diff(d) => d
            })
        }
    }
}
