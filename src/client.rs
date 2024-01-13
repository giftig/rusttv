use std::fs::File;
use std::io::{Error as IoError, Read, Write};
use std::net::TcpStream;
use std::path::Path;

use ssh2::{Error as SshError, Session};

// Buffer size for file transfers
const BUF_SIZE: usize = 1024 * 4;

pub struct SshClient {
    session: Session,
    tv_dir: String,
}

#[derive(Debug)]
pub enum ClientError {
    Ssh(SshError),
    Io(IoError)
}

impl From<SshError> for ClientError {
    fn from(err: SshError) -> ClientError {
        ClientError::Ssh(err)
    }
}

impl From<IoError> for ClientError {
    fn from(err: IoError) -> ClientError {
        ClientError::Io(err)
    }
}

type Result<T> = std::result::Result<T, ClientError>;

impl SshClient {
    pub fn connect(host: &str, port: usize, username: &str, privkey: &str, tv_dir: &str) -> Result<SshClient> {
        let mut client = SshClient { session: Session::new()?, tv_dir: tv_dir.to_string() };
        let conn = TcpStream::connect(format!("{host}:{port}"))?;
        client.session.set_tcp_stream(conn);
        client.session.handshake()?;
        client.session.userauth_pubkey_file(username, None, Path::new(privkey), None)?;

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
        let output = self.execute(&format!("ls -1 '{}'", self.tv_dir))?;
        Ok(output.split_terminator("\n").map(String::from).collect())
    }

    pub fn list_episodes(&mut self, show: &str) -> Result<Vec<String>> {
        let output = self.execute(&format!("ls -1 '{}/{}'", self.tv_dir, show))?;
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
