mod bindings;

pub use bindings::SSHPathBindings;

use std::fmt::{Debug, Formatter};
use ssh2::{Error, Session, Sftp};
use tokio::net::TcpStream;
use std::fs::File as StdFile;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::Instant;
use crate::conf::{PathError, PathsConf};

pub struct SSHClient {
    session: Session,
    sftp: Option<Sftp>,
    path_bindings: Option<SSHPathBindings>
}

#[derive(Debug)]
pub enum SSHError {
    SSH2(Error),
    Path(String),
    MkdirError(String)
}

impl From<PathError> for SSHError {
    fn from(value: PathError) -> Self {
        Self::Path(value.to_string())
    }
}

impl From<ssh2::Error> for SSHError {
    fn from(value: Error) -> Self {
        Self::SSH2(value)
    }
}

const BUFFSIZE: usize = 2 << 20;

impl SSHClient {
    pub fn new() -> Self {
        Self { session: Session::new().unwrap(), sftp: None, path_bindings: None}
    }

    pub async fn connect(
        &mut self,
        username: String,
        port: u16,
        host: String
    ) -> std::io::Result<()> {
        let stream = TcpStream::connect(format!("{host}:{port}")).await?;
        self.session.set_tcp_stream(stream);
        self.session.handshake()?;

        let mut agent = self.session.agent()?;
        agent.connect()?;
        agent.list_identities()?;
        let identities = agent.identities()?;
        let mut errors = vec![];
        for identity in identities {
            match agent.userauth(&username, &identity) {
                Ok(()) => {
                    self.session.set_blocking(true);
                    println!("creating sftp");
                    self.sftp = Some(self.session.sftp()?);
                    return Ok(())
                },
                Err(e) => errors.push(e)
            }
        }

        Err(errors.pop().unwrap())?
    }

    pub fn bind_paths(&mut self, bindings: SSHPathBindings) {
        self.path_bindings = Some(bindings)
    }

    fn sftp(&self) -> &Sftp {
        self.sftp.as_ref().unwrap()
    }

    fn paths(&self) -> &SSHPathBindings { self.path_bindings.as_ref().unwrap() }

    pub fn write(&self, rel: String, size: u64) -> Result<(), SSHError> {
        // todo: manage permission numbers...
        assert!(self.session.authenticated());
        let path = self.paths().absolute(&rel);
        let remote = Path::new(&self.paths.remote).join(&path);
        println!("uploading file {:?} to {:?}", self.paths.mid_absolute(&rel).unwrap(), remote);


        let mut local_reader = BufReader::with_capacity(BUFFSIZE, StdFile::open(path.clone()).unwrap());

        // println!("request scp send...");

        /// Recursively try to mkdir a folder
        fn recursive_mkdir(s: &Sftp, mut vec: Vec<String>) -> Result<Vec<String>, SSHError> {
            println!("recursing over {:?}", vec);
            if vec.len() == 1 {
                Err(SSHError::MkdirError("reached root, cannot recurse further".to_string()))
            } else {
                let pop = vec.pop().unwrap();
                let join = PathBuf::from(vec.join("/"));
                if let Err(e) = s.mkdir(&join, 0o755) {
                    if e.message() != "no such file" {
                        Err(SSHError::MkdirError(e.message().to_string()))
                    } else {
                        let mut vec = recursive_mkdir(s, vec)?;
                        // now we know for sure that all the parent exists.
                        vec.push(pop);
                        println!("successfully created: {:?}", join);
                        s.mkdir(&join, 0o755).unwrap();
                        Ok(vec)
                    }
                } else {
                    println!("successfully created: {:?}", join);
                    Ok(vec)
                }
            }
        }

        let remote_file = match self.sftp.as_ref().unwrap().create(remote.as_path()) {
            Ok(f) => f,
            Err(e) => {
                match e.message() {
                    "no such file" => {
                        let remote_split: Vec<String> = remote.to_str().unwrap().split("/").map(|s| s.to_string()).collect();
                        recursive_mkdir(self.sftp(), remote_split)?;
                        self.sftp.as_ref().unwrap().create(remote.as_path())?
                    }
                    _ => Err(e)?
                }
            }
        };

        let mut ch = BufWriter::with_capacity(BUFFSIZE, remote_file);

        println!("start copying data...");

        let start = Instant::now();

        std::io::copy(&mut local_reader, &mut ch).unwrap();
        let elapsed = start.elapsed();
        println!("operation took {:?}", elapsed);
        println!("operation performance: {} bytes/sec", size / elapsed.as_secs());

        Ok(())
    }

    pub fn delete(&self, path: String) -> std::io::Result<()> {
        return Ok(self.sftp.as_ref().unwrap().unlink(path.as_ref())?)
    }
}

impl Debug for SSHClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SSHClient {{ session: Session {{ blocking: {:?} }} }}", self.session.is_blocking())
    }
}
