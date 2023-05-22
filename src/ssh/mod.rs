use std::fmt::{Debug, Formatter};
use ssh2::{Error, Session, Sftp};
use tokio::net::TcpStream;
use std::fs::File as StdFile;
use std::io::{BufReader, BufWriter};
use std::fmt::Write;
use std::path::PathBuf;
use std::time::Instant;
use indicatif::{ProgressState, ProgressStyle};
use crate::conf::{PathError, PathsConf};

pub struct SSHClient {
    session: Session,
    sftp: Option<Sftp>
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

impl From<Error> for SSHError {
    fn from(value: Error) -> Self {
        Self::SSH2(value)
    }
}

const BUFF_SIZE: usize = 2 << 20;

impl SSHClient {
    pub fn new() -> Self {
        Self { session: Session::new().unwrap(), sftp: None}
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

    fn sftp(&self) -> &Sftp {
        self.sftp.as_ref().unwrap()
    }

    pub fn write(&self, paths: &PathsConf, rel: String, size: u64) -> Result<(), SSHError> {
        // todo: manage permission numbers...
        assert!(self.session.authenticated());
        let path = paths.absolute(&rel).unwrap();
        let remote = paths.to_remote(&rel);
        println!("uploading file {:?} to {:?}", paths.canonical(&rel).unwrap(), remote);


        let mut local_reader = BufReader::with_capacity(BUFF_SIZE, StdFile::open(path).unwrap());

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

        let mut ch = BufWriter::with_capacity(BUFF_SIZE, remote_file);

        let bar = indicatif::ProgressBar::new(size);
        bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
        let mut bar_reader = bar.wrap_read(&mut local_reader);

        println!("start copying data...");

        let start = Instant::now();

        std::io::copy(&mut bar_reader, &mut ch).unwrap();
        let elapsed = start.elapsed();
        println!("operation took {:?}", elapsed);
        println!("operation performance: {:0.2} bytes/sec", size as f64 / elapsed.as_secs_f64());

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
