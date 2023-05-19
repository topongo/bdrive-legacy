use std::fmt::{Debug, Formatter};
use ssh2::{OpenFlags, OpenType, Session, Sftp};
use tokio::net::TcpStream;
use std::fs::File as StdFile;
use std::io::{BufReader, BufWriter, Read, Write};
use std::time::Instant;
use crate::fs::{File, FileSuccess, Upload};
use crate::fs::state::{LocalHashed, Sync};

pub struct SSHClient {
    session: Session,
    sftp: Option<Sftp>
}

const BUFFSIZE: usize = 2 << 20;

impl SSHClient {
    pub fn new() -> Self {
        Self { session: Session::new().unwrap() }
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
                Ok(()) => return Ok(()),
                Err(e) => errors.push(e)
            }
        }
        self.session.set_blocking(true);
        self.sftp = Some(self.session.sftp()?);

        Err(errors.pop().unwrap())?
    }

    pub fn write(&self, path: String, size: u64) -> Result<(), ssh2::Error> {
        assert!(self.session.authenticated());
        println!("calling function write({}, {})", path, size);
        let mut local_reader = BufReader::with_capacity(BUFFSIZE, StdFile::open(path.clone()).unwrap());

        println!("request scp send...");

        let mut ch = BufWriter::with_capacity(BUFFSIZE, self.sftp.unwrap().create("/home/pi/test.iso".as_ref())?);

        // let mut ch = self.session.scp_send(, 0o655, size, None)?;

        println!("start copying data...");

        // let mut buffer: [u8; BUFFSIZE] = [0; BUFFSIZE];
        // println!("wrote {} bytes to remote file", ch.write(&buffer).unwrap());
        // let mut tot_r = 0;
        // let mut r;
        // let mut w;
        let start = Instant::now();
        // loop {
        //     r = local_reader.read(&mut buffer).unwrap();
        //     if r == 0 {
        //         if tot_r == size {
        //             break
        //         } else {
        //             panic!("file partially sent! ({} instead of {} bytes)", tot_r, size);
        //         }
        //     } else {
        //         tot_r += r as u64;
        //     }
        //     w = ch.write(&mut buffer).unwrap();
        //     if w != r {
        //         panic!("read {} bytes in buffer but sent only {} bytes, while reading at {}", r, w, tot_r - r as u64);
        //     }
        // }
        // println!();
        std::io::copy(&mut local_reader, &mut ch).unwrap();
        let elapsed = start.elapsed();
        println!("operation took {:?}", elapsed);
        println!("operation performance: {} bytes/sec", size / elapsed.as_secs());

        Ok(())
    }

    pub fn delete(&self) -> std::io::Result<()> {

        todo!()
    }
}

impl Debug for SSHClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SSHClient {{ session: Session {{ blocking: {:?} }} }}", self.session.is_blocking())
    }
}
