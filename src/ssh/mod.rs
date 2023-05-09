use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use ssh2::Session;
use tokio::net::TcpStream;

pub struct SSHClient {
    session: Session
}

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
        // let mut ssh_agent = self.session.agent()?;
        // ssh_agent.connect()?;

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

        Err(errors.pop().unwrap())?
    }
}

impl Debug for SSHClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SSHClient {{ session: Session }}", )
    }
}
