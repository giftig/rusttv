pub mod osmc;
pub mod upload;

use std::fs::File;
use std::io::{Error as IoError, Read};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use ssh2::{Error as SshError, Session};
use thiserror::Error;

pub struct SshClient {
    session: Session,
    tv_dir: PathBuf,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("An SSH error occurred: {0}")]
    Ssh(#[from] SshError),
    #[error("An IO error occurred: {0}")]
    Io(#[from] IoError),
    #[error("A fatal error occurred while transforming OS-specific strings")]
    PlatformError,
    #[error("An unexpected threading error occurred")]
    Thread
}

pub enum Auth {
    Password(String),
    Privkey(String)
}

type Result<T> = std::result::Result<T, ClientError>;

impl SshClient {
    // Simple sanitisation to make sure the path works ok in a single-quoted shell string
    // This undoubtedly misses some edge cases but will work ok given injection isn't a problem
    fn sanitise_shell_path(p: &Path) -> Result<String> {
        let s = p.to_str().ok_or(ClientError::PlatformError)?;
        Ok(s.replace("\"", "\\\""))
    }

    pub fn connect(
        host: &str,
        port: usize,
        username: &str,
        auth: &Auth,
        tv_dir: &Path,
    ) -> Result<SshClient> {
        let mut client = SshClient {
            session: Session::new()?,
            tv_dir: tv_dir.to_path_buf(),
        };
        let conn = TcpStream::connect(format!("{host}:{port}"))?;
        client.session.set_tcp_stream(conn);
        client.session.handshake()?;

        match auth {
            Auth::Password(pwd) => client.session.userauth_password(username, &pwd)?,
            Auth::Privkey(file) => client.session.userauth_pubkey_file(username, None, Path::new(file), None)?,
        }

        Ok(client)
    }

    fn execute(&mut self, cmd: &str) -> Result<String> {
        let mut channel = self.session.channel_session()?;
        channel.exec(cmd)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        Ok(output)
    }

    pub fn list_shows(&mut self) -> Result<Vec<String>> {
        let path_sane = Self::sanitise_shell_path(&self.tv_dir)?;
        let output = self.execute(&format!("ls -1 \"{}\"", path_sane))?;
        Ok(output.split_terminator("\n").map(String::from).collect())
    }

    pub fn list_episodes(&mut self, show: &str) -> Result<Vec<String>> {
        let mut path = self.tv_dir.clone();
        path.push(show);
        let path_sane = Self::sanitise_shell_path(&path)?;

        let output = self.execute(&format!("ls -1 \"{}\"", path_sane))?;
        Ok(output.split_terminator("\n").map(String::from).collect())
    }

    pub fn upload_file(&mut self, local: &Path, remote: &Path) -> Result<()> {
        let size = local.metadata()?.len();
        let out_chan = self.session.scp_send(remote, 0o644, size, None)?;

        let local_file = File::open(local)?;

        upload::handle_upload(local_file, out_chan, size)
    }
}
