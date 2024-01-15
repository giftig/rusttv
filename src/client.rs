use std::fs::File;
use std::io::{Error as IoError, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use ssh2::{Error as SshError, Session};
use thiserror::Error;

// Buffer size for file transfers
const BUF_SIZE: usize = 1024 * 4;

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
    PlatformError
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
        privkey: &str,
        tv_dir: &Path,
    ) -> Result<SshClient> {
        let mut client = SshClient {
            session: Session::new()?,
            tv_dir: tv_dir.to_path_buf(),
        };
        let conn = TcpStream::connect(format!("{host}:{port}"))?;
        client.session.set_tcp_stream(conn);
        client.session.handshake()?;
        client
            .session
            .userauth_pubkey_file(username, None, Path::new(privkey), None)?;

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

    // TODO: Low level logic here, maybe split into another module
    pub fn upload_file(&mut self, local: &Path, remote: &Path) -> Result<()> {
        let size = local.metadata()?.len();
        let mut out_chan = self.session.scp_send(remote, 0o644, size, None)?;

        let mut local_file = File::open(local)?;
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

        loop {
            let n = local_file.read(&mut buf)?;
            if n == 0 {
                break;
            }

            let out_buf = &buf[0..n];
            consume_buffer(&mut out_chan, out_buf)?;

            if n < BUF_SIZE {
                break;
            }
        }

        out_chan.send_eof()?;
        out_chan.wait_eof()?;
        out_chan.close()?;
        out_chan.wait_close()?;

        Ok(())
    }
}

// Completely consume the buffer, allowing the writer to backpressure where needed
fn consume_buffer(writer: &mut dyn Write, buf: &[u8]) -> Result<()> {
    let mut total: usize = 0;

    loop {
        let written = writer.write(&buf[total..])?;
        total += written;

        if total == buf.len() {
            return Ok(());
        }
    }
}
